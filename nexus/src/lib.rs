pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod integration;
pub mod monitor;
pub mod routes;
pub mod state;

use crate::monitor::PrometheusMetrics;
use axum::extract::Request;
use axum::{middleware::Next, Extension, Router};
use routes::mod_routes;
use state::AppState;
use std::sync::Arc;
use utoipa_swagger_ui::SwaggerUi;

/// 创建应用实例（用于生产环境和测试）
pub fn create_app(
    state: Arc<AppState>,
    prometheus_metrics: Arc<PrometheusMetrics>,
    include_middleware: bool,
) -> Router {
    let openapi = routes::api_doc::ApiDoc::openapi();

    let mut app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi.clone()))
        .merge(routes::health::health_routes())
        .nest("/api", mod_routes())
        .route("/metrics", axum::routing::get(metrics_handler))
        .layer(Extension(state.clone()))
        .layer(Extension(prometheus_metrics.clone()));

    if include_middleware {
        app = app.layer(axum::middleware::from_fn(
            |req: Request, next: Next| async move { error_middleware(req, next).await },
        ));
    }

    app
}

/// 创建测试用的应用实例（不包含错误处理中间件）
pub fn create_test_app() -> Router {
    let state = Arc::new(AppState::new());
    let prometheus_metrics =
        Arc::new(PrometheusMetrics::new().expect("Failed to create PrometheusMetrics"));
    create_app(state, prometheus_metrics, false)
}

/// Prometheus metrics endpoint
pub async fn metrics_handler(Extension(metrics): Extension<Arc<PrometheusMetrics>>) -> String {
    metrics.gather().unwrap_or_else(|e| {
        tracing::error!("Failed to gather metrics: {}", e);
        String::new()
    })
}

/// 错误处理中间件
pub async fn error_middleware(req: Request, next: Next) -> axum::response::Response {
    let metrics = req.extensions().get::<Arc<PrometheusMetrics>>().cloned();
    use std::time::Instant;

    let start = Instant::now();
    let path = req.uri().path().to_string();
    let method = req.method().clone();

    if let Some(ref m) = metrics {
        m.http_requests_total.inc();

        if let Some(content_length) = req.headers().get("content-length") {
            if let Ok(s) = content_length.to_str() {
                if let Ok(size) = s.parse::<f64>() {
                    m.http_request_size_bytes.observe(size);
                }
            }
        }
    }

    let response = next.run(req).await;

    let duration = start.elapsed().as_secs_f64();
    if let Some(ref m) = metrics {
        m.http_request_duration_seconds.observe(duration);

        if let Some(content_length) = response.headers().get("content-length") {
            if let Ok(s) = content_length.to_str() {
                if let Ok(size) = s.parse::<f64>() {
                    m.http_response_size_bytes.observe(size);
                }
            }
        }
    }

    if response.status().is_server_error() {
        tracing::warn!(
            method = %method,
            path = %path,
            status = %response.status(),
            duration_seconds = duration,
            "HTTP request failed"
        );
    }

    response
}
