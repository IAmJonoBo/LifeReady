#![allow(clippy::await_holding_lock)]

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use lifeready_auth::{AccessLevel, AuthConfig, Claims, Role, SensitivityTier};
use sqlx::PgPool;
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Mutex, Once};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;
use uuid::Uuid;

fn init_env() {
    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe {
        std::env::set_var("LIFEREADY_ENV", "dev");
        std::env::set_var("JWT_SECRET", "test-secret-32-chars-minimum!!");
    });
}

static ENV_LOCK: Mutex<()> = Mutex::new(());

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

#[tokio::test]
async fn healthz_exists() {
    init_env();
    let app = audit_service::app();
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn readyz_exists() {
    init_env();
    let app = audit_service::app();
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        payload.get("status").and_then(|v| v.as_str()),
        Some("not_ready")
    );
}

#[tokio::test]
async fn unauthenticated_requests_return_problem_json_with_request_id() {
    init_env();
    let app = audit_service::app();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/audit/events")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
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
async fn append_event_returns_database_unavailable_without_pool() {
    init_env();
    with_env_async(&[("DATABASE_URL", None)], || async {
        let app = audit_service::app();
        let body = serde_json::json!({
            "actor_principal_id": "00000000-0000-0000-0000-000000000001",
            "action": "case.export",
            "tier": "green",
            "case_id": "00000000-0000-0000-0000-000000000010",
            "payload": {"ok": true}
        })
        .to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/audit/events")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", test_token()))
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
async fn append_event_returns_hashes() {
    init_env();
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    sqlx::query("TRUNCATE audit_events")
        .execute(&pool)
        .await
        .unwrap();
    let app = audit_service::app();
    let payload = serde_json::json!({"ip": "127.0.0.1"});
    let body = serde_json::json!({
        "actor_principal_id": "00000000-0000-0000-0000-000000000001",
        "action": "case.export",
        "tier": "green",
        "case_id": "00000000-0000-0000-0000-000000000010",
        "payload": payload
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit/events")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", test_token()))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let prev_hash = value.get("prev_hash").and_then(|v| v.as_str()).unwrap();
    let event_hash = value.get("event_hash").and_then(|v| v.as_str()).unwrap();
    let event_id = value.get("event_id").and_then(|v| v.as_str()).unwrap();
    let created_at = value.get("created_at").and_then(|v| v.as_str()).unwrap();

    assert_eq!(prev_hash.len(), 64);
    assert_eq!(event_hash.len(), 64);
    assert!(!event_id.is_empty());
    assert!(!created_at.is_empty());
}

#[tokio::test]
async fn append_event_rejects_invalid_tier() {
    init_env();
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    sqlx::query("TRUNCATE audit_events")
        .execute(&pool)
        .await
        .unwrap();
    let app = audit_service::app();
    let body = serde_json::json!({
        "actor_principal_id": "00000000-0000-0000-0000-000000000001",
        "action": "case.export",
        "tier": "purple",
        "case_id": "00000000-0000-0000-0000-000000000010",
        "payload": {"ok": true}
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit/events")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", test_token()))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn append_event_rejects_invalid_actor_id() {
    init_env();
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    sqlx::query("TRUNCATE audit_events")
        .execute(&pool)
        .await
        .unwrap();
    let app = audit_service::app();
    let body = serde_json::json!({
        "actor_principal_id": "not-a-uuid",
        "action": "case.export",
        "tier": "green",
        "case_id": "00000000-0000-0000-0000-000000000010",
        "payload": {"ok": true}
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit/events")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", test_token()))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn append_event_rejects_invalid_case_id() {
    init_env();
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    sqlx::query("TRUNCATE audit_events")
        .execute(&pool)
        .await
        .unwrap();
    let app = audit_service::app();
    let body = serde_json::json!({
        "actor_principal_id": "00000000-0000-0000-0000-000000000001",
        "action": "case.export",
        "tier": "green",
        "case_id": "not-a-uuid",
        "payload": {"ok": true}
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit/events")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", test_token()))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn append_event_allows_missing_case_id() {
    init_env();
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    sqlx::query("TRUNCATE audit_events")
        .execute(&pool)
        .await
        .unwrap();
    let app = audit_service::app();
    let body = serde_json::json!({
        "actor_principal_id": "00000000-0000-0000-0000-000000000001",
        "action": "case.export",
        "tier": "green",
        "case_id": null,
        "payload": {"ok": true}
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit/events")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", test_token()))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn export_returns_database_unavailable_without_pool() {
    init_env();
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
    with_env_async(&[("DATABASE_URL", None)], || async {
        let app = audit_service::app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/audit/export")
                    .header("authorization", format!("Bearer {}", read_only_token()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    })
    .await;
}

#[tokio::test]
async fn export_returns_head_hash() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    let export_dir = unique_dir("audit-export");
    std::fs::create_dir_all(&export_dir).unwrap();
    with_env_async(&[("AUDIT_EXPORT_DIR", export_dir.to_str())], || async {
        sqlx::query("TRUNCATE audit_events")
            .execute(&pool)
            .await
            .unwrap();
        let app = audit_service::app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/audit/export")
                    .header("authorization", format!("Bearer {}", test_token()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let head_hash = value.get("head_hash").and_then(|v| v.as_str()).unwrap();
        let events_sha = value.get("events_sha256").and_then(|v| v.as_str()).unwrap();
        let download_url = value.get("download_url").and_then(|v| v.as_str()).unwrap();
        assert_eq!(head_hash, &"0".repeat(64));
        assert_eq!(head_hash.len(), 64);
        assert_eq!(events_sha.len(), 64);

        let export_path = download_url.trim_start_matches("file://");
        assert!(std::path::Path::new(export_path).exists());
    })
    .await;
}

#[tokio::test]
async fn export_returns_latest_event_hash() {
    init_env();
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    let export_dir = unique_dir("audit-export");
    std::fs::create_dir_all(&export_dir).unwrap();
    with_env_async(&[("AUDIT_EXPORT_DIR", export_dir.to_str())], || async {
        sqlx::query("TRUNCATE audit_events")
            .execute(&pool)
            .await
            .unwrap();
        let app = audit_service::app();

        let body = serde_json::json!({
            "actor_principal_id": "00000000-0000-0000-0000-000000000001",
            "action": "case.export",
            "tier": "green",
            "case_id": "00000000-0000-0000-0000-000000000010",
            "payload": {"ok": true}
        })
        .to_string();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/audit/events")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", test_token()))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let event_hash = value.get("event_hash").and_then(|v| v.as_str()).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/audit/export")
                    .header("authorization", format!("Bearer {}", test_token()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let head_hash = value.get("head_hash").and_then(|v| v.as_str()).unwrap();
        let download_url = value.get("download_url").and_then(|v| v.as_str()).unwrap();
        assert_eq!(head_hash, event_hash);

        let export_path = download_url.trim_start_matches("file://");
        assert!(std::path::Path::new(export_path).exists());
    })
    .await;
}

#[tokio::test]
async fn append_event_sets_prev_hash_from_latest_event() {
    init_env();
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    sqlx::query("TRUNCATE audit_events")
        .execute(&pool)
        .await
        .unwrap();
    let app = audit_service::app();

    let body = serde_json::json!({
        "actor_principal_id": "00000000-0000-0000-0000-000000000001",
        "action": "case.export",
        "tier": "green",
        "case_id": "00000000-0000-0000-0000-000000000010",
        "payload": {"ok": true}
    })
    .to_string();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit/events")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", test_token()))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let event_hash = value.get("event_hash").and_then(|v| v.as_str()).unwrap();

    let body = serde_json::json!({
        "actor_principal_id": "00000000-0000-0000-0000-000000000001",
        "action": "case.export",
        "tier": "green",
        "case_id": "00000000-0000-0000-0000-000000000010",
        "payload": {"ok": false}
    })
    .to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit/events")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", test_token()))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let prev_hash = value.get("prev_hash").and_then(|v| v.as_str()).unwrap();
    assert_eq!(prev_hash, event_hash);
}

fn test_token() -> String {
    let config = AuthConfig::new("test-secret");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::Principal,
        vec![SensitivityTier::Red],
        AccessLevel::LimitedWrite,
        None,
        300,
    );
    config.issue_token(&claims).expect("token")
}

fn read_only_token() -> String {
    let config = AuthConfig::new("test-secret");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::Principal,
        vec![SensitivityTier::Red],
        AccessLevel::ReadOnlyAll,
        None,
        300,
    );
    config.issue_token(&claims).expect("token")
}

#[tokio::test]
async fn append_event_rejects_missing_scope() {
    init_env();
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    sqlx::query("TRUNCATE audit_events")
        .execute(&pool)
        .await
        .unwrap();
    let app = audit_service::app();
    let body = serde_json::json!({
        "actor_principal_id": "00000000-0000-0000-0000-000000000001",
        "action": "case.export",
        "tier": "green",
        "case_id": "00000000-0000-0000-0000-000000000010",
        "payload": {"ok": true}
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit/events")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", read_only_token()))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn export_rejects_missing_scope() {
    init_env();
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    sqlx::query("TRUNCATE audit_events")
        .execute(&pool)
        .await
        .unwrap();
    let app = audit_service::app();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/audit/export")
                .header("authorization", format!("Bearer {}", test_token()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

async fn setup_db() -> Option<PgPool> {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("DATABASE_URL not set; skipping audit-service tests");
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
        "CREATE INDEX IF NOT EXISTS idx_audit_case_time ON audit_events(case_id, created_at DESC);",
    )
    .execute(pool)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_actor_time ON audit_events(actor_principal_id, created_at DESC);")
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
async fn append_event_rejects_insufficient_role() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    sqlx::query("TRUNCATE audit_events")
        .execute(&pool)
        .await
        .unwrap();

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

    let app = audit_service::app();
    let body = serde_json::json!({
        "action": "test",
        "tier": "amber",
        "payload": {}
    })
    .to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit/events")
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
async fn append_event_rejects_insufficient_tier() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    sqlx::query("TRUNCATE audit_events")
        .execute(&pool)
        .await
        .unwrap();

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

    let app = audit_service::app();
    let body = serde_json::json!({
        "action": "test",
        "tier": "amber",
        "payload": {}
    })
    .to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audit/events")
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
async fn export_rejects_insufficient_role() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    sqlx::query("TRUNCATE audit_events")
        .execute(&pool)
        .await
        .unwrap();

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

    let app = audit_service::app();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/audit/export")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn export_rejects_insufficient_tier() {
    init_env();
    let pool = match setup_db().await {
        Some(pool) => pool,
        None => return,
    };
    sqlx::query("TRUNCATE audit_events")
        .execute(&pool)
        .await
        .unwrap();

    let config = AuthConfig::new("test-secret-32-chars-minimum!!");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::Principal,
        vec![SensitivityTier::Green],
        AccessLevel::ReadOnlyAll,
        None,
        300,
    );
    let token = config.issue_token(&claims).expect("token");

    let app = audit_service::app();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/audit/export")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
