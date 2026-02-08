use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use chrono::Utc;
use lifeready_auth::{
    conflict, invalid_request, not_found, request_id_middleware, AuthConfig, AuthLayer,
    RequestContext, RequestId,
};
use lifeready_policy::{
    require_role, require_scope, require_scope_any, require_tier, Role, SensitivityTier,
    TierRequirement,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use std::{path::PathBuf, str::FromStr};

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
    let auth_config = Arc::new(AuthConfig::from_env());

    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/cases/emergency-pack", post(create_emergency_pack))
        .route("/v1/cases/mhca39", post(create_mhca39))
        .route("/v1/cases/{case_id}/export", post(export_case))
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Serialize)]
struct ExportManifest {
    case_id: String,
    case_type: String,
    exported_at: String,
    audit_head_hash: String,
    audit_events_sha256: String,
    documents: Vec<ManifestDocument>,
}

#[derive(Debug, Serialize, Clone)]
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

    let row = sqlx::query(
        "INSERT INTO cases (principal_id, case_type, status, blocked_reasons) \
         VALUES ($1, 'emergency_pack', 'draft', ARRAY[]::text[]) \
         RETURNING case_id, created_at, status, blocked_reasons",
    )
    .bind(principal_id)
    .fetch_one(pool)
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

    let response = CaseResponse {
        case_id: case_id.to_string(),
        case_type: "emergency_pack".into(),
        status,
        created_at: created_at.to_rfc3339(),
        blocked_reasons,
    };

    let _ = payload;

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

    let row = sqlx::query(
        "UPDATE mhca39_evidence SET document_id = $1, added_at = now() \
         WHERE case_id = $2 AND slot_name = $3 \
         RETURNING slot_name, document_id, added_at",
    )
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

    let required_slots =
        sqlx::query("SELECT required_evidence_slots FROM mhca39_cases WHERE case_id = $1")
            .bind(case_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?;

    let required_slots: Vec<String> = match required_slots {
        Some(row) => row
            .try_get("required_evidence_slots")
            .map_err(|error| db_error_to_response(error, request_id))?,
        None => return Err(not_found(Some(request_id), "mhca39 case not found")),
    };

    let missing_slots = sqlx::query(
        "SELECT slot_name FROM mhca39_evidence WHERE case_id = $1 AND document_id IS NULL",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;
    if !missing_slots.is_empty() {
        return Err(conflict(Some(request_id), "evidence slots incomplete"));
    }

    let rows = sqlx::query(
        "SELECT e.slot_name, e.document_id, d.document_type, d.title, v.sha256, v.blob_ref \
         FROM mhca39_evidence e \
         JOIN documents d ON d.document_id = e.document_id \
         JOIN LATERAL ( \
            SELECT sha256, blob_ref FROM document_versions \
            WHERE document_id = e.document_id ORDER BY created_at DESC LIMIT 1 \
         ) v ON true \
         WHERE e.case_id = $1 ORDER BY e.slot_name",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    if rows.len() != required_slots.len() {
        return Err(conflict(Some(request_id), "evidence versions missing"));
    }

    let export_dir = state
        .export_dir
        .join(case_id.to_string())
        .join(Utc::now().format("%Y%m%dT%H%M%SZ").to_string());
    let documents_dir = export_dir.join("documents");
    fs::create_dir_all(&documents_dir)
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;

    let mut manifest_documents = Vec::new();
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

    let manifest = ExportManifest {
        case_id: case_id.to_string(),
        case_type: "mhca39".into(),
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
    for doc in &manifest_documents {
        checksums.push(format!("{}  {}", doc.sha256, doc.bundle_path));
    }
    checksums.sort();
    fs::write(&checksums_path, checksums.join("\n"))
        .map_err(|error| invalid_request(Some(request_id), error.to_string()))?;

    sqlx::query(
        "INSERT INTO case_artifacts (case_id, kind, blob_ref, sha256) VALUES ($1, $2, $3, $4)",
    )
    .bind(case_id)
    .bind("mhca39_export")
    .bind(manifest_path.to_string_lossy().to_string())
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
        tracing::warn!(error = %error, "database ping failed; continuing");
        return Some(pool);
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
        "id_subject".into(),
        "id_applicant".into(),
        "address_subject".into(),
        "asset_summary".into(),
        "medical_evidence_1".into(),
        "medical_evidence_2".into(),
    ]
}

fn resolve_blob_ref(blob_ref: &str, storage_dir: &PathBuf) -> Option<PathBuf> {
    if let Some(path) = blob_ref.strip_prefix("file://") {
        return Some(PathBuf::from(path));
    }
    if blob_ref.starts_with('/') {
        return Some(PathBuf::from(blob_ref));
    }
    Some(storage_dir.join(blob_ref))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn sha256_file(path: &PathBuf) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    Ok(sha256_bytes(&bytes))
}

fn zero_hash() -> String {
    "0".repeat(64)
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
        return invalid_request(Some(request_id), db_error.message().to_string());
    }
    invalid_request(Some(request_id), error.to_string())
}
