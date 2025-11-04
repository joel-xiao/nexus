use axum::{Router, Extension};
use axum::routing::{get, delete};
use utoipa::OpenApi;
use crate::routes::handlers::config::prompts as handlers;
use std::sync::Arc;
use crate::state::AppState;

// ===== 路由定义（路径 + OpenAPI 在一起） =====

/// 注册 prompts 相关的路由
pub fn prompts_routes() -> Router {
    Router::new()
        .route("/prompts", get(list_prompts))
        .route("/prompts/{name}", get(get_prompt))
        .route("/prompts/{name}", delete(delete_prompt))
}

// ===== 带 OpenAPI 注解的包装函数（路径和 OpenAPI 定义在一起） =====

/// 列出所有提示词
#[utoipa::path(
    get,
    path = "/api/config/prompts",
    tag = "config-prompts",
    responses(
        (status = 200, description = "提示词列表")
    )
)]
pub async fn list_prompts(
    Extension(state): Extension<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    handlers::list_prompts(Extension(state)).await
}

/// 获取单个提示词配置
#[utoipa::path(
    get,
    path = "/api/config/prompts/{name}",
    tag = "config-prompts",
    params(
        ("name" = String, Path, description = "提示词名称")
    ),
    responses(
        (status = 200, description = "提示词配置详情"),
        (status = 404, description = "提示词不存在")
    )
)]
pub async fn get_prompt(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> axum::Json<serde_json::Value> {
    handlers::get_prompt(Extension(state), axum::extract::Path(name)).await
}

/// 删除提示词
#[utoipa::path(
    delete,
    path = "/api/config/prompts/{name}",
    tag = "config-prompts",
    params(
        ("name" = String, Path, description = "提示词名称")
    ),
    responses(
        (status = 200, description = "成功删除提示词")
    )
)]
pub async fn delete_prompt(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> axum::Json<serde_json::Value> {
    handlers::delete_prompt(Extension(state), axum::extract::Path(name)).await
}

// ===== OpenAPI 文档片段 =====
#[derive(OpenApi)]
#[openapi(
    paths(
        list_prompts,
        get_prompt,
        delete_prompt,
    ),
    tags(
        (name = "config-prompts", description = "提示词管理"),
    )
)]
pub struct PromptsApiDoc;
