use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use crate::domain::config::feature_flag::FeatureFlagStore;
use crate::domain::config::routing::ModelRouter;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub version: String,
    pub adapters: HashMap<String, AdapterConfig>,
    pub prompts: HashMap<String, PromptConfig>,
    pub feature_flags: HashMap<String, serde_json::Value>,
    pub routing_rules: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AdapterConfig {
    pub name: String,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub enabled: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PromptConfig {
    pub name: String,
    pub template: String,
    pub enabled: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    feature_flags: Arc<FeatureFlagStore>,
    router: Arc<ModelRouter>,
    watch_handlers: Arc<RwLock<Vec<tokio::sync::watch::Sender<Config>>>>,
}

impl ConfigManager {
    pub fn new() -> Self {
        let config = Config {
            version: "1.0.0".to_string(),
            adapters: HashMap::new(),
            prompts: HashMap::new(),
            feature_flags: HashMap::new(),
            routing_rules: Vec::new(),
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            feature_flags: Arc::new(FeatureFlagStore::new()),
            router: Arc::new(ModelRouter::new()),
            watch_handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn load_from_json(&self, json: &str) -> anyhow::Result<()> {
        let new_config: Config = serde_json::from_str(json)?;
        self.update_config(new_config).await;
        Ok(())
    }

    pub async fn update_config(&self, new_config: Config) {
        info!("Updating configuration (version: {})", new_config.version);
        
        let mut config = self.config.write().await;
        *config = new_config.clone();

        // 通知所有监听者
        let handlers = self.watch_handlers.read().await;
        for handler in handlers.iter() {
            let _ = handler.send(new_config.clone());
        }
    }

    pub async fn hot_reload_adapter(&self, adapter_config: AdapterConfig) -> anyhow::Result<()> {
        info!("Hot reloading adapter: {}", adapter_config.name);
        let mut config = self.config.write().await;
        config.adapters.insert(adapter_config.name.clone(), adapter_config.clone());
        
        // 返回配置以便外部可以注册适配器
        Ok(())
    }
    
    /// 获取所有适配器配置
    pub async fn get_all_adapter_configs(&self) -> Vec<AdapterConfig> {
        let config = self.config.read().await;
        config.adapters.values().cloned().collect()
    }

    pub async fn hot_reload_prompt(&self, prompt_config: PromptConfig) -> anyhow::Result<()> {
        info!("Hot reloading prompt: {}", prompt_config.name);
        let mut config = self.config.write().await;
        config.prompts.insert(prompt_config.name.clone(), prompt_config);
        Ok(())
    }

    pub async fn get_adapter_config(&self, name: &str) -> Option<AdapterConfig> {
        let config = self.config.read().await;
        config.adapters.get(name).cloned()
    }

    pub async fn get_prompt_config(&self, name: &str) -> Option<PromptConfig> {
        let config = self.config.read().await;
        config.prompts.get(name).cloned()
    }

    pub fn feature_flags(&self) -> Arc<FeatureFlagStore> {
        self.feature_flags.clone()
    }

    pub fn router(&self) -> Arc<ModelRouter> {
        self.router.clone()
    }

    pub async fn watch(&self) -> tokio::sync::watch::Receiver<Config> {
        let (tx, rx) = tokio::sync::watch::channel(self.config.read().await.clone());
        let mut handlers = self.watch_handlers.write().await;
        handlers.push(tx);
        rx
    }

    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }

    pub async fn export_config(&self) -> String {
        let config = self.config.read().await;
        serde_json::to_string_pretty(&*config).unwrap_or_default()
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

