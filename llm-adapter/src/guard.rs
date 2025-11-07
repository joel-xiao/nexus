use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::debug;

#[derive(Clone, Debug)]
pub struct ConcurrencyConfig {
    pub max_concurrent: usize,
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

    pub async fn acquire(&self) -> Result<ConcurrencyPermit, ConcurrencyError> {
        if !self.config.enabled {
            return Ok(ConcurrencyPermit::unlimited());
        }

        let permit = tokio::time::timeout(
            Duration::from_secs(30),
            self.semaphore.clone().acquire_owned(),
        )
        .await
        .map_err(|_| ConcurrencyError::Timeout)?
        .map_err(|_| ConcurrencyError::Closed)?;

        Ok(ConcurrencyPermit::new(permit))
    }

    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    pub fn update_config(&mut self, config: ConcurrencyConfig) {
        self.config = config.clone();

        if config.enabled {
            debug!(
                max_concurrent = config.max_concurrent,
                "Concurrency config updated"
            );
        }
    }
}

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
        Self { permit: None }
    }
}

impl Drop for ConcurrencyPermit {
    fn drop(&mut self) {
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
