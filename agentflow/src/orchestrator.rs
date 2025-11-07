/// Agent 编排器
/// 
/// 负责协调多个 Agent 的交互和工作流程

use crate::agent::{AgentFlowAgent, AgentMessage, AgentResponse, AgentContext, MessageType};
use crate::workflow::{Workflow, WorkflowEngine};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};

/// 编排配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    /// 会话ID
    pub session_id: String,
    /// 最大轮数
    #[serde(default = "default_max_rounds")]
    pub max_rounds: usize,
    /// 超时时间（秒）
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    /// 是否启用自动规划
    #[serde(default)]
    pub auto_planning: bool,
    /// 是否记录历史
    #[serde(default = "default_true")]
    pub save_history: bool,
    /// 额外配置
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_max_rounds() -> usize {
    20
}

fn default_timeout() -> u64 {
    300
}

fn default_true() -> bool {
    true
}

impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            max_rounds: default_max_rounds(),
            timeout_seconds: default_timeout(),
            auto_planning: false,
            save_history: true,
            metadata: HashMap::new(),
        }
    }
}

/// 编排结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationResult {
    /// 会话ID
    pub session_id: String,
    /// 最终结果
    pub result: String,
    /// 执行轮数
    pub rounds: usize,
    /// 是否成功完成
    pub success: bool,
    /// 参与的 Agent 列表
    pub agents_used: Vec<String>,
    /// 消息历史
    pub message_history: Vec<AgentMessage>,
    /// 耗时（秒）
    pub duration_seconds: f64,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Agent 编排器
pub struct AgentOrchestrator {
    /// 注册的 Agent
    agents: Arc<RwLock<HashMap<String, Arc<dyn AgentFlowAgent>>>>,
    /// 工作流引擎
    workflow_engine: Option<Arc<WorkflowEngine>>,
    /// 配置
    config: OrchestrationConfig,
}

