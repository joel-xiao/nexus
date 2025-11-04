pub mod flags;
pub mod routing;
pub mod adapters;
pub mod prompts;
pub mod reload;
pub mod import_export;

use axum::Router;

// 重新导出类型以供外部使用
pub use flags::{CreateFlagRequest, UpdateFlagRequest, FlagResponse};
pub use routing::CreateRuleRequest;
pub use reload::{ReloadAdapterRequest, ReloadPromptRequest};

/// 组装所有 config 相关的路由
/// 路径定义和 OpenAPI 在各子模块中，这里只负责组合
pub fn config_routes() -> Router {
    Router::new()
        .merge(flags::flags_routes())
        .merge(routing::routing_routes())
        .merge(adapters::adapters_routes())
        .merge(prompts::prompts_routes())
        .merge(reload::reload_routes())
        .merge(import_export::import_export_routes())
}

// ===== OpenAPI 文档组合 =====
use utoipa::openapi::OpenApi as OpenApiStruct;

/// 组合所有 config 子模块的 OpenAPI 文档
pub fn config_openapi() -> OpenApiStruct {
    let mut openapi = <flags::FlagsApiDoc as utoipa::OpenApi>::openapi();
    
    openapi.merge(<routing::RoutingApiDoc as utoipa::OpenApi>::openapi());
    openapi.merge(<adapters::AdaptersApiDoc as utoipa::OpenApi>::openapi());
    openapi.merge(<prompts::PromptsApiDoc as utoipa::OpenApi>::openapi());
    openapi.merge(<reload::ReloadApiDoc as utoipa::OpenApi>::openapi());
    openapi.merge(<import_export::ImportExportApiDoc as utoipa::OpenApi>::openapi());
    
    openapi
}
