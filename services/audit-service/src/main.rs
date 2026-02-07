use audit_service::app;

#[tokio::main]
async fn main() {
    let app = app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("server failed");
}

