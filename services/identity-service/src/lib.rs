use axum::{
    extract::{Extension, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration as ChronoDuration, Utc};
use lifeready_audit::{AuditEvent, InMemoryAuditSink};
use lifeready_auth::{
    invalid_request, request_id_middleware, AccessLevel, AuthConfig, AuthLayer, Claims,
    RequestContext, RequestId, Role, SensitivityTier,
};
use lifeready_policy::{require_role, require_scope, require_tier, TierRequirement};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

const TOKEN_TTL_SECONDS: i64 = 900;

#[derive(Clone)]
struct AppState {
    audit: InMemoryAuditSink,
    auth: Arc<AuthConfig>,
}

pub fn router() -> Router {
    let auth = Arc::new(
        AuthConfig::from_env_checked()
            .expect("AuthConfig misconfigured (check LIFEREADY_ENV and JWT_SECRET)"),
    );
    let state = AppState {
        audit: InMemoryAuditSink::default(),
        auth: auth.clone(),
    };

    let public_paths = ["/v1/auth/login", "/v1/auth/mfa/verify"];

    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/auth/login", post(login))
        .route("/v1/auth/mfa/verify", post(verify_mfa))
        .route("/v1/me", get(me))
        .with_state(state)
        .layer(AuthLayer::new(auth).with_allowlist(public_paths))
        .layer(axum::middleware::from_fn(request_id_middleware))
}

async fn healthz() -> &'static str {
    "ok"
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LoginRequest {
    email: String,
    password: Option<String>,
    client: Option<LoginClient>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LoginClient {
    platform: String,
    device_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct LoginChallenge {
    challenge_id: String,
    mfa_required: bool,
    mfa_methods: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MfaVerifyRequest {
    challenge_id: String,
    method: String,
    code: String,
}

#[derive(Debug, Serialize)]
struct Session {
    access_token: String,
    expires_at: String,
}

#[derive(Debug, Serialize)]
struct MeResponse {
    principal_id: String,
    email: String,
    display_name: Option<String>,
}

async fn login(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginChallenge>), axum::response::Response> {
    if payload.email.trim().is_empty() {
        return Err(invalid_request(Some(request_id), "email is required"));
    }

    let challenge = LoginChallenge {
        challenge_id: uuid::Uuid::new_v4().to_string(),
        mfa_required: true,
        mfa_methods: vec!["totp".into()],
    };

    state.audit.record(AuditEvent::new(
        payload.email,
        "identity.login_started",
        "green",
        Some(request_id.0),
        None,
        serde_json::json!({"client": payload.client.map(|c| c.platform)}),
    ));

    Ok((StatusCode::OK, Json(challenge)))
}

async fn verify_mfa(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<MfaVerifyRequest>,
) -> Result<(StatusCode, Json<Session>), axum::response::Response> {
    if payload.challenge_id.trim().is_empty() {
        return Err(invalid_request(
            Some(request_id),
            "challenge_id is required",
        ));
    }
    if payload.code.trim().is_empty() {
        return Err(invalid_request(Some(request_id), "code is required"));
    }

    let claims = Claims::new(
        uuid::Uuid::new_v4().to_string(),
        Role::Principal,
        vec![SensitivityTier::Green, SensitivityTier::Amber],
        AccessLevel::LimitedWrite,
        None,
        TOKEN_TTL_SECONDS,
    );

    let token = state
        .auth
        .issue_token(&claims)
        .map_err(|error| error.into_response(Some(request_id)))?;

    let expires_at = (Utc::now() + ChronoDuration::seconds(TOKEN_TTL_SECONDS)).to_rfc3339();
    let session = Session {
        access_token: token,
        expires_at,
    };

    state.audit.record(AuditEvent::new(
        claims.sub.clone(),
        "identity.session_issued",
        "green",
        Some(request_id.0),
        None,
        serde_json::json!({"method": payload.method}),
    ));

    Ok((StatusCode::OK, Json(session)))
}

async fn me(
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<MeResponse>, axum::response::Response> {
    require_role(
        &ctx,
        &[
            Role::Principal,
            Role::Proxy,
            Role::ExecutorNominee,
            Role::EmergencyContact,
        ],
    )
    .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Green))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "read:all").map_err(|error| error.into_response(Some(request_id)))?;

    Ok(Json(MeResponse {
        principal_id: ctx.principal_id.clone(),
        email: ctx
            .email
            .clone()
            .unwrap_or_else(|| "principal@example.com".into()),
        display_name: None,
    }))
}

pub fn addr_from_env(default_port: u16) -> SocketAddr {
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("IDENTITY_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .or_else(|| std::env::var("PORT").ok().and_then(|p| p.parse().ok()))
        .unwrap_or(default_port);
    format!("{host}:{port}").parse().expect("valid host:port")
}

pub async fn check_db() -> Option<sqlx::PgPool> {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(_) => {
            tracing::warn!("DATABASE_URL not set; skipping database check");
            return None;
        }
    };

    let pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            tracing::warn!(error = %error, "database connection failed; continuing");
            return None;
        }
    };

    if let Err(error) = sqlx::query("SELECT 1").execute(&pool).await {
        tracing::warn!(error = %error, "database ping failed; continuing");
        return Some(pool);
    }

    tracing::info!("database connected");
    Some(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env(vars: &[(&str, Option<&str>)], f: impl FnOnce()) {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let mut saved = Vec::with_capacity(vars.len());

        for (key, value) in vars {
            saved.push((*key, std::env::var(*key).ok()));
            match value {
                Some(value) => unsafe { std::env::set_var(*key, value) },
                None => unsafe { std::env::remove_var(*key) },
            }
        }

        f();

        for (key, value) in saved {
            match value {
                Some(value) => unsafe { std::env::set_var(key, value) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
    }

    async fn with_env_async<F, Fut>(vars: &[(&str, Option<&str>)], f: F)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()>,
    {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let mut saved = Vec::with_capacity(vars.len());

        for (key, value) in vars {
            saved.push((*key, std::env::var(*key).ok()));
            match value {
                Some(value) => unsafe { std::env::set_var(*key, value) },
                None => unsafe { std::env::remove_var(*key) },
            }
        }

        f().await;

        for (key, value) in saved {
            match value {
                Some(value) => unsafe { std::env::set_var(key, value) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
    }

    #[test]
    fn addr_from_env_prefers_identity_port() {
        with_env(
            &[
                ("HOST", Some("127.0.0.1")),
                ("IDENTITY_PORT", Some("5555")),
                ("PORT", Some("9999")),
            ],
            || {
                let addr = addr_from_env(8081);
                assert_eq!(addr, "127.0.0.1:5555".parse::<SocketAddr>().unwrap());
            },
        );
    }

    #[test]
    fn addr_from_env_falls_back_to_port() {
        with_env(
            &[
                ("HOST", Some("0.0.0.0")),
                ("IDENTITY_PORT", None),
                ("PORT", Some("7000")),
            ],
            || {
                let addr = addr_from_env(8081);
                assert_eq!(addr, "0.0.0.0:7000".parse::<SocketAddr>().unwrap());
            },
        );
    }

    #[tokio::test]
    async fn check_db_returns_none_without_database_url() {
        with_env_async(&[("DATABASE_URL", None)], || async {
            let result = check_db().await;
            assert!(result.is_none());
        })
        .await;
    }
}
