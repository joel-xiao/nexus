use axum::{Json, Extension};
use crate::state::AppState;
use crate::domain::config::manager::PromptConfig;
use std::sync::Arc;
use super::common::{ok_response, ok_response_with_message, error_response};

// ===== 业务逻辑处理函数 =====

pub async fn list_prompts(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let config = state.config_manager.get_config().await;
    let prompts: Vec<PromptConfig> = config.prompts.values().cloned().collect();
    ok_response(serde_json::json!({ "prompts": prompts }))
}

pub async fn get_prompt(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match state.config_manager.get_prompt_config(&name).await {
        Some(config) => ok_response(serde_json::json!({ "prompt": config })),
        None => error_response(&format!("Prompt {} not found", name))
    }
}

pub async fn delete_prompt(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let mut config = state.config_manager.get_config().await;
    config.prompts.remove(&name);
    state.config_manager.update_config(config).await;
    
    ok_response_with_message(&format!("Prompt {} deleted", name), serde_json::json!({}))
}
