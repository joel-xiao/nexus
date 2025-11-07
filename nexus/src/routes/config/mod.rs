pub mod adapters;
pub mod flags;
pub mod import_export;
pub mod prompts;
pub mod reload;
pub mod routing;

use axum::Router;

pub use flags::{CreateFlagRequest, FlagResponse, UpdateFlagRequest};
pub use reload::{ReloadAdapterRequest, ReloadPromptRequest};
pub use routing::CreateRuleRequest;

pub fn config_routes() -> Router {
    Router::new()
        .merge(flags::flags_routes())
        .merge(routing::routing_routes())
        .merge(adapters::adapters_routes())
        .merge(prompts::prompts_routes())
        .merge(reload::reload_routes())
        .merge(import_export::import_export_routes())
}

use utoipa::openapi::OpenApi as OpenApiStruct;

pub fn config_openapi() -> OpenApiStruct {
    let mut openapi = <flags::FlagsApiDoc as utoipa::OpenApi>::openapi();

    openapi.merge(<routing::RoutingApiDoc as utoipa::OpenApi>::openapi());
    openapi.merge(<adapters::AdaptersApiDoc as utoipa::OpenApi>::openapi());
    openapi.merge(<prompts::PromptsApiDoc as utoipa::OpenApi>::openapi());
    openapi.merge(<reload::ReloadApiDoc as utoipa::OpenApi>::openapi());
    openapi.merge(<import_export::ImportExportApiDoc as utoipa::OpenApi>::openapi());

    openapi
}
