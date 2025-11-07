use crate::infrastructure::messaging::mcp::message::McpMessage;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn};

pub type MessageId = String;

#[derive(Clone)]
pub struct McpBus {
    tasks: Arc<RwLock<HashMap<MessageId, McpMessage>>>,
    subscribers: Arc<RwLock<HashMap<String, mpsc::Sender<McpMessage>>>>,
}

impl McpBus {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn publish(&self, message: McpMessage) -> MessageId {
        let id = uuid::Uuid::new_v4().to_string();
        let mut tasks = self.tasks.write().await;
        tasks.insert(id.clone(), message.clone());
        info!("Published message: {}", id);

        let subscribers = self.subscribers.read().await;
        for (name, tx) in subscribers.iter() {
            if let Err(e) = tx.send(message.clone()).await {
                warn!("Failed to send to subscriber {}: {}", name, e);
            }
        }

        id
    }

    pub async fn get(&self, id: &MessageId) -> Option<McpMessage> {
        let tasks = self.tasks.read().await;
        tasks.get(id).cloned()
    }

    pub async fn update(&self, id: &MessageId, message: McpMessage) {
        let mut tasks = self.tasks.write().await;
        tasks.insert(id.clone(), message);
        info!("Updated message: {}", id);
    }

    pub async fn subscribe(&self, name: String) -> mpsc::Receiver<McpMessage> {
        let (tx, rx) = mpsc::channel(100);
        let mut subscribers = self.subscribers.write().await;
        subscribers.insert(name.clone(), tx);
        info!("Added subscriber: {}", name);
        rx
    }

    pub async fn list_tasks(&self) -> Vec<(MessageId, McpMessage)> {
        let tasks = self.tasks.read().await;
        tasks.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}

impl Default for McpBus {
    fn default() -> Self {
        Self::new()
    }
}
