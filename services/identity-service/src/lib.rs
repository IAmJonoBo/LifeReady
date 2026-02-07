use axum::{
    extract::{Extension, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration as ChronoDuration, Utc};
use lifeready_audit::{AuditEvent, InMemoryAuditSink};
use lifeready_auth::{
    auth_middleware, authorize, invalid_request, request_id_middleware, AccessLevel, AuthConfig,
    AuthLayerState, Claims, RequestId, RequiredAccess, Role, SensitivityTier,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Duration};

const TOKEN_TTL_SECONDS: i64 = 900;

#[derive(Clone)]
struct AppState {
    audit: InMemoryAuditSink,
    auth: AuthConfig,
}

pub fn router() -> Router {
    let auth = AuthConfig::from_env();
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
        .layer(axum::middleware::from_fn_with_state(
            AuthLayerState::new(auth, public_paths),
            auth_middleware,
        ))
        .layer(axum::middleware::from_fn(request_id_middleware))
}

async fn healthz() -> &'static str {
    "ok"
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: Option<String>,
    client: Option<LoginClient>,
}

#[derive(Debug, Deserialize)]
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
    Extension(claims): Extension<Claims>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<MeResponse>, axum::response::Response> {
    let required = RequiredAccess {
        min_tier: SensitivityTier::Green,
        access_level: AccessLevel::ReadOnlyAll,
        allowed_roles: None,
    };

    authorize(&claims, &required).map_err(|error| error.into_response(Some(request_id)))?;

    Ok(Json(MeResponse {
        principal_id: claims.sub.clone(),
        email: claims
            .email
            .clone()
            .unwrap_or_else(|| "principal@example.com".into()),
        display_name: None,
    }))
}

pub fn addr_from_env(default_port: u16) -> SocketAddr {
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
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
