use axum::{Router, Extension};
use axum::routing::{get, delete};
use utoipa::OpenApi;
use crate::routes::handlers::config::adapters as handlers;
use std::sync::Arc;
use crate::state::AppState;

// ===== 路由定义（路径 + OpenAPI 在一起） =====

/// 注册 adapters 相关的路由
pub fn adapters_routes() -> Router {
    Router::new()
        .route("/adapters", get(list_adapters))
        .route("/adapters/{name}", get(get_adapter))
        .route("/adapters/{name}", delete(delete_adapter))
        .route("/adapters/{name}/billing", get(get_billing_stats))
}

// ===== 带 OpenAPI 注解的包装函数（路径和 OpenAPI 定义在一起） =====

/// 列出所有适配器
#[utoipa::path(
    get,
    path = "/api/config/adapters",
    tag = "config-adapters",
    responses(
        (status = 200, description = "适配器列表")
    )
)]
pub async fn list_adapters(
    Extension(state): Extension<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    handlers::list_adapters(Extension(state)).await
}

/// 获取单个适配器配置
#[utoipa::path(
    get,
    path = "/api/config/adapters/{name}",
    tag = "config-adapters",
    params(
        ("name" = String, Path, description = "适配器名称")
    ),
    responses(
        (status = 200, description = "适配器配置详情"),
        (status = 404, description = "适配器不存在")
    )
)]
pub async fn get_adapter(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> axum::Json<serde_json::Value> {
    handlers::get_adapter(Extension(state), axum::extract::Path(name)).await
}

/// 删除适配器
#[utoipa::path(
    delete,
    path = "/api/config/adapters/{name}",
    tag = "config-adapters",
    params(
        ("name" = String, Path, description = "适配器名称")
    ),
    responses(
        (status = 200, description = "成功删除适配器")
    )
)]
pub async fn delete_adapter(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> axum::Json<serde_json::Value> {
    handlers::delete_adapter(Extension(state), axum::extract::Path(name)).await
}

/// 获取适配器计费统计
#[utoipa::path(
    get,
    path = "/api/config/adapters/{name}/billing",
    tag = "config-adapters",
    params(
        ("name" = String, Path, description = "适配器名称")
    ),
    responses(
        (status = 200, description = "计费统计")
    )
)]
pub async fn get_billing_stats(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(adapter_name): axum::extract::Path<String>,
) -> axum::Json<serde_json::Value> {
    handlers::get_billing_stats(Extension(state), axum::extract::Path(adapter_name)).await
}

// ===== OpenAPI 文档片段 =====
#[derive(OpenApi)]
#[openapi(
    paths(
        list_adapters,
        get_adapter,
        delete_adapter,
        get_billing_stats,
    ),
    tags(
        (name = "config-adapters", description = "适配器管理"),
    )
)]
pub struct AdaptersApiDoc;
