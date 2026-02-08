use axum::http::StatusCode;
use lifeready_auth::{RequestContext, RequestId};
use lifeready_policy::{Role, SensitivityTier};
use uuid::Uuid;

#[test]
fn proxy_denied_red_document() {
    let ctx = RequestContext {
        request_id: RequestId(Uuid::new_v4()),
        principal_id: "proxy-user".into(),
        roles: vec![Role::Proxy],
        allowed_tiers: vec![SensitivityTier::Green, SensitivityTier::Amber],
        scopes: vec!["read:all".into()],
        expires_at: chrono::Utc::now(),
        email: None,
    };
    let request_id = RequestId(Uuid::new_v4());

    let response = vault_service::ensure_document_access(&ctx, SensitivityTier::Red, request_id)
        .expect_err("expected forbidden");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
