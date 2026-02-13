use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use lifeready_auth::{AccessLevel, AuthConfig, Claims, Role, SensitivityTier};
use sqlx::PgPool;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;
use uuid::Uuid;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn init_env() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    unsafe {
        std::env::set_var("LIFEREADY_ENV", "dev");
        std::env::set_var("JWT_SECRET", "test-secret-32-chars-minimum!!");
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

fn token_write() -> String {
    let config = AuthConfig::new("test-secret-32-chars-minimum!!");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::Principal,
        vec![SensitivityTier::Amber],
        AccessLevel::LimitedWrite,
        None,
        300,
    );
    config.issue_token(&claims).expect("token")
}

fn token_read() -> String {
    let config = AuthConfig::new("test-secret-32-chars-minimum!!");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::Principal,
        vec![SensitivityTier::Amber],
        AccessLevel::ReadOnlyAll,
        None,
        300,
    );
    config.issue_token(&claims).expect("token")
}

fn token_invalid_principal() -> String {
    let config = AuthConfig::new("test-secret-32-chars-minimum!!");
    let claims = Claims::new(
        "not-a-uuid",
        Role::Principal,
        vec![SensitivityTier::Amber],
        AccessLevel::LimitedWrite,
        None,
        300,
    );
    config.issue_token(&claims).expect("token")
}

fn unique_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{}-{}", std::process::id(), nanos))
}

async fn setup_db() -> Option<PgPool> {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("DATABASE_URL not set; skipping vault-service db tests");
            return None;
        }
    };
    let pool = PgPool::connect(&database_url).await.ok()?;
    ensure_schema(&pool).await.ok()?;
    Some(pool)
}

async fn ensure_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";")
        .execute(pool)
        .await?;
    sqlx::query(
        "DO $$ BEGIN \
         IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'sensitivity_tier') THEN \
         CREATE TYPE sensitivity_tier AS ENUM ('green','amber','red'); \
         END IF; \
         END $$;",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "DO $$ BEGIN \
         IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'document_type') THEN \
         CREATE TYPE document_type AS ENUM (\
             'id','proof_of_address','will','advance_directive','medical_letter','policy','statement','other'\
         ); \
         END IF; \
         END $$;",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS documents (\
            document_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),\
            principal_id uuid NOT NULL,\
            document_type document_type NOT NULL,\
            title text NOT NULL,\
            sensitivity sensitivity_tier NOT NULL,\
            tags text[] NOT NULL DEFAULT ARRAY[]::text[],\
            created_at timestamptz NOT NULL DEFAULT now()\
        );",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS document_versions (\
            version_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),\
            document_id uuid NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,\
            blob_ref text NOT NULL,\
            sha256 char(64) NOT NULL,\
            byte_size bigint NOT NULL,\
            mime_type text NOT NULL,\
            created_at timestamptz NOT NULL DEFAULT now(),\
            UNIQUE(document_id, sha256)\
        );",
    )
    .execute(pool)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_documents_principal ON documents(principal_id);")
        .execute(pool)
        .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_doc_versions_doc ON document_versions(document_id);",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn reset_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("TRUNCATE document_versions, documents RESTART IDENTITY CASCADE")
        .execute(pool)
        .await?;
    Ok(())
}

