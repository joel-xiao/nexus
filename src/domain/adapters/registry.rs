use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::domain::config::manager::AdapterConfig;
use crate::domain::adapters::factory::AdapterFactory;
use crate::domain::adapters::wrapper::WrappedAdapter;
use tracing::{info, error};

pub struct AdapterRegistry {
    adapters: Arc<RwLock<HashMap<String, Arc<dyn Adapter + Send + Sync>>>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, name: &str, adapter: Arc<dyn Adapter + Send + Sync>) {
        let mut adapters = self.adapters.write().await;
        adapters.insert(name.to_string(), adapter);
        info!("Registered adapter: {}", name);
    }

    pub async fn get(&self, name: &str) -> Option<Arc<dyn Adapter + Send + Sync>> {
        let adapters = self.adapters.read().await;
        adapters.get(name).cloned()
    }

    pub async fn list(&self) -> Vec<String> {
        let adapters = self.adapters.read().await;
        adapters.keys().cloned().collect()
    }

    /// 从配置动态注册适配器
    pub async fn register_from_config(&self, config: AdapterConfig) -> anyhow::Result<()> {
        if !config.enabled {
            info!("Adapter {} is disabled, skipping registration", config.name);
            return Ok(());
        }

        // 创建通用适配器
        let adapter = AdapterFactory::create_generic_adapter(config.clone())?;

        // 创建限流器
        let rate_limiter = AdapterFactory::create_rate_limiter(&config.metadata);

        // 创建并发控制器
        let concurrency_guard = AdapterFactory::create_concurrency_guard(&config.metadata);

        // 创建计费追踪器（共享的，所有适配器可以共享）
        let billing_tracker = AdapterFactory::create_billing_tracker(&config.metadata);

        // 包装适配器
        let wrapped = Arc::new(WrappedAdapter::new(
            adapter,
            rate_limiter,
            billing_tracker,
            concurrency_guard,
        ));

        // 注册
        self.register(&config.name, wrapped).await;

        Ok(())
    }

    /// 批量从配置注册适配器
    pub async fn register_from_configs(&self, configs: Vec<AdapterConfig>) -> anyhow::Result<()> {
        for config in configs {
            if let Err(e) = self.register_from_config(config).await {
                error!("Failed to register adapter: {}", e);
                // 继续注册其他适配器
            }
        }
        Ok(())
    }

    /// 取消注册适配器
    pub async fn unregister(&self, name: &str) -> bool {
        let mut adapters = self.adapters.write().await;
        adapters.remove(name).is_some()
    }
}

#[async_trait]
pub trait Adapter: Send + Sync {
    fn name(&self) -> &str;
    async fn describe(&self) -> String;
    async fn invoke(&self, prompt: &str) -> anyhow::Result<String>;
    async fn health(&self) -> bool;
}


