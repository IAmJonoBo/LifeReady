use axum::Router;
use std::future::{pending, Future};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    init_tracing("vault_service=info,tower_http=info");

    let _db = vault_service::check_db().await;
    let addr = vault_service::addr_from_env(8083);

    tracing::info!(%addr, "vault-service listening");
    let listener = TcpListener::bind(addr).await.unwrap();
    run_with_listener(listener, pending()).await.unwrap();
}

fn init_tracing(default_filter: &str) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| default_filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn build_app() -> Router {
    vault_service::router().layer(TraceLayer::new_for_http())
}

async fn run_with_listener<F>(listener: TcpListener, shutdown: F) -> std::io::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    axum::serve(listener, build_app())
        .with_graceful_shutdown(shutdown)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tokio::sync::oneshot;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn run_with_listener_stops_on_shutdown() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let (tx, rx) = oneshot::channel::<()>();
        let handle = tokio::spawn(async move {
            run_with_listener(listener, async move {
                let _ = rx.await;
            })
            .await
        });
        let _ = tx.send(());
        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn build_app_exposes_healthz() {
        let app = build_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
