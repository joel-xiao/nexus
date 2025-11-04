use axum::{Router, Json};
use axum::routing::get;
use utoipa::OpenApi;
use crate::routes::handlers::health as handlers;

// ===== 带 OpenAPI 注解的包装函数（路径和 OpenAPI 定义在一起） =====

/// 健康检查端点
/// 
/// 用于 Kubernetes 存活探针（Liveness Probe），检查服务是否正常运行。
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "服务健康", body = handlers::HealthResponse)
    )
)]
pub async fn health_handler() -> Json<handlers::HealthResponse> {
    handlers::health_handler().await
}

/// 就绪检查端点
/// 
/// 用于 Kubernetes 就绪探针（Readiness Probe），检查服务是否准备好接收流量。
#[utoipa::path(
    get,
    path = "/ready",
    tag = "health",
    responses(
        (status = 200, description = "服务就绪状态", body = handlers::ReadinessResponse)
    )
)]
pub async fn readiness_handler(
    axum::Extension(state): axum::Extension<std::sync::Arc<crate::state::AppState>>,
) -> Json<handlers::ReadinessResponse> {
    handlers::readiness_handler(axum::Extension(state)).await
}

// ===== 路由定义 =====

/// 注册 health 相关的路由
pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
}

// ===== OpenAPI 文档片段 =====
#[derive(OpenApi)]
#[openapi(
    paths(
        health_handler,
        readiness_handler,
    ),
    components(schemas(
        handlers::HealthResponse,
        handlers::ReadinessResponse,
        handlers::HealthChecks,
    )),
    tags(
        (name = "health", description = "健康检查相关接口"),
    )
)]
pub struct HealthApiDoc;

// 重新导出类型
pub use handlers::{HealthResponse, ReadinessResponse, HealthChecks};







