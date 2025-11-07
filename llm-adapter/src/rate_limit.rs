use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::debug;

#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
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

pub struct RateLimiter {
    config: RateLimitConfig,
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

    pub async fn check(&self, key: &str) -> Result<(), RateLimitError> {
        if !self.config.enabled {
            return Ok(());
        }

        let now = Instant::now();

        if self.config.requests_per_second > 0 {
            let mut second_requests = self
                .second_window
                .entry(key.to_string())
                .or_insert_with(Vec::new);

            second_requests
                .retain(|&timestamp| now.duration_since(timestamp) < Duration::from_secs(1));

            if second_requests.len() >= self.config.requests_per_second as usize {
                return Err(RateLimitError::TooManyRequests(format!(
                    "Rate limit exceeded: {} requests per second",
                    self.config.requests_per_second
                )));
            }

            second_requests.push(now);
        }

        if self.config.requests_per_minute > 0 {
            let mut minute_requests = self
                .minute_window
                .entry(key.to_string())
                .or_insert_with(Vec::new);

            minute_requests
                .retain(|&timestamp| now.duration_since(timestamp) < Duration::from_secs(60));

            if minute_requests.len() >= self.config.requests_per_minute as usize {
                return Err(RateLimitError::TooManyRequests(format!(
                    "Rate limit exceeded: {} requests per minute",
                    self.config.requests_per_minute
                )));
            }

            minute_requests.push(now);
        }

        if self.config.requests_per_hour > 0 {
            let mut hour_requests = self
                .hour_window
                .entry(key.to_string())
                .or_insert_with(Vec::new);

            hour_requests
                .retain(|&timestamp| now.duration_since(timestamp) < Duration::from_secs(3600));

            if hour_requests.len() >= self.config.requests_per_hour as usize {
                return Err(RateLimitError::TooManyRequests(format!(
                    "Rate limit exceeded: {} requests per hour",
                    self.config.requests_per_hour
                )));
            }

            hour_requests.push(now);
        }

        Ok(())
    }

    pub fn update_config(&self, _config: RateLimitConfig) {
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
