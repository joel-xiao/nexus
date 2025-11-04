use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use tracing::{info, debug};

/// 计费配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillingConfig {
    /// 输入 token 价格（每 1000 tokens）
    pub input_price_per_1k: f64,
    /// 输出 token 价格（每 1000 tokens）
    pub output_price_per_1k: f64,
    /// 最小计费单位（tokens）
    pub min_charge_tokens: u64,
    /// 是否启用计费
    pub enabled: bool,
}

impl Default for BillingConfig {
    fn default() -> Self {
        Self {
            input_price_per_1k: 0.001,
            output_price_per_1k: 0.002,
            min_charge_tokens: 0,
            enabled: true,
        }
    }
}

/// 使用记录
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsageRecord {
    pub adapter_name: String,
    pub user_id: Option<String>,
    pub request_id: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_cost: f64,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// 计费统计器
pub struct BillingTracker {
    config: Arc<RwLock<BillingConfig>>,
    records: Arc<DashMap<String, Vec<UsageRecord>>>, // adapter_name -> records
    user_stats: Arc<DashMap<String, UserBillingStats>>, // user_id -> stats
    adapter_stats: Arc<DashMap<String, AdapterBillingStats>>, // adapter_name -> stats
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserBillingStats {
    pub user_id: String,
    pub total_requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdapterBillingStats {
    pub adapter_name: String,
    pub total_requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost: f64,
    pub last_updated: DateTime<Utc>,
}

impl BillingTracker {
    pub fn new(config: BillingConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            records: Arc::new(DashMap::new()),
            user_stats: Arc::new(DashMap::new()),
            adapter_stats: Arc::new(DashMap::new()),
        }
    }

    /// 记录使用情况
    pub async fn record_usage(
        &self,
        adapter_name: String,
        user_id: Option<String>,
        request_id: String,
        input_tokens: u64,
        output_tokens: u64,
        metadata: serde_json::Value,
    ) {
        let config = self.config.read().await;
        if !config.enabled {
            return;
        }

        // 计算成本
        let input_cost = (input_tokens as f64 / 1000.0) * config.input_price_per_1k;
        let output_cost = (output_tokens as f64 / 1000.0) * config.output_price_per_1k;
        let total_cost = input_cost + output_cost;

        let record = UsageRecord {
            adapter_name: adapter_name.clone(),
            user_id: user_id.clone(),
            request_id,
            input_tokens,
            output_tokens,
            total_cost,
            timestamp: Utc::now(),
            metadata,
        };

        // 保存记录
        self.records
            .entry(adapter_name.clone())
            .or_insert_with(Vec::new)
            .push(record.clone());

        // 更新用户统计
        if let Some(ref uid) = user_id {
            let mut stats = self.user_stats
                .entry(uid.clone())
                .or_insert_with(|| UserBillingStats {
                    user_id: uid.clone(),
                    total_requests: 0,
                    total_input_tokens: 0,
                    total_output_tokens: 0,
                    total_cost: 0.0,
                    last_updated: Utc::now(),
                });
            
            stats.total_requests += 1;
            stats.total_input_tokens += input_tokens;
            stats.total_output_tokens += output_tokens;
            stats.total_cost += total_cost;
            stats.last_updated = Utc::now();
        }

        // 更新适配器统计
        let mut adapter_stats = self.adapter_stats
            .entry(adapter_name.clone())
            .or_insert_with(|| AdapterBillingStats {
                adapter_name: adapter_name.clone(),
                total_requests: 0,
                total_input_tokens: 0,
                total_output_tokens: 0,
                total_cost: 0.0,
                last_updated: Utc::now(),
            });
        
        adapter_stats.total_requests += 1;
        adapter_stats.total_input_tokens += input_tokens;
        adapter_stats.total_output_tokens += output_tokens;
        adapter_stats.total_cost += total_cost;
        adapter_stats.last_updated = Utc::now();

        debug!(
            adapter = %adapter_name,
            input_tokens = input_tokens,
            output_tokens = output_tokens,
            cost = total_cost,
            "Billing recorded"
        );
    }

    /// 获取用户统计
    pub fn get_user_stats(&self, user_id: &str) -> Option<UserBillingStats> {
        self.user_stats.get(user_id).map(|s| s.value().clone())
    }

    /// 获取适配器统计
    pub fn get_adapter_stats(&self, adapter_name: &str) -> Option<AdapterBillingStats> {
        self.adapter_stats.get(adapter_name).map(|s| s.value().clone())
    }

    /// 获取所有用户统计
    pub fn get_all_user_stats(&self) -> Vec<UserBillingStats> {
        self.user_stats.iter().map(|e| e.value().clone()).collect()
    }

    /// 获取所有适配器统计
    pub fn get_all_adapter_stats(&self) -> Vec<AdapterBillingStats> {
        self.adapter_stats.iter().map(|e| e.value().clone()).collect()
    }

    /// 更新计费配置
    pub async fn update_config(&self, config: BillingConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
        info!("Billing config updated");
    }
}

impl Default for BillingTracker {
    fn default() -> Self {
        Self::new(BillingConfig::default())
    }
}


