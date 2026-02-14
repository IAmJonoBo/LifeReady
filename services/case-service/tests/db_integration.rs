use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use lifeready_auth::{AccessLevel, AuthConfig, Claims, Role, SensitivityTier};
use sha2::Digest;
use sqlx::{PgPool, Row};
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

fn token_other_principal() -> String {
    let config = AuthConfig::new("test-secret-32-chars-minimum!!");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000999",
        Role::Principal,
        vec![SensitivityTier::Amber],
        AccessLevel::ReadOnlyAll,
        None,
        300,
    );
    config.issue_token(&claims).expect("token")
}

fn token_read_packs() -> String {
    let config = AuthConfig::new("test-secret-32-chars-minimum!!");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::Principal,
        vec![SensitivityTier::Amber],
        AccessLevel::ReadOnlyPacks,
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

async fn setup_db() -> Option<PgPool> {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("DATABASE_URL not set; skipping case-service db tests");
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
        "DO $$ BEGIN \
         IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'case_type') THEN \
         CREATE TYPE case_type AS ENUM ('emergency_pack','mhca39','death_readiness','will_prep_sa','deceased_estate_reporting_sa'); \
         END IF; \
         END $$;",
    )
    .execute(pool)
    .await?;
    // Add new enum values if they don't exist (for existing DBs)
    sqlx::query(
        "DO $$ BEGIN \
         ALTER TYPE case_type ADD VALUE IF NOT EXISTS 'will_prep_sa'; \
         EXCEPTION WHEN duplicate_object THEN NULL; \
         END $$;",
    )
    .execute(pool)
    .await
    .ok();
    sqlx::query(
        "DO $$ BEGIN \
         ALTER TYPE case_type ADD VALUE IF NOT EXISTS 'deceased_estate_reporting_sa'; \
         EXCEPTION WHEN duplicate_object THEN NULL; \
         END $$;",
    )
    .execute(pool)
    .await
    .ok();
    sqlx::query(
        "DO $$ BEGIN \
         IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'case_status') THEN \
         CREATE TYPE case_status AS ENUM ('draft','ready','blocked','exported','closed','revoked'); \
         END IF; \
         END $$;",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS cases (\
            case_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),\
            principal_id uuid NOT NULL,\
            case_type case_type NOT NULL,\
            status case_status NOT NULL DEFAULT 'draft',\
            blocked_reasons text[] NOT NULL DEFAULT ARRAY[]::text[],\
            created_at timestamptz NOT NULL DEFAULT now()\
        );",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS case_artifacts (\
            artifact_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),\
            case_id uuid NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,\
            kind text NOT NULL,\
            blob_ref text NOT NULL,\
            sha256 char(64) NOT NULL,\
            created_at timestamptz NOT NULL DEFAULT now()\
        );",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS mhca39_cases (\
            case_id uuid PRIMARY KEY REFERENCES cases(case_id) ON DELETE CASCADE,\
            subject_person_id uuid NOT NULL,\
            applicant_person_id uuid NOT NULL,\
            relationship_to_subject text,\
            required_evidence_slots text[] NOT NULL,\
            notes text\
        );",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS mhca39_evidence (\
            evidence_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),\
            case_id uuid NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,\
            slot_name text NOT NULL,\
            document_id uuid,\
            added_at timestamptz NOT NULL DEFAULT now(),\
            UNIQUE(case_id, slot_name)\
        );",
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
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS audit_events (\
            event_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),\
            created_at timestamptz NOT NULL DEFAULT now(),\
            actor_principal_id uuid NOT NULL,\
            action text NOT NULL,\
            tier sensitivity_tier NOT NULL,\
            case_id uuid,\
            payload jsonb NOT NULL,\
            prev_hash char(64) NOT NULL,\
            event_hash char(64) NOT NULL\
        );",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS case_evidence (\
            evidence_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),\
            case_id uuid NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,\
            slot_name text NOT NULL,\
            document_id uuid,\
            added_at timestamptz NOT NULL DEFAULT now(),\
            UNIQUE(case_id, slot_name)\
        );",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS will_prep_cases (\
            case_id uuid PRIMARY KEY REFERENCES cases(case_id) ON DELETE CASCADE,\
            principal_person_id uuid NOT NULL,\
            required_evidence_slots text[] NOT NULL,\
            notes text\
        );",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS deceased_estate_cases (\
            case_id uuid PRIMARY KEY REFERENCES cases(case_id) ON DELETE CASCADE,\
            deceased_person_id uuid NOT NULL,\
            executor_person_id uuid NOT NULL,\
            estimated_estate_value_zar numeric,\
            required_evidence_slots text[] NOT NULL,\
            notes text\
        );",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn reset_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "TRUNCATE audit_events, document_versions, documents, mhca39_evidence, mhca39_cases, case_evidence, will_prep_cases, deceased_estate_cases, case_artifacts, cases RESTART IDENTITY CASCADE",
    )
        .execute(pool)
        .await?;
    Ok(())
}

