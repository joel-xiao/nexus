use crate::domain::adapters::generic::{GenericAdapter, RequestConfig, AuthType};
use crate::infrastructure::adapter::{rate_limit::RateLimiter, billing::BillingTracker, guard::ConcurrencyGuard};
use crate::domain::config::manager::AdapterConfig;
use std::sync::Arc;
use tracing::info;

/// 适配器工厂 - 从配置创建适配器
pub struct AdapterFactory;

impl AdapterFactory {
    /// 从配置创建通用适配器
    pub fn create_generic_adapter(config: AdapterConfig) -> anyhow::Result<Arc<GenericAdapter>> {
        let api_key = config.api_key
            .ok_or_else(|| anyhow::anyhow!("API key is required for adapter: {}", config.name))?;
        
        let base_url = config.base_url
            .unwrap_or_else(|| "https://api.example.com".to_string());
        
        let model = config.model
            .unwrap_or_else(|| "default".to_string());

        // 从 metadata 中提取请求配置
        let request_config = Self::parse_request_config(&config.metadata)?;

        let adapter = GenericAdapter::new(
            config.name.clone(),
            api_key,
            model,
            base_url,
            request_config,
        );

        info!("Created generic adapter: {}", config.name);
        Ok(Arc::new(adapter))
    }

    fn parse_request_config(metadata: &std::collections::HashMap<String, serde_json::Value>) -> anyhow::Result<RequestConfig> {
        // 默认配置
        let mut config = RequestConfig {
            endpoint_template: "/v1/chat/completions".to_string(),
            body_template: Some(serde_json::json!({
                "model": "{model}",
                "messages": [
                    {
                        "role": "user",
                        "content": "{prompt}"
                    }
                ]
            })),
            method: "POST".to_string(),
            auth_type: AuthType::Bearer,
            auth_header: Some("Authorization".to_string()),
            model_field: "model".to_string(),
            message_field: "messages".to_string(),
            response_path: "choices.0.message.content".to_string(),
        };

        // 从 metadata 覆盖配置
        if let Some(endpoint) = metadata.get("endpoint_template").and_then(|v| v.as_str()) {
            config.endpoint_template = endpoint.to_string();
        }

        if let Some(body_template) = metadata.get("body_template") {
            config.body_template = Some(body_template.clone());
        }

        if let Some(method) = metadata.get("method").and_then(|v| v.as_str()) {
            config.method = method.to_string();
        }

        if let Some(auth_type) = metadata.get("auth_type").and_then(|v| v.as_str()) {
            config.auth_type = match auth_type {
                "bearer" => AuthType::Bearer,
                "header" => {
                    let header_name = metadata.get("auth_header")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Authorization");
                    AuthType::Header(header_name.to_string())
                },
                "query" => {
                    let param_name = metadata.get("auth_param")
                        .and_then(|v| v.as_str())
                        .unwrap_or("api_key");
                    AuthType::Query(param_name.to_string())
                },
                _ => AuthType::None,
            };
        }

        if let Some(model_field) = metadata.get("model_field").and_then(|v| v.as_str()) {
            config.model_field = model_field.to_string();
        }

        if let Some(message_field) = metadata.get("message_field").and_then(|v| v.as_str()) {
            config.message_field = message_field.to_string();
        }

        if let Some(response_path) = metadata.get("response_path").and_then(|v| v.as_str()) {
            config.response_path = response_path.to_string();
        }

        Ok(config)
    }

    /// 创建限流配置
    pub fn create_rate_limiter(metadata: &std::collections::HashMap<String, serde_json::Value>) -> Arc<RateLimiter> {
        use crate::infrastructure::adapter::rate_limit::RateLimitConfig;
        
        let mut config = RateLimitConfig::default();

        if let Some(rps) = metadata.get("rate_limit_rps").and_then(|v| v.as_u64()) {
            config.requests_per_second = rps as u32;
        }

        if let Some(rpm) = metadata.get("rate_limit_rpm").and_then(|v| v.as_u64()) {
            config.requests_per_minute = rpm as u32;
        }

        if let Some(rph) = metadata.get("rate_limit_rph").and_then(|v| v.as_u64()) {
            config.requests_per_hour = rph as u32;
        }

        if let Some(enabled) = metadata.get("rate_limit_enabled").and_then(|v| v.as_bool()) {
            config.enabled = enabled;
        }

        Arc::new(RateLimiter::new(config))
    }

    /// 创建并发控制配置
    pub fn create_concurrency_guard(metadata: &std::collections::HashMap<String, serde_json::Value>) -> Arc<ConcurrencyGuard> {
        use crate::infrastructure::adapter::guard::ConcurrencyConfig;
        
        let mut config = ConcurrencyConfig::default();

        if let Some(max) = metadata.get("max_concurrent").and_then(|v| v.as_u64()) {
            config.max_concurrent = max as usize;
        }

        if let Some(enabled) = metadata.get("concurrency_enabled").and_then(|v| v.as_bool()) {
            config.enabled = enabled;
        }

        Arc::new(ConcurrencyGuard::new(config))
    }

    /// 创建计费配置
    pub fn create_billing_tracker(metadata: &std::collections::HashMap<String, serde_json::Value>) -> Arc<BillingTracker> {
        use crate::infrastructure::adapter::billing::BillingConfig;
        
        let mut config = BillingConfig::default();

        if let Some(price) = metadata.get("input_price_per_1k").and_then(|v| v.as_f64()) {
            config.input_price_per_1k = price;
        }

        if let Some(price) = metadata.get("output_price_per_1k").and_then(|v| v.as_f64()) {
            config.output_price_per_1k = price;
        }

        if let Some(enabled) = metadata.get("billing_enabled").and_then(|v| v.as_bool()) {
            config.enabled = enabled;
        }

        Arc::new(BillingTracker::new(config))
    }
}


