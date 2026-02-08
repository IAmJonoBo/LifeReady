use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
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
    let app = case_service::router();
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