impl AgentOrchestrator {
    pub fn new(config: OrchestrationConfig) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            workflow_engine: None,
            config,
        }
    }

    /// 注册 Agent
    pub async fn register_agent(&self, agent: Arc<dyn AgentFlowAgent>) {
        let agent_id = agent.id().to_string();
        let mut agents = self.agents.write().await;
        agents.insert(agent_id.clone(), agent);
        info!("Registered agent: {}", agent_id);
    }

    /// 移除 Agent
    pub async fn unregister_agent(&self, agent_id: &str) -> bool {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id).is_some()
    }

    /// 获取 Agent
    pub async fn get_agent(&self, agent_id: &str) -> Option<Arc<dyn AgentFlowAgent>> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// 列出所有 Agent
    pub async fn list_agents(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    /// 设置工作流引擎
    pub fn with_workflow_engine(mut self, engine: Arc<WorkflowEngine>) -> Self {
        self.workflow_engine = Some(engine);
        self
    }

    /// 执行单轮对话
    pub async fn execute_round(
        &self,
        message: AgentMessage,
        context: &mut AgentContext,
    ) -> anyhow::Result<Vec<AgentResponse>> {
        let mut responses = Vec::new();

        // 添加消息到上下文
        context.add_message(message.clone());

        // 如果指定了接收者，直接发送给该 Agent
        if let Some(ref receiver_id) = message.receiver_id {
            if let Some(agent) = self.get_agent(receiver_id).await {
                debug!("Sending message to specific agent: {}", receiver_id);
                let response = agent.process(message, context).await?;
                responses.push(response);
            } else {
                anyhow::bail!("Agent not found: {}", receiver_id);
            }
        } else {
            // 广播模式：找到所有可以处理的 Agent
            let agents = self.agents.read().await;
            for (agent_id, agent) in agents.iter() {
                if agent.can_handle(&message).await {
                    debug!("Agent {} can handle the message", agent_id);
                    match agent.process(message.clone(), context).await {
                        Ok(response) => responses.push(response),
                        Err(e) => {
                            warn!("Agent {} failed to process message: {}", agent_id, e);
                        }
                    }
                }
            }
        }

        Ok(responses)
    }

    /// 执行多轮对话
    pub async fn orchestrate(
        &self,
        initial_message: String,
        initial_agent_id: Option<String>,
    ) -> anyhow::Result<OrchestrationResult> {
        let start_time = std::time::Instant::now();
        let mut context = AgentContext::new();
        let mut agents_used = Vec::new();
        let mut current_round = 0;

        // 创建初始消息
        let mut current_message = AgentMessage::new(
            "user".to_string(),
            "User".to_string(),
            initial_agent_id,
            initial_message,
            MessageType::Text,
        );

        let mut final_result = String::new();
        let mut success = false;

        while current_round < self.config.max_rounds {
            current_round += 1;
            info!("Orchestration round {}/{}", current_round, self.config.max_rounds);

            // 执行当前轮
            let responses = match self.execute_round(current_message.clone(), &mut context).await {
                Ok(responses) => responses,
                Err(e) => {
                    warn!("Round {} failed: {}", current_round, e);
                    break;
                }
            };

            if responses.is_empty() {
                warn!("No agents responded in round {}", current_round);
                break;
            }

            // 处理响应
            let mut should_continue = false;
            for response in responses {
                // 记录使用的 Agent
                if !agents_used.contains(&response.message.sender_id) {
                    agents_used.push(response.message.sender_id.clone());
                }

                // 添加响应消息到上下文
                context.add_message(response.message.clone());

                // 更新最终结果
                final_result = response.message.content.clone();

                // 检查是否需要继续
                if response.should_continue {
                    should_continue = true;
                    
                    // 如果有指定下一个 Agent，创建新消息
                    if let Some(next_agent_id) = response.next_agent_id {
                        current_message = AgentMessage::new(
                            response.message.sender_id,
                            response.message.sender_name,
                            Some(next_agent_id),
                            response.message.content,
                            MessageType::Task,
                        );
                    } else {
                        // 广播消息
                        current_message = AgentMessage::new(
                            response.message.sender_id,
                            response.message.sender_name,
                            None,
                            response.message.content,
                            MessageType::Task,
                        );
                    }
                } else {
                    success = true;
                }
            }

            if !should_continue {
                info!("Orchestration completed successfully in {} rounds", current_round);
                break;
            }
        }

        let duration = start_time.elapsed().as_secs_f64();

        Ok(OrchestrationResult {
            session_id: self.config.session_id.clone(),
            result: final_result,
            rounds: current_round,
            success,
            agents_used,
            message_history: context.conversation_history,
            duration_seconds: duration,
            metadata: self.config.metadata.clone(),
        })
    }

    /// 使用工作流执行
    pub async fn orchestrate_with_workflow(
        &self,
        workflow: &Workflow,
        input: HashMap<String, serde_json::Value>,
    ) -> anyhow::Result<OrchestrationResult> {
        if let Some(ref engine) = self.workflow_engine {
            let start_time = std::time::Instant::now();
            
            // 执行工作流
            let workflow_result = engine.execute(workflow, input, &self.agents).await?;
            
            let duration = start_time.elapsed().as_secs_f64();

            Ok(OrchestrationResult {
                session_id: self.config.session_id.clone(),
                result: workflow_result.final_output,
                rounds: workflow_result.steps_executed.len(),
                success: workflow_result.success,
                agents_used: workflow_result.agents_used,
                message_history: Vec::new(), // 工作流执行不保存消息历史
                duration_seconds: duration,
                metadata: workflow_result.metadata,
            })
        } else {
            anyhow::bail!("Workflow engine not configured")
        }
    }

    /// 获取配置
    pub fn config(&self) -> &OrchestrationConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let config = OrchestrationConfig::default();
        let orchestrator = AgentOrchestrator::new(config);
        
        let agents = orchestrator.list_agents().await;
        assert_eq!(agents.len(), 0);
    }
}

