use axum::{Router, Extension};
use axum::routing::{get, post};
use utoipa::OpenApi;
use crate::routes::handlers::config::import_export as handlers;
use std::sync::Arc;
use crate::state::AppState;

// ===== 带 OpenAPI 注解的包装函数（路径和 OpenAPI 定义在一起） =====

/// 导出配置
#[utoipa::path(
    get,
    path = "/api/config/export",
    tag = "config-import-export",
    responses(
        (status = 200, description = "配置 JSON")
    )
)]
pub async fn export_config(
    Extension(state): Extension<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    handlers::export_config(Extension(state)).await
}

/// 导入配置
#[utoipa::path(
    post,
    path = "/api/config/import",
    tag = "config-import-export",
    request_body(content = serde_json::Value, description = "配置 JSON", content_type = "application/json"),
    responses(
        (status = 200, description = "成功导入配置")
    )
)]
pub async fn import_config(
    Extension(state): Extension<Arc<AppState>>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    handlers::import_config(Extension(state), axum::Json(payload)).await
}

// ===== OpenAPI 文档片段 =====
#[derive(OpenApi)]
#[openapi(
    paths(
        export_config,
        import_config,
    ),
    tags(
        (name = "config-import-export", description = "配置导入导出"),
    )
)]
pub struct ImportExportApiDoc;

// ===== 路由定义（路径 + OpenAPI 在一起） =====

/// 注册 import_export 相关的路由
pub fn import_export_routes() -> Router {
    Router::new()
        .route("/export", get(export_config))
        .route("/import", post(import_config))
}
