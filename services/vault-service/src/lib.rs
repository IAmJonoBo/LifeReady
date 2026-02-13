use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use lifeready_auth::{
    conflict, invalid_request, not_found, request_id_middleware, AuthConfig, AuthLayer,
    RequestContext, RequestId,
};
use lifeready_policy::{
    require_role, require_scope, require_tier, Role, SensitivityTier, TierRequirement,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::net::SocketAddr;
use std::sync::Arc;
use std::{path::PathBuf, str::FromStr};

#[derive(Clone)]
struct AppState {
    pool: Option<PgPool>,
    storage_dir: PathBuf,
}

pub fn router() -> Router {
    let state = AppState {
        pool: pool_from_env(),
        storage_dir: storage_dir_from_env(),
    };
    let auth_config = Arc::new(
        AuthConfig::from_env_checked()
            .expect("AuthConfig misconfigured (check LIFEREADY_ENV and JWT_SECRET)"),
    );

    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/documents", get(list_documents))
        .route("/v1/documents", post(init_document))
        .route(
            "/v1/documents/{document_id}/versions",
            post(commit_document),
        )
        .route("/v1/documents/{document_id}", get(get_document))
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

#[derive(Debug, Deserialize)]
struct DocumentInit {
    document_type: String,
    title: String,
    sensitivity: SensitivityTier,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct DocumentInitResponse {
    document_id: String,
    upload_url: String,
    upload_headers: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct DocumentCommit {
    blob_ref: String,
    sha256: String,
    byte_size: u64,
    mime_type: String,
}

#[derive(Debug, Serialize)]
struct DocumentVersionResponse {
    document_id: String,
    version_id: String,
    sha256: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct DocumentResponse {
    document_id: String,
    document_type: String,
    title: String,
    sensitivity: SensitivityTier,
    tags: Vec<String>,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    limit: Option<i64>,
}

#[derive(Debug, Serialize)]
struct DocumentListResponse {
    items: Vec<DocumentResponse>,
}

async fn init_document(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<DocumentInit>,
) -> Result<(StatusCode, Json<DocumentInitResponse>), axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Allowlist(vec![payload.sensitivity]))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;
    let document_type = payload.document_type.to_lowercase();

    let row = sqlx::query(
        "INSERT INTO documents (principal_id, document_type, title, sensitivity, tags) \
         VALUES ($1, $2::document_type, $3, $4::sensitivity_tier, $5) \
         RETURNING document_id, created_at",
    )
    .bind(principal_id)
    .bind(&document_type)
    .bind(&payload.title)
    .bind(tier_to_str(payload.sensitivity))
    .bind(payload.tags.clone().unwrap_or_default())
    .fetch_one(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let document_id: uuid::Uuid = row
        .try_get("document_id")
        .map_err(|error| db_error_to_response(error, request_id))?;

    let upload_path = state.storage_dir.join(document_id.to_string());
    if let Err(error) = std::fs::create_dir_all(&state.storage_dir) {
        return Err(invalid_request(Some(request_id), error.to_string()));
    }
    let upload_url = format!("file://{}", upload_path.display());

    let response = DocumentInitResponse {
        document_id: document_id.to_string(),
        upload_url,
        upload_headers: serde_json::json!({
            "x-upload-token": "stub",
            "x-blob-ref": format!("file://{}", upload_path.display()),
        }),
    };
    Ok((StatusCode::CREATED, Json(response)))
}

async fn commit_document(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Path(document_id): Path<String>,
    Json(payload): Json<DocumentCommit>,
) -> Result<(StatusCode, Json<DocumentVersionResponse>), axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "write:limited").map_err(|error| error.into_response(Some(request_id)))?;

    if !is_sha256(&payload.sha256) {
        return Err(invalid_request(Some(request_id), "invalid sha256"));
    }

    let document_id = parse_uuid(&document_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid document_id"))?;
    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;

    let exists =
        sqlx::query("SELECT 1 FROM documents WHERE document_id = $1 AND principal_id = $2")
            .bind(document_id)
            .bind(principal_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| db_error_to_response(error, request_id))?
            .is_some();
    if !exists {
        return Err(not_found(Some(request_id), "document not found"));
    }

    let blob_ref = normalize_blob_ref(
        &payload.blob_ref,
        &state.storage_dir,
        document_id,
        request_id,
    )?;
    let row = sqlx::query(
        "INSERT INTO document_versions (document_id, blob_ref, sha256, byte_size, mime_type) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING version_id, created_at",
    )
    .bind(document_id)
    .bind(&blob_ref)
    .bind(&payload.sha256)
    .bind(payload.byte_size as i64)
    .bind(&payload.mime_type)
    .fetch_one(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let version_id: uuid::Uuid = row
        .try_get("version_id")
        .map_err(|error| db_error_to_response(error.into(), request_id))?;
    let created_at: chrono::DateTime<Utc> = row
        .try_get("created_at")
        .map_err(|error| db_error_to_response(error.into(), request_id))?;

    let response = DocumentVersionResponse {
        document_id: document_id.to_string(),
        version_id: version_id.to_string(),
        sha256: payload.sha256,
        created_at: created_at.to_rfc3339(),
    };
    Ok((StatusCode::CREATED, Json(response)))
}

async fn get_document(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Path(document_id): Path<String>,
) -> Result<Json<DocumentResponse>, axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy, Role::ExecutorNominee])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "read:all").map_err(|error| error.into_response(Some(request_id)))?;

    let document_id = parse_uuid(&document_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid document_id"))?;
    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;

    let row = sqlx::query(
        "SELECT document_id, document_type, title, sensitivity, tags, created_at \
         FROM documents WHERE document_id = $1 AND principal_id = $2",
    )
    .bind(document_id)
    .bind(principal_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let row = match row {
        Some(row) => row,
        None => return Err(not_found(Some(request_id), "document not found")),
    };

    let created_at: chrono::DateTime<Utc> = row
        .try_get("created_at")
        .map_err(|error| db_error_to_response(error.into(), request_id))?;
    let sensitivity = tier_from_db(
        row.try_get::<String, _>("sensitivity")
            .map_err(|error| db_error_to_response(error.into(), request_id))?,
    )
    .ok_or_else(|| invalid_request(Some(request_id), "invalid sensitivity"))?;

    ensure_document_access(&ctx, sensitivity, request_id)?;

    Ok(Json(DocumentResponse {
        document_id: row
            .try_get::<uuid::Uuid, _>("document_id")
            .map_err(|error| db_error_to_response(error.into(), request_id))?
            .to_string(),
        document_type: row
            .try_get::<String, _>("document_type")
            .map_err(|error| db_error_to_response(error.into(), request_id))?,
        title: row
            .try_get::<String, _>("title")
            .map_err(|error| db_error_to_response(error.into(), request_id))?,
        sensitivity,
        tags: row
            .try_get::<Vec<String>, _>("tags")
            .map_err(|error| db_error_to_response(error.into(), request_id))?,
        created_at: created_at.to_rfc3339(),
    }))
}

async fn list_documents(
    State(state): State<AppState>,
    ctx: RequestContext,
    Extension(request_id): Extension<RequestId>,
    Query(query): Query<ListQuery>,
) -> Result<Json<DocumentListResponse>, axum::response::Response> {
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err(invalid_request(Some(request_id), "database unavailable")),
    };
    require_role(&ctx, &[Role::Principal, Role::Proxy, Role::ExecutorNominee])
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_tier(&ctx, TierRequirement::Min(SensitivityTier::Amber))
        .map_err(|error| error.into_response(Some(request_id)))?;
    require_scope(&ctx, "read:all").map_err(|error| error.into_response(Some(request_id)))?;

    let principal_id = parse_uuid(&ctx.principal_id)
        .ok_or_else(|| invalid_request(Some(request_id), "invalid principal_id"))?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);

    let rows = sqlx::query(
        "SELECT document_id, document_type, title, sensitivity, tags, created_at \
         FROM documents WHERE principal_id = $1 ORDER BY created_at DESC LIMIT $2",
    )
    .bind(principal_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|error| db_error_to_response(error, request_id))?;

    let mut items = Vec::new();
    for row in rows {
        let created_at: chrono::DateTime<Utc> = row
            .try_get("created_at")
            .map_err(|error| db_error_to_response(error.into(), request_id))?;
        let sensitivity = tier_from_db(
            row.try_get::<String, _>("sensitivity")
                .map_err(|error| db_error_to_response(error.into(), request_id))?,
        )
        .ok_or_else(|| invalid_request(Some(request_id), "invalid sensitivity"))?;

        if ensure_document_access(&ctx, sensitivity, request_id).is_err() {
            continue;
        }

        items.push(DocumentResponse {
            document_id: row
                .try_get::<uuid::Uuid, _>("document_id")
                .map_err(|error| db_error_to_response(error.into(), request_id))?
                .to_string(),
            document_type: row
                .try_get::<String, _>("document_type")
                .map_err(|error| db_error_to_response(error.into(), request_id))?,
            title: row
                .try_get::<String, _>("title")
                .map_err(|error| db_error_to_response(error.into(), request_id))?,
            sensitivity,
            tags: row
                .try_get::<Vec<String>, _>("tags")
                .map_err(|error| db_error_to_response(error.into(), request_id))?,
            created_at: created_at.to_rfc3339(),
        });
    }

    Ok(Json(DocumentListResponse { items }))
}

