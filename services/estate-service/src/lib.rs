use axum::{
    Json, Router,
    extract::{Extension, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use lifeready_audit::{AuditEvent, InMemoryAuditSink};
use lifeready_auth::{AuthConfig, AuthLayer, RequestContext, RequestId, request_id_middleware};
use lifeready_policy::{
    Role, SensitivityTier, TierRequirement, require_role, require_scope, require_tier,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    audit: InMemoryAuditSink,
}

pub fn router() -> Router {
    let state = AppState {
        audit: InMemoryAuditSink::default(),
    };
    let auth_config = Arc::new(
        AuthConfig::from_env_checked()
            .expect("AuthConfig misconfigured (check LIFEREADY_ENV and JWT_SECRET)"),
    );

    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/people", post(create_person).get(list_people))
        .route("/v1/assets", post(create_asset))
        .route("/v1/instructions", post(create_instruction))
        .route("/v1/roles/grants", post(create_role_grant))
        .with_state(state)
        .layer(AuthLayer::new(auth_config))
        .layer(axum::middleware::from_fn(request_id_middleware))
}

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz() -> (StatusCode, Json<serde_json::Value>) {
    let db_ready = check_db().await.is_some();
    if db_ready {
        (
            StatusCode::OK,
            Json(serde_json::json!({"status": "ready", "database": "up"})),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"status": "not_ready", "database": "down"})),
        )
    }
}

#[derive(Debug, Deserialize)]
struct PersonCreate {
    full_name: String,
    email: Option<String>,
    phone_e164: Option<String>,
    relationship: Option<String>,
}

#[derive(Debug, Serialize)]
struct PersonResponse {
    person_id: String,
    created_at: String,
    full_name: String,
    email: Option<String>,
    phone_e164: Option<String>,
    relationship: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PeopleQuery {
    limit: Option<u32>,
}

#[derive(Debug, Serialize)]
struct PeopleListResponse {
    items: Vec<PersonResponse>,
}

#[derive(Debug, Deserialize)]
struct AssetCreate {
    category: String,
    label: String,
    notes: Option<String>,
    sensitivity: Option<SensitivityTier>,
}

#[derive(Debug, Serialize)]
struct AssetResponse {
    asset_id: String,
    created_at: String,
    category: String,
    label: String,
    notes: Option<String>,
    sensitivity: SensitivityTier,
}

#[derive(Debug, Deserialize)]
struct InstructionCreate {
    title: String,
    body: String,
    sensitivity: Option<SensitivityTier>,
}

#[derive(Debug, Serialize)]
struct InstructionResponse {
    instruction_id: String,
    created_at: String,
    title: String,
    body: String,
    sensitivity: SensitivityTier,
}

#[derive(Debug, Deserialize)]
struct RoleGrantCreate {
    person_id: String,
    role: Role,
    scope: RoleScope,
}

#[derive(Debug, Deserialize, Serialize)]
struct RoleScope {
    access_level: String,
    allowed_tiers: Option<Vec<SensitivityTier>>,
    expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct RoleGrantResponse {
    grant_id: String,
    status: String,
    created_at: String,
    person_id: String,
    role: Role,
    scope: RoleScope,
}

async fn create_person(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<PersonCreate>,
) -> Result<(StatusCode, Json<PersonResponse>), axum::response::Response> {
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let response = PersonResponse {
        person_id: uuid::Uuid::new_v4().to_string(),
        created_at: Utc::now().to_rfc3339(),
        full_name: payload.full_name,
        email: payload.email,
        phone_e164: payload.phone_e164,
        relationship: payload.relationship,
    };

    state.audit.record(AuditEvent::new(
        ctx.principal_id.clone(),
        "estate.person.created",
        "amber",
        Some(request_id.0),
        None,
        serde_json::json!({"person_id": response.person_id}),
    ));

    Ok((StatusCode::CREATED, Json(response)))
}

async fn list_people(
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Query(query): Query<PeopleQuery>,
) -> Result<Json<PeopleListResponse>, axum::response::Response> {
    require_role(&ctx, &[Role::Principal, Role::Proxy, Role::ExecutorNominee])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "read:all").map_err(|error| error.into_response(Some(request_id)))?;

