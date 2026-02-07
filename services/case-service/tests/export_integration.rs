use audit_verifier::verify_bundle;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use lifeready_auth::{AccessLevel, AuthConfig, Claims, Role, SensitivityTier};
use serde_json::Value;
use sha2::Digest;
use sqlx::PgPool;
use tempfile::TempDir;
use tower::util::ServiceExt;

#[tokio::test]
async fn export_bundle_verifies_and_detects_tamper() {
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
    }
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("DATABASE_URL not set; skipping export integration test");
            return;
        }
    };
    let temp = TempDir::new().expect("temp dir");
    let storage_dir = temp.path().join("storage");
    let export_dir = temp.path().join("exports");
    let audit_export_dir = temp.path().join("audit");
    std::fs::create_dir_all(&storage_dir).expect("storage dir");
    unsafe {
        std::env::set_var(
            "LOCAL_STORAGE_DIR",
            storage_dir.to_string_lossy().to_string(),
        );
        std::env::set_var("LOCAL_EXPORT_DIR", export_dir.to_string_lossy().to_string());
        std::env::set_var(
            "AUDIT_EXPORT_DIR",
            audit_export_dir.to_string_lossy().to_string(),
        );
    }

    let pool = PgPool::connect(&database_url).await.expect("db connect");
    reset_db(&pool).await;

    let token = test_token();
    let vault = vault_service::router();
    let case_app = case_service::router();
    let audit_app = audit_service::app();

    let doc_path = storage_dir.join("doc-1.txt");
    std::fs::write(&doc_path, b"test document").expect("write doc");
    let sha256 = sha256_file(&doc_path);

    let init_body = serde_json::json!({
        "document_type": "other",
        "title": "Test Doc",
        "sensitivity": "amber",
        "tags": []
    })
    .to_string();

    let init_res = call_json(
        vault.clone(),
        Request::builder()
            .method("POST")
            .uri("/v1/documents")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(init_body))
            .unwrap(),
    )
    .await;
    assert_eq!(init_res.status(), StatusCode::CREATED);
    let init_value = body_json(init_res).await;
    let document_id = init_value
        .get("document_id")
        .and_then(|v| v.as_str())
        .expect("document_id")
        .to_string();

    let commit_body = serde_json::json!({
        "blob_ref": doc_path.to_string_lossy(),
        "sha256": sha256,
        "byte_size": 13,
        "mime_type": "text/plain"
    })
    .to_string();

    let commit_res = call_json(
        vault,
        Request::builder()
            .method("POST")
            .uri(format!("/v1/documents/{document_id}/versions"))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(commit_body))
            .unwrap(),
    )
    .await;
    assert_eq!(commit_res.status(), StatusCode::CREATED);

    let case_body = serde_json::json!({
        "subject_person_id": "00000000-0000-0000-0000-000000000010",
        "applicant_person_id": "00000000-0000-0000-0000-000000000020"
    })
    .to_string();

    let case_res = call_json(
        case_app.clone(),
        Request::builder()
            .method("POST")
            .uri("/v1/cases/mhca39")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(case_body))
            .unwrap(),
    )
    .await;
    assert_eq!(case_res.status(), StatusCode::CREATED);
    let case_value = body_json(case_res).await;
    let case_id = case_value
        .get("case_id")
        .and_then(|v| v.as_str())
        .expect("case_id")
        .to_string();

    for slot in default_slots() {
        let attach_body = serde_json::json!({ "document_id": document_id }).to_string();

        let attach_res = call_json(
            case_app.clone(),
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/cases/{case_id}/evidence/{slot}"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(attach_body))
                .unwrap(),
        )
        .await;
        assert_eq!(attach_res.status(), StatusCode::OK);
    }

    let audit_body = serde_json::json!({
        "actor_principal_id": "00000000-0000-0000-0000-000000000001",
        "action": "case.export",
        "tier": "amber",
        "case_id": case_id,
        "payload": {"note": "export"}
    })
    .to_string();

    let audit_res = call_json(
        audit_app,
        Request::builder()
            .method("POST")
            .uri("/v1/audit/events")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(audit_body))
            .unwrap(),
    )
    .await;
    assert_eq!(audit_res.status(), StatusCode::CREATED);

    let export_res = call_json(
        case_app,
        Request::builder()
            .method("POST")
            .uri(format!("/v1/cases/{case_id}/export"))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(export_res.status(), StatusCode::OK);
    let export_value = body_json(export_res).await;
    let download_url = export_value
        .get("download_url")
        .and_then(|v| v.as_str())
        .expect("download_url");
    let bundle_path = download_url.strip_prefix("file://").expect("file url");

    verify_bundle(std::path::Path::new(bundle_path)).expect("bundle valid");

    let manifest_path = std::path::Path::new(bundle_path).join("manifest.json");
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    manifest["case_id"] = Value::String("tampered".into());
    std::fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

    let tampered = verify_bundle(std::path::Path::new(bundle_path));
    assert!(tampered.is_err());
}

async fn reset_db(pool: &PgPool) {
    sqlx::query(
        "TRUNCATE audit_events, document_versions, documents, mhca39_evidence, mhca39_cases, case_artifacts, cases RESTART IDENTITY",
    )
    .execute(pool)
    .await
    .unwrap();
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

fn sha256_file(path: &std::path::Path) -> String {
    let bytes = std::fs::read(path).expect("read file");
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn default_slots() -> Vec<&'static str> {
    vec![
        "id_subject",
        "id_applicant",
        "address_subject",
        "asset_summary",
        "medical_evidence_1",
        "medical_evidence_2",
    ]
}

async fn call_json(app: Router, request: Request<Body>) -> axum::response::Response {
    app.oneshot(request).await.unwrap()
}

async fn body_json(response: axum::response::Response) -> Value {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}
