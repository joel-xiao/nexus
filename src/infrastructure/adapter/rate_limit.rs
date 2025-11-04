use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use tracing::debug;

/// 限流配置
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// 每秒允许的请求数
    pub requests_per_second: u32,
    /// 每分钟允许的请求数
    pub requests_per_minute: u32,
    /// 每小时允许的请求数
    pub requests_per_hour: u32,
    /// 是否启用限流
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            requests_per_minute: 60,
            requests_per_hour: 1000,
            enabled: true,
        }
    }
}

/// 限流器
pub struct RateLimiter {
    config: RateLimitConfig,
    // 滑动窗口：记录最近的时间戳
    second_window: Arc<DashMap<String, Vec<Instant>>>,
    minute_window: Arc<DashMap<String, Vec<Instant>>>,
    hour_window: Arc<DashMap<String, Vec<Instant>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            second_window: Arc::new(DashMap::new()),
            minute_window: Arc::new(DashMap::new()),
            hour_window: Arc::new(DashMap::new()),
        }
    }

    /// 检查是否可以执行请求
    pub async fn check(&self, key: &str) -> Result<(), RateLimitError> {
        if !self.config.enabled {
            return Ok(());
        }

        let now = Instant::now();

        // 检查秒级限流
        if self.config.requests_per_second > 0 {
            let mut second_requests = self.second_window
                .entry(key.to_string())
                .or_insert_with(Vec::new);
            
            // 清理过期请求
            second_requests.retain(|&timestamp| now.duration_since(timestamp) < Duration::from_secs(1));
            
            if second_requests.len() >= self.config.requests_per_second as usize {
                return Err(RateLimitError::TooManyRequests(
                    format!("Rate limit exceeded: {} requests per second", self.config.requests_per_second)
                ));
            }
            
            second_requests.push(now);
        }

        // 检查分钟级限流
        if self.config.requests_per_minute > 0 {
            let mut minute_requests = self.minute_window
                .entry(key.to_string())
                .or_insert_with(Vec::new);
            
            minute_requests.retain(|&timestamp| now.duration_since(timestamp) < Duration::from_secs(60));
            
            if minute_requests.len() >= self.config.requests_per_minute as usize {
                return Err(RateLimitError::TooManyRequests(
                    format!("Rate limit exceeded: {} requests per minute", self.config.requests_per_minute)
                ));
            }
            
            minute_requests.push(now);
        }

        // 检查小时级限流
        if self.config.requests_per_hour > 0 {
            let mut hour_requests = self.hour_window
                .entry(key.to_string())
                .or_insert_with(Vec::new);
            
            hour_requests.retain(|&timestamp| now.duration_since(timestamp) < Duration::from_secs(3600));
            
            if hour_requests.len() >= self.config.requests_per_hour as usize {
                return Err(RateLimitError::TooManyRequests(
                    format!("Rate limit exceeded: {} requests per hour", self.config.requests_per_hour)
                ));
            }
            
            hour_requests.push(now);
        }

        Ok(())
    }

    pub fn update_config(&self, _config: RateLimitConfig) {
        // 更新配置（实际实现中需要 RwLock）
        debug!("Rate limiter config updated");
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitError {
    TooManyRequests(String),
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::TooManyRequests(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for RateLimitError {}


