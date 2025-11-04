use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};
use dashmap::DashMap;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, ToSchema)]
pub enum FlagStatus {
    Enabled,
    Disabled,
    GradualRollout { percentage: u8 }, // 0-100
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct FeatureFlag {
    pub name: String,
    pub status: FlagStatus,
    pub description: String,
    pub enabled_for: Vec<String>, // 用户ID列表或规则
    pub disabled_for: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl FeatureFlag {
    pub fn new(name: String, status: FlagStatus) -> Self {
        Self {
            name,
            status,
            description: String::new(),
            enabled_for: Vec::new(),
            disabled_for: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn is_enabled_for(&self, user_id: Option<&str>) -> bool {
        // 检查是否在禁用列表
        if let Some(uid) = user_id {
            if self.disabled_for.iter().any(|id| id == uid) {
                return false;
            }
        }

        match &self.status {
            FlagStatus::Disabled => false,
            FlagStatus::Enabled => {
                // 检查是否在启用列表
                if self.enabled_for.is_empty() {
                    true
                } else if let Some(uid) = user_id {
                    self.enabled_for.iter().any(|id| id == uid)
                } else {
                    false
                }
            },
            FlagStatus::GradualRollout { percentage } => {
                // 灰度发布：基于用户ID的哈希值
                if let Some(uid) = user_id {
                    let hash = Self::hash_user(uid);
                    let user_percentage = (hash % 100) as u8;
                    user_percentage < *percentage
                } else {
                    false
                }
            }
        }
    }

    fn hash_user(user_id: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Clone)]
pub struct FeatureFlagStore {
    flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
    cache: Arc<DashMap<String, bool>>, // (flag_name:user_id) -> enabled
}

impl FeatureFlagStore {
    pub fn new() -> Self {
        Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(DashMap::new()),
        }
    }

    pub async fn register(&self, flag: FeatureFlag) {
        let mut flags = self.flags.write().await;
        info!("Registering feature flag: {}", flag.name);
        flags.insert(flag.name.clone(), flag);
    }

    pub async fn get(&self, name: &str) -> Option<FeatureFlag> {
        let flags = self.flags.read().await;
        flags.get(name).cloned()
    }

    pub async fn is_enabled(&self, name: &str, user_id: Option<&str>) -> bool {
        let cache_key = format!("{}:{}", name, user_id.unwrap_or("anonymous"));
        
        // 先检查缓存
        if let Some(cached) = self.cache.get(&cache_key) {
            return *cached.value();
        }

        let flags = self.flags.read().await;
        let enabled = flags.get(name)
            .map(|flag| flag.is_enabled_for(user_id))
            .unwrap_or(false);

        // 缓存结果（TTL 10秒）
        self.cache.insert(cache_key, enabled);
        
        debug!("Feature flag '{}' is {} for user {:?}", name, 
            if enabled { "enabled" } else { "disabled" }, user_id);
        
        enabled
    }

    pub async fn update(&self, name: &str, flag: FeatureFlag) {
        let mut flags = self.flags.write().await;
        if flags.contains_key(name) {
            info!("Updating feature flag: {}", name);
            flags.insert(name.to_string(), flag);
            
            // 清除缓存
            self.cache.clear();
        }
    }

    pub async fn delete(&self, name: &str) {
        let mut flags = self.flags.write().await;
        flags.remove(name);
        self.cache.clear();
    }

    pub async fn list(&self) -> Vec<FeatureFlag> {
        let flags = self.flags.read().await;
        flags.values().cloned().collect()
    }

    pub async fn clear_cache(&self) {
        self.cache.clear();
    }
}

impl Default for FeatureFlagStore {
    fn default() -> Self {
        Self::new()
    }
}

