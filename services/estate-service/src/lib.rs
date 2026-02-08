use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use lifeready_audit::{AuditEvent, InMemoryAuditSink};
use lifeready_auth::{request_id_middleware, AuthConfig, AuthLayer, RequestContext, RequestId};
use lifeready_policy::{
    require_role, require_scope, require_tier, Role, SensitivityTier, TierRequirement,
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
        tracing::warn!(error = %error, "database ping failed; continuing");
        return Some(pool);
    }

    tracing::info!("database connected");
    Some(pool)
}
