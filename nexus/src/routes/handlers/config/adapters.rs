use crate::routes::common::{error_response, ok_response, ok_response_with_message};
use crate::routes::handlers::adapter_helpers::serialize_adapter_config;
use crate::state::AppState;
use axum::{Extension, Json};
use std::sync::Arc;

pub async fn list_adapters(Extension(state): Extension<Arc<AppState>>) -> Json<serde_json::Value> {
    let config = state.config_manager.get_config().await;
    let adapters: Vec<serde_json::Value> = config
        .adapters
        .values()
        .map(serialize_adapter_config)
        .collect();

    ok_response(serde_json::json!({ "adapters": adapters }))
}

pub async fn get_adapter(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match state.config_manager.get_adapter_config(&name).await {
        Some(config) => {
            let adapter = serialize_adapter_config(&config);
            ok_response(serde_json::json!({ "adapter": adapter }))
        }
        None => error_response(&format!("Adapter {} not found", name)),
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

    ok_response_with_message(&format!("Adapter {} deleted", name), serde_json::json!({}))
}

pub async fn get_billing_stats(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(adapter_name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    if let Some(tracker) = state.adapter_registry.read().await.get_billing_tracker(&adapter_name) {
        if let Some(stats) = tracker.get_adapter_stats(&adapter_name) {
            ok_response(serde_json::json!({
                "stats": {
                    "total_tokens": stats.total_input_tokens + stats.total_output_tokens,
                    "total_cost": stats.total_cost,
                    "requests": stats.total_requests,
                    "input_tokens": stats.total_input_tokens,
                    "output_tokens": stats.total_output_tokens,
                    "last_updated": stats.last_updated.to_rfc3339()
                }
            }))
        } else {
            ok_response(serde_json::json!({
                "stats": {
                    "total_tokens": 0,
                    "total_cost": 0.0,
                    "requests": 0,
                    "input_tokens": 0,
                    "output_tokens": 0
                }
            }))
        }
    } else {
        ok_response(serde_json::json!({
            "stats": {
                "total_tokens": 0,
                "total_cost": 0.0,
                "requests": 0,
                "input_tokens": 0,
                "output_tokens": 0
            }
        }))
    }
}

pub async fn get_models_stats(Extension(state): Extension<Arc<AppState>>) -> Json<serde_json::Value> {
    let config = state.config_manager.get_config().await;
    let mut total = 0;
    let mut enabled = 0;
    let mut disabled = 0;
    let mut by_adapter: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    let mut models_list = Vec::new();

    for (adapter_name, adapter_config) in &config.adapters {
        if let Some(model_name) = &adapter_config.model {
            total += 1;
            if adapter_config.enabled {
                enabled += 1;
            } else {
                disabled += 1;
            }

            models_list.push(serde_json::json!({
                "name": model_name,
                "adapter": adapter_name,
                "enabled": adapter_config.enabled
            }));

            let adapter_count = by_adapter
                .entry(adapter_name.clone())
                .or_insert_with(|| serde_json::json!({ "count": 0, "models": [] }));
            adapter_count["count"] = serde_json::Value::Number(
                serde_json::Number::from(adapter_count["count"].as_u64().unwrap_or(0) + 1),
            );
            adapter_count["models"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!(model_name));
        }
    }

    ok_response(serde_json::json!({
        "total": total,
        "enabled": enabled,
        "disabled": disabled,
        "by_adapter": by_adapter,
        "models": models_list
    }))
}

pub async fn get_adapter_by_model(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(model_name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let config = state.config_manager.get_config().await;

    for (_adapter_name, adapter_config) in &config.adapters {
        if let Some(m) = &adapter_config.model {
            if m == &model_name {
                let adapter = serialize_adapter_config(adapter_config);
                return ok_response(serde_json::json!({ "adapter": adapter }));
            }
        }
    }

    error_response(&format!("Model {} not found", model_name))
}
