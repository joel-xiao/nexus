use crate::routes::handlers::config::routing as handlers;
use crate::state::AppState;
use axum::routing::{delete, get, post, put};
use axum::{Extension, Json, Router};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    post,
    path = "/api/config/routing/rules",
    tag = "config-routing",
    request_body = CreateRuleRequest,
    responses(
        (status = 200, description = "成功创建路由规则", content_type = "application/json"),
        (status = 500, description = "服务器错误", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn create_routing_rule(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateRuleRequest>,
) -> Json<serde_json::Value> {
    handlers::create_routing_rule(Extension(state), Json(payload)).await
}

#[utoipa::path(
    get,
    path = "/api/config/routing/rules",
    tag = "config-routing",
    responses(
        (status = 200, description = "路由规则列表", body = Vec<crate::domain::config::routing::RoutingRule>)
    )
)]
pub async fn list_routing_rules(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<Vec<crate::domain::config::routing::RoutingRule>> {
    handlers::list_routing_rules(Extension(state)).await
}

#[utoipa::path(
    get,
    path = "/api/config/routing/rules/{name}",
    tag = "config-routing",
    params(
        ("name" = String, Path, description = "规则名称")
    ),
    responses(
        (status = 200, description = "路由规则详情", content_type = "application/json"),
        (status = 404, description = "路由规则不存在", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn get_routing_rule(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    handlers::get_routing_rule(Extension(state), axum::extract::Path(name)).await
}

#[utoipa::path(
    put,
    path = "/api/config/routing/rules/{name}",
    tag = "config-routing",
    params(
        ("name" = String, Path, description = "规则名称")
    ),
    request_body = UpdateRuleRequest,
    responses(
        (status = 200, description = "成功更新路由规则", content_type = "application/json"),
        (status = 500, description = "服务器错误", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn update_routing_rule(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(payload): Json<UpdateRuleRequest>,
) -> Json<serde_json::Value> {
    handlers::update_routing_rule(Extension(state), axum::extract::Path(name), Json(payload)).await
}

#[utoipa::path(
    delete,
    path = "/api/config/routing/rules/{name}",
    tag = "config-routing",
    params(
        ("name" = String, Path, description = "规则名称")
    ),
    responses(
        (status = 200, description = "成功删除路由规则", content_type = "application/json")
    )
)]
pub async fn delete_routing_rule(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    handlers::delete_routing_rule(Extension(state), axum::extract::Path(name)).await
}

use crate::domain::config::routing::ModelWeight;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateRuleRequest {
    #[schema(example = "default_rule")]
    pub name: String,
    #[schema(example = "round_robin")]
    pub strategy: String,
    pub models: Vec<ModelWeight>,
    #[schema(example = 100)]
    pub priority: Option<u32>,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateRuleRequest {
    #[schema(example = "round_robin")]
    pub strategy: String,
    pub models: Vec<ModelWeight>,
    #[schema(example = 100)]
    pub priority: Option<u32>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_routing_rule,
        list_routing_rules,
        update_routing_rule,
        delete_routing_rule,
        get_routing_rule,
    ),
    components(schemas(
        CreateRuleRequest,
        UpdateRuleRequest,
        crate::routes::common::ErrorResponse,
    )),
    tags(
        (name = "config-routing", description = "路由规则管理"),
    )
)]
pub struct RoutingApiDoc;

pub fn routing_routes() -> Router {
    Router::new()
        .route("/routing/rules", post(create_routing_rule))
        .route("/routing/rules", get(list_routing_rules))
        .route("/routing/rules/{name}", get(get_routing_rule))
        .route("/routing/rules/{name}", put(update_routing_rule))
        .route("/routing/rules/{name}", delete(delete_routing_rule))
}
