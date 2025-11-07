use crate::billing::BillingTracker;
use crate::config::AdapterConfig;
use crate::factory::AdapterFactory;
use crate::wrapper::WrappedAdapter;
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

pub struct AdapterRegistry {
    adapters: Arc<RwLock<HashMap<String, Arc<dyn Adapter + Send + Sync>>>>,
    billing_trackers: Arc<DashMap<String, Arc<BillingTracker>>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: Arc::new(RwLock::new(HashMap::new())),
            billing_trackers: Arc::new(DashMap::new()),
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

    pub async fn register_from_config(&self, config: AdapterConfig) -> anyhow::Result<()> {
        if !config.enabled {
            info!("Adapter {} is disabled, skipping registration", config.name);
            return Ok(());
        }

        let adapter = AdapterFactory::create_adapter(config.clone())?;

        let rate_limiter = AdapterFactory::create_rate_limiter(&config.metadata);

        let concurrency_guard = AdapterFactory::create_concurrency_guard(&config.metadata);

        let billing_tracker = AdapterFactory::create_billing_tracker(&config.metadata);
        self.billing_trackers.insert(config.name.clone(), billing_tracker.clone());

        let wrapped = Arc::new(WrappedAdapter::new(
            adapter,
            rate_limiter,
            billing_tracker,
            concurrency_guard,
        ));

        self.register(&config.name, wrapped).await;

        Ok(())
    }

    pub async fn register_from_configs(&self, configs: Vec<AdapterConfig>) -> anyhow::Result<()> {
        for config in configs {
            if let Err(e) = self.register_from_config(config).await {
                error!("Failed to register adapter: {}", e);
            }
        }
        Ok(())
    }

    pub async fn unregister(&self, name: &str) -> bool {
        let mut adapters = self.adapters.write().await;
        let removed = adapters.remove(name).is_some();
        if removed {
            self.billing_trackers.remove(name);
        }
        removed
    }

    pub fn get_billing_tracker(&self, name: &str) -> Option<Arc<BillingTracker>> {
        self.billing_trackers.get(name).map(|e| e.value().clone())
    }
}

#[derive(Debug, Clone, Default)]
pub struct InvokeOptions {
    pub user_id: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[async_trait]
pub trait Adapter: Send + Sync {
    fn name(&self) -> &str;
    async fn describe(&self) -> String;
    async fn invoke(&self, prompt: &str) -> anyhow::Result<String>;
    async fn invoke_with_options(&self, prompt: &str, options: &InvokeOptions) -> anyhow::Result<String> {
        let _ = options;
        self.invoke(prompt).await
    }
    async fn health(&self) -> bool;
}
