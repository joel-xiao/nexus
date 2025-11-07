use axum::{Json, Extension};
use crate::state::AppState;
use std::sync::Arc;
use super::common::{ok_response, ok_response_with_message, error_response};

pub async fn export_config(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let config_str = state.config_manager.export_config().await;
    match serde_json::from_str::<serde_json::Value>(&config_str) {
        Ok(config) => ok_response(config),
        Err(_) => error_response("Failed to export configuration")
    }
}

pub async fn import_config(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let config_str = serde_json::to_string(&payload).unwrap_or_default();
    match state.config_manager.load_from_json(&config_str).await {
        Ok(_) => {
            let adapter_configs = state.config_manager.get_all_adapter_configs().await;
            match state.adapter_registry.write().await.register_from_configs(adapter_configs).await {
                Ok(_) => ok_response_with_message(
                    "Configuration imported and adapters registered successfully",
                    serde_json::json!({})
                ),
                Err(e) => ok_response_with_message(
                    &format!("Configuration imported but some adapters failed to register: {}", e),
                    serde_json::json!({ "status": "warning" })
                ),
            }
        },
        Err(e) => error_response(&e.to_string()),
    }
}
