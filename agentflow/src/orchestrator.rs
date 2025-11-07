use crate::agent::{AgentContext, AgentFlowAgent, AgentMessage, AgentResponse, MessageType};
use crate::workflow::{Workflow, WorkflowEngine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpeakerSelection {
    RoundRobin,
    Random,
    Manual,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    pub session_id: String,
    #[serde(default = "default_max_rounds")]
    pub max_rounds: usize,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default)]
    pub auto_planning: bool,
    #[serde(default = "default_true")]
    pub save_history: bool,
    #[serde(default = "default_speaker_selection")]
    pub speaker_selection: SpeakerSelection,
    #[serde(default)]
    pub agent_order: Vec<String>,
    #[serde(default)]
    pub termination_condition: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_speaker_selection() -> SpeakerSelection {
    SpeakerSelection::RoundRobin
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
            speaker_selection: SpeakerSelection::RoundRobin,
            agent_order: Vec::new(),
            termination_condition: Some("TERMINATE".to_string()),
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationResult {
    pub session_id: String,
    pub result: String,
    pub rounds: usize,
    pub success: bool,
    pub agents_used: Vec<String>,
    pub message_history: Vec<AgentMessage>,
    pub duration_seconds: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct AgentOrchestrator {
    agents: Arc<RwLock<HashMap<String, Arc<dyn AgentFlowAgent>>>>,
    workflow_engine: Option<Arc<WorkflowEngine>>,
    config: OrchestrationConfig,
    current_speaker_index: Arc<tokio::sync::Mutex<usize>>,
}

impl AgentOrchestrator {
    pub fn new(config: OrchestrationConfig) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            workflow_engine: None,
            config,
            current_speaker_index: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    pub async fn register_agent(&self, agent: Arc<dyn AgentFlowAgent>) {
        let agent_id = agent.id().to_string();
        let mut agents = self.agents.write().await;
        agents.insert(agent_id.clone(), agent);
        info!("Registered agent: {}", agent_id);
    }

    pub async fn unregister_agent(&self, agent_id: &str) -> bool {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id).is_some()
    }

    pub async fn get_agent(&self, agent_id: &str) -> Option<Arc<dyn AgentFlowAgent>> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    pub async fn list_agents(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    pub fn with_workflow_engine(mut self, engine: Arc<WorkflowEngine>) -> Self {
        self.workflow_engine = Some(engine);
        self
    }

    async fn select_next_speaker(&self) -> Option<String> {
        let agents = self.agents.read().await;
        let agent_ids: Vec<String> = if !self.config.agent_order.is_empty() {
            self.config
                .agent_order
                .iter()
                .filter(|id| agents.contains_key(*id))
                .cloned()
                .collect()
        } else {
            agents.keys().cloned().collect()
        };

        if agent_ids.is_empty() {
            return None;
        }

        match self.config.speaker_selection {
            SpeakerSelection::RoundRobin => {
                let mut idx = self.current_speaker_index.lock().await;
                let selected = agent_ids[*idx % agent_ids.len()].clone();
                *idx += 1;
                Some(selected)
            }
            SpeakerSelection::Random => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                std::time::SystemTime::now().hash(&mut hasher);
                let hash = hasher.finish();
                let idx = (hash as usize) % agent_ids.len();
                Some(agent_ids[idx].clone())
            }
            SpeakerSelection::Manual | SpeakerSelection::Auto => {
                let mut idx = self.current_speaker_index.lock().await;
                let selected = agent_ids[*idx % agent_ids.len()].clone();
                *idx += 1;
                Some(selected)
            }
        }
    }

    fn check_termination(&self, message: &str) -> bool {
        if let Some(ref condition) = self.config.termination_condition {
            message.to_uppercase().contains(&condition.to_uppercase())
        } else {
            false
        }
    }

    pub async fn execute_round(
        &self,
        message: AgentMessage,
        context: &mut AgentContext,
    ) -> anyhow::Result<Vec<AgentResponse>> {
        let mut responses = Vec::new();

        context.add_message(message.clone());

        if let Some(ref receiver_id) = message.receiver_id {
            if let Some(agent) = self.get_agent(receiver_id).await {
                debug!("Sending message to specific agent: {}", receiver_id);
                let response = agent.process(message, context).await?;
                responses.push(response);
            } else {
                anyhow::bail!("Agent not found: {}", receiver_id);
            }
        } else {
            match self.config.speaker_selection {
                SpeakerSelection::RoundRobin | SpeakerSelection::Random | SpeakerSelection::Manual => {
                    if let Some(speaker_id) = self.select_next_speaker().await {
                        if let Some(agent) = self.get_agent(&speaker_id).await {
                            debug!("Selected speaker: {}", speaker_id);
                            match agent.process(message.clone(), context).await {
                                Ok(response) => responses.push(response),
                                Err(e) => {
                                    warn!("Agent {} failed to process message: {}", speaker_id, e);
                                }
                            }
                        }
                    }
                }
                SpeakerSelection::Auto => {
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
            }
        }

        Ok(responses)
    }

    pub async fn orchestrate(
        &self,
        initial_message: String,
        initial_agent_id: Option<String>,
    ) -> anyhow::Result<OrchestrationResult> {
        let start_time = std::time::Instant::now();
        let mut context = AgentContext::new();
        let mut agents_used = Vec::new();
        let mut current_round = 0;

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
            info!(
                "Orchestration round {}/{}",
                current_round, self.config.max_rounds
            );

            let responses = match self
                .execute_round(current_message.clone(), &mut context)
                .await
            {
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

            let mut should_continue = false;
            let mut selected_response: Option<AgentResponse> = None;

            for response in responses {
                if !agents_used.contains(&response.message.sender_id) {
                    agents_used.push(response.message.sender_id.clone());
                }

                context.add_message(response.message.clone());

                if self.check_termination(&response.message.content) {
                    info!("Termination condition met: {}", response.message.content);
                    final_result = response.message.content.clone();
                    success = true;
                    break;
                }

                if selected_response.is_none() || response.should_continue {
                    selected_response = Some(response.clone());
                }

                final_result = response.message.content.clone();
            }

            if let Some(response) = selected_response {
                if response.should_continue && !self.check_termination(&response.message.content) {
                    should_continue = true;

                    if let Some(next_agent_id) = response.next_agent_id {
                        current_message = AgentMessage::new(
                            response.message.sender_id,
                            response.message.sender_name,
                            Some(next_agent_id),
                            response.message.content,
                            MessageType::Task,
                        );
                    } else {
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
                info!(
                    "Orchestration completed successfully in {} rounds",
                    current_round
                );
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

    pub async fn orchestrate_with_workflow(
        &self,
        workflow: &Workflow,
        input: HashMap<String, serde_json::Value>,
    ) -> anyhow::Result<OrchestrationResult> {
        if let Some(ref engine) = self.workflow_engine {
            let start_time = std::time::Instant::now();

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