#[tokio::test]
async fn init_document_commit_and_list() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("vault-storage");
    std::fs::create_dir_all(&storage_dir).unwrap();

    with_env_async(&[("LOCAL_STORAGE_DIR", storage_dir.to_str())], || async {
        let app = vault_service::router();
        let body = serde_json::json!({
            "document_type": "will",
            "title": "My will",
            "sensitivity": "amber"
        })
        .to_string();
        let response = axum::Router::into_service(app.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/documents")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token_write()))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let document_id = value.get("document_id").and_then(|v| v.as_str()).unwrap();

        let blob_path = storage_dir.join(document_id);
        std::fs::write(&blob_path, b"blob").unwrap();

        let sha256 = "a".repeat(64);
        let commit_body = serde_json::json!({
            "blob_ref": "auto",
            "sha256": sha256,
            "byte_size": 4,
            "mime_type": "text/plain"
        })
        .to_string();
        let response = axum::Router::into_service(app.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/documents/{document_id}/versions"))
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token_write()))
                    .body(Body::from(commit_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value.get("sha256").and_then(|v| v.as_str()),
            Some(sha256.as_str())
        );
        assert!(value.get("version_id").is_some());

        let response = axum::Router::into_service(app)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/documents?limit=10")
                    .header("authorization", format!("Bearer {}", token_read()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let items = value.get("items").and_then(|v| v.as_array()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].get("document_id").and_then(|v| v.as_str()),
            Some(document_id)
        );
    })
    .await;
}

#[tokio::test]
async fn get_document_returns_payload() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("vault-storage");
    std::fs::create_dir_all(&storage_dir).unwrap();

    with_env_async(&[("LOCAL_STORAGE_DIR", storage_dir.to_str())], || async {
        let app = vault_service::router();
        let body = serde_json::json!({
            "document_type": "id",
            "title": "Passport",
            "sensitivity": "amber"
        })
        .to_string();
        let response = axum::Router::into_service(app.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/documents")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token_write()))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let document_id = value.get("document_id").and_then(|v| v.as_str()).unwrap();

        let response = axum::Router::into_service(app)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/v1/documents/{document_id}"))
                    .header("authorization", format!("Bearer {}", token_read()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value.get("document_id").and_then(|v| v.as_str()),
            Some(document_id)
        );
        assert_eq!(
            value.get("title").and_then(|v| v.as_str()),
            Some("Passport")
        );
    })
    .await;
}

#[tokio::test]
async fn commit_document_rejects_invalid_sha() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("vault-storage");
    std::fs::create_dir_all(&storage_dir).unwrap();

    with_env_async(&[("LOCAL_STORAGE_DIR", storage_dir.to_str())], || async {
        let app = vault_service::router();
        let document_id = uuid::Uuid::new_v4();
        let commit_body = serde_json::json!({
            "blob_ref": "auto",
            "sha256": "not-a-sha",
            "byte_size": 4,
            "mime_type": "text/plain"
        })
        .to_string();
        let response = axum::Router::into_service(app)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/documents/{document_id}/versions"))
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token_write()))
                    .body(Body::from(commit_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    })
    .await;
}

#[tokio::test]
async fn commit_document_rejects_missing_blob() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("vault-storage");
    std::fs::create_dir_all(&storage_dir).unwrap();

    with_env_async(&[("LOCAL_STORAGE_DIR", storage_dir.to_str())], || async {
        let app = vault_service::router();
        let body = serde_json::json!({
            "document_type": "will",
            "title": "My will",
            "sensitivity": "amber"
        })
        .to_string();
        let response = axum::Router::into_service(app.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/documents")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token_write()))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let document_id = value.get("document_id").and_then(|v| v.as_str()).unwrap();

        let commit_body = serde_json::json!({
            "blob_ref": "auto",
            "sha256": "a".repeat(64),
            "byte_size": 4,
            "mime_type": "text/plain"
        })
        .to_string();
        let response = axum::Router::into_service(app)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/documents/{document_id}/versions"))
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token_write()))
                    .body(Body::from(commit_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    })
    .await;
}

#[tokio::test]
async fn commit_document_rejects_duplicate_version() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("vault-storage");
    std::fs::create_dir_all(&storage_dir).unwrap();

    with_env_async(&[("LOCAL_STORAGE_DIR", storage_dir.to_str())], || async {
        let app = vault_service::router();
        let body = serde_json::json!({
            "document_type": "will",
            "title": "My will",
            "sensitivity": "amber"
        })
        .to_string();
        let response = axum::Router::into_service(app.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/documents")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token_write()))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let document_id = value.get("document_id").and_then(|v| v.as_str()).unwrap();

        let blob_path = storage_dir.join(document_id);
        std::fs::write(&blob_path, b"blob").unwrap();

        let sha256 = "a".repeat(64);
        let commit_body = serde_json::json!({
            "blob_ref": "auto",
            "sha256": sha256,
            "byte_size": 4,
            "mime_type": "text/plain"
        })
        .to_string();

        let response = axum::Router::into_service(app.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/documents/{document_id}/versions"))
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token_write()))
                    .body(Body::from(commit_body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let response = axum::Router::into_service(app)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/documents/{document_id}/versions"))
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token_write()))
                    .body(Body::from(commit_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    })
    .await;
}

#[tokio::test]
async fn get_document_returns_not_found() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("vault-storage");
    std::fs::create_dir_all(&storage_dir).unwrap();

    with_env_async(&[("LOCAL_STORAGE_DIR", storage_dir.to_str())], || async {
        let app = vault_service::router();
        let document_id = uuid::Uuid::new_v4();
        let response = axum::Router::into_service(app)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/v1/documents/{document_id}"))
                    .header("authorization", format!("Bearer {}", token_read()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    })
    .await;
}

#[tokio::test]
async fn list_documents_filters_unavailable_tiers() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("vault-storage");
    std::fs::create_dir_all(&storage_dir).unwrap();

    with_env_async(&[("LOCAL_STORAGE_DIR", storage_dir.to_str())], || async {
        let principal_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        sqlx::query(
            "INSERT INTO documents (document_id, principal_id, document_type, title, sensitivity, tags) \
             VALUES ($1, $2, 'id', $3, 'amber', ARRAY[]::text[])",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(principal_id)
        .bind("Amber Doc")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO documents (document_id, principal_id, document_type, title, sensitivity, tags) \
             VALUES ($1, $2, 'id', $3, 'red', ARRAY[]::text[])",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(principal_id)
        .bind("Red Doc")
        .execute(&pool)
        .await
        .unwrap();

        let app = vault_service::router();
        let response = axum::Router::into_service(app)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/documents?limit=10")
                    .header("authorization", format!("Bearer {}", token_read()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let items = value.get("items").and_then(|v| v.as_array()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].get("title").and_then(|v| v.as_str()), Some("Amber Doc"));
    })
    .await;
}

#[tokio::test]
async fn init_document_rejects_invalid_principal() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("vault-storage");
    std::fs::create_dir_all(&storage_dir).unwrap();

    with_env_async(&[("LOCAL_STORAGE_DIR", storage_dir.to_str())], || async {
        let app = vault_service::router();
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
                        format!("Bearer {}", token_invalid_principal()),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    })
    .await;
}

#[tokio::test]
async fn commit_document_rejects_invalid_principal() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = vault_service::router();
    let document_id = Uuid::new_v4();
    let body = serde_json::json!({
        "blob_ref": "auto",
        "sha256": "a".repeat(64),
        "byte_size": 10,
        "mime_type": "text/plain"
    })
    .to_string();

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/documents/{document_id}/versions"))
                .header("content-type", "application/json")
                .header(
                    "authorization",
                    format!("Bearer {}", token_invalid_principal()),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_document_rejects_invalid_principal() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = vault_service::router();
    let document_id = Uuid::new_v4();

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/documents/{document_id}"))
                .header(
                    "authorization",
                    format!("Bearer {}", token_invalid_principal()),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_documents_rejects_invalid_principal() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = vault_service::router();

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/documents")
                .header(
                    "authorization",
                    format!("Bearer {}", token_invalid_principal()),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_document_rejects_unlisted_tier() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let document_id = uuid::Uuid::new_v4();
    let principal_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    sqlx::query(
        "INSERT INTO documents (document_id, principal_id, document_type, title, sensitivity, tags) \
         VALUES ($1, $2, 'id', $3, 'red', ARRAY[]::text[])",
    )
    .bind(document_id)
    .bind(principal_id)
    .bind("Red Doc")
    .execute(&pool)
    .await
    .unwrap();

    let app = vault_service::router();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/documents/{document_id}"))
                .header("authorization", format!("Bearer {}", token_read()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn commit_document_accepts_file_ref() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("vault-storage");
    std::fs::create_dir_all(&storage_dir).unwrap();

    with_env_async(&[("LOCAL_STORAGE_DIR", storage_dir.to_str())], || async {
        let app = vault_service::router();
        let body = serde_json::json!({
            "document_type": "will",
            "title": "My will",
            "sensitivity": "amber"
        })
        .to_string();
        let response = axum::Router::into_service(app.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/documents")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token_write()))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let document_id = value.get("document_id").and_then(|v| v.as_str()).unwrap();

        let blob_path = storage_dir.join("explicit-blob");
        std::fs::write(&blob_path, b"blob").unwrap();
        let blob_ref = format!("file://{}", blob_path.display());

        let commit_body = serde_json::json!({
            "blob_ref": blob_ref,
            "sha256": "a".repeat(64),
            "byte_size": 4,
            "mime_type": "text/plain"
        })
        .to_string();
        let response = axum::Router::into_service(app)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/documents/{document_id}/versions"))
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token_write()))
                    .body(Body::from(commit_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    })
    .await;
}

#[tokio::test]
async fn init_document_rejects_insufficient_role() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let config = AuthConfig::new("test-secret-32-chars-minimum!!");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::EmergencyContact,
        vec![SensitivityTier::Amber],
        AccessLevel::LimitedWrite,
        None,
        300,
    );
    let token = config.issue_token(&claims).expect("token");

    let app = vault_service::router();
    let body = serde_json::json!({
        "document_type": "will",
        "title": "Test",
        "sensitivity": "amber"
    })
    .to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/documents")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn init_document_rejects_insufficient_tier() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let config = AuthConfig::new("test-secret-32-chars-minimum!!");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::Principal,
        vec![SensitivityTier::Green],
        AccessLevel::LimitedWrite,
        None,
        300,
    );
    let token = config.issue_token(&claims).expect("token");

    let app = vault_service::router();
    let body = serde_json::json!({
        "document_type": "will",
        "title": "Test",
        "sensitivity": "amber"
    })
    .to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/documents")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn commit_document_rejects_invalid_document_id() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = vault_service::router();
    let body = serde_json::json!({
        "blob_ref": "test-blob",
        "sha256": "a".repeat(64),
        "byte_size": 10,
        "mime_type": "text/plain"
    })
    .to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/documents/not-a-uuid/versions")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn commit_document_rejects_nonexistent_document() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = vault_service::router();
    let document_id = Uuid::new_v4();
    let body = serde_json::json!({
        "blob_ref": "test-blob",
        "sha256": "a".repeat(64),
        "byte_size": 10,
        "mime_type": "text/plain"
    })
    .to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/documents/{document_id}/versions"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_document_rejects_invalid_document_id() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = vault_service::router();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/documents/not-a-uuid")
                .header("authorization", format!("Bearer {}", token_read()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_document_rejects_nonexistent_document() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = vault_service::router();
    let document_id = Uuid::new_v4();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/documents/{document_id}"))
                .header("authorization", format!("Bearer {}", token_read()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_documents_rejects_insufficient_role() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let config = AuthConfig::new("test-secret-32-chars-minimum!!");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::EmergencyContact,
        vec![SensitivityTier::Amber],
        AccessLevel::ReadOnlyAll,
        None,
        300,
    );
    let token = config.issue_token(&claims).expect("token");

    let app = vault_service::router();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/documents")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
