use axum::Router;
use axum::routing::post;
use utoipa::OpenApi;
use crate::routes::handlers::invoke as handlers;

// ===== 带 OpenAPI 注解的包装函数（路径和 OpenAPI 定义在一起） =====

/// 调用 LLM 模型处理请求
/// 
/// 根据输入文本、适配器选择策略和用户ID，调用相应的 LLM 模型进行处理。
#[utoipa::path(
    post,
    path = "/api/invoke",
    tag = "invoke",
    request_body = handlers::InvokeRequest,
    responses(
        (status = 200, description = "成功处理请求", body = handlers::InvokeResponse),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn invoke_handler(
    axum::Extension(state): axum::Extension<std::sync::Arc<crate::state::AppState>>,
    axum::Json(payload): axum::Json<handlers::InvokeRequest>,
) -> axum::Json<handlers::InvokeResponse> {
    handlers::invoke_handler(axum::Extension(state), axum::Json(payload)).await
}

// ===== 路由定义 =====

/// 注册 invoke 相关的路由
pub fn invoke_routes() -> Router {
    Router::new()
        .route("/invoke", post(invoke_handler))
}

// ===== OpenAPI 文档片段 =====
#[derive(OpenApi)]
#[openapi(
    paths(
        invoke_handler,
    ),
    components(schemas(
        handlers::InvokeRequest,
        handlers::InvokeResponse,
    )),
    tags(
        (name = "invoke", description = "模型调用接口"),
    )
)]
pub struct InvokeApiDoc;

// 重新导出类型
pub use handlers::{InvokeRequest, InvokeResponse};








