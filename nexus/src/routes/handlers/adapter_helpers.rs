use crate::state::AppState;
use llm_adapter::config::AdapterConfig;
use std::sync::Arc;
use tracing::{error, info};

pub fn record_adapter_call(metrics: &crate::monitor::Metrics, adapter_name: &str) {
    metrics.increment(&format!("adapter_calls_total:{}", adapter_name));
}

pub fn record_adapter_success(
    metrics: &crate::monitor::Metrics,
    adapter_name: &str,
    duration: f64,
) {
    metrics.increment(&format!("adapter_success_total:{}", adapter_name));
    metrics.record_histogram("adapter_duration_seconds", duration);
}

pub fn record_adapter_error(metrics: &crate::monitor::Metrics, adapter_name: &str) {
    metrics.increment(&format!("adapter_errors_total:{}", adapter_name));
}

pub async fn register_adapter_dynamically(
    state: &Arc<AppState>,
    config: AdapterConfig,
) -> Result<(), String> {
    match state
        .adapter_registry
        .write()
        .await
        .register_from_config(config)
        .await
    {
        Ok(_) => {
            info!("Adapter registered successfully");
            Ok(())
        }
        Err(e) => {
            error!("Failed to register adapter dynamically: {}", e);
            Err(format!("Failed to register adapter: {}", e))
        }
    }
}

pub fn serialize_adapter_config(config: &AdapterConfig) -> serde_json::Value {
    serde_json::json!({
        "name": config.name,
        "api_key": config.api_key,
        "model": config.model,
        "base_url": config.base_url,
        "enabled": config.enabled
    })
}