    let _limit = query.limit.unwrap_or(50);
    Ok(Json(PeopleListResponse { items: Vec::new() }))
}

async fn create_asset(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<AssetCreate>,
) -> Result<(StatusCode, Json<AssetResponse>), axum::response::Response> {
    let tier = payload.sensitivity.unwrap_or(SensitivityTier::Amber);
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Allowlist(vec![tier]))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let response = AssetResponse {
        asset_id: uuid::Uuid::new_v4().to_string(),
        created_at: Utc::now().to_rfc3339(),
        category: payload.category,
        label: payload.label,
        notes: payload.notes,
        sensitivity: tier,
    };

    state.audit.record(AuditEvent::new(
        ctx.principal_id.clone(),
        "estate.asset.created",
        format!("{:?}", tier).to_lowercase(),
        Some(request_id.0),
        None,
        serde_json::json!({"asset_id": response.asset_id}),
    ));

    Ok((StatusCode::CREATED, Json(response)))
}

async fn create_instruction(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<InstructionCreate>,
) -> Result<(StatusCode, Json<InstructionResponse>), axum::response::Response> {
    let tier = payload.sensitivity.unwrap_or(SensitivityTier::Amber);
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Allowlist(vec![tier]))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let response = InstructionResponse {
        instruction_id: uuid::Uuid::new_v4().to_string(),
        created_at: Utc::now().to_rfc3339(),
        title: payload.title,
        body: payload.body,
        sensitivity: tier,
    };

    state.audit.record(AuditEvent::new(
        ctx.principal_id.clone(),
        "estate.instruction.created",
        format!("{:?}", tier).to_lowercase(),
        Some(request_id.0),
        None,
        serde_json::json!({"instruction_id": response.instruction_id}),
    ));

    Ok((StatusCode::CREATED, Json(response)))
}

async fn create_role_grant(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<RoleGrantCreate>,
) -> Result<(StatusCode, Json<RoleGrantResponse>), axum::response::Response> {
    require_role(&ctx, &[Role::Principal])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let response = RoleGrantResponse {
        grant_id: uuid::Uuid::new_v4().to_string(),
        status: "invited".into(),
        created_at: Utc::now().to_rfc3339(),
        person_id: payload.person_id,
        role: payload.role,
        scope: payload.scope,
    };

    state.audit.record(AuditEvent::new(
        ctx.principal_id.clone(),
        "estate.role.grant_invited",
        "amber",
        Some(request_id.0),
        None,
        serde_json::json!({"grant_id": response.grant_id}),
    ));

    Ok((StatusCode::CREATED, Json(response)))
}

pub fn addr_from_env(default_port: u16) -> SocketAddr {
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("ESTATE_PORT")
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
        tracing::warn!(error = %error, "database ping failed; readiness unavailable");
        return None;
    }

    tracing::info!("database connected");
    Some(pool)
}

#[cfg(test)]
#[allow(clippy::await_holding_lock)]
mod tests {
    use super::*;
    use chrono::Utc;
    use lifeready_audit::InMemoryAuditSink;
    use lifeready_auth::{AccessLevel, RequestId};
    use lifeready_policy::{Role, SensitivityTier};
    use std::future::Future;
    use std::sync::Mutex;
    use uuid::Uuid;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn create_test_state() -> AppState {
        AppState {
            audit: InMemoryAuditSink::default(),
        }
    }

    fn create_test_ctx(
        role: Role,
        allowed_tiers: Vec<SensitivityTier>,
        scope: AccessLevel,
    ) -> RequestContext {
        let scopes = match scope {
            AccessLevel::LimitedWrite => vec!["write:limited".to_string()],
            AccessLevel::ReadOnlyAll => vec!["read:all".to_string()],
            AccessLevel::ReadOnlyPacks => vec!["read:packs".to_string()],
        };

        RequestContext {
            request_id: RequestId(Uuid::new_v4()),
            principal_id: "test-user".to_string(),
            roles: vec![role],
            allowed_tiers,
            scopes,
            expires_at: Utc::now(),
            email: None,
        }
    }

