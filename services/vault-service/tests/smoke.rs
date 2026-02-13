use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use lifeready_auth::{AccessLevel, AuthConfig, Claims, Role, SensitivityTier};
use serde_json::Value;
use std::sync::Once;
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

fn read_token() -> String {
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

fn invalid_principal_read_token() -> String {
    let config = AuthConfig::new("test-secret-32-chars-minimum!!");
    let claims = Claims::new(
        "not-a-uuid",
        Role::Principal,
        vec![SensitivityTier::Amber],
        AccessLevel::ReadOnlyAll,
        None,
        300,
    );
    config.issue_token(&claims).expect("token")
}

#[tokio::test]
async fn healthz_exists() {
    init_env();
    let app = vault_service::router();
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
    let app = vault_service::router();
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
    let app = vault_service::router();
    let req = Request::builder()
        .uri("/v1/documents")
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
async fn init_document_returns_database_unavailable_without_pool() {
    init_env();
    let app = vault_service::router();
    let body = serde_json::json!({
        "document_type": "will",
        "title": "My will",
        "sensitivity": "amber"
    })
    .to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/documents")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", auth_token()))
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_documents_returns_database_unavailable_without_pool() {
    init_env();
    let app = vault_service::router();
    let req = Request::builder()
        .uri("/v1/documents")
        .header("authorization", format!("Bearer {}", read_token()))
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn commit_document_rejects_invalid_sha256() {
    init_env();
    let app = vault_service::router();
    let body = serde_json::json!({
        "blob_ref": "auto",
        "sha256": "not-a-sha",
        "byte_size": 1,
        "mime_type": "application/pdf"
    })
    .to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/documents/00000000-0000-0000-0000-000000000010/versions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", auth_token()))
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_document_rejects_invalid_document_id() {
    init_env();
    let app = vault_service::router();
    let req = Request::builder()
        .uri("/v1/documents/not-a-uuid")
        .header("authorization", format!("Bearer {}", read_token()))
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_document_rejects_invalid_principal_id() {
    init_env();
    let app = vault_service::router();
    let req = Request::builder()
        .uri("/v1/documents/00000000-0000-0000-0000-000000000010")
        .header(
            "authorization",
            format!("Bearer {}", invalid_principal_read_token()),
        )
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn download_document_rejects_invalid_document_id() {
    init_env();
    let app = vault_service::router();
    let req = Request::builder()
        .uri("/v1/documents/not-a-uuid/download")
        .header("authorization", format!("Bearer {}", read_token()))
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn download_document_rejects_invalid_principal_id() {
    init_env();
    let app = vault_service::router();
    let req = Request::builder()
        .uri("/v1/documents/00000000-0000-0000-0000-000000000010/download")
        .header(
            "authorization",
            format!("Bearer {}", invalid_principal_read_token()),
        )
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn download_document_returns_database_unavailable_without_pool() {
    init_env();
    let app = vault_service::router();
    let req = Request::builder()
        .uri("/v1/documents/00000000-0000-0000-0000-000000000010/download")
        .header("authorization", format!("Bearer {}", read_token()))
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
