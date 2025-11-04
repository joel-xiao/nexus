use axum::{Router, Extension, Json};
use axum::routing::{get, post, put, delete};
use utoipa::OpenApi;
use crate::routes::handlers::config::flags as handlers;
use std::sync::Arc;
use crate::state::AppState;

// ===== 带 OpenAPI 注解的包装函数（路径和 OpenAPI 定义在一起） =====

/// 创建 Feature Flag
#[utoipa::path(
    post,
    path = "/api/config/flags",
    tag = "config-flags",
    request_body = CreateFlagRequest,
    responses(
        (status = 200, description = "成功创建标志", body = FlagResponse)
    )
)]
pub async fn create_flag(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateFlagRequest>,
) -> Json<FlagResponse> {
    handlers::create_flag(
        Extension(state),
        Json(payload)
    ).await
}

/// 列出所有 Feature Flags
#[utoipa::path(
    get,
    path = "/api/config/flags",
    tag = "config-flags",
    responses(
        (status = 200, description = "标志列表", body = Vec<crate::domain::config::feature_flag::FeatureFlag>)
    )
)]
pub async fn list_flags(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<Vec<crate::domain::config::feature_flag::FeatureFlag>> {
    handlers::list_flags(Extension(state)).await
}

/// 获取单个 Feature Flag
#[utoipa::path(
    get,
    path = "/api/config/flags/{name}",
    tag = "config-flags",
    params(
        ("name" = String, Path, description = "标志名称")
    ),
    responses(
        (status = 200, description = "标志详情"),
        (status = 404, description = "标志不存在")
    )
)]
pub async fn get_flag(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    handlers::get_flag(Extension(state), axum::extract::Path(name)).await
}

/// 更新 Feature Flag
#[utoipa::path(
    put,
    path = "/api/config/flags/{name}",
    tag = "config-flags",
    params(
        ("name" = String, Path, description = "标志名称")
    ),
    request_body = UpdateFlagRequest,
    responses(
        (status = 200, description = "成功更新标志", body = FlagResponse)
    )
)]
pub async fn update_flag(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(payload): Json<UpdateFlagRequest>,
) -> Json<FlagResponse> {
    handlers::update_flag(Extension(state), axum::extract::Path(name), Json(payload)).await
}

/// 删除 Feature Flag
#[utoipa::path(
    delete,
    path = "/api/config/flags/{name}",
    tag = "config-flags",
    params(
        ("name" = String, Path, description = "标志名称")
    ),
    responses(
        (status = 200, description = "成功删除标志")
    )
)]
pub async fn delete_flag(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    handlers::delete_flag(Extension(state), axum::extract::Path(name)).await
}

/// 检查 Feature Flag 状态
#[utoipa::path(
    get,
    path = "/api/config/flags/{name}/check",
    tag = "config-flags",
    params(
        ("name" = String, Path, description = "标志名称")
    ),
    responses(
        (status = 200, description = "标志状态", body = FlagResponse)
    )
)]
pub async fn check_flag(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<FlagResponse> {
    handlers::check_flag(Extension(state), axum::extract::Path(name)).await
}

// ===== OpenAPI 类型定义 =====

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateFlagRequest {
    /// 标志名称
    #[schema(example = "new_feature")]
    pub name: String,
    /// 标志状态: "enabled", "disabled", "gradual"
    #[schema(example = "enabled")]
    pub status: String,
    /// 标志描述
    #[schema(example = "新功能标志")]
    pub description: Option<String>,
    /// 渐进式发布的百分比 (0-100)
    #[schema(example = 50)]
    pub percentage: Option<u8>,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateFlagRequest {
    /// 标志状态: "enabled", "disabled", "gradual"
    #[schema(example = "enabled")]
    pub status: String,
    /// 标志描述
    #[schema(example = "新功能标志")]
    pub description: Option<String>,
    /// 渐进式发布的百分比 (0-100)
    #[schema(example = 50)]
    pub percentage: Option<u8>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct FlagResponse {
    /// 标志名称
    #[schema(example = "new_feature")]
    pub name: String,
    /// 是否启用
    #[schema(example = true)]
    pub enabled: bool,
}

// ===== OpenAPI 文档片段 =====
#[derive(OpenApi)]
#[openapi(
    paths(
        create_flag,
        check_flag,
        list_flags,
        update_flag,
        delete_flag,
        get_flag,
    ),
    components(schemas(
        CreateFlagRequest,
        UpdateFlagRequest,
        FlagResponse,
    )),
    tags(
        (name = "config-flags", description = "功能标志管理"),
    )
)]
pub struct FlagsApiDoc;

// ===== 路由定义（路径 + OpenAPI 在一起） =====

/// 注册 flags 相关的路由
pub fn flags_routes() -> Router {
    Router::new()
        .route("/flags", post(create_flag))
        .route("/flags", get(list_flags))
        .route("/flags/{name}", get(get_flag))
        .route("/flags/{name}", put(update_flag))
        .route("/flags/{name}", delete(delete_flag))
        .route("/flags/{name}/check", get(check_flag))
}
