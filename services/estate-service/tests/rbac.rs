use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use lifeready_auth::{AccessLevel, AuthConfig, Claims, Role, SensitivityTier};
use tower::util::ServiceExt;

#[tokio::test]
async fn create_person_requires_auth() {
    std::env::set_var("JWT_SECRET", "test-secret");
    let app = estate_service::router();
    let body = serde_json::json!({"full_name": "Ada Lovelace"}).to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/people")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_person_denies_insufficient_tier() {
    std::env::set_var("JWT_SECRET", "test-secret");
    let app = estate_service::router();
    let body = serde_json::json!({"full_name": "Ada Lovelace"}).to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/people")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", test_token()))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

fn test_token() -> String {
    let config = AuthConfig::new("test-secret");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::Principal,
        vec![SensitivityTier::Green],
        AccessLevel::LimitedWrite,
        None,
        300,
    );
    config.issue_token(&claims).expect("token")
}
