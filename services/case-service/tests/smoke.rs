use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::Empty;
use tower::ServiceExt;

#[tokio::test]
async fn healthz_exists() {
    let app = case_service::router();
    let req = Request::builder()
        .uri("/healthz")
        .body(Empty::<Body>::new())
        .unwrap();

    let res = axum::Router::into_service(app).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
