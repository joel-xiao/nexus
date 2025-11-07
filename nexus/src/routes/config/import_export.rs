use crate::routes::handlers::config::import_export as handlers;
use crate::state::AppState;
use axum::routing::{get, post};
use axum::{Extension, Router};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    path = "/api/config/export",
    tag = "config-import-export",
    responses(
        (status = 200, description = "配置 JSON", content_type = "application/json"),
        (status = 500, description = "导出失败", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn export_config(
    Extension(state): Extension<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    handlers::export_config(Extension(state)).await
}

#[utoipa::path(
    post,
    path = "/api/config/import",
    tag = "config-import-export",
    request_body(content = serde_json::Value, description = "配置 JSON", content_type = "application/json"),
    responses(
        (status = 200, description = "成功导入配置", content_type = "application/json"),
        (status = 500, description = "导入失败", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn import_config(
    Extension(state): Extension<Arc<AppState>>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    handlers::import_config(Extension(state), axum::Json(payload)).await
}

#[derive(OpenApi)]
#[openapi(
    paths(
        export_config,
        import_config,
    ),
    components(schemas(
        crate::routes::common::ErrorResponse,
    )),
    tags(
        (name = "config-import-export", description = "配置导入导出"),
    )
)]
pub struct ImportExportApiDoc;

pub fn import_export_routes() -> Router {
    Router::new()
        .route("/export", get(export_config))
        .route("/import", post(import_config))
}
