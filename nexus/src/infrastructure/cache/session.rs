use crate::infrastructure::cache::RedisCache;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub session_id: String,
    pub user_id: Option<String>,
    pub data: HashMap<String, String>,
    pub created_at: i64,
    pub last_accessed: i64,
}

#[derive(Clone)]
pub struct SessionCache {
    redis: RedisCache,
    memory: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionCache {
    pub fn new(redis_url: Option<&str>) -> Self {
        Self {
            redis: RedisCache::new(redis_url, "session"),
            memory: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, session_id: &str) -> Result<Option<Session>> {
        // 先尝试从 Redis 获取
        if let Ok(Some(session)) = self.redis.get::<Session>(session_id).await {
            return Ok(Some(session));
        }
        
        // 回退到内存
        let memory = self.memory.read().await;
        Ok(memory.get(session_id).cloned())
    }

    pub async fn set(&self, session: &Session, ttl_seconds: Option<u64>) -> Result<()> {
        // 写入 Redis
        let _ = self.redis.set(&session.session_id, session, ttl_seconds).await;
        
        // 同时写入内存作为备份
        let mut memory = self.memory.write().await;
        memory.insert(session.session_id.clone(), session.clone());
        
        Ok(())
    }

    pub async fn delete(&self, session_id: &str) -> Result<()> {
        let _ = self.redis.delete(session_id).await;
        let mut memory = self.memory.write().await;
        memory.remove(session_id);
        Ok(())
    }
}

impl Default for SessionCache {
    fn default() -> Self {
        Self::new(None)
    }
}

