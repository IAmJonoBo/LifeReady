use axum::{routing::get, Router};
use std::net::SocketAddr;

pub fn router() -> Router {
    Router::new().route("/healthz", get(healthz))
}

async fn healthz() -> &'static str {
    "ok"
}

pub fn addr_from_env(default_port: u16) -> SocketAddr {
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(default_port);
    format!("{host}:{port}").parse().expect("valid host:port")
}
