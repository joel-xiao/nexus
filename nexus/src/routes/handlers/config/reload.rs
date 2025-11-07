use crate::domain::config::manager::{AdapterConfig, PromptConfig};
use crate::routes::common::{error_response, ok_response_with_message};
use crate::routes::config::reload::{ReloadAdapterRequest, ReloadPromptRequest};
use crate::routes::handlers::adapter_helpers::register_adapter_dynamically;
use crate::state::AppState;
use axum::{Extension, Json};
use std::sync::Arc;
use tracing::error;

pub async fn hot_reload_adapter(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<ReloadAdapterRequest>,
) -> Json<serde_json::Value> {
    let config = AdapterConfig {
        name: payload.name.clone(),
        api_key: payload.api_key,
        model: payload.model,
        base_url: payload.base_url,
        enabled: payload.enabled,
        metadata: std::collections::HashMap::new(),
    };

    match state
        .config_manager
        .hot_reload_adapter(config.clone())
        .await
    {
        Ok(_) => match register_adapter_dynamically(&state, config).await {
            Ok(_) => ok_response_with_message(
                &format!("Adapter {} reloaded and registered", payload.name),
                serde_json::json!({}),
            ),
            Err(e) => error_response(&e),
        },
        Err(e) => error_response(&e.to_string()),
    }
}

pub async fn hot_reload_prompt(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<ReloadPromptRequest>,
) -> Json<serde_json::Value> {
    let template = payload.template.clone();
    let name = payload.name.clone();

    let config = PromptConfig {
        name: name.clone(),
        template: template.clone(),
        enabled: payload.enabled,
        metadata: std::collections::HashMap::new(),
    };

    match state.config_manager.hot_reload_prompt(config).await {
        Ok(_) => {
            let mut store = state.prompt_store.write().await;
            if payload.enabled {
                if let Err(e) = store.register_template(&name, &template) {
                    error!("Failed to register prompt template: {}", e);
                }
            } else {
                store.unregister_template(&name);
            }

            ok_response_with_message(
                &format!("Prompt {} reloaded", payload.name),
                serde_json::json!({}),
            )
        }
        Err(e) => error_response(&e.to_string()),
    }
}