fn unique_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{}-{}", std::process::id(), nanos))
}

#[tokio::test]
async fn create_emergency_pack_persists_case() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let body =
        serde_json::json!({"directive_document_ids": [], "emergency_contacts": []}).to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/emergency-pack")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();
    let case_uuid = Uuid::parse_str(case_id).unwrap();

    let row = sqlx::query("SELECT case_type FROM cases WHERE case_id = $1")
        .bind(case_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    let case_type: String = row.try_get("case_type").unwrap();
    assert_eq!(case_type, "emergency_pack");
}

#[tokio::test]
async fn create_mhca39_persists_case() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000011",
        "applicant_person_id": "00000000-0000-0000-0000-000000000022",
        "relationship_to_subject": "spouse",
        "notes": "test",
        "required_evidence_slots": ["id", "letter"]
    })
    .to_string();

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();
    let case_uuid = Uuid::parse_str(case_id).unwrap();

    let row = sqlx::query("SELECT subject_person_id FROM mhca39_cases WHERE case_id = $1")
        .bind(case_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    let subject: Uuid = row.try_get("subject_person_id").unwrap();
    assert_eq!(subject.to_string(), "00000000-0000-0000-0000-000000000011");
}

#[tokio::test]
async fn attach_evidence_and_export_case() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("case-storage");
    let export_dir = unique_dir("case-export");
    std::fs::create_dir_all(&storage_dir).unwrap();
    std::fs::create_dir_all(&export_dir).unwrap();

    let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    unsafe {
        std::env::set_var("LOCAL_STORAGE_DIR", &storage_dir);
        std::env::set_var("LOCAL_EXPORT_DIR", &export_dir);
    }

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000011",
        "applicant_person_id": "00000000-0000-0000-0000-000000000022",
        "relationship_to_subject": "spouse",
        "notes": "test",
        "required_evidence_slots": ["id"]
    })
    .to_string();

    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();

    let document_id = Uuid::new_v4();
    let blob_path = storage_dir.join(document_id.to_string());
    std::fs::write(&blob_path, b"doc").unwrap();

    sqlx::query(
        "INSERT INTO documents (document_id, principal_id, document_type, title, sensitivity, tags) \
         VALUES ($1, $2, 'id', $3, 'amber', ARRAY[]::text[])",
    )
    .bind(document_id)
    .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
    .bind("ID")
    .execute(&pool)
    .await
    .unwrap();

    let blob_ref = format!("file://{}", blob_path.display());
    let sha256 = "a".repeat(64);
    sqlx::query(
        "INSERT INTO document_versions (document_id, blob_ref, sha256, byte_size, mime_type) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(document_id)
    .bind(blob_ref)
    .bind(&sha256)
    .bind(3_i64)
    .bind("text/plain")
    .execute(&pool)
    .await
    .unwrap();

    let attach_body = serde_json::json!({"document_id": document_id.to_string()}).to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/id"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/cases/{case_id}/export"))
                .header("authorization", format!("Bearer {}", token_read()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let download_url = value.get("download_url").and_then(|v| v.as_str()).unwrap();
    let manifest_sha = value
        .get("manifest_sha256")
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(manifest_sha.len(), 64);

    let export_path = download_url.trim_start_matches("file://");
    let manifest_path = PathBuf::from(export_path).join("manifest.json");
    assert!(manifest_path.exists());
}

#[tokio::test]
async fn attach_evidence_rejects_missing_document() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("case-storage");
    std::fs::create_dir_all(&storage_dir).unwrap();

    let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    unsafe {
        std::env::set_var("LOCAL_STORAGE_DIR", &storage_dir);
    }

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000011",
        "applicant_person_id": "00000000-0000-0000-0000-000000000022",
        "required_evidence_slots": ["id"]
    })
    .to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();

    let missing_doc_id = Uuid::new_v4();
    let attach_body = serde_json::json!({"document_id": missing_doc_id.to_string()}).to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/id"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn export_case_rejects_incomplete_evidence() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let export_dir = unique_dir("case-export");
    std::fs::create_dir_all(&export_dir).unwrap();

    let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    unsafe {
        std::env::set_var("LOCAL_EXPORT_DIR", &export_dir);
    }

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000011",
        "applicant_person_id": "00000000-0000-0000-0000-000000000022",
        "required_evidence_slots": ["id", "letter"]
    })
    .to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/cases/{case_id}/export"))
                .header("authorization", format!("Bearer {}", token_read()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn export_case_rejects_wrong_principal() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let export_dir = unique_dir("case-export");
    std::fs::create_dir_all(&export_dir).unwrap();

    let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    unsafe {
        std::env::set_var("LOCAL_EXPORT_DIR", &export_dir);
    }

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000011",
        "applicant_person_id": "00000000-0000-0000-0000-000000000022",
        "required_evidence_slots": ["id"]
    })
    .to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/cases/{case_id}/export"))
                .header(
                    "authorization",
                    format!("Bearer {}", token_other_principal()),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn export_case_rejects_missing_versions() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("case-storage");
    let export_dir = unique_dir("case-export");
    std::fs::create_dir_all(&storage_dir).unwrap();
    std::fs::create_dir_all(&export_dir).unwrap();

    let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    unsafe {
        std::env::set_var("LOCAL_STORAGE_DIR", &storage_dir);
        std::env::set_var("LOCAL_EXPORT_DIR", &export_dir);
    }

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000011",
        "applicant_person_id": "00000000-0000-0000-0000-000000000022",
        "required_evidence_slots": ["id"]
    })
    .to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();

    let document_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO documents (document_id, principal_id, document_type, title, sensitivity, tags) \
         VALUES ($1, $2, 'id', $3, 'amber', ARRAY[]::text[])",
    )
    .bind(document_id)
    .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
    .bind("ID")
    .execute(&pool)
    .await
    .unwrap();

    let attach_body = serde_json::json!({"document_id": document_id.to_string()}).to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/id"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/cases/{case_id}/export"))
                .header("authorization", format!("Bearer {}", token_read()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn attach_evidence_rejects_unknown_slot() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("case-storage");
    std::fs::create_dir_all(&storage_dir).unwrap();

    let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    unsafe {
        std::env::set_var("LOCAL_STORAGE_DIR", &storage_dir);
    }

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000011",
        "applicant_person_id": "00000000-0000-0000-0000-000000000022",
        "required_evidence_slots": ["id"]
    })
    .to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();

    let document_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO documents (document_id, principal_id, document_type, title, sensitivity, tags) \
         VALUES ($1, $2, 'id', $3, 'amber', ARRAY[]::text[])",
    )
    .bind(document_id)
    .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
    .bind("ID")
    .execute(&pool)
    .await
    .unwrap();

    let attach_body = serde_json::json!({"document_id": document_id.to_string()}).to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/unknown"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_mhca39_uses_default_slots() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000011",
        "applicant_person_id": "00000000-0000-0000-0000-000000000022"
    })
    .to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();
    let case_uuid = Uuid::parse_str(case_id).unwrap();

    let row = sqlx::query("SELECT required_evidence_slots FROM mhca39_cases WHERE case_id = $1")
        .bind(case_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    let slots: Vec<String> = row.try_get("required_evidence_slots").unwrap();
    assert!(slots.len() >= 7);
    assert!(slots.contains(&"medical_certificate_1".to_string()));
}

