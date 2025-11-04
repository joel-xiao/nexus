use crate::domain::adapters::registry::Adapter;
use crate::infrastructure::adapter::rate_limit::RateLimiter;
use crate::infrastructure::adapter::billing::BillingTracker;
use crate::infrastructure::adapter::guard::ConcurrencyGuard;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{warn, error};
use uuid::Uuid;

/// 包装的 Adapter - 集成限流、计费、并发控制
pub struct WrappedAdapter {
    inner: Arc<dyn Adapter + Send + Sync>,
    rate_limiter: Arc<RateLimiter>,
    billing_tracker: Arc<BillingTracker>,
    concurrency_guard: Arc<ConcurrencyGuard>,
    adapter_name: String,
}

impl WrappedAdapter {
    pub fn new(
        inner: Arc<dyn Adapter + Send + Sync>,
        rate_limiter: Arc<RateLimiter>,
        billing_tracker: Arc<BillingTracker>,
        concurrency_guard: Arc<ConcurrencyGuard>,
    ) -> Self {
        let adapter_name = inner.name().to_string();
        Self {
            inner,
            rate_limiter,
            billing_tracker,
            concurrency_guard,
            adapter_name,
        }
    }
}

#[async_trait]
impl Adapter for WrappedAdapter {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn describe(&self) -> String {
        format!("{} (with rate limiting, billing, concurrency control)", self.inner.describe().await)
    }

    async fn invoke(&self, prompt: &str) -> anyhow::Result<String> {
        let request_id = Uuid::new_v4().to_string();
        let user_id = None; // 可以从上下文获取
        
        // 1. 并发控制
        let _permit = self.concurrency_guard.acquire().await
            .map_err(|e| {
                error!("Concurrency limit exceeded: {}", e);
                anyhow::anyhow!("Service busy, please try again later")
            })?;

        // 2. 限流检查
        let rate_limit_key = format!("{}:{}", self.adapter_name, user_id.as_deref().unwrap_or("anonymous"));
        self.rate_limiter.check(&rate_limit_key).await
            .map_err(|e| {
                warn!("Rate limit exceeded for {}: {}", rate_limit_key, e);
                anyhow::anyhow!("Rate limit exceeded: {}", e)
            })?;

        // 3. 调用原始适配器
        let start = std::time::Instant::now();
        let result = self.inner.invoke(prompt).await;
        let duration = start.elapsed();

        // 4. 计费统计（简化：根据 prompt 长度估算 tokens）
        // 实际应该从 API 响应中获取真实的 token 数
        let input_tokens = estimate_tokens(prompt);
        let output_tokens = match &result {
            Ok(ref output) => estimate_tokens(output),
            Err(_) => 0,
        };

        // 记录使用情况
        self.billing_tracker.record_usage(
            self.adapter_name.clone(),
            user_id,
            request_id,
            input_tokens,
            output_tokens,
            serde_json::json!({
                "duration_ms": duration.as_millis(),
                "success": result.is_ok(),
            }),
        ).await;

        result
    }

    async fn health(&self) -> bool {
        self.inner.health().await
    }
}

/// 估算 token 数量（简化版：实际应该使用 tiktoken 等库）
fn estimate_tokens(text: &str) -> u64 {
    // 简单估算：英文约 4 字符 = 1 token，中文约 1.5 字符 = 1 token
    let chars: usize = text.chars().count();
    let chinese_chars = text.chars().filter(|c| (*c as u32) >= 0x4E00 && (*c as u32) <= 0x9FFF).count();
    let english_chars = chars - chinese_chars;
    
    ((english_chars as f64 / 4.0) + (chinese_chars as f64 / 1.5)) as u64
}


