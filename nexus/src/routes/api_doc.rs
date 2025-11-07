use crate::routes::agents::AgentsApiDoc;
use crate::routes::config;
use crate::routes::health::HealthApiDoc;
use crate::routes::invoke::InvokeApiDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(info(
    title = "Nexus API",
    description = "Nexus - 多模型 LLM 统一网关 API 文档",
    version = "0.1.0",
    contact(name = "Nexus Team")
))]
pub struct ApiDoc;

impl ApiDoc {
    pub fn openapi() -> utoipa::openapi::OpenApi {
        let mut openapi = <ApiDoc as utoipa::OpenApi>::openapi();
        openapi.merge(<HealthApiDoc as utoipa::OpenApi>::openapi());
        openapi.merge(<InvokeApiDoc as utoipa::OpenApi>::openapi());
        openapi.merge(<AgentsApiDoc as utoipa::OpenApi>::openapi());
        openapi.merge(config::config_openapi());
        openapi
    }
}
