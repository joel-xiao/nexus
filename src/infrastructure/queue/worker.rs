use crate::infrastructure::queue::task::{Task, TaskStatus};
use crate::infrastructure::queue::manager::TaskQueue;
use crate::domain::adapters::registry::AdapterRegistry;
use crate::application::Planner;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};
use std::time::Duration;

pub struct TaskWorker {
    queue: Arc<TaskQueue>,
    adapter_registry: Arc<RwLock<AdapterRegistry>>,
    planner: Arc<Planner>,
    concurrency: usize,
}

impl TaskWorker {
    pub fn new(
        queue: Arc<TaskQueue>,
        adapter_registry: Arc<RwLock<AdapterRegistry>>,
        planner: Arc<Planner>,
        concurrency: usize,
    ) -> Self {
        Self {
            queue,
            adapter_registry,
            planner,
            concurrency,
        }
    }

    pub async fn start(&self) {
        info!("Starting {} worker(s)", self.concurrency);
        
        for i in 0..self.concurrency {
            let queue = self.queue.clone();
            let adapter_registry = self.adapter_registry.clone();
            let planner = self.planner.clone();
            
            tokio::spawn(async move {
                Self::worker_loop(i, queue, adapter_registry, planner).await;
            });
        }
    }

    async fn worker_loop(
        worker_id: usize,
        queue: Arc<TaskQueue>,
        adapter_registry: Arc<RwLock<AdapterRegistry>>,
        planner: Arc<Planner>,
    ) {
        info!("Worker {} started", worker_id);
        
        loop {
            // 获取待处理任务
            let pending_tasks = queue.list_tasks(Some(TaskStatus::Pending));
            
            if pending_tasks.is_empty() {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
            
            // 按优先级排序
            let mut tasks: Vec<Task> = pending_tasks.into_iter().collect();
            tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
            
            if let Some(mut task) = tasks.first().cloned() {
                info!("Worker {} processing task {}", worker_id, task.id);
                
                // 更新任务状态
                task.mark_processing();
                queue.update_task(task.clone());
                
                // 执行任务
                match Self::execute_task(&task, &adapter_registry, &planner).await {
                    Ok(result) => {
                        task.mark_completed(result);
                        info!("Worker {} completed task {}", worker_id, task.id);
                    },
                    Err(e) => {
                        let error_msg = e.to_string();
                        task.mark_failed(error_msg);
                        
                        if task.can_retry() {
                            warn!("Worker {} will retry task {} (attempt {}/{})", 
                                worker_id, task.id, task.retry_count, task.max_retries);
                            // 延迟重试
                            let delay_secs: u64 = 2_u64.pow(task.retry_count);
                            tokio::time::sleep(Duration::from_secs(delay_secs)).await;
                            task.status = TaskStatus::Pending;
                        } else {
                            error!("Worker {} failed task {} after {} retries", 
                                worker_id, task.id, task.max_retries);
                        }
                    }
                }
                
                queue.update_task(task);
            }
            
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn execute_task(
        task: &Task,
        adapter_registry: &Arc<RwLock<AdapterRegistry>>,
        planner: &Arc<Planner>,
    ) -> anyhow::Result<serde_json::Value> {
        match task.task_type.as_str() {
            "invoke" => {
                let input = task.payload["input"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing input"))?;
                
                // 使用 Planner 拆分任务
                let _subtasks = planner.split_task(input).await;
                
                // 调用适配器
                let adapter_name = task.payload["adapter"].as_str().unwrap_or("mock");
                let registry = adapter_registry.read().await;
                
                if let Some(adapter) = registry.get(adapter_name).await {
                    let result = adapter.invoke(input).await?;
                    Ok(serde_json::json!({
                        "result": result,
                        "adapter": adapter_name
                    }))
                } else {
                    anyhow::bail!("Adapter not found: {}", adapter_name)
                }
            },
            _ => {
                anyhow::bail!("Unknown task type: {}", task.task_type)
            }
        }
    }
}

