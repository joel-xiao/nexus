use crate::agent::AgentConfig;
use crate::workflow::Workflow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFlowConfig {
    #[serde(default = "default_version")]
    pub version: String,

    #[serde(default)]
    pub agents: Vec<AgentConfig>,

    #[serde(default)]
    pub workflows: Vec<Workflow>,

    #[serde(default)]
    pub global_config: GlobalConfig,

    #[serde(default = "default_now")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[serde(default = "default_now")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_max_rounds")]
    pub default_max_rounds: usize,

    #[serde(default = "default_timeout")]
    pub default_timeout_seconds: u64,

    #[serde(default = "default_true")]
    pub enable_logging: bool,

    #[serde(default = "default_true")]
    pub save_history: bool,

    #[serde(default = "default_log_level")]
    pub log_level: String,

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

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            default_max_rounds: default_max_rounds(),
            default_timeout_seconds: default_timeout(),
            enable_logging: true,
            save_history: true,
            log_level: default_log_level(),
            metadata: HashMap::new(),
        }
    }
}

impl Default for AgentFlowConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            agents: Vec::new(),
            workflows: Vec::new(),
            global_config: GlobalConfig::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl AgentFlowConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_agent(&mut self, agent: AgentConfig) {
        self.agents.push(agent);
        self.updated_at = chrono::Utc::now();
    }

    pub fn remove_agent(&mut self, agent_id: &str) -> bool {
        let len_before = self.agents.len();
        self.agents.retain(|a| a.id != agent_id);
        let removed = self.agents.len() < len_before;
        if removed {
            self.updated_at = chrono::Utc::now();
        }
        removed
    }

    pub fn get_agent(&self, agent_id: &str) -> Option<&AgentConfig> {
        self.agents.iter().find(|a| a.id == agent_id)
    }

    pub fn update_agent(&mut self, agent: AgentConfig) -> bool {
        if let Some(existing) = self.agents.iter_mut().find(|a| a.id == agent.id) {
            *existing = agent;
            self.updated_at = chrono::Utc::now();
            true
        } else {
            false
        }
    }

    pub fn add_workflow(&mut self, workflow: Workflow) {
        self.workflows.push(workflow);
        self.updated_at = chrono::Utc::now();
    }

    pub fn remove_workflow(&mut self, workflow_id: &str) -> bool {
        let len_before = self.workflows.len();
        self.workflows.retain(|w| w.config.id != workflow_id);
        let removed = self.workflows.len() < len_before;
        if removed {
            self.updated_at = chrono::Utc::now();
        }
        removed
    }

    pub fn get_workflow(&self, workflow_id: &str) -> Option<&Workflow> {
        self.workflows.iter().find(|w| w.config.id == workflow_id)
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> anyhow::Result<AgentFlowConfig> {
    let content = std::fs::read_to_string(path)?;
    AgentFlowConfig::from_json(&content)
}

pub fn save_config<P: AsRef<Path>>(config: &AgentFlowConfig, path: P) -> anyhow::Result<()> {
    let json = config.to_json()?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load_config_from_env() -> anyhow::Result<AgentFlowConfig> {
    let config_path = std::env::var("AGENTFLOW_CONFIG_PATH")
        .unwrap_or_else(|_| "agentflow_config.json".to_string());

    if Path::new(&config_path).exists() {
        load_config(&config_path)
    } else {
        Ok(AgentFlowConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentRole;

    #[test]
    fn test_config_creation() {
        let config = AgentFlowConfig::new();
        assert_eq!(config.agents.len(), 0);
        assert_eq!(config.workflows.len(), 0);
    }

    #[test]
    fn test_add_agent() {
        let mut config = AgentFlowConfig::new();
        let agent = AgentConfig::new(
            "agent1".to_string(),
            "Test Agent".to_string(),
            AgentRole::Assistant,
            "A test agent".to_string(),
            "You are a helpful assistant".to_string(),
            "mock".to_string(),
        );

        config.add_agent(agent);
        assert_eq!(config.agents.len(), 1);
        assert!(config.get_agent("agent1").is_some());
    }

    #[test]
    fn test_remove_agent() {
        let mut config = AgentFlowConfig::new();
        let agent = AgentConfig::new(
            "agent1".to_string(),
            "Test Agent".to_string(),
            AgentRole::Assistant,
            "A test agent".to_string(),
            "You are a helpful assistant".to_string(),
            "mock".to_string(),
        );

        config.add_agent(agent);
        assert_eq!(config.agents.len(), 1);

        let removed = config.remove_agent("agent1");
        assert!(removed);
        assert_eq!(config.agents.len(), 0);
    }

    #[test]
    fn test_json_serialization() {
        let config = AgentFlowConfig::new();
        let json = config.to_json().unwrap();
        let deserialized = AgentFlowConfig::from_json(&json).unwrap();

        assert_eq!(config.version, deserialized.version);
        assert_eq!(config.agents.len(), deserialized.agents.len());
    }
}
