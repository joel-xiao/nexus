use crate::application::postprocessor::ProcessingContext;
use crate::infrastructure::queue::task::{Task, TaskPriority};
use crate::monitor::event::{Event, EventLevel};
use crate::routes::handlers::adapter_helpers::{
    record_adapter_call, record_adapter_error, record_adapter_success, register_adapter_dynamically,
};
use crate::state::AppState;
use axum::{Extension, Json};
use llm_adapter::config::AdapterConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::warn;
use tracing::{error, info};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({
    "input": "Hello, world!",
    "adapter": "openai",
    "api_key": "sk-xxxx",
    "model": "gpt-4o-mini",
    "user_id": "user123",
    "prompt_name": "default"
}))]
pub struct InvokeRequest {
    #[schema(example = "Hello, world!")]
    pub input: String,
    #[serde(default)]
    #[schema(example = "mock")]
    pub adapter: Option<String>,
    #[serde(default)]
    #[schema(example = "sk-xxxx")]
    pub api_key: Option<String>,
    #[serde(default)]
    #[schema(example = "gpt-4o-mini")]
    pub model: Option<String>,
    #[serde(default)]
    #[schema(example = "https://api.openai.com")]
    pub base_url: Option<String>,
    #[serde(default)]
    #[schema(example = "user123")]
    pub user_id: Option<String>,
    #[serde(default)]
    #[schema(example = "default")]
    pub prompt_name: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct InvokeResponse {
    #[schema(example = "Generated response")]
    pub result: String,
    #[schema(example = json!([]))]
    pub tasks: Vec<serde_json::Value>,
    #[schema(example = "mock")]
    pub adapter_used: String,
}

pub async fn invoke_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<InvokeRequest>,
) -> Json<serde_json::Value> {
    use crate::routes::common::{error_response, ok_response};
    info!("Processing invoke request: {}", payload.input);

    state.metrics.increment("invoke_requests_total");

    state.audit_log.log_action(
        "invoke",
        "request",
        "new",
        None,
        "started",
        serde_json::json!({"input": payload.input}),
    );

    let user_id = payload.user_id.as_deref();

    let mut adapter_candidate = payload.adapter.clone();
    let mut inline_api_key = payload.api_key.clone();

    if inline_api_key.is_none() {
        if let Some(ref name) = adapter_candidate {
            let adapter_exists = state
                .adapter_registry
                .read()
                .await
                .get(name)
                .await
                .is_some();
            if !adapter_exists && is_potential_api_key(name) {
                inline_api_key = Some(name.clone());
                adapter_candidate = None;
            }
        }
    }

    let adapter_name = match adapter_candidate {
        Some(name) => name,
        None => {
            if inline_api_key.is_some() {
                "openai".to_string()
            } else {
                match state
                    .config_manager
                    .router()
                    .select_model(user_id, None)
                    .await
                {
                    Some((_model, adapter)) => adapter,
                    None => {
                        warn!("No adapter available from routing configuration");
                        return error_response(
                            "No adapter available. Please specify an adapter or configure routing.",
                        );
                    }
                }
            }
        }
    };

    if let Some(api_key) = inline_api_key {
        let mut config = AdapterConfig::new(adapter_name.clone());
        config.api_key = Some(api_key);

        if let Some(model) = &payload.model {
            config.model = Some(model.clone());
        }

        if let Some(base_url) = &payload.base_url {
            config.base_url = Some(base_url.clone());
        }

        if let Err(e) = register_adapter_dynamically(&state, config).await {
            return error_response(&e);
        }
    }

    let tasks = state.planner.split_task(&payload.input).await;
    let task_messages: Vec<serde_json::Value> = tasks
        .iter()
        .filter_map(|t| serde_json::to_value(t).ok())
        .collect();

    for task in &tasks {
        state.mcp_bus.publish(task.clone()).await;
    }

    let kb_enabled = state
        .config_manager
        .feature_flags()
        .is_enabled("knowledge_base", user_id)
        .await;
    
    let kb_results = if kb_enabled {
        state
            .knowledge_base
            .read()
            .await
            .query(&payload.input)
            .await
    } else {
        Vec::new()
    };
    let context = if !kb_results.is_empty() {
        Some(kb_results.join("\n\n"))
    } else {
        None
    };

    let final_prompt = if let Some(prompt_name) = &payload.prompt_name {
        let prompt_config = state.config_manager.get_prompt_config(prompt_name).await;
        if let Some(config) = prompt_config {
            if !config.enabled {
                warn!("Prompt template {} is disabled, using default", prompt_name);
                format!("{}{}", payload.input, if let Some(ctx) = context {
                    format!("\n\n相关上下文：\n{}", ctx)
                } else {
                    String::new()
                })
            } else {
                match state.prompt_store.read().await.render(prompt_name, &serde_json::json!({
                    "input": payload.input,
                    "context": context
                })) {
                    Ok(rendered) => {
                        info!("Prompt rendered with template: {}", prompt_name);
                        rendered
                    }
                    Err(e) => {
                        warn!("Failed to render prompt template {}: {}, using default", prompt_name, e);
                        format!("{}{}", payload.input, if let Some(ctx) = context {
                            format!("\n\n相关上下文：\n{}", ctx)
                        } else {
                            String::new()
                        })
                    }
                }
            }
        } else {
            warn!("Prompt template {} not found, using default", prompt_name);
            format!("{}{}", payload.input, if let Some(ctx) = context {
                format!("\n\n相关上下文：\n{}", ctx)
            } else {
                String::new()
            })
        }
    } else {
        match state.prompt_store.write().await.render_string(
            "{{input}}{{#if context}}\n\n相关上下文：\n{{context}}{{/if}}",
            &serde_json::json!({
                "input": payload.input,
                "context": context
            }),
        ) {
            Ok(rendered) => {
                info!("Prompt rendered with default template");
                rendered
            }
            Err(_) => payload.input.clone(),
        }
    };

    let task_payload = serde_json::json!({
        "input": final_prompt,
        "adapter": adapter_name.clone(),
    });

    let queue_task = Task::new(
        "invoke".to_string(),
        task_payload.clone(),
        TaskPriority::Normal,
    );

    let task_id = match state.task_queue.enqueue(queue_task) {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to enqueue task: {}", e);
            return error_response("Failed to queue task");
        }
    };

    let event = Event::new(
        "task.enqueued".to_string(),
        "invoke_handler".to_string(),
        serde_json::json!({"task_id": task_id, "input": &payload.input}),
        EventLevel::Info,
    );
    state.event_bus.publish(event);

    let mut context = ProcessingContext::new(
        payload.user_id.clone(),
        adapter_name.clone(),
        final_prompt.clone(),
    );

    if let Err(e) = state.postprocessor_chain.pre_process(&mut context).await {
        error!("Pre-processing failed: {}", e);
    }

    let prompt_to_use = context.processed_input.as_ref().unwrap_or(&final_prompt);

    match state.adapter_registry.read().await.get(&adapter_name).await {
        Some(adapter) => {
            info!("Using adapter: {}", adapter_name);
            record_adapter_call(&state.metrics, &adapter_name);

            let start = std::time::Instant::now();

            let options = llm_adapter::InvokeOptions {
                user_id: payload.user_id.clone(),
                model: payload.model.clone(),
                temperature: None,
                max_tokens: None,
                metadata: std::collections::HashMap::new(),
            };

            let res = match adapter.invoke_with_options(prompt_to_use, &options).await {
                Ok(res) => {
                    let duration = start.elapsed().as_secs_f64();
                    record_adapter_success(&state.metrics, &adapter_name, duration);
                    res
                }
                Err(e) => {
                    error!("Adapter invocation failed: {}", e);
                    record_adapter_error(&state.metrics, &adapter_name);
                    format!("Error: {}", e)
                }
            };

            context = context.with_output(res);
        }
        None => {
            error!("Adapter not found: {}", adapter_name);
            let error_msg = "No adapter available".to_string();
            context = context.with_output(error_msg);
        }
    };

    if let Err(e) = state.postprocessor_chain.post_process(&mut context).await {
        error!("Post-processing failed: {}", e);
    }

    let final_result = context
        .processed_output
        .unwrap_or_else(|| context.original_output.clone());

    ok_response(InvokeResponse {
        result: final_result,
        tasks: task_messages,
        adapter_used: adapter_name.to_string(),
    })
}

fn is_potential_api_key(value: &str) -> bool {
    value.starts_with("sk-")
        || value.len() >= 40
            && value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
