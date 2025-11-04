use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retrying,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub id: String,
    pub task_type: String,
    pub payload: serde_json::Value,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub retry_count: u32,
    pub max_retries: u32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub result: Option<serde_json::Value>,
    pub idempotency_key: Option<String>, // 幂等性键
}

impl Task {
    pub fn new(task_type: String, payload: serde_json::Value, priority: TaskPriority) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type,
            payload,
            status: TaskStatus::Pending,
            priority,
            retry_count: 0,
            max_retries: 3,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
            result: None,
            idempotency_key: None,
        }
    }

    pub fn with_idempotency(mut self, key: String) -> Self {
        self.idempotency_key = Some(key);
        self
    }

    pub fn with_max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    pub fn mark_processing(&mut self) {
        self.status = TaskStatus::Processing;
        self.started_at = Some(Utc::now());
    }

    pub fn mark_completed(&mut self, result: serde_json::Value) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.result = Some(result);
    }

    pub fn mark_failed(&mut self, error: String) {
        if self.can_retry() {
            self.status = TaskStatus::Retrying;
            self.retry_count += 1;
        } else {
            self.status = TaskStatus::Failed;
            self.completed_at = Some(Utc::now());
        }
        self.error = Some(error);
    }
}

