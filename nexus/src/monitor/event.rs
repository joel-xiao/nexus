use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tokio::sync::mpsc;
use std::sync::Arc;
use dashmap::DashMap;
use tracing::{info, warn};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub id: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub data: serde_json::Value,
    pub level: EventLevel,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum EventLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone)]
pub struct EventBus {
    subscribers: Arc<DashMap<String, mpsc::UnboundedSender<Event>>>,
    buffer: Arc<DashMap<String, Vec<Event>>>, // 临时缓冲区
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(DashMap::new()),
            buffer: Arc::new(DashMap::new()),
        }
    }

    pub fn subscribe(&self, name: String) -> mpsc::UnboundedReceiver<Event> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.subscribers.insert(name.clone(), tx);
        info!("Event subscriber registered: {}", name);
        rx
    }

    pub fn publish(&self, event: Event) {
        info!("Publishing event: {} ({})", event.id, event.event_type);
        
        // 广播给所有订阅者
        let mut failed = Vec::new();
        for entry in self.subscribers.iter() {
            if let Err(e) = entry.value().send(event.clone()) {
                warn!("Failed to send event to subscriber {}: {}", entry.key(), e);
                failed.push(entry.key().clone());
            }
        }
        
        // 移除失败的订阅者
        for name in failed {
            self.subscribers.remove(&name);
        }
        
        // 保存到缓冲区（用于审计）
        let buffer_key = format!("{}:{}", event.source, event.timestamp.format("%Y-%m-%d"));
        let mut buffer = self.buffer.entry(buffer_key).or_insert_with(Vec::new);
        buffer.push(event);
        
        // 限制缓冲区大小
        if buffer.len() > 1000 {
            buffer.remove(0);
        }
    }

    pub fn get_events(&self, source: &str, date: &str) -> Vec<Event> {
        let key = format!("{}:{}", source, date);
        self.buffer.get(&key)
            .map(|v| v.value().clone())
            .unwrap_or_default()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Event {
    pub fn new(event_type: String, source: String, data: serde_json::Value, level: EventLevel) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            source,
            data,
            level,
        }
    }
}

