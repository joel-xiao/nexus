use axum::{Router, Extension, Json};
use axum::routing::{get, post, put, delete};
use utoipa::OpenApi;
use crate::routes::handlers::config::routing as handlers;
use std::sync::Arc;
use crate::state::AppState;

// ===== 带 OpenAPI 注解的包装函数（路径和 OpenAPI 定义在一起） =====

/// 创建路由规则
#[utoipa::path(
    post,
    path = "/api/config/routing/rules",
    tag = "config-routing",
    request_body = CreateRuleRequest,
    responses(
        (status = 200, description = "成功创建路由规则")
    )
)]
pub async fn create_routing_rule(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateRuleRequest>,
) -> Json<serde_json::Value> {
    handlers::create_routing_rule(Extension(state), Json(payload)).await
}

/// 列出所有路由规则
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

/// 获取单个路由规则
#[utoipa::path(
    get,
    path = "/api/config/routing/rules/{name}",
    tag = "config-routing",
    params(
        ("name" = String, Path, description = "规则名称")
    ),
    responses(
        (status = 200, description = "路由规则详情"),
        (status = 404, description = "路由规则不存在")
    )
)]
pub async fn get_routing_rule(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    handlers::get_routing_rule(Extension(state), axum::extract::Path(name)).await
}

/// 更新路由规则
#[utoipa::path(
    put,
    path = "/api/config/routing/rules/{name}",
    tag = "config-routing",
    params(
        ("name" = String, Path, description = "规则名称")
    ),
    request_body = CreateRuleRequest,
    responses(
        (status = 200, description = "成功更新路由规则")
    )
)]
pub async fn update_routing_rule(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(payload): Json<CreateRuleRequest>,
) -> Json<serde_json::Value> {
    handlers::update_routing_rule(Extension(state), axum::extract::Path(name), Json(payload)).await
}

/// 删除路由规则
#[utoipa::path(
    delete,
    path = "/api/config/routing/rules/{name}",
    tag = "config-routing",
    params(
        ("name" = String, Path, description = "规则名称")
    ),
    responses(
        (status = 200, description = "成功删除路由规则")
    )
)]
pub async fn delete_routing_rule(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    handlers::delete_routing_rule(Extension(state), axum::extract::Path(name)).await
}

// ===== OpenAPI 类型定义 =====
use crate::domain::config::routing::ModelWeight;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateRuleRequest {
    /// 规则名称
    #[schema(example = "default_rule")]
    pub name: String,
    /// 路由策略: "round_robin", "random", "weighted", "least_connections", "user_based", "hash_based"
    #[schema(example = "round_robin")]
    pub strategy: String,
    /// 模型权重列表
    pub models: Vec<ModelWeight>,
    /// 优先级，数字越大优先级越高
    #[schema(example = 100)]
    pub priority: Option<u32>,
}

// ===== OpenAPI 文档片段 =====
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
    )),
    tags(
        (name = "config-routing", description = "路由规则管理"),
    )
)]
pub struct RoutingApiDoc;

// ===== 路由定义（路径 + OpenAPI 在一起） =====

/// 注册 routing 相关的路由
pub fn routing_routes() -> Router {
    Router::new()
        .route("/routing/rules", post(create_routing_rule))
        .route("/routing/rules", get(list_routing_rules))
        .route("/routing/rules/{name}", get(get_routing_rule))
        .route("/routing/rules/{name}", put(update_routing_rule))
        .route("/routing/rules/{name}", delete(delete_routing_rule))
}