#[tokio::test]
async fn export_case_allows_read_packs_scope() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("case-storage");
    let export_dir = unique_dir("case-export");
    std::fs::create_dir_all(&storage_dir).unwrap();
    std::fs::create_dir_all(&export_dir).unwrap();

    let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    unsafe {
        std::env::set_var("LOCAL_STORAGE_DIR", &storage_dir);
        std::env::set_var("LOCAL_EXPORT_DIR", &export_dir);
    }

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000011",
        "applicant_person_id": "00000000-0000-0000-0000-000000000022",
        "required_evidence_slots": ["id"]
    })
    .to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();

    let document_id = Uuid::new_v4();
    let blob_path = storage_dir.join(document_id.to_string());
    std::fs::write(&blob_path, b"doc").unwrap();

    sqlx::query(
        "INSERT INTO documents (document_id, principal_id, document_type, title, sensitivity, tags) \
         VALUES ($1, $2, 'id', $3, 'amber', ARRAY[]::text[])",
    )
    .bind(document_id)
    .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
    .bind("ID")
    .execute(&pool)
    .await
    .unwrap();

    let blob_ref = format!("file://{}", blob_path.display());
    sqlx::query(
        "INSERT INTO document_versions (document_id, blob_ref, sha256, byte_size, mime_type) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(document_id)
    .bind(blob_ref)
    .bind("a".repeat(64))
    .bind(3_i64)
    .bind("text/plain")
    .execute(&pool)
    .await
    .unwrap();

    let attach_body = serde_json::json!({"document_id": document_id.to_string()}).to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/id"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/cases/{case_id}/export"))
                .header("authorization", format!("Bearer {}", token_read_packs()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn export_case_includes_audit_events() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("case-storage");
    let export_dir = unique_dir("case-export");
    std::fs::create_dir_all(&storage_dir).unwrap();
    std::fs::create_dir_all(&export_dir).unwrap();

    let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    unsafe {
        std::env::set_var("LOCAL_STORAGE_DIR", &storage_dir);
        std::env::set_var("LOCAL_EXPORT_DIR", &export_dir);
    }

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000011",
        "applicant_person_id": "00000000-0000-0000-0000-000000000022",
        "required_evidence_slots": ["id"]
    })
    .to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();

    let document_id = Uuid::new_v4();
    let blob_path = storage_dir.join(document_id.to_string());
    std::fs::write(&blob_path, b"doc").unwrap();

    sqlx::query(
        "INSERT INTO documents (document_id, principal_id, document_type, title, sensitivity, tags) \
         VALUES ($1, $2, 'id', $3, 'amber', ARRAY[]::text[])",
    )
    .bind(document_id)
    .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
    .bind("ID")
    .execute(&pool)
    .await
    .unwrap();

    let blob_ref = format!("file://{}", blob_path.display());
    let version_hash = "a".repeat(64);
    sqlx::query(
        "INSERT INTO document_versions (document_id, blob_ref, sha256, byte_size, mime_type) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(document_id)
    .bind(blob_ref)
    .bind(&version_hash)
    .bind(3_i64)
    .bind("text/plain")
    .execute(&pool)
    .await
    .unwrap();

    let attach_body = serde_json::json!({"document_id": document_id.to_string()}).to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/id"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let audit_event_id = Uuid::new_v4();
    let audit_hash = "b".repeat(64);
    sqlx::query(
        "INSERT INTO audit_events (event_id, actor_principal_id, action, tier, case_id, payload, prev_hash, event_hash) \
         VALUES ($1, $2, $3, $4::sensitivity_tier, $5, $6, $7, $8)",
    )
    .bind(audit_event_id)
    .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
    .bind("case.export")
    .bind("green")
    .bind(Uuid::parse_str(case_id).unwrap())
    .bind(serde_json::json!({"ok": true}))
    .bind("0".repeat(64))
    .bind(&audit_hash)
    .execute(&pool)
    .await
    .unwrap();

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/cases/{case_id}/export"))
                .header("authorization", format!("Bearer {}", token_read()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let download_url = value.get("download_url").and_then(|v| v.as_str()).unwrap();
    let export_path = download_url.trim_start_matches("file://");
    let audit_path = PathBuf::from(export_path).join("audit.jsonl");
    let audit_contents = std::fs::read_to_string(audit_path).unwrap();
    assert!(audit_contents.contains(&audit_event_id.to_string()));
}

#[tokio::test]
async fn export_case_rejects_missing_blob() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let export_dir = unique_dir("case-export");
    std::fs::create_dir_all(&export_dir).unwrap();

    let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    unsafe {
        std::env::set_var("LOCAL_EXPORT_DIR", &export_dir);
    }

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000011",
        "applicant_person_id": "00000000-0000-0000-0000-000000000022",
        "required_evidence_slots": ["id"]
    })
    .to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();

    let document_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO documents (document_id, principal_id, document_type, title, sensitivity, tags) \
         VALUES ($1, $2, 'id', $3, 'amber', ARRAY[]::text[])",
    )
    .bind(document_id)
    .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
    .bind("ID")
    .execute(&pool)
    .await
    .unwrap();

    let blob_ref = "file:///missing";
    sqlx::query(
        "INSERT INTO document_versions (document_id, blob_ref, sha256, byte_size, mime_type) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(document_id)
    .bind(blob_ref)
    .bind("a".repeat(64))
    .bind(3_i64)
    .bind("text/plain")
    .execute(&pool)
    .await
    .unwrap();

    let attach_body = serde_json::json!({"document_id": document_id.to_string()}).to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/id"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/cases/{case_id}/export"))
                .header("authorization", format!("Bearer {}", token_read()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_emergency_pack_rejects_insufficient_role() {
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

    let app = case_service::router();
    let body =
        serde_json::json!({"directive_document_ids": [], "emergency_contacts": []}).to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/emergency-pack")
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
async fn create_emergency_pack_rejects_insufficient_tier() {
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

    let app = case_service::router();
    let body =
        serde_json::json!({"directive_document_ids": [], "emergency_contacts": []}).to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/emergency-pack")
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
async fn create_emergency_pack_rejects_readonly_scope() {
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
        vec![SensitivityTier::Amber],
        AccessLevel::ReadOnlyAll,
        None,
        300,
    );
    let token = config.issue_token(&claims).expect("token");

    let app = case_service::router();
    let body =
        serde_json::json!({"directive_document_ids": [], "emergency_contacts": []}).to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/emergency-pack")
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
async fn create_mhca39_rejects_invalid_subject_person_id() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "not-a-uuid",
        "applicant_person_id": Uuid::new_v4().to_string(),
    })
    .to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
async fn create_mhca39_rejects_invalid_applicant_person_id() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": Uuid::new_v4().to_string(),
        "applicant_person_id": "not-a-uuid",
    })
    .to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
