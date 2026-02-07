use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn append_event_returns_hashes() {
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
    let app = audit_service::app();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/audit/export")
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
