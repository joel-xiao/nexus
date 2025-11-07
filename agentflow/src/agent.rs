use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    User,
    Assistant,
    Planner,
    Executor,
    Reviewer,
    Coordinator,
    Expert { domain: String },
    Custom { role_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub description: String,
    pub system_prompt: String,
    pub adapter_name: String,
    #[serde(default)]
    pub capabilities: Vec<AgentCapability>,
    #[serde(default)]
    pub max_turns: Option<usize>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_enabled() -> bool {
    true
}

impl AgentConfig {
    pub fn new(
        id: String,
        name: String,
        role: AgentRole,
        description: String,
        system_prompt: String,
        adapter_name: String,
    ) -> Self {
        Self {
            id,
            name,
            role,
            description,
            system_prompt,
            adapter_name,
            capabilities: Vec::new(),
            max_turns: None,
            temperature: None,
            enabled: true,
            metadata: HashMap::new(),
        }
    }

    pub fn with_capability(mut self, capability: AgentCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    pub fn with_max_turns(mut self, max_turns: usize) -> Self {
        self.max_turns = Some(max_turns);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub receiver_id: Option<String>,
    pub content: String,
    pub message_type: MessageType,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Text,
    System,
    Task,
    Result,
    Error,
}

impl AgentMessage {
    pub fn new(
        sender_id: String,
        sender_name: String,
        receiver_id: Option<String>,
        content: String,
        message_type: MessageType,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender_id,
            sender_name,
            receiver_id,
            content,
            message_type,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub conversation_history: Vec<AgentMessage>,
    pub shared_state: HashMap<String, serde_json::Value>,
    pub local_state: HashMap<String, serde_json::Value>,
}

impl AgentContext {
    pub fn new() -> Self {
        Self {
            conversation_history: Vec::new(),
            shared_state: HashMap::new(),
            local_state: HashMap::new(),
        }
    }

    pub fn add_message(&mut self, message: AgentMessage) {
        self.conversation_history.push(message);
    }

    pub fn get_shared(&self, key: &str) -> Option<&serde_json::Value> {
        self.shared_state.get(key)
    }

    pub fn set_shared(&mut self, key: String, value: serde_json::Value) {
        self.shared_state.insert(key, value);
    }

    pub fn get_local(&self, key: &str) -> Option<&serde_json::Value> {
        self.local_state.get(key)
    }

    pub fn set_local(&mut self, key: String, value: serde_json::Value) {
        self.local_state.insert(key, value);
    }

    pub fn get_last_n_messages(&self, n: usize) -> Vec<&AgentMessage> {
        let len = self.conversation_history.len();
        if len <= n {
            self.conversation_history.iter().collect()
        } else {
            self.conversation_history[len - n..].iter().collect()
        }
    }
}

impl Default for AgentContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub message: AgentMessage,
    pub should_continue: bool,
    pub next_agent_id: Option<String>,
    pub confidence: Option<f32>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AgentResponse {
    pub fn new(message: AgentMessage) -> Self {
        Self {
            message,
            should_continue: true,
            next_agent_id: None,
            confidence: None,
            metadata: HashMap::new(),
        }
    }

    pub fn done(mut self) -> Self {
        self.should_continue = false;
        self
    }

    pub fn next(mut self, agent_id: String) -> Self {
        self.next_agent_id = Some(agent_id);
        self
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[async_trait]
pub trait AgentFlowAgent: Send + Sync {
    fn config(&self) -> &AgentConfig;

    fn id(&self) -> &str {
        &self.config().id
    }

    fn name(&self) -> &str {
        &self.config().name
    }

    async fn process(
        &self,
        message: AgentMessage,
        context: &mut AgentContext,
    ) -> anyhow::Result<AgentResponse>;

    async fn can_handle(&self, message: &AgentMessage) -> bool {
        message
            .receiver_id
            .as_ref()
            .map_or(true, |id| id == self.id())
    }

    fn build_prompt(&self, message: &AgentMessage, context: &AgentContext) -> String {
        let mut prompt = format!(
            "系统角色: {}\n当前任务: {}\n\n",
            self.config().system_prompt,
            message.content
        );

        let recent_messages = context.get_last_n_messages(5);
        if !recent_messages.is_empty() {
            prompt.push_str("最近对话:\n");
            for msg in recent_messages {
                prompt.push_str(&format!("[{}]: {}\n", msg.sender_name, msg.content));
            }
            prompt.push('\n');
        }

        prompt
    }
}
