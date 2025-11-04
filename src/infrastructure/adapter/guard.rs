use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::debug;
use std::time::Duration;

/// 并发控制配置
#[derive(Clone, Debug)]
pub struct ConcurrencyConfig {
    /// 最大并发数
    pub max_concurrent: usize,
    /// 是否启用并发控制
    pub enabled: bool,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            enabled: true,
        }
    }
}

/// 并发控制器（使用 Semaphore）
pub struct ConcurrencyGuard {
    semaphore: Arc<Semaphore>,
    config: ConcurrencyConfig,
}

impl ConcurrencyGuard {
    pub fn new(config: ConcurrencyConfig) -> Self {
        let permits = if config.enabled {
            config.max_concurrent
        } else {
            usize::MAX // 无限制
        };

        Self {
            semaphore: Arc::new(Semaphore::new(permits)),
            config,
        }
    }

    /// 获取许可（如果可用）
    pub async fn acquire(&self) -> Result<ConcurrencyPermit, ConcurrencyError> {
        if !self.config.enabled {
            return Ok(ConcurrencyPermit::unlimited());
        }

        // 尝试获取许可，带超时
        let permit = tokio::time::timeout(
            Duration::from_secs(30),
            self.semaphore.clone().acquire_owned(),
        )
        .await
        .map_err(|_| ConcurrencyError::Timeout)?
        .map_err(|_| ConcurrencyError::Closed)?;

        Ok(ConcurrencyPermit::new(permit))
    }

    /// 获取当前可用许可数
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// 更新配置
    pub fn update_config(&mut self, config: ConcurrencyConfig) {
        self.config = config.clone();
        
        if config.enabled {
            // 动态调整 semaphore 的许可数比较复杂
            // 这里简单记录配置更新
            debug!(
                max_concurrent = config.max_concurrent,
                "Concurrency config updated"
            );
        }
    }
}

/// 并发许可（RAII 模式）
pub struct ConcurrencyPermit {
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
}

impl ConcurrencyPermit {
    fn new(permit: tokio::sync::OwnedSemaphorePermit) -> Self {
        Self {
            permit: Some(permit),
        }
    }

    fn unlimited() -> Self {
        Self {
            permit: None,
        }
    }
}

impl Drop for ConcurrencyPermit {
    fn drop(&mut self) {
        // 自动释放许可
        drop(self.permit.take());
    }
}

#[derive(Debug, Clone)]
pub enum ConcurrencyError {
    Timeout,
    Closed,
}

impl std::fmt::Display for ConcurrencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConcurrencyError::Timeout => write!(f, "Concurrency limit timeout"),
            ConcurrencyError::Closed => write!(f, "Semaphore closed"),
        }
    }
}

impl std::error::Error for ConcurrencyError {}


