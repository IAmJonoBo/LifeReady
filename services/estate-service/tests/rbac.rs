use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use lifeready_auth::{AccessLevel, AuthConfig, Claims, Role, SensitivityTier};
use tower::util::ServiceExt;

#[tokio::test]
async fn create_person_requires_auth() {
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
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
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
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

#[tokio::test]
async fn create_person_allows_valid_token() {
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
    let app = estate_service::router();
    let body = serde_json::json!({"full_name": "Ada Lovelace"}).to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/people")
                .header("content-type", "application/json")
                .header(
                    "authorization",
                    format!("Bearer {}", test_token_with_tier(SensitivityTier::Amber)),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
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

fn test_token_with_tier(tier: SensitivityTier) -> String {
    let config = AuthConfig::new("test-secret");
    let claims = Claims::new(
        "00000000-0000-0000-0000-000000000001",
        Role::Principal,
        vec![tier],
        AccessLevel::LimitedWrite,
        None,
        300,
    );
    config.issue_token(&claims).expect("token")
}
