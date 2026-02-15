use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::{get, patch, post, put},
};
use chrono::Utc;
use lifeready_audit::zero_hash;
use lifeready_auth::{
    AuthConfig, AuthLayer, RequestContext, RequestId, conflict, invalid_request, not_found,
    request_id_middleware,
};
use lifeready_policy::{
    Role, SensitivityTier, TierRequirement, require_role, require_scope, require_scope_any,
    require_tier,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use std::fs;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::{path::PathBuf, str::FromStr};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

#[derive(Clone)]
struct AppState {
    pool: Option<PgPool>,
    export_dir: PathBuf,
    storage_dir: PathBuf,
}

pub fn router() -> Router {
    let state = AppState {
        pool: pool_from_env(),
        export_dir: export_dir_from_env(),
        storage_dir: storage_dir_from_env(),
    };
    let auth_config = Arc::new(
        AuthConfig::from_env_checked()
            .expect("AuthConfig misconfigured (check LIFEREADY_ENV and JWT_SECRET)"),
    );

    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/cases/emergency-pack", post(create_emergency_pack))
        .route("/v1/cases/mhca39", post(create_mhca39))
        .route("/v1/cases/will-prep-sa", post(create_will_prep_sa))
        .route(
            "/v1/cases/deceased-estate-sa",
            post(create_deceased_estate_sa),
        )
        .route("/v1/cases/popia-incident", post(create_popia_incident))
        .route("/v1/cases/death-readiness", post(create_death_readiness))
        .route("/v1/cases/{case_id}", patch(update_case))
        .route("/v1/cases/{case_id}/link", post(link_case))
        .route("/v1/cases/{case_id}/revoke", post(revoke_case))
        .route("/v1/cases/{case_id}/export", post(export_case))
        .route("/v1/cases/{case_id}/transition", post(transition_case))
        .route(
            "/v1/cases/{case_id}/evidence/{slot_name}",
            put(attach_evidence),
        )
        .with_state(state)
        .layer(AuthLayer::new(auth_config))
        .layer(axum::middleware::from_fn(request_id_middleware))
}

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let db_ready = match &state.pool {
        Some(pool) => sqlx::query("SELECT 1").execute(pool).await.is_ok(),
        None => false,
    };

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

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
struct EmergencyContact {
    name: String,
    phone_e164: String,
    relationship: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EmergencyPackRequest {
    directive_document_ids: Vec<String>,
    emergency_contacts: Vec<EmergencyContact>,
}

#[derive(Debug, Deserialize)]
struct Mhca39Create {
    subject_person_id: String,
    applicant_person_id: String,
    relationship_to_subject: Option<String>,
    notes: Option<String>,
    required_evidence_slots: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct WillPrepCreate {
    principal_person_id: String,
    notes: Option<String>,
    required_evidence_slots: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct DeceasedEstateCreate {
    deceased_person_id: String,
    executor_person_id: String,
    estimated_estate_value_zar: Option<f64>,
    notes: Option<String>,
    required_evidence_slots: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct CaseResponse {
    case_id: String,
    case_type: String,
    status: String,
    created_at: String,
    blocked_reasons: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ExportResponse {
    download_url: String,
    expires_at: String,
    manifest_sha256: String,
}

#[derive(Debug, Deserialize)]
struct EvidenceAttach {
    document_id: String,
}

#[derive(Debug, Serialize)]
struct EvidenceSlotResponse {
    slot_name: String,
    document_id: String,
    added_at: String,
}

#[derive(Debug, Deserialize)]
struct PopiaIncidentCreate {
    incident_title: String,
    description: Option<String>,
    affected_data_classes: Vec<String>,
    affected_user_count: Option<i32>,
    mitigation_steps: Option<String>,
    notes: Option<String>,
    required_evidence_slots: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct TransitionRequest {
    to_status: String,
    reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct TransitionResponse {
    case_id: String,
    from_status: String,
    to_status: String,
    transitioned_at: String,
}

#[derive(Debug, Deserialize)]
struct DeathReadinessCreate {
    executor_nominee_person_id: String,
    asset_document_ids: Option<Vec<String>>,
    contact_document_ids: Option<Vec<String>>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CaseUpdate {
    summary: Option<String>,
    mitigation_steps: Option<String>,
    affected_data_classes: Option<Vec<String>>,
    affected_user_count: Option<i32>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LinkRequest {
    expires_in_hours: Option<i32>,
}

#[derive(Debug, Serialize)]
struct LinkResponse {
    share_url: String,
    expires_at: String,
}

#[derive(Debug, Serialize)]
struct RevokeResponse {
    case_id: String,
    revoked_at: String,
}

#[derive(Debug, Serialize)]
struct ExportManifest {
    case_id: String,
    case_type: String,
    exported_at: String,
    audit_head_hash: String,
    audit_events_sha256: String,
    documents: Vec<ManifestDocument>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ManifestDocument {
    slot_name: String,
    document_id: String,
    document_type: String,
    title: String,
    sha256: String,
    bundle_path: String,
}

async fn create_emergency_pack(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<EmergencyPackRequest>,
) -> Result<(StatusCode, Json<CaseResponse>), axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;

    let directive_ids: Vec<uuid::Uuid> = payload
        .directive_document_ids
        .iter()
        .map(|id| {
            parse_uuid(id).ok_or_else(|| invalid_request(Some(request_id), "invalid document_id"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let contacts_json = serde_json::to_value(&payload.emergency_contacts)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let row = sqlx::query(
        "INSERT INTO cases (principal_id, case_type, status, blocked_reasons) \
         VALUES ($1, 'emergency_pack', 'draft', ARRAY[]::text[]) \
         RETURNING case_id, created_at, status, blocked_reasons",
    )
    .bind(principal_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let case_id: uuid::Uuid = row
        .try_get("case_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let created_at: chrono::DateTime<Utc> = row
        .try_get("created_at")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let status: String = row
        .try_get("status")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let blocked_reasons: Vec<String> = row
        .try_get("blocked_reasons")
        .map_err(|error| db_error_to_response(error, request_id))?;

    sqlx::query(
        "INSERT INTO emergency_pack_cases (case_id, directive_document_ids, emergency_contacts) \
         VALUES ($1, $2, $3)",
    )
    .bind(case_id)
    .bind(&directive_ids)
    .bind(&contacts_json)
    .execute(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    tx.commit()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let response = CaseResponse {
        case_id: case_id.to_string(),
        case_type: "emergency_pack".into(),
        status,
        created_at: created_at.to_rfc3339(),
        blocked_reasons,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn create_mhca39(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<Mhca39Create>,
) -> Result<(StatusCode, Json<CaseResponse>), axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;
    let subject_person_id = parse_uuid(&payload.subject_person_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid subject_person_id"))?;
    let applicant_person_id = parse_uuid(&payload.applicant_person_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid applicant_person_id"))?;
    let required_slots = payload
        .required_evidence_slots
        .clone()
        .unwrap_or_else(default_mhca39_slots);

    let mut tx = pool
        .begin()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;
    let row = sqlx::query(
        "INSERT INTO cases (principal_id, case_type, status, blocked_reasons) \
         VALUES ($1, 'mhca39', 'blocked', ARRAY['evidence incomplete']) \
         RETURNING case_id, created_at, status, blocked_reasons",
    )
    .bind(principal_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let case_id: uuid::Uuid = row
        .try_get("case_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let created_at: chrono::DateTime<Utc> = row
        .try_get("created_at")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let status: String = row
        .try_get("status")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let blocked_reasons: Vec<String> = row
        .try_get("blocked_reasons")
        .map_err(|error| db_error_to_response(error, request_id))?;

    sqlx::query(
        "INSERT INTO mhca39_cases (case_id, subject_person_id, applicant_person_id, relationship_to_subject, required_evidence_slots, notes) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(case_id)
    .bind(subject_person_id)
    .bind(applicant_person_id)
    .bind(payload.relationship_to_subject)
    .bind(required_slots.clone())
    .bind(payload.notes)
    .execute(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    for slot in &required_slots {
        sqlx::query("INSERT INTO mhca39_evidence (case_id, slot_name) VALUES ($1, $2)")
            .bind(case_id)
            .bind(slot)
            .execute(&mut *tx)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
    }

    tx.commit()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let response = CaseResponse {
        case_id: case_id.to_string(),
        case_type: "mhca39".into(),
        status,
        created_at: created_at.to_rfc3339(),
        blocked_reasons,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn create_will_prep_sa(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<WillPrepCreate>,
) -> Result<(StatusCode, Json<CaseResponse>), axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;
    let principal_person_id = parse_uuid(&payload.principal_person_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_person_id"))?;
    let required_slots = payload
        .required_evidence_slots
        .clone()
        .unwrap_or_else(default_will_prep_slots);

    let mut tx = pool
        .begin()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;
    let row = sqlx::query(
        "INSERT INTO cases (principal_id, case_type, status, blocked_reasons) \
         VALUES ($1, 'will_prep_sa', 'blocked', ARRAY['evidence incomplete']) \
         RETURNING case_id, created_at, status, blocked_reasons",
    )
    .bind(principal_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let case_id: uuid::Uuid = row
        .try_get("case_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let created_at: chrono::DateTime<Utc> = row
        .try_get("created_at")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let status: String = row
        .try_get("status")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let blocked_reasons: Vec<String> = row
        .try_get("blocked_reasons")
        .map_err(|error| db_error_to_response(error, request_id))?;

    sqlx::query(
        "INSERT INTO will_prep_cases (case_id, principal_person_id, required_evidence_slots, notes) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(case_id)
    .bind(principal_person_id)
    .bind(required_slots.clone())
    .bind(payload.notes)
    .execute(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    for slot in &required_slots {
        sqlx::query("INSERT INTO case_evidence (case_id, slot_name) VALUES ($1, $2)")
            .bind(case_id)
            .bind(slot)
            .execute(&mut *tx)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
    }

    tx.commit()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let response = CaseResponse {
        case_id: case_id.to_string(),
        case_type: "will_prep_sa".into(),
        status,
        created_at: created_at.to_rfc3339(),
        blocked_reasons,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn create_deceased_estate_sa(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<DeceasedEstateCreate>,
) -> Result<(StatusCode, Json<CaseResponse>), axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;
    let deceased_person_id = parse_uuid(&payload.deceased_person_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid deceased_person_id"))?;
    let executor_person_id = parse_uuid(&payload.executor_person_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid executor_person_id"))?;
    let required_slots = payload
        .required_evidence_slots
        .clone()
        .unwrap_or_else(default_deceased_estate_slots);

    let mut tx = pool
        .begin()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;
    let row = sqlx::query(
        "INSERT INTO cases (principal_id, case_type, status, blocked_reasons) \
         VALUES ($1, 'deceased_estate_reporting_sa', 'blocked', ARRAY['evidence incomplete']) \
         RETURNING case_id, created_at, status, blocked_reasons",
    )
    .bind(principal_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let case_id: uuid::Uuid = row
        .try_get("case_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let created_at: chrono::DateTime<Utc> = row
        .try_get("created_at")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let status: String = row
        .try_get("status")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let blocked_reasons: Vec<String> = row
        .try_get("blocked_reasons")
        .map_err(|error| db_error_to_response(error, request_id))?;

    sqlx::query(
        "INSERT INTO deceased_estate_cases (case_id, deceased_person_id, executor_person_id, estimated_estate_value_zar, required_evidence_slots, notes) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(case_id)
    .bind(deceased_person_id)
    .bind(executor_person_id)
    .bind(payload.estimated_estate_value_zar)
    .bind(required_slots.clone())
    .bind(payload.notes)
    .execute(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    for slot in &required_slots {
        sqlx::query("INSERT INTO case_evidence (case_id, slot_name) VALUES ($1, $2)")
            .bind(case_id)
            .bind(slot)
            .execute(&mut *tx)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
    }

    tx.commit()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let response = CaseResponse {
        case_id: case_id.to_string(),
        case_type: "deceased_estate_reporting_sa".into(),
        status,
        created_at: created_at.to_rfc3339(),
        blocked_reasons,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn create_popia_incident(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<PopiaIncidentCreate>,
) -> Result<(StatusCode, Json<CaseResponse>), axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;
    let required_slots = payload
        .required_evidence_slots
        .clone()
        .unwrap_or_else(default_popia_incident_slots);

    let mut tx = pool
        .begin()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;
    let row = sqlx::query(
        "INSERT INTO cases (principal_id, case_type, status, blocked_reasons) \
         VALUES ($1, 'popia_incident', 'draft', ARRAY[]::text[]) \
         RETURNING case_id, created_at, status, blocked_reasons",
    )
    .bind(principal_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let case_id: uuid::Uuid = row
        .try_get("case_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let created_at: chrono::DateTime<Utc> = row
        .try_get("created_at")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let status: String = row
        .try_get("status")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let blocked_reasons: Vec<String> = row
        .try_get("blocked_reasons")
        .map_err(|error| db_error_to_response(error, request_id))?;

    sqlx::query(
        "INSERT INTO popia_incident_cases (case_id, incident_title, description, \
         affected_data_classes, affected_user_count, mitigation_steps, \
         required_evidence_slots, notes) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(case_id)
    .bind(&payload.incident_title)
    .bind(&payload.description)
    .bind(&payload.affected_data_classes)
    .bind(payload.affected_user_count)
    .bind(&payload.mitigation_steps)
    .bind(&required_slots)
    .bind(&payload.notes)
    .execute(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    for slot in &required_slots {
        sqlx::query("INSERT INTO case_evidence (case_id, slot_name) VALUES ($1, $2)")
            .bind(case_id)
            .bind(slot)
            .execute(&mut *tx)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
    }

    tx.commit()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let response = CaseResponse {
        case_id: case_id.to_string(),
        case_type: "popia_incident".into(),
        status,
        created_at: created_at.to_rfc3339(),
        blocked_reasons,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn create_death_readiness(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<DeathReadinessCreate>,
) -> Result<(StatusCode, Json<CaseResponse>), axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;
    let executor_nominee_id = parse_uuid(&payload.executor_nominee_person_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid executor_nominee_person_id"))?;

    let asset_ids: Vec<uuid::Uuid> = payload
        .asset_document_ids
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|id| {
            parse_uuid(id)
                .ok_or_else(|| invalid_request(Some(request_id), "invalid asset_document_id"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let contact_ids: Vec<uuid::Uuid> = payload
        .contact_document_ids
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|id| {
            parse_uuid(id)
                .ok_or_else(|| invalid_request(Some(request_id), "invalid contact_document_id"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let row = sqlx::query(
        "INSERT INTO cases (principal_id, case_type, status, blocked_reasons) \
         VALUES ($1, 'death_readiness', 'draft', ARRAY[]::text[]) \
         RETURNING case_id, created_at, status, blocked_reasons",
    )
    .bind(principal_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let case_id: uuid::Uuid = row
        .try_get("case_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let created_at: chrono::DateTime<Utc> = row
        .try_get("created_at")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let status: String = row
        .try_get("status")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let blocked_reasons: Vec<String> = row
        .try_get("blocked_reasons")
        .map_err(|error| db_error_to_response(error, request_id))?;

    sqlx::query(
        "INSERT INTO death_readiness_cases \
         (case_id, executor_nominee_person_id, asset_document_ids, contact_document_ids, notes) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(case_id)
    .bind(executor_nominee_id)
    .bind(&asset_ids)
    .bind(&contact_ids)
    .bind(&payload.notes)
    .execute(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    tx.commit()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let response = CaseResponse {
        case_id: case_id.to_string(),
        case_type: "death_readiness".into(),
        status,
        created_at: created_at.to_rfc3339(),
        blocked_reasons,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn update_case(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Path(case_id): Path<String>,
    Json(payload): Json<CaseUpdate>,
) -> Result<Json<CaseResponse>, axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let case_id =
        parse_uuid(&case_id).ok_or_else(|| invalid_request(Some(request_id), "invalid case_id"))?;
    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;
    ensure_case_access(pool, case_id, principal_id, request_id).await?;

    let case_type = fetch_case_type(pool, case_id, request_id).await?;
    if case_type != "popia_incident" {
        return Err(invalid_request(
            Some(request_id),
            "PATCH updates are only supported for popia_incident cases",
        ));
    }

    // Append-only: record a new revision, never overwrite existing data
    let revision_number: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(revision_number), 0) + 1 FROM incident_revisions WHERE case_id = $1",
    )
    .bind(case_id)
    .fetch_one(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    sqlx::query(
        "INSERT INTO incident_revisions \
         (case_id, revision_number, summary, mitigation_steps, \
          affected_data_classes, affected_user_count, notes, actor_principal_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(case_id)
    .bind(revision_number as i32)
    .bind(&payload.summary)
    .bind(&payload.mitigation_steps)
    .bind(payload.affected_data_classes.as_deref().unwrap_or_default())
    .bind(payload.affected_user_count)
    .bind(&payload.notes)
    .bind(principal_id)
    .execute(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let row = sqlx::query(
        "SELECT case_id, case_type::text, status::text, created_at, blocked_reasons \
         FROM cases WHERE case_id = $1",
    )
    .bind(case_id)
    .fetch_one(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let response = CaseResponse {
        case_id: row
            .try_get::<uuid::Uuid, _>("case_id")
            .map_err(|error| db_error_to_response(error, request_id))?
            .to_string(),
        case_type: row
            .try_get::<String, _>("case_type")
            .map_err(|error| db_error_to_response(error, request_id))?,
        status: row
            .try_get::<String, _>("status")
            .map_err(|error| db_error_to_response(error, request_id))?,
        created_at: row
            .try_get::<chrono::DateTime<Utc>, _>("created_at")
            .map_err(|error| db_error_to_response(error, request_id))?
            .to_rfc3339(),
        blocked_reasons: row
            .try_get::<Vec<String>, _>("blocked_reasons")
            .map_err(|error| db_error_to_response(error, request_id))?,
    };

    Ok(Json(response))
}

async fn link_case(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Path(case_id): Path<String>,
    Json(payload): Json<LinkRequest>,
) -> Result<Json<LinkResponse>, axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let case_id =
        parse_uuid(&case_id).ok_or_else(|| invalid_request(Some(request_id), "invalid case_id"))?;
    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;
    ensure_case_access(pool, case_id, principal_id, request_id).await?;

    let case_type = fetch_case_type(pool, case_id, request_id).await?;
    if case_type != "emergency_pack" {
        return Err(invalid_request(
            Some(request_id),
            "link issuance is only supported for emergency_pack cases",
        ));
    }

    let expires_in_hours = payload.expires_in_hours.unwrap_or(24).clamp(1, 168);
    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + chrono::Duration::hours(i64::from(expires_in_hours));

    sqlx::query(
        "UPDATE emergency_pack_cases SET share_link_token = $1, share_link_expires_at = $2 \
         WHERE case_id = $3",
    )
    .bind(&token)
    .bind(expires_at)
    .bind(case_id)
    .execute(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    // Transition to link_issued if currently in ready state
    let status_row = sqlx::query("SELECT status::text FROM cases WHERE case_id = $1")
        .bind(case_id)
        .fetch_one(pool)
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;
    let current_status: String = status_row
        .try_get("status")
        .map_err(|error| db_error_to_response(error, request_id))?;

    let allowed = allowed_transitions(&case_type, &current_status);
    if allowed.contains(&"link_issued") {
        sqlx::query("UPDATE cases SET status = 'link_issued' WHERE case_id = $1")
            .bind(case_id)
            .execute(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
    }

    let share_url = format!("https://api.lifeready.local/case/v1/share/{}", token);
    let response = LinkResponse {
        share_url,
        expires_at: expires_at.to_rfc3339(),
    };

    Ok(Json(response))
}

async fn revoke_case(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Path(case_id): Path<String>,
) -> Result<Json<RevokeResponse>, axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let case_id =
        parse_uuid(&case_id).ok_or_else(|| invalid_request(Some(request_id), "invalid case_id"))?;
    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;
    ensure_case_access(pool, case_id, principal_id, request_id).await?;

    // Clear the share link immediately
    sqlx::query(
        "UPDATE emergency_pack_cases SET share_link_token = NULL, share_link_expires_at = NULL \
         WHERE case_id = $1",
    )
    .bind(case_id)
    .execute(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    // Transition to revoked
    sqlx::query("UPDATE cases SET status = 'revoked' WHERE case_id = $1")
        .bind(case_id)
        .execute(pool)
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let response = RevokeResponse {
        case_id: case_id.to_string(),
        revoked_at: Utc::now().to_rfc3339(),
    };

    Ok(Json(response))
}

/// Allowed state transitions per case type, based on PRD §7 state machines.
fn allowed_transitions(case_type: &str, from: &str) -> &'static [&'static str] {
    match (case_type, from) {
        // §7.1 Emergency Directive Pack
        ("emergency_pack", "draft") => &["ready"],
        ("emergency_pack", "ready") => &["link_issued"],
        ("emergency_pack", "link_issued") => &["accessed", "revoked", "expired"],
        // §7.2 MHCA 39 Case
        ("mhca39", "blocked") => &["evidence_collecting"],
        ("mhca39", "evidence_collecting") => &["draft_generated", "blocked"],
        ("mhca39", "draft_generated") => &["awaiting_oath"],
        ("mhca39", "awaiting_oath") => &["exported"],
        ("mhca39", "exported") => &["closed"],
        // §7.3 Death Readiness Pack (will_prep_sa, deceased_estate_reporting_sa)
        ("will_prep_sa", "blocked") => &["ready"],
        ("will_prep_sa", "ready") => &["exported"],
        ("will_prep_sa", "exported") => &["accessed", "revoked"],
        ("deceased_estate_reporting_sa", "blocked") => &["ready"],
        ("deceased_estate_reporting_sa", "ready") => &["exported"],
        ("deceased_estate_reporting_sa", "exported") => &["accessed", "revoked"],
        // POPIA Incident
        ("popia_incident", "draft") => &["ready"],
        ("popia_incident", "ready") => &["exported"],
        ("popia_incident", "exported") => &["closed"],
        // §7.3 Death Readiness Pack
        ("death_readiness", "draft") => &["ready"],
        ("death_readiness", "ready") => &["exported"],
        ("death_readiness", "exported") => &["closed"],
        _ => {
            tracing::debug!(
                case_type = case_type,
                from_status = from,
                "no transitions defined for this case_type/status combination"
            );
            &[]
        }
    }
}

async fn transition_case(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Path(case_id): Path<String>,
    Json(payload): Json<TransitionRequest>,
) -> Result<Json<TransitionResponse>, axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let case_id =
        parse_uuid(&case_id).ok_or_else(|| invalid_request(Some(request_id), "invalid case_id"))?;
    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;

    ensure_case_access(pool, case_id, principal_id, request_id).await?;

    let row = sqlx::query("SELECT case_type::text, status::text FROM cases WHERE case_id = $1")
        .bind(case_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;
    let row = match row {
        Some(row) => row,
        None => return Err(not_found(Some(request_id), "case not found")),
    };
    let case_type: String = row
        .try_get("case_type")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let current_status: String = row
        .try_get("status")
        .map_err(|error| db_error_to_response(error, request_id))?;

    let valid_targets = allowed_transitions(&case_type, &current_status);
    if !valid_targets.contains(&payload.to_status.as_str()) {
        return Err(conflict(
            Some(request_id),
            format!(
                "transition from '{}' to '{}' not allowed for {}",
                current_status, payload.to_status, case_type
            ),
        ));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    sqlx::query("UPDATE cases SET status = $1::case_status WHERE case_id = $2")
        .bind(&payload.to_status)
        .bind(case_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    sqlx::query(
        "INSERT INTO case_transitions (case_id, from_status, to_status, actor_principal_id, reason) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(case_id)
    .bind(&current_status)
    .bind(&payload.to_status)
    .bind(principal_id)
    .bind(&payload.reason)
    .execute(&mut *tx)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    tx.commit()
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    Ok(Json(TransitionResponse {
        case_id: case_id.to_string(),
        from_status: current_status,
        to_status: payload.to_status,
        transitioned_at: Utc::now().to_rfc3339(),
    }))
}

async fn attach_evidence(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Path((case_id, slot_name)): Path<(String, String)>,
    Json(payload): Json<EvidenceAttach>,
) -> Result<Json<EvidenceSlotResponse>, axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let case_id =
        parse_uuid(&case_id).ok_or_else(|| invalid_request(Some(request_id), "invalid case_id"))?;
    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;
    let document_id = parse_uuid(&payload.document_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid document_id"))?;

    ensure_case_access(pool, case_id, principal_id, request_id).await?;

    let exists = sqlx::query("SELECT 1 FROM documents WHERE document_id = $1")
        .bind(document_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| db_error_to_response(error, request_id))?
        .is_some();
    if !exists {
        return Err(not_found(Some(request_id), "document not found"));
    }

    // Determine case type to update the correct evidence table.
    // Table names are compile-time literals from the match below, not user input.
    let case_type = fetch_case_type(pool, case_id, request_id).await?;
    let evidence_table = match case_type.as_str() {
        "mhca39" => "mhca39_evidence",
        _ => "case_evidence",
    };

    let query = format!(
        "UPDATE {} SET document_id = $1, added_at = now() \
         WHERE case_id = $2 AND slot_name = $3 \
         RETURNING slot_name, document_id, added_at",
        evidence_table
    );
    let row = sqlx::query(&query)
        .bind(document_id)
        .bind(case_id)
        .bind(&slot_name)
        .fetch_optional(pool)
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let row = match row {
        Some(row) => row,
        None => return Err(not_found(Some(request_id), "evidence slot not found")),
    };

    let added_at: chrono::DateTime<Utc> = row
        .try_get("added_at")
        .map_err(|error| db_error_to_response(error, request_id))?;

    Ok(Json(EvidenceSlotResponse {
        slot_name: row
            .try_get::<String, _>("slot_name")
            .map_err(|error| db_error_to_response(error, request_id))?,
        document_id: row
            .try_get::<uuid::Uuid, _>("document_id")
            .map_err(|error| db_error_to_response(error, request_id))?
            .to_string(),
        added_at: added_at.to_rfc3339(),
    }))
}

async fn export_case(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Path(case_id): Path<String>,
) -> Result<Json<ExportResponse>, axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy, Role::ExecutorNominee])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope_any(&ctx, &["read:packs", "read:all"])
        .map_err(|error| error.into_response(Some(request_id)))?;
    let include_audit = ctx.scopes.iter().any(|scope| scope == "read:all");

    let case_id =
        parse_uuid(&case_id).ok_or_else(|| invalid_request(Some(request_id), "invalid case_id"))?;
    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;
    ensure_case_access(pool, case_id, principal_id, request_id).await?;

    // Determine case type to fetch from the correct tables
    let case_type = fetch_case_type(pool, case_id, request_id).await?;
    let (evidence_table, slots_query, required_slots) = match case_type.as_str() {
        "emergency_pack" => {
            // Emergency pack uses directive_document_ids, not evidence slots.
            // We treat each directive_document_id as a synthetic slot.
            let row = sqlx::query(
                "SELECT directive_document_ids FROM emergency_pack_cases WHERE case_id = $1",
            )
            .bind(case_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
            let doc_ids: Vec<uuid::Uuid> = match row {
                Some(r) => r
                    .try_get("directive_document_ids")
                    .map_err(|error| db_error_to_response(error, request_id))?,
                None => {
                    return Err(not_found(Some(request_id), "emergency_pack case not found"));
                }
            };
            let slots: Vec<String> = doc_ids.iter().map(|id| id.to_string()).collect();
            // Use empty table markers; we fetch documents directly below.
            ("__emergency_pack__", "__emergency_pack__", slots)
        }
        "mhca39" => {
            let row =
                sqlx::query("SELECT required_evidence_slots FROM mhca39_cases WHERE case_id = $1")
                    .bind(case_id)
                    .fetch_optional(pool)
                    .await
                    .map_err(|error| db_error_to_response(error, request_id))?;
            let slots: Vec<String> = match row {
                Some(r) => r
                    .try_get("required_evidence_slots")
                    .map_err(|error| db_error_to_response(error, request_id))?,
                None => return Err(not_found(Some(request_id), "mhca39 case not found")),
            };
            ("mhca39_evidence", "mhca39_evidence", slots)
        }
        "will_prep_sa" => {
            let row = sqlx::query(
                "SELECT required_evidence_slots FROM will_prep_cases WHERE case_id = $1",
            )
            .bind(case_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
            let slots: Vec<String> = match row {
                Some(r) => r
                    .try_get("required_evidence_slots")
                    .map_err(|error| db_error_to_response(error, request_id))?,
                None => return Err(not_found(Some(request_id), "will_prep_sa case not found")),
            };
            ("case_evidence", "case_evidence", slots)
        }
        "deceased_estate_reporting_sa" => {
            let row = sqlx::query(
                "SELECT required_evidence_slots FROM deceased_estate_cases WHERE case_id = $1",
            )
            .bind(case_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
            let slots: Vec<String> = match row {
                Some(r) => r
                    .try_get("required_evidence_slots")
                    .map_err(|error| db_error_to_response(error, request_id))?,
                None => {
                    return Err(not_found(
                        Some(request_id),
                        "deceased_estate case not found",
                    ));
                }
            };
            ("case_evidence", "case_evidence", slots)
        }
        "popia_incident" => {
            let row = sqlx::query(
                "SELECT required_evidence_slots FROM popia_incident_cases WHERE case_id = $1",
            )
            .bind(case_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
            let slots: Vec<String> = match row {
                Some(r) => r
                    .try_get("required_evidence_slots")
                    .map_err(|error| db_error_to_response(error, request_id))?,
                None => {
                    return Err(not_found(Some(request_id), "popia_incident case not found"));
                }
            };
            ("case_evidence", "case_evidence", slots)
        }
        "death_readiness" => {
            // Death readiness uses document references, not evidence slots.
            let row = sqlx::query(
                "SELECT asset_document_ids, contact_document_ids \
                 FROM death_readiness_cases WHERE case_id = $1",
            )
            .bind(case_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
            let row = match row {
                Some(r) => r,
                None => {
                    return Err(not_found(
                        Some(request_id),
                        "death_readiness case not found",
                    ));
                }
            };
            let asset_ids: Vec<uuid::Uuid> = row
                .try_get("asset_document_ids")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let contact_ids: Vec<uuid::Uuid> = row
                .try_get("contact_document_ids")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let mut all_ids = Vec::new();
            all_ids.extend(asset_ids.iter().map(|id| id.to_string()));
            all_ids.extend(contact_ids.iter().map(|id| id.to_string()));
            ("__death_readiness__", "__death_readiness__", all_ids)
        }
        _ => {
            return Err(invalid_request(
                Some(request_id),
                "unsupported case type for export",
            ));
        }
    };

    // Safety: evidence_table and slots_query are compile-time string literals
    // selected by the exhaustive match above; they are never user-supplied.

    let export_dir = state
        .export_dir
        .join(case_id.to_string())
        .join(Utc::now().format("%Y%m%dT%H%M%SZ").to_string());
    let documents_dir = export_dir.join("documents");
    fs::create_dir_all(&documents_dir)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;

    let mut manifest_documents = Vec::new();

    if evidence_table == "__emergency_pack__" {
        // Emergency pack: fetch documents directly from directive_document_ids
        if required_slots.is_empty() {
            return Err(conflict(
                Some(request_id),
                "no directive documents attached",
            ));
        }
        for (idx, doc_id_str) in required_slots.iter().enumerate() {
            let document_id = parse_uuid(doc_id_str)
                .ok_or_else(|| invalid_request(Some(request_id), "invalid document_id"))?;
            let row = sqlx::query(
                "SELECT d.document_id, d.document_type, d.title, v.sha256, v.blob_ref \
                 FROM documents d \
                 JOIN LATERAL ( \
                    SELECT sha256, blob_ref FROM document_versions \
                    WHERE document_id = d.document_id ORDER BY created_at DESC LIMIT 1 \
                 ) v ON true \
                 WHERE d.document_id = $1",
            )
            .bind(document_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
            let row = match row {
                Some(r) => r,
                None => return Err(not_found(Some(request_id), "directive document not found")),
            };
            let blob_ref: String = row
                .try_get("blob_ref")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let source_path = resolve_blob_ref(&blob_ref, &state.storage_dir)
                .ok_or_else(|| invalid_request(Some(request_id), "invalid blob_ref"))?;
            if !source_path.exists() {
                return Err(not_found(Some(request_id), "document blob not found"));
            }
            let dest_path = documents_dir.join(document_id.to_string());
            fs::copy(&source_path, &dest_path)
                .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;

            let sha256: String = row
                .try_get("sha256")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let document_type: String = row
                .try_get("document_type")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let title: String = row
                .try_get("title")
                .map_err(|error| db_error_to_response(error, request_id))?;
            manifest_documents.push(ManifestDocument {
                slot_name: format!("directive_{}", idx),
                document_id: document_id.to_string(),
                document_type,
                title,
                sha256,
                bundle_path: format!("documents/{}", document_id),
            });
        }
    } else if evidence_table == "__death_readiness__" {
        // Death readiness: fetch documents directly by ID
        for (idx, doc_id_str) in required_slots.iter().enumerate() {
            let document_id = parse_uuid(doc_id_str)
                .ok_or_else(|| invalid_request(Some(request_id), "invalid document_id"))?;
            let row = sqlx::query(
                "SELECT d.document_id, d.document_type, d.title, v.sha256, v.blob_ref \
                 FROM documents d \
                 JOIN LATERAL ( \
                    SELECT sha256, blob_ref FROM document_versions \
                    WHERE document_id = d.document_id ORDER BY created_at DESC LIMIT 1 \
                 ) v ON true \
                 WHERE d.document_id = $1",
            )
            .bind(document_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
            let row = match row {
                Some(r) => r,
                None => continue, // Skip missing documents gracefully
            };
            let blob_ref: String = row
                .try_get("blob_ref")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let source_path = resolve_blob_ref(&blob_ref, &state.storage_dir)
                .ok_or_else(|| invalid_request(Some(request_id), "invalid blob_ref"))?;
            if !source_path.exists() {
                continue; // Skip missing blobs gracefully
            }
            let dest_path = documents_dir.join(document_id.to_string());
            fs::copy(&source_path, &dest_path)
                .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;

            let sha256: String = row
                .try_get("sha256")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let document_type: String = row
                .try_get("document_type")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let title: String = row
                .try_get("title")
                .map_err(|error| db_error_to_response(error, request_id))?;
            manifest_documents.push(ManifestDocument {
                slot_name: format!("doc_{}", idx),
                document_id: document_id.to_string(),
                document_type,
                title,
                sha256,
                bundle_path: format!("documents/{}", document_id),
            });
        }
    } else {
        let missing_query = format!(
            "SELECT slot_name FROM {} WHERE case_id = $1 AND document_id IS NULL",
            evidence_table
        );
        let missing_slots = sqlx::query(&missing_query)
            .bind(case_id)
            .fetch_all(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;
        if !missing_slots.is_empty() {
            return Err(conflict(Some(request_id), "evidence slots incomplete"));
        }

        let evidence_join_query = format!(
            "SELECT e.slot_name, e.document_id, d.document_type, d.title, v.sha256, v.blob_ref \
             FROM {} e \
             JOIN documents d ON d.document_id = e.document_id \
             JOIN LATERAL ( \
                SELECT sha256, blob_ref FROM document_versions \
                WHERE document_id = e.document_id ORDER BY created_at DESC LIMIT 1 \
             ) v ON true \
             WHERE e.case_id = $1 ORDER BY e.slot_name",
            slots_query
        );
        let rows = sqlx::query(&evidence_join_query)
            .bind(case_id)
            .fetch_all(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;

        if rows.len() != required_slots.len() {
            return Err(conflict(Some(request_id), "evidence versions missing"));
        }

        for row in rows {
            let document_id: uuid::Uuid = row
                .try_get("document_id")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let blob_ref: String = row
                .try_get("blob_ref")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let source_path = resolve_blob_ref(&blob_ref, &state.storage_dir)
                .ok_or_else(|| invalid_request(Some(request_id), "invalid blob_ref"))?;
            if !source_path.exists() {
                return Err(not_found(Some(request_id), "document blob not found"));
            }
            let dest_path = documents_dir.join(document_id.to_string());
            fs::copy(&source_path, &dest_path)
                .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;

            let sha256: String = row
                .try_get("sha256")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let slot_name: String = row
                .try_get("slot_name")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let document_type: String = row
                .try_get("document_type")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let title: String = row
                .try_get("title")
                .map_err(|error| db_error_to_response(error, request_id))?;
            let bundle_path = format!("documents/{}", document_id);

            manifest_documents.push(ManifestDocument {
                slot_name,
                document_id: document_id.to_string(),
                document_type,
                title,
                sha256,
                bundle_path,
            });
        }
    }

    manifest_documents.sort_by(|a, b| a.slot_name.cmp(&b.slot_name));

    let audit_events = if include_audit {
        fetch_audit_events(pool).await?
    } else {
        Vec::new()
    };
    let audit_head_hash = audit_events
        .last()
        .map(|event| event.event_hash.clone())
        .unwrap_or_else(zero_hash);
    let audit_path = export_dir.join("audit.jsonl");
    write_audit_jsonl(&audit_path, &audit_events)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
    let audit_sha256 = sha256_file(&audit_path)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;

    // Generate type-specific template output and instructions
    let (template_filename, template_bytes, instructions_filename, instructions) = match case_type
        .as_str()
    {
        "emergency_pack" => {
            let template =
                generate_emergency_pack_template(pool, case_id, &manifest_documents, request_id)
                    .await?;
            let t_bytes = serde_json::to_vec_pretty(&template)
                .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
            let instr = generate_emergency_pack_instructions(&template);
            (
                "emergency_pack.json".to_string(),
                t_bytes,
                "emergency_instructions.md".to_string(),
                instr,
            )
        }
        "mhca39" => {
            let mhca39_template =
                generate_mhca39_template(pool, case_id, &manifest_documents, request_id).await?;
            let t_bytes = serde_json::to_vec_pretty(&mhca39_template)
                .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
            let instr = generate_mhca39_instructions(&mhca39_template);
            (
                "MHCA39_draft.json".to_string(),
                t_bytes,
                "MHCA39_instructions.md".to_string(),
                instr,
            )
        }
        "will_prep_sa" => {
            let template =
                generate_will_prep_template(pool, case_id, &manifest_documents, request_id).await?;
            let t_bytes = serde_json::to_vec_pretty(&template)
                .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
            let instr = generate_will_prep_instructions();
            (
                "will_prep_draft.json".to_string(),
                t_bytes,
                "witnessing_instructions.md".to_string(),
                instr,
            )
        }
        "deceased_estate_reporting_sa" => {
            let template =
                generate_deceased_estate_template(pool, case_id, &manifest_documents, request_id)
                    .await?;
            let t_bytes = serde_json::to_vec_pretty(&template)
                .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
            let estimated_value = template.estimated_estate_value_zar;
            let instr = generate_deceased_estate_instructions(estimated_value);
            (
                "deceased_estate_draft.json".to_string(),
                t_bytes,
                "instructions.md".to_string(),
                instr,
            )
        }
        "popia_incident" => {
            let template =
                generate_popia_incident_template(pool, case_id, &manifest_documents, request_id)
                    .await?;
            let t_bytes = serde_json::to_vec_pretty(&template)
                .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
            let instr = generate_popia_incident_instructions(&template);
            (
                "popia_notification_pack.json".to_string(),
                t_bytes,
                "popia_instructions.md".to_string(),
                instr,
            )
        }
        "death_readiness" => {
            let template =
                generate_death_readiness_template(pool, case_id, &manifest_documents, request_id)
                    .await?;
            let t_bytes = serde_json::to_vec_pretty(&template)
                .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
            let instr = generate_death_readiness_instructions(&template);
            (
                "death_readiness.json".to_string(),
                t_bytes,
                "instructions.md".to_string(),
                instr,
            )
        }
        _ => {
            return Err(invalid_request(
                Some(request_id),
                "unsupported case type for export",
            ));
        }
    };

    let template_path = export_dir.join(&template_filename);
    fs::write(&template_path, &template_bytes)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
    let template_sha256 = sha256_bytes(&template_bytes);

    let instructions_path = export_dir.join(&instructions_filename);
    fs::write(&instructions_path, &instructions)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
    let instructions_sha256 = sha256_bytes(instructions.as_bytes());

    let manifest = ExportManifest {
        case_id: case_id.to_string(),
        case_type: case_type.clone(),
        exported_at: Utc::now().to_rfc3339(),
        audit_head_hash: audit_head_hash.clone(),
        audit_events_sha256: audit_sha256.clone(),
        documents: manifest_documents.clone(),
    };

    let manifest_path = export_dir.join("manifest.json");
    let manifest_bytes = serde_json::to_vec(&manifest)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
    fs::write(&manifest_path, &manifest_bytes)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;
    let manifest_sha256 = sha256_bytes(&manifest_bytes);

    let checksums_path = export_dir.join("checksums.txt");
    let mut checksums = Vec::new();
    checksums.push(format!("{}  manifest.json", manifest_sha256));
    checksums.push(format!("{}  audit.jsonl", audit_sha256));
    checksums.push(format!("{}  {}", template_sha256, template_filename));
    checksums.push(format!(
        "{}  {}",
        instructions_sha256, instructions_filename
    ));
    for doc in &manifest_documents {
        checksums.push(format!("{}  {}", doc.sha256, doc.bundle_path));
    }
    checksums.sort();
    fs::write(&checksums_path, checksums.join("\n"))
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;

    let zip_path = export_dir.with_extension("zip");
    create_zip(&export_dir, &zip_path)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;

    let artifact_kind = match case_type.as_str() {
        "emergency_pack" => "emergency_pack_export",
        "mhca39" => "mhca39_export",
        "will_prep_sa" => "will_prep_export",
        "deceased_estate_reporting_sa" => "deceased_estate_export",
        "popia_incident" => "popia_notification_export",
        "death_readiness" => "death_readiness_export",
        _ => "case_export",
    };

    sqlx::query(
        "INSERT INTO case_artifacts (case_id, kind, blob_ref, sha256) VALUES ($1, $2, $3, $4)",
    )
    .bind(case_id)
    .bind(artifact_kind)
    .bind(zip_path.to_string_lossy().to_string())
    .bind(&manifest_sha256)
    .execute(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    sqlx::query("UPDATE cases SET status = 'exported' WHERE case_id = $1")
        .bind(case_id)
        .execute(pool)
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let response = ExportResponse {
        download_url: format!("file://{}", export_dir.display()),
        expires_at: Utc::now().to_rfc3339(),
        manifest_sha256,
    };

    Ok(Json(response))
}

/// MHCA39 template output structure
#[derive(Debug, Serialize, Deserialize)]
struct Mhca39Template {
    /// Case identifier
    case_id: String,
    /// Export timestamp
    exported_at: String,
    /// Subject person ID
    subject_person_id: String,
    /// Applicant person ID
    applicant_person_id: String,
    /// Relationship to subject
    relationship_to_subject: Option<String>,
    /// Notes
    notes: Option<String>,
    /// Evidence checklist with slot status
    evidence_checklist: Vec<EvidenceChecklistItem>,
    /// Disclaimer
    disclaimer: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EvidenceChecklistItem {
    slot_name: String,
    required: bool,
    attached: bool,
    document_id: Option<String>,
    document_type: Option<String>,
    title: Option<String>,
}

async fn generate_mhca39_template(
    pool: &PgPool,
    case_id: uuid::Uuid,
    manifest_documents: &[ManifestDocument],
    request_id: RequestId,
) -> Result<Mhca39Template, axum::response::Response> {
    let case_row = sqlx::query(
        "SELECT m.subject_person_id, m.applicant_person_id, m.relationship_to_subject, m.notes, m.required_evidence_slots \
         FROM mhca39_cases m WHERE m.case_id = $1",
    )
    .bind(case_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let case_row = match case_row {
        Some(row) => row,
        None => return Err(not_found(Some(request_id), "mhca39 case not found")),
    };

    let subject_person_id: uuid::Uuid = case_row
        .try_get("subject_person_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let applicant_person_id: uuid::Uuid = case_row
        .try_get("applicant_person_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let relationship_to_subject: Option<String> = case_row
        .try_get("relationship_to_subject")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let notes: Option<String> = case_row
        .try_get("notes")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let required_slots: Vec<String> = case_row
        .try_get("required_evidence_slots")
        .map_err(|error| db_error_to_response(error, request_id))?;

    let evidence_rows = sqlx::query(
        "SELECT e.slot_name, e.document_id, d.document_type, d.title \
         FROM mhca39_evidence e \
         LEFT JOIN documents d ON d.document_id = e.document_id \
         WHERE e.case_id = $1 \
         ORDER BY e.slot_name",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let mut checklist = Vec::new();
    for row in evidence_rows {
        let slot_name: String = row
            .try_get("slot_name")
            .map_err(|error| db_error_to_response(error, request_id))?;
        let document_id: Option<uuid::Uuid> = row
            .try_get("document_id")
            .map_err(|error| db_error_to_response(error, request_id))?;
        let document_type: Option<String> = row
            .try_get("document_type")
            .map_err(|error| db_error_to_response(error, request_id))?;
        let title: Option<String> = row
            .try_get("title")
            .map_err(|error| db_error_to_response(error, request_id))?;

        checklist.push(EvidenceChecklistItem {
            slot_name: slot_name.clone(),
            required: required_slots.contains(&slot_name),
            attached: document_id.is_some(),
            document_id: document_id.map(|id| id.to_string()),
            document_type,
            title,
        });
    }

    let _ = manifest_documents;

    Ok(Mhca39Template {
        case_id: case_id.to_string(),
        exported_at: Utc::now().to_rfc3339(),
        subject_person_id: subject_person_id.to_string(),
        applicant_person_id: applicant_person_id.to_string(),
        relationship_to_subject,
        notes,
        evidence_checklist: checklist,
        disclaimer: "DISCLAIMER: This document pack is generated by LifeReady SA for \
            evidentiary purposes only. It does NOT constitute a legal determination of \
            incapacity. The applicant must follow the official MHCA 39 process with the \
            Master of the High Court. LifeReady SA does not provide legal or medical advice."
            .to_string(),
    })
}

fn generate_mhca39_instructions(template: &Mhca39Template) -> String {
    let mut md = String::new();
    md.push_str("# MHCA 39 Submission Pack Instructions\n\n");
    md.push_str("## Overview\n\n");
    md.push_str("This export pack contains evidence documents for an MHCA 39 application ");
    md.push_str("to the Master of the High Court for administration appointment.\n\n");
    md.push_str(&format!("**Case ID:** `{}`\n\n", template.case_id));
    md.push_str(&format!("**Exported:** {}\n\n", template.exported_at));

    md.push_str("## Disclaimer\n\n");
    md.push_str("> ");
    md.push_str(&template.disclaimer);
    md.push_str("\n\n");

    md.push_str("## Evidence Checklist\n\n");
    md.push_str("| Slot | Required | Attached | Document |\n");
    md.push_str("|------|----------|----------|----------|\n");
    for item in &template.evidence_checklist {
        let required = if item.required { "✓" } else { "-" };
        let attached = if item.attached { "✓" } else { "✗" };
        let doc = item.title.as_deref().unwrap_or("-");
        md.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            item.slot_name, required, attached, doc
        ));
    }
    md.push('\n');

    md.push_str("## Next Steps\n\n");
    md.push_str("1. Review all documents in the `documents/` folder\n");
    md.push_str("2. Complete the official MHCA 39 form from the Master of the High Court\n");
    md.push_str(
        "3. Have the applicant swear/affirm the application before a Commissioner of Oaths\n",
    );
    md.push_str("4. Submit the application with this evidence pack to the Master's Office\n");
    md.push_str("5. Retain this pack and the `checksums.txt` for verification purposes\n\n");

    md.push_str("## Verification\n\n");
    md.push_str("Use the `audit-verifier` CLI tool to verify the integrity of this bundle:\n\n");
    md.push_str("```bash\n");
    md.push_str("audit-verifier verify-bundle --bundle <path-to-export>\n");
    md.push_str("```\n");

    md
}

/// Will prep template output structure
#[derive(Debug, Serialize, Deserialize)]
struct WillPrepTemplate {
    case_id: String,
    exported_at: String,
    principal_person_id: String,
    notes: Option<String>,
    evidence_checklist: Vec<EvidenceChecklistItem>,
    disclaimer: String,
}

async fn generate_will_prep_template(
    pool: &PgPool,
    case_id: uuid::Uuid,
    _manifest_documents: &[ManifestDocument],
    request_id: RequestId,
) -> Result<WillPrepTemplate, axum::response::Response> {
    let case_row = sqlx::query(
        "SELECT w.principal_person_id, w.notes, w.required_evidence_slots \
         FROM will_prep_cases w WHERE w.case_id = $1",
    )
    .bind(case_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let case_row = match case_row {
        Some(row) => row,
        None => return Err(not_found(Some(request_id), "will_prep_sa case not found")),
    };

    let principal_person_id: uuid::Uuid = case_row
        .try_get("principal_person_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let notes: Option<String> = case_row
        .try_get("notes")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let required_slots: Vec<String> = case_row
        .try_get("required_evidence_slots")
        .map_err(|error| db_error_to_response(error, request_id))?;

    let evidence_rows = sqlx::query(
        "SELECT e.slot_name, e.document_id, d.document_type, d.title \
         FROM case_evidence e \
         LEFT JOIN documents d ON d.document_id = e.document_id \
         WHERE e.case_id = $1 \
         ORDER BY e.slot_name",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let mut checklist = Vec::new();
    for row in evidence_rows {
        let slot_name: String = row
            .try_get("slot_name")
            .map_err(|error| db_error_to_response(error, request_id))?;
        let document_id: Option<uuid::Uuid> = row
            .try_get("document_id")
            .map_err(|error| db_error_to_response(error, request_id))?;
        let document_type: Option<String> = row
            .try_get("document_type")
            .map_err(|error| db_error_to_response(error, request_id))?;
        let title: Option<String> = row
            .try_get("title")
            .map_err(|error| db_error_to_response(error, request_id))?;

        checklist.push(EvidenceChecklistItem {
            slot_name: slot_name.clone(),
            required: required_slots.contains(&slot_name),
            attached: document_id.is_some(),
            document_id: document_id.map(|id| id.to_string()),
            document_type,
            title,
        });
    }

    Ok(WillPrepTemplate {
        case_id: case_id.to_string(),
        exported_at: Utc::now().to_rfc3339(),
        principal_person_id: principal_person_id.to_string(),
        notes,
        evidence_checklist: checklist,
        disclaimer: "DISCLAIMER: This pack is generated by LifeReady SA for preparation purposes only. \
            It does NOT constitute legal advice. Consult a qualified legal professional for will execution. \
            LifeReady SA does not provide legal advice."
            .to_string(),
    })
}

fn generate_will_prep_instructions() -> String {
    let mut md = String::new();
    md.push_str("# Will Preparation Pack — SA Witnessing Instructions\n\n");
    md.push_str("## Overview\n");
    md.push_str("This export pack contains draft will documents and supporting schedules.\n\n");
    md.push_str("## SA Witnessing Formalities\n");
    md.push_str("Under South African law, a valid will requires:\n");
    md.push_str("1. The testator must sign the will at the end of the last page, in the presence of two competent witnesses\n");
    md.push_str("2. Both witnesses must be present simultaneously when the testator signs\n");
    md.push_str(
        "3. Each witness must sign the will in the presence of the testator and of each other\n",
    );
    md.push_str("4. Every page of the will must be signed by the testator and both witnesses\n");
    md.push_str(
        "5. If the testator cannot sign, a commissioner of oaths may sign on their behalf\n\n",
    );
    md.push_str("## Important\n");
    md.push_str(
        "> DISCLAIMER: This pack is generated by LifeReady SA for preparation purposes only.\n",
    );
    md.push_str("> It does NOT constitute legal advice. Consult a qualified legal professional for will execution.\n");
    md.push_str("> LifeReady SA does not provide legal advice.\n\n");
    md.push_str("## Verification\n");
    md.push_str("Use the `audit-verifier` CLI to verify bundle integrity.\n");
    md
}

/// Deceased estate template output structure
#[derive(Debug, Serialize, Deserialize)]
struct DeceasedEstateTemplate {
    case_id: String,
    exported_at: String,
    deceased_person_id: String,
    executor_person_id: String,
    estimated_estate_value_zar: Option<f64>,
    notes: Option<String>,
    evidence_checklist: Vec<EvidenceChecklistItem>,
    disclaimer: String,
}

async fn generate_deceased_estate_template(
    pool: &PgPool,
    case_id: uuid::Uuid,
    _manifest_documents: &[ManifestDocument],
    request_id: RequestId,
) -> Result<DeceasedEstateTemplate, axum::response::Response> {
    let case_row = sqlx::query(
        "SELECT d.deceased_person_id, d.executor_person_id, d.estimated_estate_value_zar, d.notes, d.required_evidence_slots \
         FROM deceased_estate_cases d WHERE d.case_id = $1",
    )
    .bind(case_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let case_row = match case_row {
        Some(row) => row,
        None => {
            return Err(not_found(
                Some(request_id),
                "deceased_estate case not found",
            ));
        }
    };

    let deceased_person_id: uuid::Uuid = case_row
        .try_get("deceased_person_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let executor_person_id: uuid::Uuid = case_row
        .try_get("executor_person_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let estimated_estate_value_zar: Option<rust_decimal::Decimal> = case_row
        .try_get("estimated_estate_value_zar")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let notes: Option<String> = case_row
        .try_get("notes")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let required_slots: Vec<String> = case_row
        .try_get("required_evidence_slots")
        .map_err(|error| db_error_to_response(error, request_id))?;

    let evidence_rows = sqlx::query(
        "SELECT e.slot_name, e.document_id, d.document_type, d.title \
         FROM case_evidence e \
         LEFT JOIN documents d ON d.document_id = e.document_id \
         WHERE e.case_id = $1 \
         ORDER BY e.slot_name",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let mut checklist = Vec::new();
    for row in evidence_rows {
        let slot_name: String = row
            .try_get("slot_name")
            .map_err(|error| db_error_to_response(error, request_id))?;
        let document_id: Option<uuid::Uuid> = row
            .try_get("document_id")
            .map_err(|error| db_error_to_response(error, request_id))?;
        let document_type: Option<String> = row
            .try_get("document_type")
            .map_err(|error| db_error_to_response(error, request_id))?;
        let title: Option<String> = row
            .try_get("title")
            .map_err(|error| db_error_to_response(error, request_id))?;

        checklist.push(EvidenceChecklistItem {
            slot_name: slot_name.clone(),
            required: required_slots.contains(&slot_name),
            attached: document_id.is_some(),
            document_id: document_id.map(|id| id.to_string()),
            document_type,
            title,
        });
    }

    Ok(DeceasedEstateTemplate {
        case_id: case_id.to_string(),
        exported_at: Utc::now().to_rfc3339(),
        deceased_person_id: deceased_person_id.to_string(),
        executor_person_id: executor_person_id.to_string(),
        estimated_estate_value_zar: match estimated_estate_value_zar {
            Some(v) => {
                use rust_decimal::prelude::ToPrimitive;
                Some(v.to_f64().ok_or_else(|| {
                    invalid_request(Some(request_id), "estate value out of f64 range")
                })?)
            }
            None => None,
        },
        notes,
        evidence_checklist: checklist,
        disclaimer:
            "DISCLAIMER: This pack is generated by LifeReady SA for preparation purposes only. \
            It does NOT constitute legal advice or claim executor appointment. \
            LifeReady SA does not provide legal advice."
                .to_string(),
    })
}

fn generate_deceased_estate_instructions(estimated_value: Option<f64>) -> String {
    let mut md = String::new();
    md.push_str("# Deceased Estate Reporting Pack — SA Instructions\n\n");
    md.push_str("## Overview\n");
    md.push_str("This export contains documents required for reporting a deceased estate in South Africa.\n\n");
    md.push_str("## Estimated Estate Value\n");
    md.push_str("Based on the captured estimated estate value:\n");
    match estimated_value {
        Some(value) if value > 250_000.0 => {
            md.push_str("- If the estate value exceeds R250,000 or if there is a valid will: Apply for **Letters of Executorship** from the Master of the High Court\n");
        }
        Some(_) => {
            md.push_str("- If the estate value is R250,000 or below and there is no will: Apply for **Letters of Authority** from the Master of the High Court\n");
        }
        None => {
            md.push_str("- If the estate value exceeds R250,000 or if there is a valid will: Apply for **Letters of Executorship** from the Master of the High Court\n");
            md.push_str("- If the estate value is R250,000 or below and there is no will: Apply for **Letters of Authority** from the Master of the High Court\n");
        }
    }
    md.push_str("\n## Required Steps\n");
    md.push_str("1. Report the death to the Department of Home Affairs within 72 hours\n");
    md.push_str("2. Report the estate to the Master of the High Court within 14 days\n");
    md.push_str("3. Submit the required documents (see manifest)\n\n");
    md.push_str("## Important\n");
    md.push_str(
        "> DISCLAIMER: This pack is generated by LifeReady SA for preparation purposes only.\n",
    );
    md.push_str("> It does NOT constitute legal advice or claim executor appointment.\n");
    md.push_str("> LifeReady SA does not provide legal advice.\n\n");
    md.push_str("## Verification\n");
    md.push_str("Use the `audit-verifier` CLI to verify bundle integrity.\n");
    md
}

/// Emergency pack template output structure
#[derive(Debug, Serialize, Deserialize)]
struct EmergencyPackTemplate {
    /// Case identifier
    case_id: String,
    /// Export timestamp
    exported_at: String,
    /// Directive document references
    directive_documents: Vec<ManifestDocument>,
    /// Emergency contacts
    emergency_contacts: Vec<serde_json::Value>,
    /// Disclaimer
    disclaimer: String,
}

async fn generate_emergency_pack_template(
    pool: &PgPool,
    case_id: uuid::Uuid,
    manifest_documents: &[ManifestDocument],
    request_id: RequestId,
) -> Result<EmergencyPackTemplate, axum::response::Response> {
    let case_row =
        sqlx::query("SELECT emergency_contacts FROM emergency_pack_cases WHERE case_id = $1")
            .bind(case_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;

    let case_row = match case_row {
        Some(row) => row,
        None => return Err(not_found(Some(request_id), "emergency_pack case not found")),
    };

    let contacts: serde_json::Value = case_row
        .try_get("emergency_contacts")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let contacts_vec = match contacts {
        serde_json::Value::Array(arr) => arr,
        _ => Vec::new(),
    };

    Ok(EmergencyPackTemplate {
        case_id: case_id.to_string(),
        exported_at: Utc::now().to_rfc3339(),
        directive_documents: manifest_documents.to_vec(),
        emergency_contacts: contacts_vec,
        disclaimer: "DISCLAIMER: This emergency directive pack is generated by LifeReady SA \
            for preparedness purposes only. It does NOT constitute medical advice or a \
            legally binding advance directive unless properly witnessed and executed \
            under South African law. LifeReady SA does not provide legal or medical advice."
            .to_string(),
    })
}

fn generate_emergency_pack_instructions(template: &EmergencyPackTemplate) -> String {
    let mut md = String::new();
    md.push_str("# Emergency Directive Pack Instructions\n\n");
    md.push_str("## Overview\n\n");
    md.push_str("This export pack contains your advance directive documents and ");
    md.push_str("emergency contact information for rapid access.\n\n");
    md.push_str(&format!("**Case ID:** `{}`\n\n", template.case_id));
    md.push_str(&format!("**Exported:** {}\n\n", template.exported_at));

    md.push_str("## Disclaimer\n\n> ");
    md.push_str(&template.disclaimer);
    md.push_str("\n\n");

    md.push_str("## Documents Included\n\n");
    for doc in &template.directive_documents {
        md.push_str(&format!(
            "- **{}**: {} ({})\n",
            doc.slot_name, doc.title, doc.document_type
        ));
    }
    md.push('\n');

    md.push_str("## Emergency Contacts\n\n");
    for contact in &template.emergency_contacts {
        let name = contact
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let phone = contact
            .get("phone_e164")
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        md.push_str(&format!("- **{}**: {}\n", name, phone));
    }
    md.push('\n');

    md.push_str("## Sharing\n\n");
    md.push_str("Share this pack via a time-limited link. All accesses are logged.\n");
    md.push_str("Revoke the link at any time to immediately invalidate access.\n\n");

    md.push_str("## Verification\n\n");
    md.push_str("Use the `audit-verifier` CLI tool to verify the integrity of this bundle:\n\n");
    md.push_str("```bash\naudit-verifier verify-bundle --bundle <path-to-export>\n```\n");

    md
}

/// POPIA incident notification template output structure
#[derive(Debug, Serialize, Deserialize)]
struct PopiaIncidentTemplate {
    /// Case identifier
    case_id: String,
    /// Export timestamp
    exported_at: String,
    /// Incident title
    incident_title: String,
    /// Incident description
    description: Option<String>,
    /// Affected data classes (e.g., health, financial, identity)
    affected_data_classes: Vec<String>,
    /// Estimated number of affected users
    affected_user_count: Option<i32>,
    /// Mitigation steps taken
    mitigation_steps: Option<String>,
    /// When the incident was reported internally
    reported_at: String,
    /// Evidence checklist
    evidence_checklist: Vec<EvidenceChecklistItem>,
    /// Disclaimer
    disclaimer: String,
}

async fn generate_popia_incident_template(
    pool: &PgPool,
    case_id: uuid::Uuid,
    _manifest_documents: &[ManifestDocument],
    request_id: RequestId,
) -> Result<PopiaIncidentTemplate, axum::response::Response> {
    let case_row = sqlx::query(
        "SELECT p.incident_title, p.description, p.affected_data_classes, \
         p.affected_user_count, p.mitigation_steps, p.reported_at, \
         p.required_evidence_slots, p.notes \
         FROM popia_incident_cases p WHERE p.case_id = $1",
    )
    .bind(case_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let case_row = match case_row {
        Some(row) => row,
        None => return Err(not_found(Some(request_id), "popia_incident case not found")),
    };

    let incident_title: String = case_row
        .try_get("incident_title")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let description: Option<String> = case_row
        .try_get("description")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let affected_data_classes: Vec<String> = case_row
        .try_get("affected_data_classes")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let affected_user_count: Option<i32> = case_row
        .try_get("affected_user_count")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let mitigation_steps: Option<String> = case_row
        .try_get("mitigation_steps")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let reported_at: chrono::DateTime<Utc> = case_row
        .try_get("reported_at")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let required_slots: Vec<String> = case_row
        .try_get("required_evidence_slots")
        .map_err(|error| db_error_to_response(error, request_id))?;

    let evidence_rows = sqlx::query(
        "SELECT e.slot_name, e.document_id, d.document_type, d.title \
         FROM case_evidence e \
         LEFT JOIN documents d ON d.document_id = e.document_id \
         WHERE e.case_id = $1 \
         ORDER BY e.slot_name",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let mut checklist = Vec::new();
    for row in evidence_rows {
        let slot_name: String = row
            .try_get("slot_name")
            .map_err(|error| db_error_to_response(error, request_id))?;
        let document_id: Option<uuid::Uuid> = row
            .try_get("document_id")
            .map_err(|error| db_error_to_response(error, request_id))?;
        let document_type: Option<String> = row
            .try_get("document_type")
            .map_err(|error| db_error_to_response(error, request_id))?;
        let title: Option<String> = row
            .try_get("title")
            .map_err(|error| db_error_to_response(error, request_id))?;

        checklist.push(EvidenceChecklistItem {
            slot_name: slot_name.clone(),
            required: required_slots.contains(&slot_name),
            attached: document_id.is_some(),
            document_id: document_id.map(|id| id.to_string()),
            document_type,
            title,
        });
    }

    Ok(PopiaIncidentTemplate {
        case_id: case_id.to_string(),
        exported_at: Utc::now().to_rfc3339(),
        incident_title,
        description,
        affected_data_classes,
        affected_user_count,
        mitigation_steps,
        reported_at: reported_at.to_rfc3339(),
        evidence_checklist: checklist,
        disclaimer: "DISCLAIMER: This POPIA security compromise notification pack is generated \
            by LifeReady SA. It is intended to support compliance with Section 22 of the \
            Protection of Personal Information Act (POPIA). This does NOT constitute legal \
            advice. Consult a qualified legal professional for regulatory submissions. \
            LifeReady SA does not provide legal advice."
            .to_string(),
    })
}

fn generate_popia_incident_instructions(template: &PopiaIncidentTemplate) -> String {
    let mut md = String::new();
    md.push_str("# POPIA Security Compromise Notification Pack\n\n");
    md.push_str("## Overview\n\n");
    md.push_str("This export pack supports POPIA Section 22 notification obligations.\n\n");
    md.push_str(&format!("**Case ID:** `{}`\n\n", template.case_id));
    md.push_str(&format!("**Incident:** {}\n\n", template.incident_title));
    md.push_str(&format!("**Reported:** {}\n\n", template.reported_at));
    md.push_str(&format!("**Exported:** {}\n\n", template.exported_at));

    md.push_str("## Disclaimer\n\n> ");
    md.push_str(&template.disclaimer);
    md.push_str("\n\n");

    md.push_str("## Incident Details\n\n");
    if let Some(desc) = &template.description {
        md.push_str(&format!("**Description:** {}\n\n", desc));
    }
    md.push_str(&format!(
        "**Affected Data Classes:** {}\n\n",
        template.affected_data_classes.join(", ")
    ));
    if let Some(count) = template.affected_user_count {
        md.push_str(&format!("**Estimated Affected Users:** {}\n\n", count));
    }
    if let Some(steps) = &template.mitigation_steps {
        md.push_str(&format!("**Mitigation Steps:** {}\n\n", steps));
    }

    md.push_str("## Required Actions (POPIA Section 22)\n\n");
    md.push_str("1. Notify the Information Regulator as soon as reasonably possible\n");
    md.push_str("2. Notify affected data subjects if the compromise may cause harm\n");
    md.push_str("3. Document all steps taken to address the compromise\n");
    md.push_str("4. Retain this pack and audit trail for compliance evidence\n\n");

    md.push_str("## Evidence Checklist\n\n");
    md.push_str("| Slot | Required | Attached | Document |\n");
    md.push_str("|------|----------|----------|----------|\n");
    for item in &template.evidence_checklist {
        let required = if item.required { "✓" } else { "-" };
        let attached = if item.attached { "✓" } else { "✗" };
        let doc = item.title.as_deref().unwrap_or("-");
        md.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            item.slot_name, required, attached, doc
        ));
    }
    md.push('\n');

    md.push_str("## Verification\n\n");
    md.push_str("Use the `audit-verifier` CLI tool to verify the integrity of this bundle:\n\n");
    md.push_str("```bash\naudit-verifier verify-bundle --bundle <path-to-export>\n```\n");

    md
}

/// Death Readiness template output structure
#[derive(Debug, Serialize, Deserialize)]
struct DeathReadinessTemplate {
    /// Case identifier
    case_id: String,
    /// Export timestamp
    exported_at: String,
    /// Executor nominee person ID (not legally appointed yet)
    executor_nominee_person_id: String,
    /// Asset document index
    asset_documents: Vec<ManifestDocument>,
    /// Contact document index
    contact_documents: Vec<ManifestDocument>,
    /// User-provided notes
    notes: Option<String>,
}

async fn generate_death_readiness_template(
    pool: &PgPool,
    case_id: uuid::Uuid,
    manifest_documents: &[ManifestDocument],
    request_id: RequestId,
) -> Result<DeathReadinessTemplate, axum::response::Response> {
    let row = sqlx::query(
        "SELECT executor_nominee_person_id, asset_document_ids, contact_document_ids, notes \
         FROM death_readiness_cases WHERE case_id = $1",
    )
    .bind(case_id)
    .fetch_one(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let executor_nominee_id: uuid::Uuid = row
        .try_get("executor_nominee_person_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let asset_ids: Vec<uuid::Uuid> = row
        .try_get("asset_document_ids")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let contact_ids: Vec<uuid::Uuid> = row
        .try_get("contact_document_ids")
        .map_err(|error| db_error_to_response(error, request_id))?;
    let notes: Option<String> = row
        .try_get("notes")
        .map_err(|error| db_error_to_response(error, request_id))?;

    let asset_set: std::collections::HashSet<String> =
        asset_ids.iter().map(|id| id.to_string()).collect();
    let contact_set: std::collections::HashSet<String> =
        contact_ids.iter().map(|id| id.to_string()).collect();

    let asset_documents: Vec<ManifestDocument> = manifest_documents
        .iter()
        .filter(|d| asset_set.contains(&d.document_id))
        .cloned()
        .collect();
    let contact_documents: Vec<ManifestDocument> = manifest_documents
        .iter()
        .filter(|d| contact_set.contains(&d.document_id))
        .cloned()
        .collect();

    Ok(DeathReadinessTemplate {
        case_id: case_id.to_string(),
        exported_at: Utc::now().to_rfc3339(),
        executor_nominee_person_id: executor_nominee_id.to_string(),
        asset_documents,
        contact_documents,
        notes,
    })
}

fn generate_death_readiness_instructions(template: &DeathReadinessTemplate) -> String {
    let mut md = String::new();
    md.push_str("# Death Readiness Pack — Instructions\n\n");
    md.push_str("**Jurisdiction:** South Africa\n\n");
    md.push_str("> **IMPORTANT:** This pack does not constitute legal advice.\n");
    md.push_str("> The \"Executor nominee\" referenced below is a nominated person who has NOT yet been legally appointed by the Master of the High Court.\n\n");
    md.push_str("## Executor Nominee\n\n");
    md.push_str(&format!(
        "- Person ID: `{}`\n\n",
        template.executor_nominee_person_id
    ));
    md.push_str("## Asset Map\n\n");
    if template.asset_documents.is_empty() {
        md.push_str("No asset documents attached.\n\n");
    } else {
        for doc in &template.asset_documents {
            md.push_str(&format!(
                "- {} ({}): `{}`\n",
                doc.title, doc.document_type, doc.document_id
            ));
        }
        md.push('\n');
    }
    md.push_str("## Contacts\n\n");
    if template.contact_documents.is_empty() {
        md.push_str("No contact documents attached.\n\n");
    } else {
        for doc in &template.contact_documents {
            md.push_str(&format!(
                "- {} ({}): `{}`\n",
                doc.title, doc.document_type, doc.document_id
            ));
        }
        md.push('\n');
    }
    md.push_str("## No Credential Release\n\n");
    md.push_str("v0.1 does not release any high-risk secrets or credentials.\n\n");
    md.push_str("## Verification\n\n");
    md.push_str("Verify bundle integrity using:\n\n");
    md.push_str("```\naudit-verifier verify-bundle --bundle <export-dir>\n```\n");
    md
}

pub fn addr_from_env(default_port: u16) -> SocketAddr {
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("CASE_PORT")
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

fn pool_from_env() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect_lazy(&database_url).ok()
}

fn export_dir_from_env() -> PathBuf {
    std::env::var("LOCAL_EXPORT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("exports").join("cases"))
}

fn storage_dir_from_env() -> PathBuf {
    std::env::var("LOCAL_STORAGE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("storage"))
}

fn parse_uuid(value: &str) -> Option<uuid::Uuid> {
    uuid::Uuid::from_str(value).ok()
}

fn default_mhca39_slots() -> Vec<String> {
    vec![
        "medical_certificate_1".into(),
        "medical_certificate_2".into(),
        "assets_income_schedule".into(),
        "applicant_id_copy".into(),
        "patient_id_copy".into(),
        "supporting_affidavit".into(),
        "mhca39_form_data".into(),
    ]
}

fn default_will_prep_slots() -> Vec<String> {
    vec![
        "draft_will_document".into(),
        "asset_schedule".into(),
        "beneficiary_schedule".into(),
        "executor_nomination".into(),
        "witness_instruction_ack".into(),
    ]
}

fn default_deceased_estate_slots() -> Vec<String> {
    vec![
        "death_certificate".into(),
        "id_of_deceased".into(),
        "id_of_executor".into(),
        "original_will".into(),
        "inventory_assets_liabilities".into(),
        "nomination_acceptance".into(),
        "proof_of_address_executor".into(),
    ]
}

fn default_popia_incident_slots() -> Vec<String> {
    vec![
        "incident_report".into(),
        "affected_data_summary".into(),
        "mitigation_evidence".into(),
        "regulator_notification_draft".into(),
        "data_subject_notification_draft".into(),
    ]
}

fn resolve_blob_ref(blob_ref: &str, storage_dir: &std::path::Path) -> Option<PathBuf> {
    let resolved = if let Some(path) = blob_ref.strip_prefix("file://") {
        PathBuf::from(path)
    } else if blob_ref.starts_with('/') {
        PathBuf::from(blob_ref)
    } else {
        storage_dir.join(blob_ref)
    };

    // Prevent path traversal: resolved path must be within storage_dir.
    if let (Ok(canonical_storage), Ok(canonical_resolved)) =
        (storage_dir.canonicalize(), resolved.canonicalize())
        && !canonical_resolved.starts_with(&canonical_storage)
    {
        tracing::warn!(
            blob_ref = blob_ref,
            "resolve_blob_ref rejected: path escapes storage directory"
        );
        return None;
    }

    Some(resolved)
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn sha256_file(path: &std::path::Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    Ok(sha256_bytes(&bytes))
}

fn create_zip(
    source_dir: &std::path::Path,
    zip_path: &std::path::Path,
) -> Result<(), std::io::Error> {
    let file = fs::File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in walkdir::WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let relative = path
            .strip_prefix(source_dir)
            .map_err(std::io::Error::other)?;

        if relative.as_os_str().is_empty() {
            continue;
        }

        let name = relative.to_string_lossy().replace('\\', "/");
        if path.is_dir() {
            zip.add_directory(&name, options)
                .map_err(std::io::Error::other)?;
        } else {
            zip.start_file(&name, options)
                .map_err(std::io::Error::other)?;
            let data = fs::read(path)?;
            zip.write_all(&data)?;
        }
    }

    zip.finish().map_err(std::io::Error::other)?;
    Ok(())
}

async fn ensure_case_access(
    pool: &PgPool,
    case_id: uuid::Uuid,
    principal_id: uuid::Uuid,
    request_id: RequestId,
) -> Result<(), axum::response::Response> {
    let row = sqlx::query("SELECT principal_id FROM cases WHERE case_id = $1")
        .bind(case_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let row = match row {
        Some(row) => row,
        None => return Err(not_found(Some(request_id), "case not found")),
    };

    let owner: uuid::Uuid = row
        .try_get("principal_id")
        .map_err(|error| db_error_to_response(error, request_id))?;
    if owner != principal_id {
        return Err(not_found(Some(request_id), "case not found"));
    }

    Ok(())
}

async fn fetch_case_type(
    pool: &PgPool,
    case_id: uuid::Uuid,
    request_id: RequestId,
) -> Result<String, axum::response::Response> {
    let row = sqlx::query("SELECT case_type::text FROM cases WHERE case_id = $1")
        .bind(case_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| db_error_to_response(error, request_id))?;

    let row = match row {
        Some(row) => row,
        None => return Err(not_found(Some(request_id), "case not found")),
    };

    row.try_get::<String, _>("case_type")
        .map_err(|error| db_error_to_response(error, request_id))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AuditAppend {
    actor_principal_id: String,
    action: String,
    tier: String,
    case_id: Option<String>,
    payload: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AuditEventLine {
    event_id: String,
    created_at: String,
    prev_hash: String,
    event_hash: String,
    event: AuditAppend,
}

async fn fetch_audit_events(
    pool: &PgPool,
) -> Result<Vec<AuditEventLine>, axum::response::Response> {
    let rows = sqlx::query(
        "SELECT event_id, created_at, actor_principal_id, action, tier, case_id, payload, prev_hash, event_hash \
         FROM audit_events ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| invalid_request(None, error.to_string()))?;

    let mut events = Vec::new();
    for row in rows {
        let event = AuditEventLine {
            event_id: row
                .try_get::<uuid::Uuid, _>("event_id")
                .map_err(|error| invalid_request(None, error.to_string()))?
                .to_string(),
            created_at: row
                .try_get::<chrono::DateTime<Utc>, _>("created_at")
                .map_err(|error| invalid_request(None, error.to_string()))?
                .to_rfc3339(),
            prev_hash: row
                .try_get::<String, _>("prev_hash")
                .map_err(|error| invalid_request(None, error.to_string()))?,
            event_hash: row
                .try_get::<String, _>("event_hash")
                .map_err(|error| invalid_request(None, error.to_string()))?,
            event: AuditAppend {
                actor_principal_id: row
                    .try_get::<uuid::Uuid, _>("actor_principal_id")
                    .map_err(|error| invalid_request(None, error.to_string()))?
                    .to_string(),
                action: row
                    .try_get::<String, _>("action")
                    .map_err(|error| invalid_request(None, error.to_string()))?,
                tier: row
                    .try_get::<String, _>("tier")
                    .map_err(|error| invalid_request(None, error.to_string()))?,
                case_id: row
                    .try_get::<Option<uuid::Uuid>, _>("case_id")
                    .map_err(|error| invalid_request(None, error.to_string()))?
                    .map(|id| id.to_string()),
                payload: row
                    .try_get::<Value, _>("payload")
                    .map_err(|error| invalid_request(None, error.to_string()))?,
            },
        };
        events.push(event);
    }

    Ok(events)
}

fn write_audit_jsonl(path: &PathBuf, events: &[AuditEventLine]) -> Result<(), std::io::Error> {
    let mut lines = Vec::new();
    for event in events {
        let line = serde_json::to_string(event).unwrap_or_default();
        lines.push(line);
    }
    fs::write(path, lines.join("\n"))
}

fn db_error_to_response(error: sqlx::Error, request_id: RequestId) -> axum::response::Response {
    if let sqlx::Error::Database(db_error) = &error {
        if db_error.code().as_deref() == Some("23505") {
            return conflict(Some(request_id), "duplicate record");
        }
        tracing::warn!(
            request_id = %request_id.0,
            error = %db_error.message(),
            "database error"
        );
        return invalid_request(Some(request_id), "database operation failed");
    }
    tracing::warn!(
        request_id = %request_id.0,
        error = %error,
        "database error"
    );
    invalid_request(Some(request_id), "database operation failed")
}

#[cfg(test)]
#[allow(clippy::await_holding_lock)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use lifeready_auth::{AccessLevel, AuthConfig, Claims, Role, SensitivityTier};
    use std::future::Future;
    use std::net::SocketAddr;
    use std::sync::Mutex;
    use tower::util::ServiceExt;
    use uuid::Uuid;

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
    fn parse_uuid_accepts_valid() {
        let value = uuid::Uuid::new_v4().to_string();
        assert!(parse_uuid(&value).is_some());
        assert!(parse_uuid("not-a-uuid").is_none());
    }

    #[test]
    fn default_mhca39_slots_contains_expected_items() {
        let slots = default_mhca39_slots();
        assert!(slots.contains(&"medical_certificate_1".to_string()));
        assert!(slots.contains(&"mhca39_form_data".to_string()));
        assert!(slots.len() >= 7);
    }

    #[test]
    fn resolve_blob_ref_handles_prefixes() {
        let base = std::env::temp_dir().join(format!("resolve-blob-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();

        // Relative path inside storage_dir
        let inner_file = base.join("blob.bin");
        std::fs::write(&inner_file, b"").unwrap();
        let relative = resolve_blob_ref("blob.bin", &base).unwrap();
        assert_eq!(relative, base.join("blob.bin"));

        // file:// inside storage_dir
        let file_inside =
            resolve_blob_ref(&format!("file://{}", inner_file.display()), &base).unwrap();
        assert_eq!(file_inside, inner_file);

        // Absolute path outside storage_dir should be rejected
        let outside = resolve_blob_ref("/etc/passwd", &base);
        assert!(
            outside.is_none(),
            "path outside storage_dir must be rejected"
        );

        // file:// path outside storage_dir should be rejected
        let file_outside = resolve_blob_ref("file:///etc/passwd", &base);
        assert!(
            file_outside.is_none(),
            "file:// outside storage_dir must be rejected"
        );

        // Traversal via ../ should be rejected when target escapes
        let traversal = resolve_blob_ref("../../../etc/passwd", &base);
        assert!(
            traversal.is_none(),
            "traversal outside storage_dir must be rejected"
        );
    }

    #[test]
    fn sha256_helpers_work() {
        let digest = sha256_bytes(b"hello");
        assert_eq!(digest.len(), 64);

        let dir = std::env::temp_dir().join(format!("case-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("file.txt");
        std::fs::write(&path, b"hello").unwrap();
        let file_digest = sha256_file(&path).unwrap();
        assert_eq!(file_digest, digest);
    }

    #[test]
    fn zero_hash_is_64_chars() {
        let value = zero_hash();
        assert_eq!(value.len(), 64);
        assert!(value.chars().all(|c| c == '0'));
    }

    #[test]
    fn env_dirs_use_defaults_when_unset() {
        with_env(
            &[("LOCAL_EXPORT_DIR", None), ("LOCAL_STORAGE_DIR", None)],
            || {
                assert_eq!(
                    export_dir_from_env(),
                    PathBuf::from("exports").join("cases")
                );
                assert_eq!(storage_dir_from_env(), PathBuf::from("storage"));
            },
        );
    }

    #[test]
    fn env_dirs_honor_overrides() {
        with_env(
            &[
                ("LOCAL_EXPORT_DIR", Some("custom-exports")),
                ("LOCAL_STORAGE_DIR", Some("custom-storage")),
            ],
            || {
                assert_eq!(export_dir_from_env(), PathBuf::from("custom-exports"));
                assert_eq!(storage_dir_from_env(), PathBuf::from("custom-storage"));
            },
        );
    }

    #[test]
    fn write_audit_jsonl_writes_events() {
        let dir = std::env::temp_dir().join(format!("case-audit-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("audit.jsonl");

        let events = vec![AuditEventLine {
            event_id: Uuid::new_v4().to_string(),
            created_at: Utc::now().to_rfc3339(),
            prev_hash: zero_hash(),
            event_hash: zero_hash(),
            event: AuditAppend {
                actor_principal_id: Uuid::new_v4().to_string(),
                action: "case.export".into(),
                tier: "amber".into(),
                case_id: Some(Uuid::new_v4().to_string()),
                payload: serde_json::json!({"ok": true}),
            },
        }];

        write_audit_jsonl(&path, &events).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("case.export"));
    }

    #[test]
    fn db_error_to_response_returns_bad_request() {
        let response = db_error_to_response(sqlx::Error::RowNotFound, RequestId(Uuid::new_v4()));
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn addr_from_env_prefers_case_port_then_port_then_default() {
        with_env(
            &[
                ("HOST", Some("127.0.0.1")),
                ("CASE_PORT", Some("6010")),
                ("PORT", Some("7010")),
            ],
            || {
                let addr = addr_from_env(8084);
                assert_eq!(addr, "127.0.0.1:6010".parse::<SocketAddr>().unwrap());
            },
        );

        with_env(
            &[
                ("HOST", Some("0.0.0.0")),
                ("CASE_PORT", None),
                ("PORT", Some("7010")),
            ],
            || {
                let addr = addr_from_env(8084);
                assert_eq!(addr, "0.0.0.0:7010".parse::<SocketAddr>().unwrap());
            },
        );

        with_env(
            &[
                ("HOST", Some("0.0.0.0")),
                ("CASE_PORT", None),
                ("PORT", None),
            ],
            || {
                let addr = addr_from_env(8084);
                assert_eq!(addr, "0.0.0.0:8084".parse::<SocketAddr>().unwrap());
            },
        );
    }

    #[tokio::test]
    async fn check_db_returns_none_without_database_url() {
        with_env_async(&[("DATABASE_URL", None)], || async {
            assert!(check_db().await.is_none());
        })
        .await;
    }

    fn auth_token(access: AccessLevel) -> String {
        let config = AuthConfig::new("test-secret-32-chars-minimum!!");
        let claims = Claims::new(
            "00000000-0000-0000-0000-000000000001",
            Role::Principal,
            vec![SensitivityTier::Amber],
            access,
            None,
            300,
        );
        config.issue_token(&claims).expect("token")
    }

    fn auth_token_for_principal(
        principal_id: &str,
        role: Role,
        tiers: Vec<SensitivityTier>,
        access: AccessLevel,
    ) -> String {
        let config = AuthConfig::new("test-secret-32-chars-minimum!!");
        let claims = Claims::new(principal_id, role, tiers, access, None, 300);
        config.issue_token(&claims).expect("token")
    }

    fn auth_token_with(role: Role, tiers: Vec<SensitivityTier>, access: AccessLevel) -> String {
        auth_token_for_principal("00000000-0000-0000-0000-000000000001", role, tiers, access)
    }

    #[tokio::test]
    async fn emergency_pack_returns_bad_request_without_database_pool() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                ("DATABASE_URL", None),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "directive_document_ids": [],
                    "emergency_contacts": []
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/emergency-pack")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn create_mhca39_rejects_invalid_subject_person_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "subject_person_id": "not-a-uuid",
                    "applicant_person_id": "00000000-0000-0000-0000-000000000002"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/mhca39")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn create_mhca39_rejects_invalid_applicant_person_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "subject_person_id": "00000000-0000-0000-0000-000000000003",
                    "applicant_person_id": "not-a-uuid"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/mhca39")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn create_mhca39_rejects_invalid_principal_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "subject_person_id": "00000000-0000-0000-0000-000000000003",
                    "applicant_person_id": "00000000-0000-0000-0000-000000000004"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/mhca39")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_for_principal(
                                        "not-a-uuid",
                                        Role::Principal,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn create_mhca39_rejects_insufficient_role() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "subject_person_id": "00000000-0000-0000-0000-000000000003",
                    "applicant_person_id": "00000000-0000-0000-0000-000000000004"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/mhca39")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::EmergencyContact,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn create_mhca39_rejects_insufficient_tier() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "subject_person_id": "00000000-0000-0000-0000-000000000003",
                    "applicant_person_id": "00000000-0000-0000-0000-000000000004"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/mhca39")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::Principal,
                                        vec![SensitivityTier::Green],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn create_mhca39_rejects_missing_scope() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "subject_person_id": "00000000-0000-0000-0000-000000000003",
                    "applicant_person_id": "00000000-0000-0000-0000-000000000004"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/mhca39")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::Principal,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::ReadOnlyAll,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn attach_evidence_rejects_invalid_case_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "document_id": "00000000-0000-0000-0000-000000000003"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("PUT")
                            .uri("/v1/cases/not-a-uuid/evidence/slot")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn attach_evidence_rejects_invalid_principal_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "document_id": "00000000-0000-0000-0000-000000000003"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("PUT")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000002/evidence/slot")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_for_principal(
                                        "not-a-uuid",
                                        Role::Principal,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn attach_evidence_rejects_invalid_document_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "document_id": "not-a-uuid"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("PUT")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000002/evidence/slot")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn export_case_rejects_invalid_case_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/not-a-uuid/export")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::ReadOnlyPacks)),
                            )
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn emergency_pack_rejects_insufficient_role() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "directive_document_ids": [],
                    "emergency_contacts": []
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/emergency-pack")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::EmergencyContact,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn emergency_pack_rejects_insufficient_tier() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "directive_document_ids": [],
                    "emergency_contacts": []
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/emergency-pack")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::Principal,
                                        vec![SensitivityTier::Green],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn emergency_pack_rejects_read_only_scope() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "directive_document_ids": [],
                    "emergency_contacts": []
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/emergency-pack")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::Principal,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::ReadOnlyAll,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn attach_evidence_rejects_insufficient_role() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "document_id": "00000000-0000-0000-0000-000000000003"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("PUT")
                            .uri("/v1/cases/not-a-uuid/evidence/slot")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::EmergencyContact,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn export_case_rejects_missing_scope() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/not-a-uuid/export")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::Principal,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn export_case_rejects_insufficient_role() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000002/export")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::EmergencyContact,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::ReadOnlyPacks,
                                    )
                                ),
                            )
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn export_case_rejects_insufficient_tier() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000002/export")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::Principal,
                                        vec![SensitivityTier::Green],
                                        AccessLevel::ReadOnlyPacks,
                                    )
                                ),
                            )
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn export_case_rejects_invalid_principal_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000002/export")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_for_principal(
                                        "not-a-uuid",
                                        Role::Principal,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::ReadOnlyPacks,
                                    )
                                ),
                            )
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[test]
    fn default_will_prep_slots_contains_expected_items() {
        let slots = default_will_prep_slots();
        assert!(slots.contains(&"draft_will_document".to_string()));
        assert!(slots.contains(&"executor_nomination".to_string()));
        assert!(slots.contains(&"witness_instruction_ack".to_string()));
        assert_eq!(slots.len(), 5);
    }

    #[test]
    fn default_deceased_estate_slots_contains_expected_items() {
        let slots = default_deceased_estate_slots();
        assert!(slots.contains(&"death_certificate".to_string()));
        assert!(slots.contains(&"id_of_deceased".to_string()));
        assert!(slots.contains(&"original_will".to_string()));
        assert!(slots.contains(&"nomination_acceptance".to_string()));
        assert_eq!(slots.len(), 7);
    }

    #[tokio::test]
    async fn create_will_prep_sa_rejects_invalid_principal_person_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "principal_person_id": "not-a-uuid"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/will-prep-sa")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn create_will_prep_sa_rejects_insufficient_role() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "principal_person_id": "00000000-0000-0000-0000-000000000003"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/will-prep-sa")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::EmergencyContact,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn create_deceased_estate_sa_rejects_invalid_deceased_person_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "deceased_person_id": "not-a-uuid",
                    "executor_person_id": "00000000-0000-0000-0000-000000000002"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/deceased-estate-sa")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn create_deceased_estate_sa_rejects_insufficient_role() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "deceased_person_id": "00000000-0000-0000-0000-000000000003",
                    "executor_person_id": "00000000-0000-0000-0000-000000000004"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/deceased-estate-sa")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::EmergencyContact,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    // === Phase 3: State machine transition tests ===

    #[test]
    fn allowed_transitions_emergency_pack_draft_to_ready() {
        let transitions = allowed_transitions("emergency_pack", "draft");
        assert_eq!(transitions, &["ready"]);
    }

    #[test]
    fn allowed_transitions_emergency_pack_ready_to_link_issued() {
        let transitions = allowed_transitions("emergency_pack", "ready");
        assert_eq!(transitions, &["link_issued"]);
    }

    #[test]
    fn allowed_transitions_emergency_pack_link_issued() {
        let transitions = allowed_transitions("emergency_pack", "link_issued");
        assert!(transitions.contains(&"accessed"));
        assert!(transitions.contains(&"revoked"));
        assert!(transitions.contains(&"expired"));
        assert_eq!(transitions.len(), 3);
    }

    #[test]
    fn allowed_transitions_mhca39_full_workflow() {
        assert_eq!(
            allowed_transitions("mhca39", "blocked"),
            &["evidence_collecting"]
        );
        assert!(allowed_transitions("mhca39", "evidence_collecting").contains(&"draft_generated"));
        assert!(allowed_transitions("mhca39", "evidence_collecting").contains(&"blocked"));
        assert_eq!(
            allowed_transitions("mhca39", "draft_generated"),
            &["awaiting_oath"]
        );
        assert_eq!(
            allowed_transitions("mhca39", "awaiting_oath"),
            &["exported"]
        );
        assert_eq!(allowed_transitions("mhca39", "exported"), &["closed"]);
    }

    #[test]
    fn allowed_transitions_will_prep_workflow() {
        assert_eq!(allowed_transitions("will_prep_sa", "blocked"), &["ready"]);
        assert_eq!(allowed_transitions("will_prep_sa", "ready"), &["exported"]);
        assert!(allowed_transitions("will_prep_sa", "exported").contains(&"accessed"));
        assert!(allowed_transitions("will_prep_sa", "exported").contains(&"revoked"));
    }

    #[test]
    fn allowed_transitions_deceased_estate_workflow() {
        assert_eq!(
            allowed_transitions("deceased_estate_reporting_sa", "blocked"),
            &["ready"]
        );
        assert_eq!(
            allowed_transitions("deceased_estate_reporting_sa", "ready"),
            &["exported"]
        );
        assert!(
            allowed_transitions("deceased_estate_reporting_sa", "exported").contains(&"accessed")
        );
        assert!(
            allowed_transitions("deceased_estate_reporting_sa", "exported").contains(&"revoked")
        );
    }

    #[test]
    fn allowed_transitions_popia_incident_workflow() {
        assert_eq!(allowed_transitions("popia_incident", "draft"), &["ready"]);
        assert_eq!(
            allowed_transitions("popia_incident", "ready"),
            &["exported"]
        );
        assert_eq!(
            allowed_transitions("popia_incident", "exported"),
            &["closed"]
        );
    }

    #[test]
    fn allowed_transitions_invalid_returns_empty() {
        assert!(allowed_transitions("unknown_type", "draft").is_empty());
        assert!(allowed_transitions("mhca39", "closed").is_empty());
        assert!(allowed_transitions("emergency_pack", "exported").is_empty());
    }

    // === POPIA incident tests ===

    #[test]
    fn default_popia_incident_slots_contains_expected_items() {
        let slots = default_popia_incident_slots();
        assert!(slots.contains(&"incident_report".to_string()));
        assert!(slots.contains(&"regulator_notification_draft".to_string()));
        assert!(slots.contains(&"data_subject_notification_draft".to_string()));
        assert_eq!(slots.len(), 5);
    }

    #[tokio::test]
    async fn create_popia_incident_rejects_insufficient_role() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "incident_title": "Test Incident",
                    "affected_data_classes": ["health"]
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/popia-incident")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::EmergencyContact,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn create_popia_incident_rejects_insufficient_tier() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "incident_title": "Test Incident",
                    "affected_data_classes": ["health"]
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/popia-incident")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::Principal,
                                        vec![SensitivityTier::Green],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    // === State transition endpoint tests ===

    #[tokio::test]
    async fn transition_case_rejects_invalid_case_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "to_status": "ready"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/not-a-uuid/transition")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn transition_case_rejects_insufficient_role() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "to_status": "ready"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000002/transition")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::EmergencyContact,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn transition_case_rejects_invalid_principal_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "to_status": "ready"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000002/transition")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_for_principal(
                                        "not-a-uuid",
                                        Role::Principal,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    // === Emergency pack template tests ===

    #[test]
    fn generate_emergency_pack_instructions_includes_contacts() {
        let template = EmergencyPackTemplate {
            case_id: Uuid::new_v4().to_string(),
            exported_at: Utc::now().to_rfc3339(),
            directive_documents: vec![],
            emergency_contacts: vec![serde_json::json!({
                "name": "Jane Doe",
                "phone_e164": "+27821234567"
            })],
            disclaimer: "Test disclaimer".into(),
        };
        let instructions = generate_emergency_pack_instructions(&template);
        assert!(instructions.contains("Emergency Directive Pack Instructions"));
        assert!(instructions.contains("Jane Doe"));
        assert!(instructions.contains("+27821234567"));
        assert!(instructions.contains("audit-verifier"));
    }

    // === POPIA incident template tests ===

    #[test]
    fn generate_popia_incident_instructions_includes_section22() {
        let template = PopiaIncidentTemplate {
            case_id: Uuid::new_v4().to_string(),
            exported_at: Utc::now().to_rfc3339(),
            incident_title: "Data Breach Q3".into(),
            description: Some("Unauthorized access to records".into()),
            affected_data_classes: vec!["health".into(), "identity".into()],
            affected_user_count: Some(42),
            mitigation_steps: Some("Revoked access tokens".into()),
            reported_at: Utc::now().to_rfc3339(),
            evidence_checklist: vec![],
            disclaimer: "Test disclaimer".into(),
        };
        let instructions = generate_popia_incident_instructions(&template);
        assert!(instructions.contains("POPIA Security Compromise"));
        assert!(instructions.contains("Section 22"));
        assert!(instructions.contains("Data Breach Q3"));
        assert!(instructions.contains("health, identity"));
        assert!(instructions.contains("42"));
        assert!(instructions.contains("Revoked access tokens"));
        assert!(instructions.contains("Information Regulator"));
    }

    // === Death readiness tests ===

    #[tokio::test]
    async fn death_readiness_returns_bad_request_without_database_pool() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                ("DATABASE_URL", None),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "executor_nominee_person_id": "00000000-0000-0000-0000-000000000002"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/death-readiness")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn death_readiness_rejects_insufficient_role() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "executor_nominee_person_id": "00000000-0000-0000-0000-000000000002"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/death-readiness")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::EmergencyContact,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn death_readiness_rejects_invalid_executor_nominee_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "executor_nominee_person_id": "not-a-uuid"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/death-readiness")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    // === Case update (PATCH) tests ===

    #[tokio::test]
    async fn update_case_returns_bad_request_without_database_pool() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                ("DATABASE_URL", None),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "summary": "Updated incident summary"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("PATCH")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000001")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn update_case_rejects_insufficient_role() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "summary": "Updated incident"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("PATCH")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000001")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::EmergencyContact,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    // === Link/revoke tests ===

    #[tokio::test]
    async fn link_case_returns_bad_request_without_database_pool() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                ("DATABASE_URL", None),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "expires_in_hours": 24
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000001/link")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn link_case_rejects_insufficient_role() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "expires_in_hours": 24
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000001/link")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::EmergencyContact,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn revoke_case_returns_bad_request_without_database_pool() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                ("DATABASE_URL", None),
            ],
            || async {
                let app = router();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000001/revoke")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn revoke_case_rejects_insufficient_role() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/00000000-0000-0000-0000-000000000001/revoke")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::EmergencyContact,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    // === Death readiness template + instructions tests ===

    #[test]
    fn generate_death_readiness_instructions_includes_executor_nominee() {
        let template = DeathReadinessTemplate {
            case_id: Uuid::new_v4().to_string(),
            exported_at: Utc::now().to_rfc3339(),
            executor_nominee_person_id: "00000000-0000-0000-0000-000000000099".into(),
            asset_documents: vec![],
            contact_documents: vec![],
            notes: Some("Test notes".into()),
        };
        let instructions = generate_death_readiness_instructions(&template);
        assert!(instructions.contains("Death Readiness Pack"));
        assert!(instructions.contains("Executor nominee"));
        assert!(instructions.contains("South Africa"));
        assert!(instructions.contains("does not constitute legal advice"));
        assert!(instructions.contains("00000000-0000-0000-0000-000000000099"));
        assert!(instructions.contains("No Credential Release"));
        assert!(instructions.contains("audit-verifier"));
    }

    // === State machine transition tests for new types ===

    #[test]
    fn allowed_transitions_death_readiness_full_workflow() {
        assert_eq!(allowed_transitions("death_readiness", "draft"), &["ready"]);
        assert_eq!(
            allowed_transitions("death_readiness", "ready"),
            &["exported"]
        );
        assert_eq!(
            allowed_transitions("death_readiness", "exported"),
            &["closed"]
        );
    }

    // === Tier gating / IDOR negative tests ===

    #[tokio::test]
    async fn death_readiness_rejects_green_tier() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "executor_nominee_person_id": "00000000-0000-0000-0000-000000000002"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/cases/death-readiness")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        Role::Principal,
                                        vec![SensitivityTier::Green],
                                        AccessLevel::LimitedWrite,
                                    )
                                ),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn update_case_rejects_invalid_case_id() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                (
                    "DATABASE_URL",
                    Some("postgres://postgres:postgres@127.0.0.1:5432/lifeready"),
                ),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "summary": "test"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("PATCH")
                            .uri("/v1/cases/not-a-uuid")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::LimitedWrite)),
                            )
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            },
        )
        .await;
    }
}
