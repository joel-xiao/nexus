use axum::{Json, Extension};
use std::sync::Arc;
use crate::state::AppState;
use crate::routes::common::ok_response;

pub async fn health_handler() -> Json<serde_json::Value> {
    ok_response(serde_json::json!({
        "status": "healthy"
    }))
}

pub async fn readiness_handler(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let adapter_count = state.adapter_registry.read().await.list().await.len();
    let ready = adapter_count > 0;
    
    ok_response(serde_json::json!({
        "ready": ready
    }))
}
