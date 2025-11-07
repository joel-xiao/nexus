/// LLM Agent 桥接实现
/// 
/// 将 llm-adapter 的 Adapter 适配为 agentflow 的 Agent

use agentflow::{Agent, AgentConfig, AgentMessage, AgentResponse, AgentContext, MessageType};
use llm_adapter::Adapter;
use std::sync::Arc;
use async_trait::async_trait;

/// LLM Agent - 桥接 Adapter 和 Agent
pub struct LLMAgent {
    config: AgentConfig,
    adapter: Arc<dyn Adapter + Send + Sync>,
}

impl LLMAgent {
    pub fn new(config: AgentConfig, adapter: Arc<dyn Adapter + Send + Sync>) -> Self {
        Self { config, adapter }
    }
}

#[async_trait]
impl Agent for LLMAgent {
    fn config(&self) -> &AgentConfig {
        &self.config
    }

    async fn process(
        &self,
        message: AgentMessage,
        context: &mut AgentContext,
    ) -> anyhow::Result<AgentResponse> {
        // 构建提示词
        let prompt = self.build_prompt(&message, context);
        
        // 添加消息到历史
        context.add_message(message.clone());
        
        // 调用 LLM
        let result = self.adapter.invoke(&prompt).await?;
        
        // 创建响应消息
        let response_msg = AgentMessage::new(
            self.config.id.clone(),
            self.config.name.clone(),
            message.sender_id.into(),
            result,
            MessageType::Result,
        );
        
        Ok(AgentResponse::new(response_msg))
    }
}

