use anyhow::Result;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct RedisCache {
    client: Arc<RwLock<Option<Client>>>,
    prefix: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheEntry<T> {
    pub value: T,
    pub expires_at: Option<i64>,
}

impl RedisCache {
    pub fn new(redis_url: Option<&str>, prefix: &str) -> Self {
        let client = if let Some(url) = redis_url {
            match Client::open(url) {
                Ok(c) => {
                    info!("Redis connection initialized: {}", url);
                    Some(c)
                }
                Err(e) => {
                    error!(
                        "Failed to connect to Redis: {}, using in-memory fallback",
                        e
                    );
                    None
                }
            }
        } else {
            debug!("Redis not configured, using in-memory fallback");
            None
        };

        Self {
            client: Arc::new(RwLock::new(client)),
            prefix: prefix.to_string(),
        }
    }

    fn key(&self, k: &str) -> String {
        format!("{}:{}", self.prefix, k)
    }

    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let client = self.client.read().await;
        if let Some(client) = client.as_ref() {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let value: Option<String> = conn.get(self.key(key)).await?;

            if let Some(v) = value {
                let entry: CacheEntry<T> = serde_json::from_str(&v)?;

                if let Some(expires) = entry.expires_at {
                    let now = chrono::Utc::now().timestamp();
                    if now > expires {
                        debug!("Cache key expired: {}", key);
                        let _: () = conn.del(self.key(key)).await?;
                        return Ok(None);
                    }
                }

                return Ok(Some(entry.value));
            }
        }
        Ok(None)
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> Result<()>
    where
        T: Serialize,
    {
        let client = self.client.read().await;
        if let Some(client) = client.as_ref() {
            let mut conn = client.get_multiplexed_async_connection().await?;

            let expires_at = ttl_seconds.map(|ttl| chrono::Utc::now().timestamp() + ttl as i64);

            let entry = CacheEntry { value, expires_at };

            let json = serde_json::to_string(&entry)?;

            if let Some(ttl) = ttl_seconds {
                conn.set_ex::<_, _, ()>(self.key(key), json, ttl).await?;
            } else {
                conn.set::<_, _, ()>(self.key(key), json).await?;
            }

            debug!("Cached key: {} (ttl: {:?})", key, ttl_seconds);
        }
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let client = self.client.read().await;
        if let Some(client) = client.as_ref() {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let _: () = conn.del(self.key(key)).await?;
        }
        Ok(())
    }

    pub async fn exists(&self, key: &str) -> Result<bool> {
        let client = self.client.read().await;
        if let Some(client) = client.as_ref() {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let exists: bool = conn.exists(self.key(key)).await?;
            Ok(exists)
        } else {
            Ok(false)
        }
    }
}

impl Default for RedisCache {
    fn default() -> Self {
        Self::new(None, "nexus")
    }
}
