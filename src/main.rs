// main.rs 作为二进制入口点
use nexus::*;

use std::net::SocketAddr;
use tokio::net::TcpListener;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化结构化 JSON 日志和异步写入
    monitor::init_logging()?;
    
    // 初始化 Jaeger tracing（如果配置了 JAEGER_ENDPOINT）
    if std::env::var("JAEGER_ENDPOINT").is_ok() {
        let _ = monitor::init_tracing("nexus");
    }
    
    let state = Arc::new(state::AppState::new());
    
    // 初始化 Prometheus metrics
    let prometheus_metrics = Arc::new(monitor::PrometheusMetrics::new()?);

    // 创建应用实例（包含错误处理中间件）
    let app = create_app(state, prometheus_metrics, true);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(&addr).await?;
    
    tracing::info!("Nexus running on http://{}", addr);
    tracing::info!("Swagger UI available at http://{}/docs", addr);
    tracing::info!("Prometheus metrics available at http://{}/metrics", addr);

    axum::serve(listener, app).await?;

    Ok(())
}















