use crate::monitor::event::{Event, EventBus, EventLevel};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuditRecord {
    pub id: String,
    pub action: String,
    pub user_id: Option<String>,
    pub resource_type: String,
    pub resource_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub result: String,
    pub metadata: serde_json::Value,
}

#[derive(Clone)]
pub struct AuditLog {
    event_bus: Arc<EventBus>,
}

impl AuditLog {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }

    pub fn log(&self, record: AuditRecord) {
        let event = Event::new(
            "audit".to_string(),
            "audit".to_string(),
            serde_json::to_value(&record).unwrap_or_default(),
            EventLevel::Info,
        );

        self.event_bus.publish(event);
    }

    pub fn log_action(
        &self,
        action: &str,
        resource_type: &str,
        resource_id: &str,
        user_id: Option<&str>,
        result: &str,
        metadata: serde_json::Value,
    ) {
        let record = AuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            action: action.to_string(),
            user_id: user_id.map(|s| s.to_string()),
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            timestamp: chrono::Utc::now(),
            result: result.to_string(),
            metadata,
        };

        self.log(record);
    }
}
