use crate::integration::llm_agent::LLMAgent;
use crate::routes::common::{error_response, ok_response};
use crate::state::AppState;
use agentflow::{Agent as AgentFlowAgent, AgentConfig, AgentOrchestrator, AgentRole, OrchestrationConfig, SpeakerSelection};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ConversationRequest {
    pub message: String,
    pub agent_ids: Option<Vec<String>>,
    pub max_rounds: Option<usize>,
    pub user_id: Option<String>,
    #[serde(default)]
    pub speaker_selection: Option<String>,
    #[serde(default)]
    pub agent_order: Option<Vec<String>>,
    #[serde(default)]
    pub termination_condition: Option<String>,
    #[serde(default)]
    pub adapter: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub agent_configs: Option<Vec<AgentRoleConfig>>,
}

#[derive(Deserialize, ToSchema)]
pub struct AgentRoleConfig {
    pub agent_id: String,
    pub role: String,
    pub system_prompt: Option<String>,
    pub name: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct OrchestrateRequest {
    pub initial_message: String,
    pub initial_agent_id: Option<String>,
    pub agent_configs: Vec<AgentConfigRequest>,
    pub max_rounds: Option<usize>,
    #[serde(default)]
    pub speaker_selection: Option<String>,
    #[serde(default)]
    pub agent_order: Option<Vec<String>>,
    #[serde(default)]
    pub termination_condition: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct AgentConfigRequest {
    pub id: String,
    pub name: String,
    pub role: String,
    pub system_prompt: String,
    pub adapter_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct ConversationResponse {
    pub result: String,
    pub rounds: usize,
    pub agents_used: Vec<String>,
    pub success: bool,
    pub duration_seconds: f64,
}

pub async fn start_conversation(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<ConversationRequest>,
) -> Json<serde_json::Value> {
    let max_rounds = payload.max_rounds.unwrap_or(10);
    let speaker_selection = match payload.speaker_selection.as_deref() {
        Some("round_robin") => SpeakerSelection::RoundRobin,
        Some("random") => SpeakerSelection::Random,
        Some("manual") => SpeakerSelection::Manual,
        Some("auto") => SpeakerSelection::Auto,
        _ => SpeakerSelection::RoundRobin,
    };
    let config = OrchestrationConfig {
        session_id: uuid::Uuid::new_v4().to_string(),
        max_rounds,
        timeout_seconds: 300,
        auto_planning: false,
        save_history: true,
        speaker_selection,
        agent_order: payload.agent_order.unwrap_or_default(),
        termination_condition: payload.termination_condition.or(Some("TERMINATE".to_string())),
        metadata: std::collections::HashMap::new(),
    };

    let orchestrator = AgentOrchestrator::new(config);

    let config = state.config_manager.get_config().await;
    
    let mut role_configs = std::collections::HashMap::new();
    if let Some(ref configs) = payload.agent_configs {
        for role_cfg in configs {
            role_configs.insert(role_cfg.agent_id.clone(), role_cfg);
        }
    }

    for (adapter_name, adapter_config) in &config.adapters {
        if !adapter_config.enabled {
            continue;
        }

        if let Some(agent_ids) = &payload.agent_ids {
            if !agent_ids.contains(adapter_name) {
                continue;
            }
        }

        let adapter = match state.adapter_registry.read().await.get(adapter_name).await {
            Some(adapter) => adapter,
            None => continue,
        };

        let role_config = role_configs.get(adapter_name);
        let (role, name, system_prompt) = if let Some(cfg) = role_config {
            (
                parse_agent_role(&cfg.role),
                cfg.name.clone().unwrap_or_else(|| format!("Agent-{}", adapter_name)),
                cfg.system_prompt.clone().unwrap_or_else(|| get_default_system_prompt(&parse_agent_role(&cfg.role)))
            )
        } else {
            (
                AgentRole::Assistant,
                format!("Agent-{}", adapter_name),
                "You are a helpful assistant.".to_string()
            )
        };

        let mut agent_config = AgentConfig::new(
            adapter_name.clone(),
            name,
            role,
            adapter_name.clone(),
            system_prompt,
            adapter_name.clone(),
        );

        if let Some(model) = &payload.model {
            agent_config.metadata.insert("model".to_string(), serde_json::json!(model));
        }
        if let Some(temp) = payload.temperature {
            agent_config.temperature = Some(temp);
        }

        let llm_provider: Arc<dyn agentflow::LLMProvider> = Arc::new(
            crate::integration::LLMAdapterProvider::new(adapter_name.clone(), adapter)
        );
        let llm_agent: Arc<dyn AgentFlowAgent> = Arc::new(LLMAgent::new(agent_config, llm_provider));
        orchestrator.register_agent(llm_agent).await;
    }

    match orchestrator
        .orchestrate(payload.message, None)
        .await
    {
        Ok(result) => ok_response(serde_json::json!({
            "result": result.result,
            "rounds": result.rounds,
            "agents_used": result.agents_used,
            "success": result.success,
            "duration_seconds": result.duration_seconds
        })),
        Err(e) => error_response(&format!("Orchestration failed: {}", e)),
    }
}

pub async fn orchestrate_agents(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<OrchestrateRequest>,
) -> Json<serde_json::Value> {
    let max_rounds = payload.max_rounds.unwrap_or(20);
    let speaker_selection = match payload.speaker_selection.as_deref() {
        Some("round_robin") => SpeakerSelection::RoundRobin,
        Some("random") => SpeakerSelection::Random,
        Some("manual") => SpeakerSelection::Manual,
        Some("auto") => SpeakerSelection::Auto,
        _ => SpeakerSelection::RoundRobin,
    };
    let agent_order = payload.agent_order.unwrap_or_else(|| {
        payload.agent_configs.iter().map(|a| a.id.clone()).collect()
    });
    let config = OrchestrationConfig {
        session_id: uuid::Uuid::new_v4().to_string(),
        max_rounds,
        timeout_seconds: 300,
        auto_planning: false,
        save_history: true,
        speaker_selection,
        agent_order,
        termination_condition: payload.termination_condition.or(Some("TERMINATE".to_string())),
        metadata: std::collections::HashMap::new(),
    };

    let orchestrator = AgentOrchestrator::new(config);

    for agent_req in &payload.agent_configs {
        let adapter = match state
            .adapter_registry
            .read()
            .await
            .get(&agent_req.adapter_name)
            .await
        {
            Some(adapter) => adapter,
            None => {
                return error_response(&format!(
                    "Adapter {} not found",
                    agent_req.adapter_name
                ));
            }
        };

        let role = parse_agent_role(&agent_req.role);

        let agent_config = AgentConfig::new(
            agent_req.id.clone(),
            agent_req.name.clone(),
            role,
            agent_req.id.clone(),
            agent_req.system_prompt.clone(),
            agent_req.adapter_name.clone(),
        );

        let llm_provider: Arc<dyn agentflow::LLMProvider> = Arc::new(
            crate::integration::LLMAdapterProvider::new(agent_req.adapter_name.clone(), adapter)
        );
        let llm_agent: Arc<dyn AgentFlowAgent> = Arc::new(LLMAgent::new(agent_config, llm_provider));
        orchestrator.register_agent(llm_agent).await;
    }

    match orchestrator
        .orchestrate(payload.initial_message, payload.initial_agent_id)
        .await
    {
        Ok(result) => ok_response(serde_json::json!({
            "result": result.result,
            "rounds": result.rounds,
            "agents_used": result.agents_used,
            "success": result.success,
            "duration_seconds": result.duration_seconds,
            "message_history": result.message_history
        })),
        Err(e) => error_response(&format!("Orchestration failed: {}", e)),
    }
}

pub async fn list_agents(Extension(state): Extension<Arc<AppState>>) -> Json<serde_json::Value> {
    let config = state.config_manager.get_config().await;
    let mut agents = Vec::new();

    for (adapter_name, adapter_config) in &config.adapters {
        if adapter_config.enabled {
            agents.push(serde_json::json!({
                "id": adapter_name,
                "name": format!("Agent-{}", adapter_name),
                "adapter": adapter_name,
                "enabled": true
            }));
        }
    }

    ok_response(serde_json::json!({ "agents": agents }))
}

fn parse_agent_role(role_str: &str) -> AgentRole {
    match role_str.to_lowercase().as_str() {
        "user" => AgentRole::User,
        "assistant" => AgentRole::Assistant,
        "planner" => AgentRole::Planner,
        "executor" => AgentRole::Executor,
        "reviewer" => AgentRole::Reviewer,
        "coordinator" => AgentRole::Coordinator,
        role if role.starts_with("expert:") => {
            let domain = role.strip_prefix("expert:").unwrap_or("general").to_string();
            AgentRole::Expert { domain }
        },
        role if role.starts_with("custom:") => {
            let role_name = role.strip_prefix("custom:").unwrap_or(role).to_string();
            AgentRole::Custom { role_name }
        },
        _ => AgentRole::Assistant,
    }
}

fn get_default_system_prompt(role: &AgentRole) -> String {
    match role {
        AgentRole::User => "You are a user participating in a multi-agent conversation.".to_string(),
        AgentRole::Assistant => "You are a helpful assistant.".to_string(),
        AgentRole::Planner => "You are a planning agent. Your role is to break down complex tasks into smaller, manageable steps and create execution plans.".to_string(),
        AgentRole::Executor => "You are an execution agent. Your role is to execute tasks and provide results based on the plans provided.".to_string(),
        AgentRole::Reviewer => "You are a review agent. Your role is to review and evaluate the results from other agents, providing feedback and suggestions.".to_string(),
        AgentRole::Coordinator => "You are a coordinator agent. Your role is to coordinate between different agents and manage the overall workflow.".to_string(),
        AgentRole::Expert { domain } => format!("You are an expert in {}. Provide specialized knowledge and insights in your domain.", domain),
        AgentRole::Custom { role_name } => format!("You are a {} agent. Fulfill your role effectively.", role_name),
    }
}