async fn create_emergency_pack_rejects_invalid_principal_id() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let body =
        serde_json::json!({"directive_document_ids": [], "emergency_contacts": []}).to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/emergency-pack")
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
async fn create_mhca39_rejects_invalid_principal_id() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": Uuid::new_v4().to_string(),
        "applicant_person_id": Uuid::new_v4().to_string(),
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
async fn attach_evidence_rejects_insufficient_role() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router().clone();
    let body = serde_json::json!({
        "subject_person_id": Uuid::new_v4().to_string(),
        "applicant_person_id": Uuid::new_v4().to_string(),
    })
    .to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();

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

    let app = case_service::router();
    let attach_body = serde_json::json!({"document_id": Uuid::new_v4().to_string()}).to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/id_subject"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn attach_evidence_rejects_invalid_case_id() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let attach_body = serde_json::json!({"document_id": Uuid::new_v4().to_string()}).to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/v1/cases/not-a-uuid/evidence/id_subject")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn attach_evidence_rejects_invalid_document_id() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router().clone();
    let body = serde_json::json!({
        "subject_person_id": Uuid::new_v4().to_string(),
        "applicant_person_id": Uuid::new_v4().to_string(),
    })
    .to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();

    let app = case_service::router();
    let attach_body = serde_json::json!({"document_id": "not-a-uuid"}).to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/id_subject"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn attach_evidence_rejects_nonexistent_case() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let case_id = Uuid::new_v4();
    let attach_body = serde_json::json!({"document_id": Uuid::new_v4().to_string()}).to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/id_subject"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn attach_evidence_rejects_invalid_principal_id() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let case_id = Uuid::new_v4();
    let attach_body = serde_json::json!({"document_id": Uuid::new_v4().to_string()}).to_string();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/id_subject"))
                .header("content-type", "application/json")
                .header(
                    "authorization",
                    format!("Bearer {}", token_invalid_principal()),
                )
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn export_case_rejects_invalid_case_id() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/not-a-uuid/export")
                .header("authorization", format!("Bearer {}", token_read()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn export_case_rejects_nonexistent_case() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let case_id = Uuid::new_v4();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/cases/{case_id}/export"))
                .header("authorization", format!("Bearer {}", token_read()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn export_case_rejects_invalid_principal_id() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let case_id = Uuid::new_v4();
    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/cases/{case_id}/export"))
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

fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[tokio::test]
async fn create_will_prep_sa_persists_case() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let body = serde_json::json!({
        "principal_person_id": "00000000-0000-0000-0000-000000000011",
        "notes": "test will"
    })
    .to_string();

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/will-prep-sa")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();
    let case_uuid = Uuid::parse_str(case_id).unwrap();

    let row = sqlx::query("SELECT case_type FROM cases WHERE case_id = $1")
        .bind(case_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    let case_type: String = row.try_get("case_type").unwrap();
    assert_eq!(case_type, "will_prep_sa");

    let row = sqlx::query("SELECT required_evidence_slots FROM will_prep_cases WHERE case_id = $1")
        .bind(case_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    let slots: Vec<String> = row.try_get("required_evidence_slots").unwrap();
    assert!(slots.len() >= 5);
    assert!(slots.contains(&"draft_will_document".to_string()));
}

#[tokio::test]
async fn create_deceased_estate_sa_persists_case() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let app = case_service::router();
    let body = serde_json::json!({
        "deceased_person_id": "00000000-0000-0000-0000-000000000033",
        "executor_person_id": "00000000-0000-0000-0000-000000000044",
        "estimated_estate_value_zar": 500000.0,
        "notes": "test estate"
    })
    .to_string();

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/deceased-estate-sa")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap();
    let case_uuid = Uuid::parse_str(case_id).unwrap();

    let row = sqlx::query("SELECT case_type FROM cases WHERE case_id = $1")
        .bind(case_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    let case_type: String = row.try_get("case_type").unwrap();
    assert_eq!(case_type, "deceased_estate_reporting_sa");

    let row = sqlx::query(
        "SELECT required_evidence_slots FROM deceased_estate_cases WHERE case_id = $1",
    )
    .bind(case_uuid)
    .fetch_one(&pool)
    .await
    .unwrap();
    let slots: Vec<String> = row.try_get("required_evidence_slots").unwrap();
    assert!(slots.len() >= 7);
    assert!(slots.contains(&"death_certificate".to_string()));
}

#[tokio::test]
async fn will_prep_sa_attach_and_export() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("will-storage");
    let export_dir = unique_dir("will-exports");
    std::fs::create_dir_all(&storage_dir).unwrap();
    std::fs::create_dir_all(&export_dir).unwrap();
    unsafe {
        std::env::set_var(
            "LOCAL_STORAGE_DIR",
            storage_dir.to_string_lossy().to_string(),
        );
        std::env::set_var("LOCAL_EXPORT_DIR", export_dir.to_string_lossy().to_string());
    }

    let principal_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let app = case_service::router();
    let body = serde_json::json!({
        "principal_person_id": principal_id.to_string(),
        "required_evidence_slots": ["draft_will_document"]
    })
    .to_string();

    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/will-prep-sa")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap().to_string();

    let document_id = Uuid::new_v4();
    let blob_path = storage_dir.join(format!("{}.bin", document_id));
    std::fs::write(&blob_path, b"draft will content").unwrap();
    let sha256 = sha256_bytes(b"draft will content");

    sqlx::query(
        "INSERT INTO documents (document_id, principal_id, document_type, title, sensitivity, tags) VALUES ($1, $2, $3::document_type, $4, $5::sensitivity_tier, $6)",
    )
    .bind(document_id)
    .bind(principal_id)
    .bind("will")
    .bind("Draft Will")
    .bind("amber")
    .bind(Vec::<String>::new())
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO document_versions (document_id, blob_ref, sha256, byte_size, mime_type) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(document_id)
    .bind(format!("file://{}", blob_path.display()))
    .bind(&sha256)
    .bind(18_i64)
    .bind("application/pdf")
    .execute(&pool)
    .await
    .unwrap();

    let attach_body = serde_json::json!({"document_id": document_id.to_string()}).to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/draft_will_document"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/cases/{case_id}/export"))
                .header("authorization", format!("Bearer {}", token_read()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let download_url = value.get("download_url").and_then(|v| v.as_str()).unwrap();
    let bundle_path = download_url.strip_prefix("file://").unwrap();

    let manifest_path = std::path::Path::new(bundle_path).join("manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["case_type"].as_str().unwrap(), "will_prep_sa");

    let instructions_path = std::path::Path::new(bundle_path).join("witnessing_instructions.md");
    let instructions = std::fs::read_to_string(&instructions_path).unwrap();
    assert!(instructions.contains("two competent witnesses"));
    assert!(instructions.contains("present simultaneously"));
}

#[tokio::test]
async fn deceased_estate_sa_attach_and_export() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    reset_db(&pool).await.unwrap();

    let storage_dir = unique_dir("estate-storage");
    let export_dir = unique_dir("estate-exports");
    std::fs::create_dir_all(&storage_dir).unwrap();
    std::fs::create_dir_all(&export_dir).unwrap();
    unsafe {
        std::env::set_var(
            "LOCAL_STORAGE_DIR",
            storage_dir.to_string_lossy().to_string(),
        );
        std::env::set_var("LOCAL_EXPORT_DIR", export_dir.to_string_lossy().to_string());
    }

    let principal_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let app = case_service::router();
    let body = serde_json::json!({
        "deceased_person_id": "00000000-0000-0000-0000-000000000033",
        "executor_person_id": "00000000-0000-0000-0000-000000000044",
        "estimated_estate_value_zar": 300000.0,
        "required_evidence_slots": ["death_certificate"]
    })
    .to_string();

    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/deceased-estate-sa")
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
    let case_id = value.get("case_id").and_then(|v| v.as_str()).unwrap().to_string();

    let document_id = Uuid::new_v4();
    let blob_path = storage_dir.join(format!("{}.bin", document_id));
    std::fs::write(&blob_path, b"death certificate content").unwrap();
    let sha256 = sha256_bytes(b"death certificate content");

    sqlx::query(
        "INSERT INTO documents (document_id, principal_id, document_type, title, sensitivity, tags) VALUES ($1, $2, $3::document_type, $4, $5::sensitivity_tier, $6)",
    )
    .bind(document_id)
    .bind(principal_id)
    .bind("other")
    .bind("Death Certificate")
    .bind("red")
    .bind(Vec::<String>::new())
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO document_versions (document_id, blob_ref, sha256, byte_size, mime_type) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(document_id)
    .bind(format!("file://{}", blob_path.display()))
    .bind(&sha256)
    .bind(25_i64)
    .bind("application/pdf")
    .execute(&pool)
    .await
    .unwrap();

    let attach_body = serde_json::json!({"document_id": document_id.to_string()}).to_string();
    let response = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/death_certificate"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token_write()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/cases/{case_id}/export"))
                .header("authorization", format!("Bearer {}", token_read()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let download_url = value.get("download_url").and_then(|v| v.as_str()).unwrap();
    let bundle_path = download_url.strip_prefix("file://").unwrap();

    let manifest_path = std::path::Path::new(bundle_path).join("manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    assert_eq!(
        manifest["case_type"].as_str().unwrap(),
        "deceased_estate_reporting_sa"
    );

    let instructions_path = std::path::Path::new(bundle_path).join("instructions.md");
    let instructions = std::fs::read_to_string(&instructions_path).unwrap();
    assert!(instructions.contains("Letters of Executorship"));
    assert!(instructions.contains("Letters of Authority"));
}
