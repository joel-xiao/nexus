use crate::infrastructure::queue::task::{Task, TaskStatus};
use crate::infrastructure::cache::RedisCache;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};

pub struct TaskQueue {
    pending_tasks: Arc<mpsc::UnboundedSender<Task>>,
    task_storage: Arc<DashMap<String, Task>>,
    #[allow(dead_code)]
    idempotency_map: Arc<DashMap<String, String>>, // idempotency_key -> task_id
    redis: Option<RedisCache>,
}

impl TaskQueue {
    pub fn new(redis_url: Option<&str>) -> Self {
        let (tx, mut rx): (mpsc::UnboundedSender<Task>, mpsc::UnboundedReceiver<Task>) = mpsc::unbounded_channel();
        
        let task_storage = Arc::new(DashMap::<String, Task>::new());
        let idempotency_map = Arc::new(DashMap::<String, String>::new());
        let redis = redis_url.map(|url| RedisCache::new(Some(url), "task"));
        let storage_clone = task_storage.clone();
        let idempotency_clone = idempotency_map.clone();
        let redis_clone = redis.clone();
        
        // 后台任务处理循环
        tokio::spawn(async move {
            while let Some(mut task) = rx.recv().await {
                info!("Received task: {} ({})", task.id, task.task_type);
                
                // 检查幂等性
                if let Some(ref key) = task.idempotency_key {
                    if let Some(existing_id) = idempotency_clone.get(key as &str) {
                        if let Some(existing_task) = storage_clone.get(existing_id.value()) {
                            if existing_task.status == TaskStatus::Completed {
                                warn!("Duplicate task detected (idempotency key: {}), skipping", key);
                                continue;
                            }
                        }
                    }
                    idempotency_clone.insert(key.clone(), task.id.clone());
                }
                
                // 保存任务状态
                task.mark_processing();
                storage_clone.insert(task.id.clone(), task.clone());
                
                // 如果有 Redis，也保存
                if let Some(ref _cache) = redis_clone {
                    let _ = _cache.set(&format!("task:{}", task.id), &task, Some(86400)).await;
                }
                
                debug!("Task {} queued for processing", task.id);
            }
        });
        
        Self {
            pending_tasks: Arc::new(tx),
            task_storage,
            idempotency_map,
            redis,
        }
    }

    pub fn enqueue(&self, task: Task) -> Result<String, mpsc::error::SendError<Task>> {
        let task_id = task.id.clone();
        self.pending_tasks.send(task)?;
        info!("Task enqueued: {}", task_id);
        Ok(task_id)
    }

    pub fn get_task(&self, task_id: &str) -> Option<Task> {
        // 先从内存获取
        if let Some(task) = self.task_storage.get(task_id) {
            return Some(task.clone());
        }
        
        // 如果有 Redis，从 Redis 获取
        if let Some(ref _cache) = self.redis {
            // TODO: 异步获取（需要改成 async 方法）
        }
        
        None
    }

    pub fn update_task(&self, task: Task) {
        self.task_storage.insert(task.id.clone(), task.clone());
        
        if let Some(ref cache) = self.redis {
            let task_id = task.id.clone();
            let task_clone = task.clone();
            let cache_clone = cache.clone();
            tokio::spawn(async move {
                let _ = cache_clone.set(&format!("task:{}", task_id), &task_clone, Some(86400)).await;
            });
        }
    }

    pub fn list_tasks(&self, status: Option<TaskStatus>) -> Vec<Task> {
        self.task_storage
            .iter()
            .filter(|entry| {
                if let Some(ref s) = status {
                    entry.value().status == *s
                } else {
                    true
                }
            })
            .map(|entry| entry.value().clone())
            .collect()
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new(None)
    }
}