pub fn addr_from_env(default_port: u16) -> SocketAddr {
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("VAULT_PORT")
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

fn storage_dir_from_env() -> PathBuf {
    std::env::var("LOCAL_STORAGE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("storage"))
}

fn parse_uuid(value: &str) -> Option<uuid::Uuid> {
    uuid::Uuid::from_str(value).ok()
}

fn tier_to_str(tier: SensitivityTier) -> &'static str {
    match tier {
        SensitivityTier::Green => "green",
        SensitivityTier::Amber => "amber",
        SensitivityTier::Red => "red",
    }
}

fn tier_from_db(value: String) -> Option<SensitivityTier> {
    match value.as_str() {
        "green" => Some(SensitivityTier::Green),
        "amber" => Some(SensitivityTier::Amber),
        "red" => Some(SensitivityTier::Red),
        _ => None,
    }
}

pub fn ensure_document_access(
    ctx: &RequestContext,
    sensitivity: SensitivityTier,
    request_id: RequestId,
) -> Result<(), axum::response::Response> {
    require_tier(ctx, TierRequirement::Allowlist(vec![sensitivity]))
        .map_err(|error| error.into_response(Some(request_id)))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

fn db_error_to_response(error: sqlx::Error, request_id: RequestId) -> axum::response::Response {
    if let sqlx::Error::Database(db_error) = &error {
        if db_error.code().as_deref() == Some("23505") {
            return conflict(Some(request_id), "duplicate version for document");
        }
        return invalid_request(Some(request_id), db_error.message().to_string());
    }
    invalid_request(Some(request_id), error.to_string())
}

fn normalize_blob_ref(
    blob_ref: &str,
    storage_dir: &PathBuf,
    document_id: uuid::Uuid,
    request_id: RequestId,
) -> Result<String, axum::response::Response> {
    let candidate = if blob_ref.trim().is_empty() || blob_ref == "auto" {
        format!(
            "file://{}",
            storage_dir.join(document_id.to_string()).display()
        )
    } else if blob_ref.starts_with("file://") || blob_ref.starts_with('/') {
        blob_ref.to_string()
    } else {
        format!("file://{}", storage_dir.join(blob_ref).display())
    };

    let path = candidate.strip_prefix("file://").unwrap_or(&candidate);
    if !std::path::Path::new(path).exists() {
        return Err(invalid_request(Some(request_id), "blob_ref does not exist"));
    }

    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use lifeready_auth::{AccessLevel, AuthConfig, Claims, Role};
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
    fn tier_string_roundtrip() {
        assert_eq!(tier_to_str(SensitivityTier::Green), "green");
        assert_eq!(tier_to_str(SensitivityTier::Amber), "amber");
        assert_eq!(tier_to_str(SensitivityTier::Red), "red");

        assert_eq!(
            tier_from_db("green".to_string()),
            Some(SensitivityTier::Green)
        );
        assert_eq!(
            tier_from_db("amber".to_string()),
            Some(SensitivityTier::Amber)
        );
        assert_eq!(tier_from_db("red".to_string()), Some(SensitivityTier::Red));
        assert_eq!(tier_from_db("unknown".to_string()), None);
    }

    #[test]
    fn sha256_validation() {
        let valid = "a".repeat(64);
        let invalid = "g".repeat(64);
        assert!(is_sha256(&valid));
        assert!(!is_sha256(&invalid));
        assert!(!is_sha256("short"));
    }

    #[test]
    fn parse_uuid_accepts_valid() {
        let value = uuid::Uuid::new_v4().to_string();
        assert!(parse_uuid(&value).is_some());
        assert!(parse_uuid("not-a-uuid").is_none());
    }

    #[test]
    fn normalize_blob_ref_resolves_paths() {
        let base = std::env::temp_dir().join(format!("vault-test-{}", Uuid::new_v4()));
        let document_id = Uuid::new_v4();
        let path = base.join(document_id.to_string());
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(&path, "").unwrap();

        let request_id = RequestId(Uuid::new_v4());
        let auto = normalize_blob_ref("", &base, document_id, request_id).unwrap();
        assert!(auto.starts_with("file://"));

        let custom = normalize_blob_ref("file:///tmp", &base, document_id, request_id).unwrap();
        assert_eq!(custom, "file:///tmp");

        let missing = normalize_blob_ref("missing", &base, document_id, request_id);
        assert_eq!(missing.unwrap_err().status(), StatusCode::BAD_REQUEST);

        let relative_path = base.join("relative-blob");
        std::fs::write(&relative_path, "").unwrap();
        let relative = normalize_blob_ref("relative-blob", &base, document_id, request_id)
            .expect("relative path resolves");
        assert!(relative.starts_with("file://"));
        assert!(relative.contains("relative-blob"));
    }

    #[test]
    fn db_error_to_response_returns_bad_request() {
        let response = db_error_to_response(sqlx::Error::RowNotFound, RequestId(Uuid::new_v4()));
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn ensure_document_access_respects_tier_allowlist() {
        let request_id = RequestId(Uuid::new_v4());
        let ctx = RequestContext {
            request_id,
            principal_id: Uuid::new_v4().to_string(),
            roles: vec![lifeready_policy::Role::Principal],
            allowed_tiers: vec![SensitivityTier::Amber],
            scopes: vec!["read:all".to_string()],
            expires_at: Utc::now(),
            email: None,
        };

        assert!(ensure_document_access(&ctx, SensitivityTier::Amber, request_id).is_ok());
        assert!(ensure_document_access(&ctx, SensitivityTier::Red, request_id).is_err());
    }

    #[test]
    fn storage_dir_defaults_and_overrides() {
        with_env(&[("LOCAL_STORAGE_DIR", None)], || {
            assert_eq!(storage_dir_from_env(), PathBuf::from("storage"));
        });

        with_env(&[("LOCAL_STORAGE_DIR", Some("custom-storage"))], || {
            assert_eq!(storage_dir_from_env(), PathBuf::from("custom-storage"));
        });
    }

    #[test]
    fn addr_from_env_prefers_vault_port_then_port_then_default() {
        with_env(
            &[
                ("HOST", Some("127.0.0.1")),
                ("VAULT_PORT", Some("6123")),
                ("PORT", Some("7123")),
            ],
            || {
                let addr = addr_from_env(8083);
                assert_eq!(addr, "127.0.0.1:6123".parse::<SocketAddr>().unwrap());
            },
        );

        with_env(
            &[
                ("HOST", Some("0.0.0.0")),
                ("VAULT_PORT", None),
                ("PORT", Some("7123")),
            ],
            || {
                let addr = addr_from_env(8083);
                assert_eq!(addr, "0.0.0.0:7123".parse::<SocketAddr>().unwrap());
            },
        );

        with_env(
            &[
                ("HOST", Some("0.0.0.0")),
                ("VAULT_PORT", None),
                ("PORT", None),
            ],
            || {
                let addr = addr_from_env(8083);
                assert_eq!(addr, "0.0.0.0:8083".parse::<SocketAddr>().unwrap());
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

    fn auth_token_with(
        principal_id: &str,
        role: Role,
        tiers: Vec<SensitivityTier>,
        access: AccessLevel,
    ) -> String {
        let config = AuthConfig::new("test-secret-32-chars-minimum!!");
        let claims = Claims::new(principal_id, role, tiers, access, None, 300);
        config.issue_token(&claims).expect("token")
    }

    #[tokio::test]
    async fn init_document_returns_bad_request_without_database_pool() {
        with_env_async(
            &[
                ("LIFEREADY_ENV", Some("dev")),
                ("JWT_SECRET", Some("test-secret-32-chars-minimum!!")),
                ("DATABASE_URL", None),
            ],
            || async {
                let app = router();
                let body = serde_json::json!({
                    "document_type": "will",
                    "title": "My will",
                    "sensitivity": "amber"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/documents")
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
    async fn init_document_rejects_insufficient_role() {
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
                    "document_type": "will",
                    "title": "My will",
                    "sensitivity": "amber"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/documents")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        "00000000-0000-0000-0000-000000000001",
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
    async fn init_document_rejects_insufficient_tier() {
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
                    "document_type": "will",
                    "title": "My will",
                    "sensitivity": "red"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/documents")
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

                assert_eq!(response.status(), StatusCode::FORBIDDEN);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn init_document_rejects_missing_scope() {
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
                    "document_type": "will",
                    "title": "My will",
                    "sensitivity": "amber"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/documents")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::ReadOnlyAll)),
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
    async fn commit_document_rejects_invalid_sha256() {
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
                    "blob_ref": "auto",
                    "sha256": "not-a-sha",
                    "byte_size": 1,
                    "mime_type": "application/pdf"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/documents/00000000-0000-0000-0000-000000000010/versions")
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
    async fn get_document_rejects_invalid_document_id() {
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
                            .method("GET")
                            .uri("/v1/documents/not-a-uuid")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::ReadOnlyAll)),
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
    async fn commit_document_rejects_insufficient_role() {
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
                    "blob_ref": "auto",
                    "sha256": format!("{:0<64}", "a"),
                    "byte_size": 1,
                    "mime_type": "application/pdf"
                })
                .to_string();
                let response = axum::Router::into_service(app)
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/v1/documents/00000000-0000-0000-0000-000000000010/versions")
                            .header("content-type", "application/json")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        "00000000-0000-0000-0000-000000000001",
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
    async fn get_document_rejects_missing_scope() {
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
                            .method("GET")
                            .uri("/v1/documents/00000000-0000-0000-0000-000000000010")
                            .header(
                                "authorization",
                                format!("Bearer {}", auth_token(AccessLevel::ReadOnlyPacks)),
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
    async fn get_document_rejects_invalid_principal_id() {
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
                            .method("GET")
                            .uri("/v1/documents/00000000-0000-0000-0000-000000000010")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        "not-a-uuid",
                                        Role::Principal,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::ReadOnlyAll,
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

    #[tokio::test]
    async fn list_documents_rejects_invalid_principal_id() {
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
                            .method("GET")
                            .uri("/v1/documents")
                            .header(
                                "authorization",
                                format!(
                                    "Bearer {}",
                                    auth_token_with(
                                        "not-a-uuid",
                                        Role::Principal,
                                        vec![SensitivityTier::Amber],
                                        AccessLevel::ReadOnlyAll,
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
}
