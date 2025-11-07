use crate::routes::handlers::config::prompts as handlers;
use crate::state::AppState;
use axum::routing::{delete, get};
use axum::{Extension, Router};
use std::sync::Arc;
use utoipa::OpenApi;

pub fn prompts_routes() -> Router {
    Router::new()
        .route("/prompts", get(list_prompts))
        .route("/prompts/{name}", get(get_prompt))
        .route("/prompts/{name}", delete(delete_prompt))
}

#[utoipa::path(
    get,
    path = "/api/config/prompts",
    tag = "config-prompts",
    responses(
        (status = 200, description = "提示词列表", content_type = "application/json"),
        (status = 500, description = "服务器错误", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn list_prompts(
    Extension(state): Extension<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    handlers::list_prompts(Extension(state)).await
}

#[utoipa::path(
    get,
    path = "/api/config/prompts/{name}",
    tag = "config-prompts",
    params(
        ("name" = String, Path, description = "提示词名称")
    ),
    responses(
        (status = 200, description = "提示词配置详情", content_type = "application/json"),
        (status = 404, description = "提示词不存在", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn get_prompt(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> axum::Json<serde_json::Value> {
    handlers::get_prompt(Extension(state), axum::extract::Path(name)).await
}

#[utoipa::path(
    delete,
    path = "/api/config/prompts/{name}",
    tag = "config-prompts",
    params(
        ("name" = String, Path, description = "提示词名称")
    ),
    responses(
        (status = 200, description = "成功删除提示词", content_type = "application/json")
    )
)]
pub async fn delete_prompt(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> axum::Json<serde_json::Value> {
    handlers::delete_prompt(Extension(state), axum::extract::Path(name)).await
}

#[derive(OpenApi)]
#[openapi(
    paths(
        list_prompts,
        get_prompt,
        delete_prompt,
    ),
    components(schemas(
        crate::routes::common::ErrorResponse,
    )),
    tags(
        (name = "config-prompts", description = "提示词管理"),
    )
)]
pub struct PromptsApiDoc;
