use axum::{
    body::Body,
    http::{Request, StatusCode, header},
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
    });
}

fn auth_token() -> String {
    let config = AuthConfig::new("test-secret-32-chars-minimum!!");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::Principal,
        vec![SensitivityTier::Green],
        AccessLevel::ReadOnlyAll,
        Some("principal@example.com".into()),
        300,
    );
    config.issue_token(&claims).expect("token")
}

#[tokio::test]
async fn healthz_exists() {
    init_env();
    let app = identity_service::router();
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
    let app = identity_service::router();
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
    let app = identity_service::router();
    let req = Request::builder()
        .uri("/v1/me")
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
async fn login_rejects_empty_email() {
    init_env();
    let app = identity_service::router();
    let body = serde_json::json!({"email": ""}).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn login_accepts_email() {
    init_env();
    let app = identity_service::router();
    let body = serde_json::json!({"email": "principal@example.com"}).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn verify_mfa_issues_token() {
    init_env();
    let app = identity_service::router();
    let body = serde_json::json!({
        "challenge_id": "challenge",
        "method": "totp",
        "code": "123456"
    })
    .to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/auth/mfa/verify")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let payload: Value = serde_json::from_slice(&body).unwrap();
    assert!(
        payload
            .get("access_token")
            .and_then(|v| v.as_str())
            .is_some()
    );
}

#[tokio::test]
async fn me_accepts_valid_token() {
    init_env();
    let app = identity_service::router();
    let req = Request::builder()
        .uri("/v1/me")
        .header("authorization", format!("Bearer {}", auth_token()))
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
