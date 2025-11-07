use crate::routes::handlers::config::adapters as handlers;
use crate::state::AppState;
use axum::routing::{delete, get};
use axum::{Extension, Router};
use std::sync::Arc;
use utoipa::OpenApi;

pub fn adapters_routes() -> Router {
    Router::new()
        .route("/adapters", get(list_adapters))
        .route("/adapters/stats", get(get_models_stats))
        .route("/adapters/by-model/{model_name}", get(get_adapter_by_model))
        .route("/adapters/{name}", get(get_adapter))
        .route("/adapters/{name}", delete(delete_adapter))
        .route("/adapters/{name}/billing", get(get_billing_stats))
}

#[utoipa::path(
    get,
    path = "/api/config/adapters",
    tag = "config-adapters",
    responses(
        (status = 200, description = "适配器列表", content_type = "application/json"),
        (status = 500, description = "服务器错误", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn list_adapters(
    Extension(state): Extension<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    handlers::list_adapters(Extension(state)).await
}

#[utoipa::path(
    get,
    path = "/api/config/adapters/{name}",
    tag = "config-adapters",
    params(
        ("name" = String, Path, description = "适配器名称")
    ),
    responses(
        (status = 200, description = "适配器配置详情", content_type = "application/json"),
        (status = 404, description = "适配器不存在", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn get_adapter(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> axum::Json<serde_json::Value> {
    handlers::get_adapter(Extension(state), axum::extract::Path(name)).await
}

#[utoipa::path(
    delete,
    path = "/api/config/adapters/{name}",
    tag = "config-adapters",
    params(
        ("name" = String, Path, description = "适配器名称")
    ),
    responses(
        (status = 200, description = "成功删除适配器", content_type = "application/json")
    )
)]
pub async fn delete_adapter(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> axum::Json<serde_json::Value> {
    handlers::delete_adapter(Extension(state), axum::extract::Path(name)).await
}

#[utoipa::path(
    get,
    path = "/api/config/adapters/{name}/billing",
    tag = "config-adapters",
    params(
        ("name" = String, Path, description = "适配器名称")
    ),
    responses(
        (status = 200, description = "计费统计", content_type = "application/json")
    )
)]
pub async fn get_billing_stats(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(adapter_name): axum::extract::Path<String>,
) -> axum::Json<serde_json::Value> {
    handlers::get_billing_stats(Extension(state), axum::extract::Path(adapter_name)).await
}

#[utoipa::path(
    get,
    path = "/api/config/adapters/stats",
    tag = "config-adapters",
    responses(
        (status = 200, description = "模型统计信息", content_type = "application/json")
    )
)]
pub async fn get_models_stats(
    Extension(state): Extension<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    handlers::get_models_stats(Extension(state)).await
}

#[utoipa::path(
    get,
    path = "/api/config/adapters/by-model/{model_name}",
    tag = "config-adapters",
    params(
        ("model_name" = String, Path, description = "模型名称")
    ),
    responses(
        (status = 200, description = "适配器配置详情", content_type = "application/json"),
        (status = 404, description = "模型不存在", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn get_adapter_by_model(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(model_name): axum::extract::Path<String>,
) -> axum::Json<serde_json::Value> {
    handlers::get_adapter_by_model(Extension(state), axum::extract::Path(model_name)).await
}

#[derive(OpenApi)]
#[openapi(
    paths(
        list_adapters,
        get_adapter,
        delete_adapter,
        get_billing_stats,
        get_models_stats,
        get_adapter_by_model,
    ),
    components(schemas(
        crate::routes::common::ErrorResponse,
    )),
    tags(
        (name = "config-adapters", description = "适配器管理"),
    )
)]
pub struct AdaptersApiDoc;
