use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use tokio::sync::Mutex;

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
    events: Arc<Mutex<Vec<AuditEvent>>>,
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
    let state = AppState::default();
    Router::new()
        .route("/v1/audit/events", post(append_audit_event))
        .route("/v1/audit/export", get(export_audit))
        .with_state(state)
}

pub fn addr_from_env(default_port: u16) -> SocketAddr {
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(default_port);
    format!("{host}:{port}").parse().expect("valid host:port")
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
    Json(input): Json<AuditAppend>,
) -> Result<(StatusCode, Json<AuditEventResponse>), StatusCode> {
    let mut events = state.events.lock().await;
    let prev_hash = events
        .last()
        .map(|e| e.event_hash.clone())
        .unwrap_or_else(zero_hash);
    let now = Utc::now().to_rfc3339();

    let mut event = AuditEvent {
        event_id: uuid(),
        created_at: now,
        prev_hash: prev_hash.clone(),
        event_hash: String::new(),
        event: input,
    };
    event.event_hash = compute_event_hash(&event.prev_hash, &event);
    events.push(event.clone());

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
    query: axum::extract::Query<AuditExportQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = query.case_id.as_deref();
    let events = state.events.lock().await;
    let head_hash = events
        .last()
        .map(|e| e.event_hash.clone())
        .unwrap_or_else(zero_hash);
    let response = serde_json::json!({
        "download_url": format!("https://api.lifeready.local/audit/exports/{}", head_hash),
        "expires_at": Utc::now().to_rfc3339(),
        "head_hash": head_hash,
    });
    Ok(Json(response))
}

fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
