use crate::routes::handlers::invoke as handlers;
use axum::routing::post;
use axum::Router;
use utoipa::OpenApi;

#[utoipa::path(
    post,
    path = "/api/invoke",
    tag = "invoke",
    request_body = handlers::InvokeRequest,
    responses(
        (status = 200, description = "成功处理请求", content_type = "application/json"),
        (status = 500, description = "服务器内部错误", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn invoke_handler(
    axum::Extension(state): axum::Extension<std::sync::Arc<crate::state::AppState>>,
    axum::Json(payload): axum::Json<handlers::InvokeRequest>,
) -> axum::Json<serde_json::Value> {
    handlers::invoke_handler(axum::Extension(state), axum::Json(payload)).await
}

pub fn invoke_routes() -> Router {
    Router::new().route("/invoke", post(invoke_handler))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        invoke_handler,
    ),
    components(schemas(
        handlers::InvokeRequest,
        handlers::InvokeResponse,
        crate::routes::common::ErrorResponse,
    )),
    tags(
        (name = "invoke", description = "模型调用接口"),
    )
)]
pub struct InvokeApiDoc;

pub use handlers::{InvokeRequest, InvokeResponse};
