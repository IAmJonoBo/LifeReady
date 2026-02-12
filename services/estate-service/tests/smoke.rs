use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use lifeready_auth::{AccessLevel, AuthConfig, Claims, Role, SensitivityTier};
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

fn auth_token(tiers: Vec<SensitivityTier>, access: AccessLevel) -> String {
    let config = AuthConfig::new("test-secret-32-chars-minimum!!");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::Principal,
        tiers,
        access,
        None,
        300,
    );
    config.issue_token(&claims).expect("token")
}

#[tokio::test]
async fn healthz_exists() {
    init_env();
    let app = estate_service::router();
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn unauthenticated_requests_return_problem_json_with_request_id() {
    init_env();
    let app = estate_service::router();
    let req = Request::builder()
        .uri("/v1/people")
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
async fn create_person_returns_created() {
    init_env();
    let app = estate_service::router();
    let body = serde_json::json!({"full_name": "Ada"}).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/people")
        .header("content-type", "application/json")
        .header(
            "authorization",
            format!(
                "Bearer {}",
                auth_token(vec![SensitivityTier::Amber], AccessLevel::LimitedWrite)
            ),
        )
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn list_people_returns_ok() {
    init_env();
    let app = estate_service::router();
    let req = Request::builder()
        .uri("/v1/people")
        .header(
            "authorization",
            format!(
                "Bearer {}",
                auth_token(vec![SensitivityTier::Amber], AccessLevel::ReadOnlyAll)
            ),
        )
        .body(Body::empty())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn create_asset_returns_created() {
    init_env();
    let app = estate_service::router();
    let body = serde_json::json!({"category": "bank", "label": "Savings", "sensitivity": "red"})
        .to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/assets")
        .header("content-type", "application/json")
        .header(
            "authorization",
            format!(
                "Bearer {}",
                auth_token(vec![SensitivityTier::Red], AccessLevel::LimitedWrite)
            ),
        )
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn create_instruction_returns_created() {
    init_env();
    let app = estate_service::router();
    let body = serde_json::json!({"title": "Note", "body": "Remember"}).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/instructions")
        .header("content-type", "application/json")
        .header(
            "authorization",
            format!(
                "Bearer {}",
                auth_token(vec![SensitivityTier::Amber], AccessLevel::LimitedWrite)
            ),
        )
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn create_role_grant_returns_created() {
    init_env();
    let app = estate_service::router();
    let body = serde_json::json!({
        "person_id": "00000000-0000-0000-0000-000000000002",
        "role": "proxy",
        "scope": {"access_level": "write:limited"}
    })
    .to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/v1/roles/grants")
        .header("content-type", "application/json")
        .header(
            "authorization",
            format!(
                "Bearer {}",
                auth_token(vec![SensitivityTier::Amber], AccessLevel::LimitedWrite)
            ),
        )
        .body(Body::from(body))
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}
