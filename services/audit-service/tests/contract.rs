use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use lifeready_auth::{AccessLevel, AuthConfig, Claims, Role, SensitivityTier};
use sqlx::PgPool;
use std::sync::Once;
use tower::util::ServiceExt;
use uuid::Uuid;

fn init_env() {
    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe {
        std::env::set_var("LIFEREADY_ENV", "dev");
        std::env::set_var("JWT_SECRET", "test-secret-32-chars-minimum!!");
    });
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
async fn export_returns_head_hash() {
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

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let head_hash = value.get("head_hash").and_then(|v| v.as_str()).unwrap();
    assert_eq!(head_hash.len(), 64);
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

async fn setup_db() -> Option<PgPool> {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("DATABASE_URL not set; skipping audit-service tests");
            return None;
        }
    };
    PgPool::connect(&database_url).await.ok()
}
