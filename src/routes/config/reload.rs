use axum::{Router, Extension, Json};
use axum::routing::put;
use utoipa::OpenApi;
use crate::routes::handlers::config::reload as handlers;
use std::sync::Arc;
use crate::state::AppState;

/// 热重载适配器配置
#[utoipa::path(
    put,
    path = "/api/config/reload/adapter",
    tag = "config-reload",
    request_body = ReloadAdapterRequest,
    responses(
        (status = 200, description = "成功重载适配器")
    )
)]
pub async fn hot_reload_adapter(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<ReloadAdapterRequest>,
) -> axum::Json<serde_json::Value> {
    handlers::hot_reload_adapter(Extension(state), axum::Json(payload)).await
}

/// 热重载提示词配置
#[utoipa::path(
    put,
    path = "/api/config/reload/prompt",
    tag = "config-reload",
    request_body = ReloadPromptRequest,
    responses(
        (status = 200, description = "成功重载提示词")
    )
)]
pub async fn hot_reload_prompt(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<ReloadPromptRequest>,
) -> axum::Json<serde_json::Value> {
    handlers::hot_reload_prompt(Extension(state), axum::Json(payload)).await
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct ReloadAdapterRequest {
    /// 适配器名称
    #[schema(example = "openai")]
    pub name: String,
    /// API 密钥
    #[schema(example = "sk-...")]
    pub api_key: Option<String>,
    /// 模型名称
    #[schema(example = "gpt-4")]
    pub model: Option<String>,
    /// 基础 URL
    #[schema(example = "https://api.openai.com/v1")]
    pub base_url: Option<String>,
    /// 是否启用
    #[schema(example = true)]
    pub enabled: bool,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct ReloadPromptRequest {
    /// 提示词名称
    #[schema(example = "default_prompt")]
    pub name: String,
    /// 提示词模板
    #[schema(example = "You are a helpful assistant. {{input}}")]
    pub template: String,
    /// 是否启用
    #[schema(example = true)]
    pub enabled: bool,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        hot_reload_adapter,
        hot_reload_prompt,
    ),
    components(schemas(
        ReloadAdapterRequest,
        ReloadPromptRequest,
    )),
    tags(
        (name = "config-reload", description = "配置热重载"),
    )
)]
pub struct ReloadApiDoc;

pub fn reload_routes() -> Router {
    Router::new()
        .route("/reload/adapter", put(hot_reload_adapter))
        .route("/reload/prompt", put(hot_reload_prompt))
}
