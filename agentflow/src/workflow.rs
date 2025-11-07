use crate::agent::{AgentContext, AgentFlowAgent, AgentMessage, MessageType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    AgentExecution,
    ParallelExecution,
    ConditionalBranch,
    Loop,
    HumanReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub step_type: StepType,
    pub agent_id: Option<String>,
    #[serde(default)]
    pub agent_ids: Vec<String>,
    #[serde(default)]
    pub input_mapping: HashMap<String, String>,
    pub output_key: String,
    pub condition: Option<String>,
    pub next_step_id: Option<String>,
    pub true_step_id: Option<String>,
    pub false_step_id: Option<String>,
    pub max_iterations: Option<usize>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}

impl WorkflowStep {
    pub fn new_agent_execution(
        id: String,
        name: String,
        agent_id: String,
        output_key: String,
    ) -> Self {
        Self {
            id,
            name,
            step_type: StepType::AgentExecution,
            agent_id: Some(agent_id),
            agent_ids: Vec::new(),
            input_mapping: HashMap::new(),
            output_key,
            condition: None,
            next_step_id: None,
            true_step_id: None,
            false_step_id: None,
            max_iterations: None,
            enabled: true,
            metadata: HashMap::new(),
        }
    }

    pub fn new_parallel_execution(
        id: String,
        name: String,
        agent_ids: Vec<String>,
        output_key: String,
    ) -> Self {
        Self {
            id,
            name,
            step_type: StepType::ParallelExecution,
            agent_id: None,
            agent_ids,
            input_mapping: HashMap::new(),
            output_key,
            condition: None,
            next_step_id: None,
            true_step_id: None,
            false_step_id: None,
            max_iterations: None,
            enabled: true,
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_now")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default = "default_now")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Untitled Workflow".to_string(),
            description: String::new(),
            version: default_version(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            enabled: true,
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub config: WorkflowConfig,
    pub steps: Vec<WorkflowStep>,
    pub start_step_id: String,
}

impl Workflow {
    pub fn new(config: WorkflowConfig, steps: Vec<WorkflowStep>, start_step_id: String) -> Self {
        Self {
            config,
            steps,
            start_step_id,
        }
    }

    pub fn get_step(&self, step_id: &str) -> Option<&WorkflowStep> {
        self.steps.iter().find(|s| s.id == step_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub workflow_id: String,
    pub success: bool,
    pub final_output: String,
    pub steps_executed: Vec<String>,
    pub agents_used: Vec<String>,
    pub step_outputs: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
    pub duration_seconds: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct WorkflowEngine {
    max_steps: usize,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self { max_steps: 100 }
    }

    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    pub async fn execute(
        &self,
        workflow: &Workflow,
        initial_input: HashMap<String, serde_json::Value>,
        agents: &Arc<RwLock<HashMap<String, Arc<dyn AgentFlowAgent>>>>,
    ) -> anyhow::Result<WorkflowResult> {
        let start_time = std::time::Instant::now();
        let mut context = AgentContext::new();

        for (key, value) in initial_input {
            context.set_shared(key, value);
        }

        let mut steps_executed = Vec::new();
        let mut agents_used = Vec::new();
        let mut step_outputs = HashMap::new();
        let mut current_step_id = Some(workflow.start_step_id.clone());
        let mut iterations = 0;

        info!("Starting workflow execution: {}", workflow.config.name);

        while let Some(step_id) = current_step_id {
            if iterations >= self.max_steps {
                error!("Workflow exceeded maximum steps: {}", self.max_steps);
                return Ok(WorkflowResult {
                    workflow_id: workflow.config.id.clone(),
                    success: false,
                    final_output: String::new(),
                    steps_executed,
                    agents_used,
                    step_outputs,
                    error: Some("Exceeded maximum steps".to_string()),
                    duration_seconds: start_time.elapsed().as_secs_f64(),
                    metadata: HashMap::new(),
                });
            }

            iterations += 1;

            let step = match workflow.get_step(&step_id) {
                Some(step) => step,
                None => {
                    error!("Step not found: {}", step_id);
                    break;
                }
            };

            if !step.enabled {
                debug!("Skipping disabled step: {}", step.name);
                current_step_id = step.next_step_id.clone();
                continue;
            }

            info!("Executing step: {} ({})", step.name, step.id);
            steps_executed.push(step.id.clone());

            let step_input = self.build_step_input(step, &context);

            let (output, next_id) = match step.step_type {
                StepType::AgentExecution => {
                    self.execute_agent_step(
                        step,
                        step_input,
                        &mut context,
                        agents,
                        &mut agents_used,
                    )
                    .await?
                }
                StepType::ParallelExecution => {
                    self.execute_parallel_step(
                        step,
                        step_input,
                        &mut context,
                        agents,
                        &mut agents_used,
                    )
                    .await?
                }
                StepType::ConditionalBranch => self.execute_conditional_step(step, &context)?,
                StepType::Loop => (serde_json::Value::Null, step.next_step_id.clone()),
                StepType::HumanReview => (serde_json::Value::Null, step.next_step_id.clone()),
            };

            step_outputs.insert(step.output_key.clone(), output.clone());
            context.set_shared(step.output_key.clone(), output);

            current_step_id = next_id;
        }

        let duration = start_time.elapsed().as_secs_f64();
        let final_output = step_outputs
            .get("final_result")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        info!(
            "Workflow execution completed in {} steps",
            steps_executed.len()
        );

        Ok(WorkflowResult {
            workflow_id: workflow.config.id.clone(),
            success: true,
            final_output,
            steps_executed,
            agents_used,
            step_outputs,
            error: None,
            duration_seconds: duration,
            metadata: HashMap::new(),
        })
    }

    fn build_step_input(&self, step: &WorkflowStep, context: &AgentContext) -> String {
        if step.input_mapping.is_empty() {
            context
                .get_shared("input")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default()
        } else {
            let mut input_parts = Vec::new();
            for (key, source_key) in &step.input_mapping {
                if let Some(value) = context.get_shared(source_key) {
                    input_parts.push(format!("{}: {}", key, value));
                }
            }
            input_parts.join("\n")
        }
    }

    async fn execute_agent_step(
        &self,
        step: &WorkflowStep,
        input: String,
        context: &mut AgentContext,
        agents: &Arc<RwLock<HashMap<String, Arc<dyn AgentFlowAgent>>>>,
        agents_used: &mut Vec<String>,
    ) -> anyhow::Result<(serde_json::Value, Option<String>)> {
        let agent_id = step
            .agent_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No agent_id for AgentExecution step"))?;

        let agents_lock = agents.read().await;
        let agent = agents_lock
            .get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        if !agents_used.contains(agent_id) {
            agents_used.push(agent_id.clone());
        }

        let message = AgentMessage::new(
            "workflow".to_string(),
            "Workflow".to_string(),
            Some(agent_id.clone()),
            input,
            MessageType::Task,
        );

        let response = agent.process(message, context).await?;
        let output = serde_json::json!({
            "content": response.message.content,
            "agent": agent_id,
        });

        Ok((output, step.next_step_id.clone()))
    }

    async fn execute_parallel_step(
        &self,
        step: &WorkflowStep,
        input: String,
        context: &mut AgentContext,
        agents: &Arc<RwLock<HashMap<String, Arc<dyn AgentFlowAgent>>>>,
        agents_used: &mut Vec<String>,
    ) -> anyhow::Result<(serde_json::Value, Option<String>)> {
        let agents_lock = agents.read().await;
        let mut handles = Vec::new();

        for agent_id in &step.agent_ids {
            if let Some(agent) = agents_lock.get(agent_id) {
                let agent_clone = agent.clone();
                let input_clone = input.clone();
                let agent_id_clone = agent_id.clone();
                let mut context_clone = context.clone();

                let handle = tokio::spawn(async move {
                    let message = AgentMessage::new(
                        "workflow".to_string(),
                        "Workflow".to_string(),
                        Some(agent_id_clone.clone()),
                        input_clone,
                        MessageType::Task,
                    );

                    let response = agent_clone.process(message, &mut context_clone).await?;
                    Ok::<_, anyhow::Error>((agent_id_clone, response.message.content))
                });

                handles.push(handle);
            }
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok((agent_id, content))) => {
                    if !agents_used.contains(&agent_id) {
                        agents_used.push(agent_id.clone());
                    }
                    results.push(serde_json::json!({
                        "agent": agent_id,
                        "content": content,
                    }));
                }
                Ok(Err(e)) => {
                    error!("Agent execution failed: {}", e);
                }
                Err(e) => {
                    error!("Task join failed: {}", e);
                }
            }
        }

        let output = serde_json::json!({ "results": results });
        Ok((output, step.next_step_id.clone()))
    }

    fn execute_conditional_step(
        &self,
        step: &WorkflowStep,
        _context: &AgentContext,
    ) -> anyhow::Result<(serde_json::Value, Option<String>)> {
        let condition_met = step.condition.as_ref().map_or(true, |_| true);

        let next_id = if condition_met {
            step.true_step_id.clone()
        } else {
            step.false_step_id.clone()
        };

        Ok((serde_json::json!({"condition_met": condition_met}), next_id))
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}
