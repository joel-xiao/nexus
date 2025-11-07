use nexus::*;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    monitor::init_logging()?;

    if std::env::var("JAEGER_ENDPOINT").is_ok() {
        let _ = monitor::init_tracing("nexus");
    }

    let state = Arc::new(state::AppState::new());

    let prometheus_metrics = Arc::new(monitor::PrometheusMetrics::new()?);

    let app = create_app(state, prometheus_metrics, true);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("Nexus running on http://{}", addr);
    tracing::info!("Swagger UI available at http://{}/docs", addr);
    tracing::info!("Prometheus metrics available at http://{}/metrics", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
