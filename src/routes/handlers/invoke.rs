use axum::{Json, Extension};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::infrastructure::queue::task::{Task, TaskPriority};
use crate::monitor::event::{Event, EventLevel};
use crate::application::postprocessor::ProcessingContext;
use std::sync::Arc;
use tracing::{info, error};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({
    "input": "Hello, world!",
    "adapter": "mock",
    "user_id": "user123"
}))]
pub struct InvokeRequest {
    /// 输入文本内容
    #[schema(example = "Hello, world!")]
    pub input: String,
    /// 可选的适配器名称，如果不指定则使用路由策略
    #[serde(default)]
    #[schema(example = "mock")]
    pub adapter: Option<String>,
    /// 可选的用户ID，用于路由策略
    #[serde(default)]
    #[schema(example = "user123")]
    pub user_id: Option<String>,
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
    use crate::routes::common::{ok_response, error_response};
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
    
    let tasks = state.planner.split_task(&payload.input).await;
    let task_messages: Vec<serde_json::Value> = tasks.iter()
        .map(|t| serde_json::to_value(t).unwrap_or(serde_json::Value::Null))
        .collect();
    
    for task in &tasks {
        let _msg_id = state.mcp_bus.publish(task.clone()).await;
    }
    
    let kb_results = state.knowledge_base.read().await.query(&payload.input).await;
    let context = if !kb_results.is_empty() {
        Some(kb_results.join("\n\n"))
    } else {
        None
    };
    
    let final_prompt = match state.prompt_store.write().await.render_string(
        "{{input}}{{#if context}}\n\n相关上下文：\n{{context}}{{/if}}",
        &serde_json::json!({
            "input": payload.input,
            "context": context
        })
    ) {
        Ok(rendered) => {
            info!("Prompt rendered with template");
            rendered
        },
        Err(_) => payload.input.clone()
    };
    
    let task_payload = serde_json::json!({
        "input": final_prompt,
        "adapter": payload.adapter.as_deref().unwrap_or("mock"),
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
        serde_json::json!({"task_id": task_id, "input": payload.input}),
        EventLevel::Info,
    );
    state.event_bus.publish(event);
    
    let user_id = payload.user_id.as_deref();
    let adapter_name = if let Some(adapter) = payload.adapter.as_deref() {
        adapter.to_string()
    } else {
        match state.config_manager.router().select_model(user_id, None).await {
            Some((_model, adapter)) => adapter,
            None => "mock".to_string()
        }
    };
    
    let mut context = ProcessingContext::new(
        payload.user_id.clone(),
        adapter_name.clone(),
        final_prompt.clone(),
    );
    
    if let Err(e) = state.postprocessor_chain.pre_process(&mut context).await {
        error!("Pre-processing failed: {}", e);
    }
    
    let prompt_to_use = context.processed_input.as_ref()
        .unwrap_or(&final_prompt);
    
    let _result = match state.adapter_registry.read().await.get(&adapter_name).await {
        Some(adapter) => {
            info!("Using adapter: {}", adapter_name);
            state.metrics.increment(&format!("adapter_calls_total:{}", adapter_name));
            
            let start = std::time::Instant::now();
            
            let res = match adapter.invoke(prompt_to_use).await {
                Ok(res) => {
                    state.metrics.increment(&format!("adapter_success_total:{}", adapter_name));
                    let duration = start.elapsed().as_secs_f64();
                    state.metrics.record_histogram("adapter_duration_seconds", duration);
                    res
                },
                Err(e) => {
                    error!("Adapter invocation failed: {}", e);
                    state.metrics.increment(&format!("adapter_errors_total:{}", adapter_name));
                    format!("Error: {}", e)
                }
            };
            
            context = context.with_output(res.clone());
            res
        },
        None => {
            error!("Adapter not found: {}", adapter_name);
            let error_msg = "No adapter available".to_string();
            context = context.with_output(error_msg.clone());
            error_msg
        }
    };
    
    if let Err(e) = state.postprocessor_chain.post_process(&mut context).await {
        error!("Post-processing failed: {}", e);
    }
    
    let final_result = context.processed_output
        .unwrap_or_else(|| context.original_output.clone());
    
    ok_response(InvokeResponse {
        result: final_result,
        tasks: task_messages,
        adapter_used: adapter_name.to_string(),
    })
}