    fn create_test_payload(tier: Option<SensitivityTier>) -> AssetCreate {
        AssetCreate {
            category: "test-category".to_string(),
            label: "test-label".to_string(),
            notes: Some("test-notes".to_string()),
            sensitivity: tier,
        }
    }
    #[tokio::test]
    async fn create_asset_success() {
        let state = create_test_state();
        let ctx = create_test_ctx(
            Role::Principal,
            vec![SensitivityTier::Amber],
            AccessLevel::LimitedWrite,
        );
        let request_id = RequestId(Uuid::new_v4());
        let payload = create_test_payload(Some(SensitivityTier::Amber));

        let result = create_asset(
            State(state.clone()),
            ctx,
            Extension(request_id),
            Json(payload),
        )
        .await;

        match result {
            Ok((status, Json(response))) => {
                assert_eq!(status, StatusCode::CREATED);
                assert_eq!(response.category, "test-category");
                assert_eq!(response.label, "test-label");
                assert_eq!(response.sensitivity, SensitivityTier::Amber);
                assert!(response.notes.is_some());

                // Verify audit
                let events = state.audit.snapshot();
                assert_eq!(events.len(), 1);
                assert_eq!(events[0].action, "estate.asset.created");
                assert_eq!(events[0].tier, "amber");
            }
            Err(_) => panic!("expected success"),
        }
    }
    #[tokio::test]
    async fn create_asset_fails_wrong_role() {
        let state = create_test_state();
        let ctx = create_test_ctx(
            Role::ExecutorNominee,
            vec![SensitivityTier::Amber],
            AccessLevel::LimitedWrite,
        );
        let request_id = RequestId(Uuid::new_v4());
        let payload = create_test_payload(Some(SensitivityTier::Amber));

        let result = create_asset(State(state), ctx, Extension(request_id), Json(payload)).await;

        match result {
            Ok(_) => panic!("expected failure"),
            Err(response) => assert_eq!(response.status(), StatusCode::FORBIDDEN),
        }
    }

    #[tokio::test]
    async fn create_asset_fails_wrong_tier() {
        let state = create_test_state();
        let ctx = create_test_ctx(
            Role::Principal,
            vec![SensitivityTier::Green],
            AccessLevel::LimitedWrite,
        );
        let request_id = RequestId(Uuid::new_v4());
        let payload = create_test_payload(Some(SensitivityTier::Amber));

        let result = create_asset(State(state), ctx, Extension(request_id), Json(payload)).await;

        match result {
            Ok(_) => panic!("expected failure"),
            Err(response) => assert_eq!(response.status(), StatusCode::FORBIDDEN),
        }
    }

    #[tokio::test]
    async fn create_asset_fails_wrong_scope() {
        let state = create_test_state();
        let ctx = create_test_ctx(
            Role::Principal,
            vec![SensitivityTier::Amber],
            AccessLevel::ReadOnlyAll,
        );
        let request_id = RequestId(Uuid::new_v4());
        let payload = create_test_payload(Some(SensitivityTier::Amber));

        let result = create_asset(State(state), ctx, Extension(request_id), Json(payload)).await;

        match result {
            Ok(_) => panic!("expected failure"),
            Err(response) => assert_eq!(response.status(), StatusCode::FORBIDDEN),
        }
    }
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
    fn addr_from_env_prefers_estate_port() {
        with_env(
            &[
                ("HOST", Some("127.0.0.1")),
                ("ESTATE_PORT", Some("5556")),
                ("PORT", Some("9000")),
            ],
            || {
                let addr = addr_from_env(8082);
                assert_eq!(addr, "127.0.0.1:5556".parse::<SocketAddr>().unwrap());
            },
        );
    }

    #[test]
    fn addr_from_env_falls_back_to_port() {
        with_env(
            &[
                ("HOST", Some("0.0.0.0")),
                ("ESTATE_PORT", None),
                ("PORT", Some("7002")),
            ],
            || {
                let addr = addr_from_env(8082);
                assert_eq!(addr, "0.0.0.0:7002".parse::<SocketAddr>().unwrap());
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
