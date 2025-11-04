use axum::{Json, Extension};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::infrastructure::queue::task::{Task, TaskPriority};
use crate::monitor::event::{Event, EventLevel};
use crate::application::postprocessor::ProcessingContext;
use std::sync::Arc;
use tracing::{info, error};
use utoipa::ToSchema;

// ===== 请求/响应类型定义 =====

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
    /// 处理结果
    #[schema(example = "Generated response")]
    pub result: String,
    /// 创建的任务列表
    #[schema(example = json!([]))]
    pub tasks: Vec<String>,
    /// 实际使用的适配器名称
    #[schema(example = "mock")]
    pub adapter_used: String,
}

// ===== 业务逻辑处理函数 =====

pub async fn invoke_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<InvokeRequest>,
) -> Json<InvokeResponse> {
    info!("Processing invoke request: {}", payload.input);
    
    // 记录指标
    state.metrics.increment("invoke_requests_total");
    
    // 审计日志
    state.audit_log.log_action(
        "invoke",
        "request",
        "new",
        None,
        "started",
        serde_json::json!({"input": payload.input}),
    );
    
    // 1. 使用 Planner 拆分任务
    let tasks = state.planner.split_task(&payload.input).await;
    let task_messages: Vec<String> = tasks.iter()
        .map(|t| serde_json::to_string(t).unwrap_or_default())
        .collect();
    
    // 2. 发布到 MCP 消息总线
    for task in &tasks {
        let _msg_id = state.mcp_bus.publish(task.clone()).await;
    }
    
    // 3. RAG: 从知识库检索相关文档
    let kb_results = state.knowledge_base.read().await.query(&payload.input).await;
    let context = if !kb_results.is_empty() {
        Some(kb_results.join("\n\n"))
    } else {
        None
    };
    
    // 4. 使用 Prompt Store 渲染提示（如果有模板）
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
        Err(_) => {
            // 模板渲染失败，使用原始输入
            payload.input.clone()
        }
    };
    
    // 5. 创建一个异步任务并加入队列
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
            return Json(InvokeResponse {
                result: "Failed to queue task".to_string(),
                tasks: task_messages,
                adapter_used: "none".to_string(),
            });
        }
    };
    
    // 发布事件
    let event = Event::new(
        "task.enqueued".to_string(),
        "invoke_handler".to_string(),
        serde_json::json!({"task_id": task_id, "input": payload.input}),
        EventLevel::Info,
    );
    state.event_bus.publish(event);
    
    // 6. 使用路由策略选择模型（如果指定了用户ID）
    let user_id = payload.user_id.as_deref();
    let adapter_name = if let Some(adapter) = payload.adapter.as_deref() {
        adapter.to_string()
    } else {
        match state.config_manager.router().select_model(user_id, None).await {
            Some((_model, adapter)) => adapter,
            None => "mock".to_string()
        }
    };
    
    // 创建处理上下文
    let mut context = ProcessingContext::new(
        payload.user_id.clone(),
        adapter_name.clone(),
        final_prompt.clone(),
    );
    
    // 预处理：执行后处理器链的预处理
    if let Err(e) = state.postprocessor_chain.pre_process(&mut context).await {
        error!("Pre-processing failed: {}", e);
    }
    
    // 使用处理后的输入（如果有）或原始输入
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
    
    // 后处理：执行后处理器链的后处理
    if let Err(e) = state.postprocessor_chain.post_process(&mut context).await {
        error!("Post-processing failed: {}", e);
    }
    
    // 使用处理后的输出（如果有）或原始输出
    let final_result = context.processed_output
        .unwrap_or_else(|| context.original_output.clone());
    
    Json(InvokeResponse {
        result: final_result,
        tasks: task_messages,
        adapter_used: adapter_name.to_string(),
    })
}
