use crate::routes::handlers::health as handlers;
use axum::routing::get;
use axum::Router;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "服务健康", content_type = "application/json")
    )
)]
pub async fn health_handler() -> axum::Json<serde_json::Value> {
    handlers::health_handler().await
}

#[utoipa::path(
    get,
    path = "/ready",
    tag = "health",
    responses(
        (status = 200, description = "服务就绪状态", content_type = "application/json")
    )
)]
pub async fn readiness_handler(
    axum::Extension(state): axum::Extension<std::sync::Arc<crate::state::AppState>>,
) -> axum::Json<serde_json::Value> {
    handlers::readiness_handler(axum::Extension(state)).await
}

pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health_handler,
        readiness_handler,
    ),
    tags(
        (name = "health", description = "健康检查相关接口"),
    )
)]
pub struct HealthApiDoc;
