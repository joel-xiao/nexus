use crate::infrastructure::cache::RedisCache;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use md5;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmbeddingEntry {
    pub text: String,
    pub embedding: Vec<f32>,
    pub model: String,
}

#[derive(Clone)]
pub struct EmbeddingCache {
    redis: RedisCache,
}

impl EmbeddingCache {
    pub fn new(redis_url: Option<&str>) -> Self {
        Self {
            redis: RedisCache::new(redis_url, "embedding"),
        }
    }

    fn key(&self, text: &str, model: &str) -> String {
        let content = format!("{}:{}", model, text);
        let hash = md5::compute(content.as_bytes());
        format!("{:x}", hash)
    }

    pub async fn get(&self, text: &str, model: &str) -> Result<Option<Vec<f32>>> {
        let key = self.key(text, model);
        if let Ok(Some(entry)) = self.redis.get::<EmbeddingEntry>(&key).await {
            Ok(Some(entry.embedding))
        } else {
            Ok(None)
        }
    }

    pub async fn set(&self, text: &str, model: &str, embedding: Vec<f32>, ttl_seconds: Option<u64>) -> Result<()> {
        let key = self.key(text, model);
        let entry = EmbeddingEntry {
            text: text.to_string(),
            embedding,
            model: model.to_string(),
        };
        self.redis.set(&key, &entry, ttl_seconds).await
    }
}

impl Default for EmbeddingCache {
    fn default() -> Self {
        Self::new(None)
    }
}

