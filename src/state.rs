use crate::domain::adapters::{AdapterRegistry, implementations::MockAdapter};
use crate::infrastructure::messaging::mcp::bus::McpBus;
use crate::application::Planner;
use crate::application::PromptStore;
use crate::application::KnowledgeBase;
use crate::infrastructure::cache::{SessionCache, EmbeddingCache};
use crate::infrastructure::queue::{TaskQueue, TaskWorker};
use crate::monitor::{EventBus, AuditLog, Metrics};
use crate::domain::config::manager::ConfigManager;
use crate::application::PostprocessorChain;
use crate::application::postprocessor::{RedactionMode, FormatMode, MergeStrategy};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub adapter_registry: Arc<RwLock<AdapterRegistry>>,
    pub mcp_bus: McpBus,
    pub planner: Arc<Planner>,
    pub prompt_store: Arc<RwLock<PromptStore>>,
    pub knowledge_base: Arc<RwLock<KnowledgeBase>>,
    pub session_cache: SessionCache,
    pub embedding_cache: EmbeddingCache,
    pub task_queue: Arc<TaskQueue>,
    pub task_worker: Arc<TaskWorker>,
    pub event_bus: Arc<EventBus>,
    pub audit_log: Arc<AuditLog>,
    pub metrics: Arc<Metrics>,
    pub config_manager: Arc<ConfigManager>,
    pub postprocessor_chain: Arc<PostprocessorChain>,
}

impl AppState {
    pub fn new() -> Self {
        let registry = AdapterRegistry::new();
        
        // 初始化配置管理
        let config_manager = Arc::new(ConfigManager::new());
        
        // 异步初始化：从配置加载适配器
        let registry_clone = Arc::new(RwLock::new(registry));
        let config_manager_clone = config_manager.clone();
        let registry_for_spawn = registry_clone.clone();
        tokio::spawn(async move {
            // 注册 Mock 适配器用于测试
            registry_for_spawn.write().await.register("mock", Arc::new(MockAdapter::new("mock".to_string()))).await;
            
            // 从配置加载适配器
            let config = config_manager_clone.get_config().await;
            if !config.adapters.is_empty() {
                let configs: Vec<_> = config.adapters.values().cloned().collect();
                if let Err(e) = registry_for_spawn.write().await.register_from_configs(configs).await {
                    tracing::error!("Failed to load adapters from config: {}", e);
                }
            }
        });
        
        // TODO: 从配置或环境变量加载 API keys
        let redis_url = std::env::var("REDIS_URL").ok();
        let redis_url_str = redis_url.as_deref();
        
        // 初始化缓存
        let session_cache = SessionCache::new(redis_url_str);
        let embedding_cache = EmbeddingCache::new(redis_url_str);
        
        // 初始化任务队列
        let task_queue = Arc::new(TaskQueue::new(redis_url_str));
        let planner = Arc::new(Planner::new());
        
        // 初始化 Worker
        let task_worker = Arc::new(TaskWorker::new(
            task_queue.clone(),
            registry_clone.clone(),
            planner.clone(),
            4, // 并发数
        ));
        
        // 启动 Worker
        let worker_clone = task_worker.clone();
        tokio::spawn(async move {
            worker_clone.start().await;
        });
        
        // 初始化监控
        let event_bus = Arc::new(EventBus::new());
        let audit_log = Arc::new(AuditLog::new(event_bus.clone()));
        let metrics = Arc::new(Metrics::new());
        
        // 初始化后处理器链
        let postprocessor_chain = Arc::new(
            PostprocessorChain::with_defaults(
                audit_log.clone(),
                RedactionMode::Mask, // 默认使用遮盖模式
                FormatMode::Plain,   // 默认纯文本格式
                MergeStrategy::Concatenate, // 默认拼接策略
            )
        );
        
        Self {
            adapter_registry: registry_clone,
            mcp_bus: McpBus::new(),
            planner,
            prompt_store: Arc::new(RwLock::new(PromptStore::new())),
            knowledge_base: Arc::new(RwLock::new(KnowledgeBase::new())),
            session_cache,
            embedding_cache,
            task_queue,
            task_worker,
            event_bus,
            audit_log,
            metrics,
            config_manager,
            postprocessor_chain,
        }
    }
}


