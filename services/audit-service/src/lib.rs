use std::time::Duration;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use lifeready_auth::{
    auth_middleware, authorize, conflict, invalid_request, request_id_middleware, AccessLevel,
    AuthConfig, AuthLayerState, Claims, RequestId, RequiredAccess, SensitivityTier,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use std::net::SocketAddr;
use std::{fs, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditAppend {
    pub actor_principal_id: String,
    pub action: String,
    pub tier: String,
    pub case_id: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub created_at: String,
    pub prev_hash: String,
    pub event_hash: String,
    pub event: AuditAppend,
}

#[derive(Clone, Default)]
struct AppState {
    pool: Option<PgPool>,
    export_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct AuditExportQuery {
    case_id: Option<String>,
}

#[derive(Serialize)]
struct AuditEventResponse {
    event_id: String,
    created_at: String,
    prev_hash: String,
    event_hash: String,
}

pub fn app() -> Router {
    let state = AppState {
        pool: pool_from_env(),
        export_dir: export_dir_from_env(),
    };
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/audit/events", post(append_audit_event))
        .route("/v1/audit/export", get(export_audit))
        .with_state(state)
        .layer(axum::middleware::from_fn_with_state(
            AuthLayerState::new(AuthConfig::from_env(), Vec::<&'static str>::new()),
            auth_middleware,
        ))
        .layer(axum::middleware::from_fn(request_id_middleware))
}

async fn healthz() -> &'static str {
    "ok"
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

pub fn compute_event_hash(prev_hash: &str, event: &AuditEvent) -> String {
    let canonical = canonical_event_json(event);
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(canonical.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn canonical_event_json(event: &AuditEvent) -> String {
    let value = serde_json::json!({
        "event_id": event.event_id,
        "created_at": event.created_at,
        "actor_principal_id": event.event.actor_principal_id,
        "action": event.event.action,
        "tier": event.event.tier,
        "case_id": event.event.case_id,
        "payload": event.event.payload,
    });
    let canonical_value = canonicalize_value(&value);
    serde_json::to_string(&canonical_value).unwrap_or_default()
}

fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut ordered = Map::new();
            for key in keys {
                if let Some(inner) = map.get(&key) {
                    ordered.insert(key, canonicalize_value(inner));
                }
            }
            Value::Object(ordered)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_value).collect()),
        _ => value.clone(),
    }
}

pub fn zero_hash() -> String {
    "0".repeat(64)
}

async fn append_audit_event(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(request_id): Extension<RequestId>,
    Json(input): Json<AuditAppend>,
) -> Result<(StatusCode, Json<AuditEventResponse>), axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    let tier = match input.tier.as_str() {
        "green" => SensitivityTier::Green,
        "amber" => SensitivityTier::Amber,
        "red" => SensitivityTier::Red,
        _ => return Err(invalid_request(Some(request_id), "invalid tier")),
    };
    let required = RequiredAccess {
        min_tier: tier,
        access_level: AccessLevel::LimitedWrite,
        allowed_roles: None,
    };
    authorize(&claims, &required).map_err(|error| error.into_response(Some(request_id)))?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;
    let prev_hash =
        sqlx::query("SELECT event_hash FROM audit_events ORDER BY created_at DESC LIMIT 1")
            .fetch_optional(&mut *tx)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?
            .and_then(|row| row.try_get::<String, _>("event_hash").ok())
            .unwrap_or_else(zero_hash);

    let created_at = Utc::now().to_rfc3339();
    let event_id = Uuid::new_v4();
    let actor_principal_id = Uuid::parse_str(&input.actor_principal_id)
        .map_err(|_| invalid_request(Some(request_id), "invalid actor_principal_id"))?;
    let case_id = input
        .case_id
        .as_ref()
        .map(|value| Uuid::parse_str(value))
        .transpose()
        .map_err(|_| invalid_request(Some(request_id), "invalid case_id"))?;
    let mut event = AuditEvent {
        event_id: event_id.to_string(),
        created_at: created_at.clone(),
        prev_hash: prev_hash.clone(),
        event_hash: String::new(),
        event: input,
    };
    event.event_hash = compute_event_hash(&event.prev_hash, &event);

    sqlx::query(
        "INSERT INTO audit_events (event_id, created_at, actor_principal_id, action, tier, case_id, payload, prev_hash, event_hash) \
         VALUES ($1, $2, $3, $4, $5::sensitivity_tier, $6, $7, $8, $9)",
    )
    .bind(event_id)
    .bind(created_at)
    .bind(actor_principal_id)
    .bind(&event.event.action)
    .bind(&event.event.tier)
    .bind(case_id)
    .bind(&event.event.payload)
    .bind(&event.prev_hash)
    .bind(&event.event_hash)
    .execute(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    tx.commit()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let response = AuditEventResponse {
        event_id: event.event_id,
        created_at: event.created_at,
        prev_hash: event.prev_hash,
        event_hash: event.event_hash,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn export_audit(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(request_id): Extension<RequestId>,
    query: axum::extract::Query<AuditExportQuery>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    let required = RequiredAccess {
        min_tier: SensitivityTier::Green,
        access_level: AccessLevel::ReadOnlyAll,
        allowed_roles: None,
    };
    authorize(&claims, &required).map_err(|error| error.into_response(Some(request_id)))?;

    let _ = query.case_id.as_deref();
    let rows = sqlx::query(
        "SELECT event_id, created_at, actor_principal_id, action, tier, case_id, payload, prev_hash, event_hash \
         FROM audit_events ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let mut events = Vec::new();
    for row in rows {
        events.push(AuditEvent {
            event_id: row
                .try_get::<uuid::Uuid, _>("event_id")
                .map_err(|error| db_error_to_response(error, request_id))?
                .to_string(),
            created_at: row
                .try_get::<chrono::DateTime<Utc>, _>("created_at")
                .map_err(|error| db_error_to_response(error, request_id))?
                .to_rfc3339(),
            prev_hash: row
                .try_get::<String, _>("prev_hash")
                .map_err(|error| db_error_to_response(error, request_id))?,
            event_hash: row
                .try_get::<String, _>("event_hash")
                .map_err(|error| db_error_to_response(error, request_id))?,
            event: AuditAppend {
                actor_principal_id: row
                    .try_get::<uuid::Uuid, _>("actor_principal_id")
                    .map_err(|error| db_error_to_response(error, request_id))?
                    .to_string(),
                action: row
                    .try_get::<String, _>("action")
                    .map_err(|error| db_error_to_response(error, request_id))?,
                tier: row
                    .try_get::<String, _>("tier")
                    .map_err(|error| db_error_to_response(error, request_id))?,
                case_id: row
                    .try_get::<Option<uuid::Uuid>, _>("case_id")
                    .map_err(|error| db_error_to_response(error, request_id))?
                    .map(|value| value.to_string()),
                payload: row
                    .try_get::<Value, _>("payload")
                    .map_err(|error| db_error_to_response(error, request_id))?,
            },
        });
    }

    let head_hash = events
        .last()
        .map(|event| event.event_hash.clone())
        .unwrap_or_else(zero_hash);
    let export_dir = state
        .export_dir
        .join(Utc::now().format("%Y%m%dT%H%M%SZ").to_string());
    fs::create_dir_all(&export_dir)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
    let export_path = export_dir.join("audit.jsonl");
    write_audit_jsonl(&export_path, &events)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
    let events_sha256 = sha256_file(&export_path)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;

    let response = serde_json::json!({
        "download_url": format!("file://{}", export_path.display()),
        "expires_at": Utc::now().to_rfc3339(),
        "head_hash": head_hash,
        "events_sha256": events_sha256,
    });
    Ok(Json(response))
}

fn pool_from_env() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect_lazy(&database_url).ok()
}

fn export_dir_from_env() -> PathBuf {
    std::env::var("AUDIT_EXPORT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("exports").join("audit"))
}

fn write_audit_jsonl(path: &PathBuf, events: &[AuditEvent]) -> Result<(), std::io::Error> {
    let mut lines = Vec::new();
    for event in events {
        let line = serde_json::to_string(event).unwrap_or_default();
        lines.push(line);
    }
    fs::write(path, lines.join("\n"))
}

fn sha256_file(path: &PathBuf) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}

fn db_error_to_response(error: sqlx::Error, request_id: RequestId) -> axum::response::Response {
    if let sqlx::Error::Database(db_error) = &error {
        if db_error.code().as_deref() == Some("23505") {
            return conflict(Some(request_id), "duplicate audit event");
        }
        return invalid_request(Some(request_id), db_error.message().to_string());
    }
    invalid_request(Some(request_id), error.to_string())
}
