/// Agent 定义模块
/// 
/// 定义了 Agent 的核心接口和配置结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

/// Agent 角色类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    /// 用户代理
    User,
    /// 助手
    Assistant,
    /// 规划者 - 负责任务规划
    Planner,
    /// 执行者 - 负责具体执行
    Executor,
    /// 审查者 - 负责结果审查
    Reviewer,
    /// 协调者 - 协调多个代理
    Coordinator,
    /// 专家 - 特定领域专家
    Expert {
        domain: String
    },
    /// 自定义角色
    Custom {
        role_name: String
    },
}

/// Agent 能力定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    /// 能力名称
    pub name: String,
    /// 能力描述
    pub description: String,
    /// 是否启用
    pub enabled: bool,
    /// 能力参数
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent 唯一标识
    pub id: String,
    /// Agent 名称
    pub name: String,
    /// Agent 角色
    pub role: AgentRole,
    /// Agent 描述
    pub description: String,
    /// 系统提示词
    pub system_prompt: String,
    /// 使用的适配器名称
    pub adapter_name: String,
    /// 能力列表
    #[serde(default)]
    pub capabilities: Vec<AgentCapability>,
    /// 最大交互轮数
    #[serde(default)]
    pub max_turns: Option<usize>,
    /// 温度参数（0.0-1.0）
    #[serde(default)]
    pub temperature: Option<f32>,
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 额外配置
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

/// Agent 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// 消息 ID
    pub id: String,
    /// 发送者 Agent ID
    pub sender_id: String,
    /// 发送者 Agent 名称
    pub sender_name: String,
    /// 接收者 Agent ID（None 表示广播）
    pub receiver_id: Option<String>,
    /// 消息内容
    pub content: String,
    /// 消息类型
    pub message_type: MessageType,
    /// 元数据
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    /// 普通文本消息
    Text,
    /// 系统消息
    System,
    /// 任务消息
    Task,
    /// 结果消息
    Result,
    /// 错误消息
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

/// Agent 上下文
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// 会话历史
    pub conversation_history: Vec<AgentMessage>,
    /// 共享状态
    pub shared_state: HashMap<String, serde_json::Value>,
    /// Agent 本地状态
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

/// Agent 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// 响应消息
    pub message: AgentMessage,
    /// 是否继续对话
    pub should_continue: bool,
    /// 建议的下一个 Agent ID
    pub next_agent_id: Option<String>,
    /// 置信度（0.0-1.0）
    pub confidence: Option<f32>,
    /// 额外数据
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

/// Agent trait - 所有 Agent 必须实现
#[async_trait]
pub trait AgentFlowAgent: Send + Sync {
    /// 获取 Agent 配置
    fn config(&self) -> &AgentConfig;

    /// 获取 Agent ID
    fn id(&self) -> &str {
        &self.config().id
    }

    /// 获取 Agent 名称
    fn name(&self) -> &str {
        &self.config().name
    }

    /// 处理消息
    async fn process(
        &self,
        message: AgentMessage,
        context: &mut AgentContext,
    ) -> anyhow::Result<AgentResponse>;

    /// 检查是否可以处理此消息
    async fn can_handle(&self, message: &AgentMessage) -> bool {
        // 默认：检查是否是发给自己的消息或广播消息
        message.receiver_id.as_ref().map_or(true, |id| id == self.id())
    }

    /// 构建提示词（供实现使用）
    fn build_prompt(&self, message: &AgentMessage, context: &AgentContext) -> String {
        let mut prompt = String::new();
        
        // 添加系统提示
        prompt.push_str(&format!("系统角色: {}\n", self.config().system_prompt));
        prompt.push_str(&format!("当前任务: {}\n\n", message.content));
        
        // 添加对话历史（最近5条）
        let recent_messages = context.get_last_n_messages(5);
        if !recent_messages.is_empty() {
            prompt.push_str("最近对话:\n");
            for msg in recent_messages {
                prompt.push_str(&format!("[{}]: {}\n", msg.sender_name, msg.content));
            }
            prompt.push_str("\n");
        }
        
        prompt
    }
}

