use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategy {
    RoundRobin,
    Random,
    Weighted,
    LeastConnections,
    UserBased,
    HashBased,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct RoutingRule {
    pub name: String,
    pub strategy: RoutingStrategy,
    pub models: Vec<ModelWeight>,
    pub condition: Option<String>,
    pub priority: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct ModelWeight {
    pub model_name: String,
    pub adapter_name: String,
    pub weight: u32,
    pub enabled: bool,
}

#[derive(Clone)]
pub struct ModelRouter {
    rules: Arc<RwLock<Vec<RoutingRule>>>,
    round_robin_index: Arc<DashMap<String, usize>>,
    connection_counts: Arc<RwLock<HashMap<String, u64>>>,
}

impl ModelRouter {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            round_robin_index: Arc::new(DashMap::new()),
            connection_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_rule(&self, rule: RoutingRule) {
        let mut rules = self.rules.write().await;
        rules.push(rule.clone());
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        info!("Added routing rule: {} ({:?})", rule.name, rule.strategy);
    }

    pub async fn select_model(
        &self,
        user_id: Option<&str>,
        context: Option<&HashMap<String, serde_json::Value>>,
    ) -> Option<(String, String)> {
        let rules = self.rules.read().await;

        for rule in rules.iter() {
            if let Some(ref condition) = rule.condition {
                if !self.evaluate_condition(condition, context) {
                    continue;
                }
            }

            let enabled_models: Vec<&ModelWeight> =
                rule.models.iter().filter(|m| m.enabled).collect();

            if enabled_models.is_empty() {
                continue;
            }

            let selected = match rule.strategy {
                RoutingStrategy::RoundRobin => self.select_round_robin(&rule.name, &enabled_models),
                RoutingStrategy::Random => self.select_random(&enabled_models),
                RoutingStrategy::Weighted => self.select_weighted(&enabled_models),
                RoutingStrategy::LeastConnections => {
                    self.select_least_connections(&enabled_models).await
                }
                RoutingStrategy::UserBased => self.select_user_based(user_id, &enabled_models),
                RoutingStrategy::HashBased => self.select_hash_based(user_id, &enabled_models),
            };

            if let Some((model, adapter)) = selected {
                let mut counts = self.connection_counts.write().await;
                *counts.entry(model.clone()).or_insert(0) += 1;

                debug!(
                    "Selected model: {} (adapter: {}) for rule {}",
                    model, adapter, rule.name
                );
                return Some((model, adapter));
            }
        }

        warn!("No matching routing rule found");
        None
    }

    fn select_round_robin(
        &self,
        rule_name: &str,
        models: &[&ModelWeight],
    ) -> Option<(String, String)> {
        let mut idx = self
            .round_robin_index
            .entry(rule_name.to_string())
            .or_insert_with(|| 0);
        let selected = models[*idx % models.len()];
        *idx += 1;
        Some((selected.model_name.clone(), selected.adapter_name.clone()))
    }

    fn select_random(&self, models: &[&ModelWeight]) -> Option<(String, String)> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        let hash = hasher.finish();
        let idx = (hash as usize) % models.len();
        let selected = models[idx];
        Some((selected.model_name.clone(), selected.adapter_name.clone()))
    }

    fn select_weighted(&self, models: &[&ModelWeight]) -> Option<(String, String)> {
        let total_weight: u32 = models.iter().map(|m| m.weight).sum();
        if total_weight == 0 {
            return None;
        }

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        let mut random = (hasher.finish() % total_weight as u64) as u32;

        for model in models {
            if random < model.weight {
                return Some((model.model_name.clone(), model.adapter_name.clone()));
            }
            random -= model.weight;
        }

        None
    }

    async fn select_least_connections(&self, models: &[&ModelWeight]) -> Option<(String, String)> {
        let counts = self.connection_counts.read().await;
        let selected = models
            .iter()
            .min_by_key(|m| counts.get(&m.model_name).unwrap_or(&0))
            .map(|m| (m.model_name.clone(), m.adapter_name.clone()));
        selected
    }

    fn select_user_based(
        &self,
        user_id: Option<&str>,
        models: &[&ModelWeight],
    ) -> Option<(String, String)> {
        if let Some(uid) = user_id {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            uid.hash(&mut hasher);
            let hash = hasher.finish();
            let idx = (hash as usize) % models.len();
            let selected = models[idx];
            Some((selected.model_name.clone(), selected.adapter_name.clone()))
        } else {
            None
        }
    }

    fn select_hash_based(
        &self,
        user_id: Option<&str>,
        models: &[&ModelWeight],
    ) -> Option<(String, String)> {
        let key = user_id.unwrap_or("default");
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let idx = (hash as usize) % models.len();
        let selected = models[idx];
        Some((selected.model_name.clone(), selected.adapter_name.clone()))
    }

    fn evaluate_condition(
        &self,
        _condition: &str,
        context: Option<&HashMap<String, serde_json::Value>>,
    ) -> bool {
        if let Some(_ctx) = context {
            true
        } else {
            false
        }
    }

    pub async fn update_rule(&self, name: &str, rule: RoutingRule) {
        let mut rules = self.rules.write().await;
        if let Some(idx) = rules.iter().position(|r| r.name == name) {
            rules[idx] = rule;
            info!("Updated routing rule: {}", name);
        }
    }

    pub async fn remove_rule(&self, name: &str) {
        let mut rules = self.rules.write().await;
        rules.retain(|r| r.name != name);
        info!("Removed routing rule: {}", name);
    }

    pub async fn list_rules(&self) -> Vec<RoutingRule> {
        let rules = self.rules.read().await;
        rules.clone()
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}
