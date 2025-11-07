use agentflow::{Agent, AgentConfig, AgentContext, AgentMessage, AgentResponse, LLMInvokeOptions, LLMProvider, MessageType};
use async_trait::async_trait;
use std::sync::Arc;

pub struct LLMAgent {
    config: AgentConfig,
    llm_provider: Arc<dyn LLMProvider>,
}

impl LLMAgent {
    pub fn new(config: AgentConfig, llm_provider: Arc<dyn LLMProvider>) -> Self {
        Self { config, llm_provider }
    }

    fn build_prompt(&self, message: &AgentMessage, context: &AgentContext) -> String {
        let role_name = match &self.config.role {
            agentflow::AgentRole::User => "User",
            agentflow::AgentRole::Assistant => "Assistant",
            agentflow::AgentRole::Planner => "Planner",
            agentflow::AgentRole::Executor => "Executor",
            agentflow::AgentRole::Reviewer => "Reviewer",
            agentflow::AgentRole::Coordinator => "Coordinator",
            agentflow::AgentRole::Expert { domain } => return self.build_expert_prompt(message, context, domain),
            agentflow::AgentRole::Custom { role_name } => return self.build_custom_prompt(message, context, role_name),
        };

        let mut prompt = format!("=== {} Agent: {} ===\n", role_name, self.config.name);
        prompt.push_str(&format!("System Instructions: {}\n\n", self.config.system_prompt));

        let recent_messages = context.get_last_n_messages(10);
        if !recent_messages.is_empty() {
            prompt.push_str("=== Conversation History ===\n");
            for msg in recent_messages {
                let sender_role = if msg.sender_id == "user" {
                    "User"
                } else {
                    "Agent"
                };
                prompt.push_str(&format!("[{}] {}: {}\n", sender_role, msg.sender_name, msg.content));
            }
            prompt.push_str("\n");
        }

        prompt.push_str("=== Current Message ===\n");
        prompt.push_str(&format!("From: {}\n", message.sender_name));
        prompt.push_str(&format!("Content: {}\n\n", message.content));
        prompt.push_str(&format!("=== Your Response as {} ===\n", role_name));

        prompt
    }

    fn build_expert_prompt(&self, message: &AgentMessage, context: &AgentContext, domain: &str) -> String {
        let mut prompt = format!("=== Expert Agent: {} (Domain: {}) ===\n", self.config.name, domain);
        prompt.push_str(&format!("System Instructions: {}\n\n", self.config.system_prompt));

        let recent_messages = context.get_last_n_messages(10);
        if !recent_messages.is_empty() {
            prompt.push_str("=== Conversation History ===\n");
            for msg in recent_messages {
                prompt.push_str(&format!("[{}]: {}\n", msg.sender_name, msg.content));
            }
            prompt.push_str("\n");
        }

        prompt.push_str("=== Current Request ===\n");
        prompt.push_str(&format!("From: {}\n", message.sender_name));
        prompt.push_str(&format!("Request: {}\n\n", message.content));
        prompt.push_str(&format!("=== Your Expert Response (Domain: {}) ===\n", domain));

        prompt
    }

    fn build_custom_prompt(&self, message: &AgentMessage, context: &AgentContext, role_name: &str) -> String {
        let mut prompt = format!("=== {} Agent: {} ===\n", role_name, self.config.name);
        prompt.push_str(&format!("System Instructions: {}\n\n", self.config.system_prompt));

        let recent_messages = context.get_last_n_messages(10);
        if !recent_messages.is_empty() {
            prompt.push_str("=== Conversation History ===\n");
            for msg in recent_messages {
                prompt.push_str(&format!("[{}]: {}\n", msg.sender_name, msg.content));
            }
            prompt.push_str("\n");
        }

        prompt.push_str("=== Current Message ===\n");
        prompt.push_str(&format!("From: {}\n", message.sender_name));
        prompt.push_str(&format!("Content: {}\n\n", message.content));
        prompt.push_str(&format!("=== Your Response as {} ===\n", role_name));

        prompt
    }
}

#[async_trait]
impl Agent for LLMAgent {
    fn config(&self) -> &AgentConfig {
        &self.config
    }

    async fn can_handle(&self, message: &AgentMessage) -> bool {
        if let Some(ref receiver_id) = message.receiver_id {
            if receiver_id != &self.config.id {
                return false;
            }
        }

        match &self.config.role {
            agentflow::AgentRole::User => true,
            agentflow::AgentRole::Assistant => true,
            agentflow::AgentRole::Planner => {
                message.content.to_lowercase().contains("plan") ||
                message.content.to_lowercase().contains("规划") ||
                message.content.to_lowercase().contains("步骤") ||
                message.message_type == MessageType::Task
            },
            agentflow::AgentRole::Executor => {
                message.content.to_lowercase().contains("execute") ||
                message.content.to_lowercase().contains("执行") ||
                message.content.to_lowercase().contains("完成") ||
                message.message_type == MessageType::Task
            },
            agentflow::AgentRole::Reviewer => {
                message.content.to_lowercase().contains("review") ||
                message.content.to_lowercase().contains("review") ||
                message.content.to_lowercase().contains("检查") ||
                message.message_type == MessageType::Result
            },
            agentflow::AgentRole::Coordinator => true,
            agentflow::AgentRole::Expert { domain } => {
                message.content.to_lowercase().contains(&domain.to_lowercase())
            },
            agentflow::AgentRole::Custom { .. } => true,
        }
    }

    async fn process(
        &self,
        message: AgentMessage,
        context: &mut AgentContext,
    ) -> anyhow::Result<AgentResponse> {
        let prompt = self.build_prompt(&message, context);

        context.add_message(message.clone());

        let options = LLMInvokeOptions {
            user_id: Some(message.sender_id.clone()),
            model: self.config.metadata.get("model").and_then(|v| v.as_str().map(|s| s.to_string())),
            temperature: self.config.temperature,
            max_tokens: self.config.metadata.get("max_tokens").and_then(|v| v.as_u64().map(|u| u as u32)),
            metadata: self.config.metadata.clone(),
        };

        let result = self.llm_provider.invoke(&prompt, &options).await?;

        let response_msg = AgentMessage::new(
            self.config.id.clone(),
            self.config.name.clone(),
            Some(message.sender_id.clone()),
            result,
            MessageType::Result,
        );

        let mut response = AgentResponse::new(response_msg);
        
        match &self.config.role {
            agentflow::AgentRole::Planner => {
                if response.message.content.to_lowercase().contains("next") ||
                   response.message.content.to_lowercase().contains("下一步") {
                    response = response.next("executor".to_string());
                }
            },
            agentflow::AgentRole::Executor => {
                if response.message.content.to_lowercase().contains("review") ||
                   response.message.content.to_lowercase().contains("检查") {
                    response = response.next("reviewer".to_string());
                }
            },
            _ => {}
        }

        Ok(response)
    }
}
