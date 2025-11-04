use axum::{Json, Extension};
use crate::state::AppState;
use std::sync::Arc;
use super::common::{ok_response, error_response, ok_response_with_message};

// ===== 业务逻辑处理函数 =====

pub async fn list_adapters(
    Extension(_state): Extension<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let adapters = _state.adapter_registry.read().await.list().await;
    ok_response(serde_json::json!({ "adapters": adapters }))
}

pub async fn get_adapter(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match state.config_manager.get_adapter_config(&name).await {
        Some(config) => ok_response(serde_json::json!({ "adapter": config })),
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
    axum::extract::Path(adapter_name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    // TODO: 从 billing tracker 获取统计
    ok_response(serde_json::json!({
        "adapter": adapter_name,
        "stats": {}
    }))
}
