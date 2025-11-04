use utoipa::OpenApi;
use crate::routes::health::HealthApiDoc;
use crate::routes::invoke::InvokeApiDoc;
use crate::routes::config;

/// 主 OpenAPI 文档
/// 
/// 自动组合所有模块的 OpenAPI 文档片段
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Nexus API",
        description = "Nexus - 多模型 LLM 统一网关 API 文档",
        version = "0.1.0",
        contact(
            name = "Nexus Team"
        )
    )
)]
pub struct ApiDoc;

impl ApiDoc {
    /// 构建完整的 OpenAPI 文档
    pub fn openapi() -> utoipa::openapi::OpenApi {
        // 从基础 ApiDoc 开始（只包含 info）
        let mut openapi = <ApiDoc as utoipa::OpenApi>::openapi();
        
        // 合并各个模块的 OpenAPI
        openapi.merge(<HealthApiDoc as utoipa::OpenApi>::openapi());
        openapi.merge(<InvokeApiDoc as utoipa::OpenApi>::openapi());
        openapi.merge(config::config_openapi());
        
        openapi
    }
}

