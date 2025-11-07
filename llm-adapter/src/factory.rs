use crate::generic::{GenericAdapter, RequestConfig, AuthType};
use crate::registry::Adapter;
use crate::providers::{
    OpenAIAdapter, DeepSeekAdapter, ZhipuAdapter, DoubaoAdapter, QianwenAdapter
};
use crate::{RateLimiter, BillingTracker, ConcurrencyGuard};
use crate::config::AdapterConfig;
use std::sync::Arc;
use tracing::info;

/// 适配器工厂 - 从配置创建适配器
pub struct AdapterFactory;

impl AdapterFactory {
    /// 从配置创建适配器（智能选择专用或通用实现）
    pub fn create_adapter(config: AdapterConfig) -> anyhow::Result<Arc<dyn Adapter + Send + Sync>> {
        let api_key = config.api_key
            .clone()
            .ok_or_else(|| anyhow::anyhow!("API key is required for adapter: {}", config.name))?;
        
        // 根据名称或类型选择专用适配器
        let adapter: Arc<dyn Adapter + Send + Sync> = match config.name.as_str() {
            "openai" => {
                info!("Creating built-in OpenAI adapter");
                Arc::new(OpenAIAdapter::new(api_key, config.model.clone()))
            },
            "deepseek" => {
                info!("Creating built-in DeepSeek adapter");
                Arc::new(DeepSeekAdapter::new(api_key, config.model.clone()))
            },
            "zhipu" => {
                info!("Creating built-in Zhipu adapter");
                Arc::new(ZhipuAdapter::new(api_key, config.model.clone()))
            },
            "doubao" => {
                info!("Creating built-in Doubao adapter");
                Arc::new(DoubaoAdapter::new(api_key, config.model.clone()))
            },
            "qianwen" => {
                // 检查是否使用兼容模式
                if let Some(base_url) = &config.base_url {
                    if base_url.contains("compatible-mode") {
                        info!("Creating generic adapter for Qianwen (OpenAI-compatible mode)");
                        Self::create_generic_adapter(config)?
                    } else {
                        info!("Creating built-in Qianwen adapter (native API)");
                        Arc::new(QianwenAdapter::new(api_key, config.model.clone()))
                    }
                } else {
                    info!("Creating built-in Qianwen adapter (native API)");
                    Arc::new(QianwenAdapter::new(api_key, config.model.clone()))
                }
            },
            _ => {
                info!("Creating generic adapter for: {}", config.name);
                Self::create_generic_adapter(config)?
            }
        };

        Ok(adapter)
    }

    /// 从配置创建通用适配器
    pub fn create_generic_adapter(config: AdapterConfig) -> anyhow::Result<Arc<dyn Adapter + Send + Sync>> {
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
        Ok(Arc::new(adapter) as Arc<dyn Adapter + Send + Sync>)
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
        use crate::rate_limit::RateLimitConfig;
        
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
        use crate::guard::ConcurrencyConfig;
        
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
        use crate::billing::BillingConfig;
        
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


