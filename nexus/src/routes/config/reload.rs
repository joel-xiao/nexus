use crate::routes::handlers::config::reload as handlers;
use crate::state::AppState;
use axum::routing::put;
use axum::{Extension, Json, Router};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    put,
    path = "/api/config/reload/adapter",
    tag = "config-reload",
    request_body = ReloadAdapterRequest,
    responses(
        (status = 200, description = "成功重载适配器", content_type = "application/json"),
        (status = 500, description = "重载失败", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn hot_reload_adapter(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<ReloadAdapterRequest>,
) -> axum::Json<serde_json::Value> {
    handlers::hot_reload_adapter(Extension(state), axum::Json(payload)).await
}

#[utoipa::path(
    put,
    path = "/api/config/reload/prompt",
    tag = "config-reload",
    request_body = ReloadPromptRequest,
    responses(
        (status = 200, description = "成功重载提示词", content_type = "application/json"),
        (status = 500, description = "重载失败", body = crate::routes::common::ErrorResponse)
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
    #[schema(example = "openai")]
    pub name: String,
    #[schema(example = "sk-...")]
    pub api_key: Option<String>,
    #[schema(example = "gpt-4")]
    pub model: Option<String>,
    #[schema(example = "https://api.openai.com/v1")]
    pub base_url: Option<String>,
    #[schema(example = true)]
    pub enabled: bool,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct ReloadPromptRequest {
    #[schema(example = "default_prompt")]
    pub name: String,
    #[schema(example = "You are a helpful assistant. {{input}}")]
    pub template: String,
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
        crate::routes::common::ErrorResponse,
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
