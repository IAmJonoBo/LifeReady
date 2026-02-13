use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use lifeready_auth::{AccessLevel, AuthConfig, Claims, Role, SensitivityTier};
use serde_json::Value;
use sha2::Digest;
use sqlx::{PgPool, Row};
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;
use uuid::Uuid;

fn init_env() {
    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe {
        std::env::set_var("LIFEREADY_ENV", "dev");
        std::env::set_var("JWT_SECRET", "test-secret-32-chars-minimum!!");
        std::env::set_var(
            "DATABASE_URL",
            "postgres://postgres:postgres@127.0.0.1:5432/lifeready",
        );
    });
}

fn auth_token() -> String {
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

fn read_packs_token() -> String {
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

fn invalid_principal_token() -> String {
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

fn read_all_token() -> String {
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

fn unique_dir(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{}-{}", std::process::id(), nanos))
}

async fn setup_db() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&database_url).await.ok()?;
    ensure_schema(&pool).await.ok()?;
    reset_db(&pool).await.ok()?;
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
         CREATE TYPE case_type AS ENUM ('emergency_pack','mhca39','death_readiness'); \
         END IF; \
         END $$;",
    )
    .execute(pool)
    .await?;
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
    Ok(())
}

async fn reset_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "TRUNCATE audit_events, document_versions, documents, mhca39_evidence, mhca39_cases, case_artifacts, cases RESTART IDENTITY CASCADE",
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn healthz_exists() {
    init_env();
    let app = case_service::router();
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn readyz_exists() {
    init_env();
    let app = case_service::router();
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let payload: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        payload.get("status").and_then(|v| v.as_str()),
        Some("not_ready")
    );
}

#[tokio::test]
async fn unauthenticated_requests_return_problem_json_with_request_id() {
    init_env();
    let app = case_service::router();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/cases/emergency-pack")
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    let request_id = res
        .headers()
        .get(lifeready_auth::REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .expect("x-request-id header");
    Uuid::parse_str(request_id).expect("x-request-id must be a UUID");
}

#[tokio::test]
async fn emergency_pack_returns_database_unavailable_without_pool() {
    init_env();
    let app = case_service::router();
    let body =
        serde_json::json!({"directive_document_ids": [], "emergency_contacts": []}).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/cases/emergency-pack")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", auth_token()))
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn mhca39_rejects_invalid_subject_person_id() {
    init_env();
    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "not-a-uuid",
        "applicant_person_id": "00000000-0000-0000-0000-000000000002"
    })
    .to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/cases/mhca39")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", auth_token()))
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn attach_evidence_rejects_invalid_case_id() {
    init_env();
    let app = case_service::router();
    let body = serde_json::json!({
        "document_id": "00000000-0000-0000-0000-000000000003"
    })
    .to_string();
    let req = Request::builder()
        .method("PUT")
        .uri("/v1/cases/not-a-uuid/evidence/slot")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", auth_token()))
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn mhca39_rejects_invalid_applicant_person_id() {
    init_env();
    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000001",
        "applicant_person_id": "not-a-uuid"
    })
    .to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/cases/mhca39")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", auth_token()))
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn mhca39_rejects_invalid_principal_id() {
    init_env();
    let app = case_service::router();
    let body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000001",
        "applicant_person_id": "00000000-0000-0000-0000-000000000002"
    })
    .to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/cases/mhca39")
        .header("content-type", "application/json")
        .header(
            "authorization",
            format!("Bearer {}", invalid_principal_token()),
        )
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn attach_evidence_rejects_invalid_document_id() {
    init_env();
    let app = case_service::router();
    let body = serde_json::json!({"document_id": "not-a-uuid"}).to_string();
    let req = Request::builder()
        .method("PUT")
        .uri("/v1/cases/00000000-0000-0000-0000-000000000010/evidence/slot")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", auth_token()))
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn export_case_rejects_invalid_case_id() {
    init_env();
    let app = case_service::router();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/cases/not-a-uuid/export")
        .header("authorization", format!("Bearer {}", read_packs_token()))
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn mhca39_flow_succeeds_with_database() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };

    let storage_dir = unique_dir("case-smoke-storage");
    let export_dir = unique_dir("case-smoke-export");
    std::fs::create_dir_all(&storage_dir).unwrap();
    std::fs::create_dir_all(&export_dir).unwrap();
    unsafe {
        std::env::set_var(
            "LOCAL_STORAGE_DIR",
            storage_dir.to_str().unwrap_or("storage"),
        );
        std::env::set_var(
            "LOCAL_EXPORT_DIR",
            export_dir.to_str().unwrap_or("exports/cases"),
        );
    }

    let app = case_service::router();
    let create_body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-0000000000aa",
        "applicant_person_id": "00000000-0000-0000-0000-0000000000ab",
        "required_evidence_slots": ["id_subject"]
    })
    .to_string();
    let create_res = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/cases/mhca39")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", auth_token()))
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);
    let create_payload: Value =
        serde_json::from_slice(&create_res.into_body().collect().await.unwrap().to_bytes())
            .unwrap();
    let case_id = create_payload
        .get("case_id")
        .and_then(|value| value.as_str())
        .unwrap()
        .to_string();

    let document_id = Uuid::new_v4();
    let principal_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let blob_path = storage_dir.join(format!("{}.bin", document_id));
    std::fs::write(&blob_path, b"fixture").unwrap();

    sqlx::query(
        "INSERT INTO documents (document_id, principal_id, document_type, title, sensitivity, tags) VALUES ($1, $2, $3::document_type, $4, $5::sensitivity_tier, $6)",
    )
    .bind(document_id)
    .bind(principal_id)
    .bind("id")
    .bind("Subject ID")
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
    .bind(hex::encode(sha2::Sha256::digest(b"fixture")))
    .bind(7_i64)
    .bind("application/octet-stream")
    .execute(&pool)
    .await
    .unwrap();

    let attach_body = serde_json::json!({
        "document_id": document_id.to_string()
    })
    .to_string();
    let attach_res = axum::Router::into_service(app.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{}/evidence/id_subject", case_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", auth_token()))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(attach_res.status(), StatusCode::OK);

    let export_res = axum::Router::into_service(app)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/cases/{}/export", case_id))
                .header("authorization", format!("Bearer {}", read_all_token()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(export_res.status(), StatusCode::OK);

    let exports: i64 = sqlx::query("SELECT COUNT(*) AS count FROM case_artifacts")
        .fetch_one(&pool)
        .await
        .unwrap()
        .try_get("count")
        .unwrap();
    assert!(exports >= 1);
}
