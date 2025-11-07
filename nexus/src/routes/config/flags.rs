use crate::routes::handlers::config::flags as handlers;
use crate::state::AppState;
use axum::routing::{delete, get, post, put};
use axum::{Extension, Json, Router};
use std::sync::Arc;
use utoipa::OpenApi;

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
    handlers::create_flag(Extension(state), Json(payload)).await
}

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

#[utoipa::path(
    get,
    path = "/api/config/flags/{name}",
    tag = "config-flags",
    params(
        ("name" = String, Path, description = "标志名称")
    ),
    responses(
        (status = 200, description = "标志详情", content_type = "application/json"),
        (status = 404, description = "标志不存在", body = crate::routes::common::ErrorResponse)
    )
)]
pub async fn get_flag(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    handlers::get_flag(Extension(state), axum::extract::Path(name)).await
}

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

#[utoipa::path(
    delete,
    path = "/api/config/flags/{name}",
    tag = "config-flags",
    params(
        ("name" = String, Path, description = "标志名称")
    ),
    responses(
        (status = 200, description = "成功删除标志", content_type = "application/json")
    )
)]
pub async fn delete_flag(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    handlers::delete_flag(Extension(state), axum::extract::Path(name)).await
}

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

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateFlagRequest {
    #[schema(example = "new_feature")]
    pub name: String,
    #[schema(example = "enabled")]
    pub status: String,
    #[schema(example = "新功能标志")]
    pub description: Option<String>,
    #[schema(example = 50)]
    pub percentage: Option<u8>,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateFlagRequest {
    #[schema(example = "enabled")]
    pub status: String,
    #[schema(example = "新功能标志")]
    pub description: Option<String>,
    #[schema(example = 50)]
    pub percentage: Option<u8>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct FlagResponse {
    #[schema(example = "new_feature")]
    pub name: String,
    #[schema(example = true)]
    pub enabled: bool,
}

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
        crate::routes::common::ErrorResponse,
    )),
    tags(
        (name = "config-flags", description = "功能标志管理"),
    )
)]
pub struct FlagsApiDoc;

pub fn flags_routes() -> Router {
    Router::new()
        .route("/flags", post(create_flag))
        .route("/flags", get(list_flags))
        .route("/flags/{name}", get(get_flag))
        .route("/flags/{name}", put(update_flag))
        .route("/flags/{name}", delete(delete_flag))
        .route("/flags/{name}/check", get(check_flag))
}
