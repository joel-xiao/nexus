use axum::{Json, Extension};
use crate::state::AppState;
use std::sync::Arc;
use super::common::{ok_response, error_response, ok_response_with_message};

pub async fn list_adapters(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let config = state.config_manager.get_config().await;
    let adapters: Vec<serde_json::Value> = config.adapters.values().map(|adapter| {
        serde_json::json!({
            "name": adapter.name,
            "api_key": adapter.api_key,
            "model": adapter.model,
            "base_url": adapter.base_url,
            "enabled": adapter.enabled
        })
    }).collect();
    
    ok_response(serde_json::json!({ "adapters": adapters }))
}

pub async fn get_adapter(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match state.config_manager.get_adapter_config(&name).await {
        Some(config) => {
            let adapter = serde_json::json!({
                "name": config.name,
                "api_key": config.api_key,
                "model": config.model,
                "base_url": config.base_url,
                "enabled": config.enabled
            });
            ok_response(serde_json::json!({ "adapter": adapter }))
        },
        None => error_response(&format!("Adapter {} not found", name))
    }
}

pub async fn delete_adapter(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let mut config = state.config_manager.get_config().await;
    config.adapters.remove(&name);
    state.config_manager.update_config(config).await;
    
    let _ = state.adapter_registry.write().await.unregister(&name).await;
    
    ok_response_with_message(
        &format!("Adapter {} deleted", name),
        serde_json::json!({})
    )
}

pub async fn get_billing_stats(
    Extension(_state): Extension<Arc<AppState>>,
    axum::extract::Path(_adapter_name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    ok_response(serde_json::json!({
        "stats": {
            "total_tokens": 0,
            "total_cost": 0.0,
            "requests": 0
        }
    }))
}
