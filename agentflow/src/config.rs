/// AgentFlow 配置管理
/// 
/// 提供配置的加载、保存和管理功能

use crate::agent::AgentConfig;
use crate::workflow::Workflow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// AgentFlow 完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFlowConfig {
    /// 配置版本
    #[serde(default = "default_version")]
    pub version: String,
    
    /// Agent 配置列表
    #[serde(default)]
    pub agents: Vec<AgentConfig>,
    
    /// 工作流配置列表
    #[serde(default)]
    pub workflows: Vec<Workflow>,
    
    /// 全局配置
    #[serde(default)]
    pub global_config: GlobalConfig,
    
    /// 创建时间
    #[serde(default = "default_now")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// 更新时间
    #[serde(default = "default_now")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

/// 全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// 默认最大轮数
    #[serde(default = "default_max_rounds")]
    pub default_max_rounds: usize,
    
    /// 默认超时时间（秒）
    #[serde(default = "default_timeout")]
    pub default_timeout_seconds: u64,
    
    /// 是否启用日志
    #[serde(default = "default_true")]
    pub enable_logging: bool,
    
    /// 是否保存历史
    #[serde(default = "default_true")]
    pub save_history: bool,
    
    /// 日志级别
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
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

    /// 添加 Agent 配置
    pub fn add_agent(&mut self, agent: AgentConfig) {
        self.agents.push(agent);
        self.updated_at = chrono::Utc::now();
    }

    /// 移除 Agent 配置
    pub fn remove_agent(&mut self, agent_id: &str) -> bool {
        let len_before = self.agents.len();
        self.agents.retain(|a| a.id != agent_id);
        let removed = self.agents.len() < len_before;
        if removed {
            self.updated_at = chrono::Utc::now();
        }
        removed
    }

    /// 获取 Agent 配置
    pub fn get_agent(&self, agent_id: &str) -> Option<&AgentConfig> {
        self.agents.iter().find(|a| a.id == agent_id)
    }

    /// 更新 Agent 配置
    pub fn update_agent(&mut self, agent: AgentConfig) -> bool {
        if let Some(existing) = self.agents.iter_mut().find(|a| a.id == agent.id) {
            *existing = agent;
            self.updated_at = chrono::Utc::now();
            true
        } else {
            false
        }
    }

    /// 添加工作流
    pub fn add_workflow(&mut self, workflow: Workflow) {
        self.workflows.push(workflow);
        self.updated_at = chrono::Utc::now();
    }

    /// 移除工作流
    pub fn remove_workflow(&mut self, workflow_id: &str) -> bool {
        let len_before = self.workflows.len();
        self.workflows.retain(|w| w.config.id != workflow_id);
        let removed = self.workflows.len() < len_before;
        if removed {
            self.updated_at = chrono::Utc::now();
        }
        removed
    }

    /// 获取工作流
    pub fn get_workflow(&self, workflow_id: &str) -> Option<&Workflow> {
        self.workflows.iter().find(|w| w.config.id == workflow_id)
    }

    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// 从 JSON 字符串解析
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

/// 从文件加载配置
pub fn load_config<P: AsRef<Path>>(path: P) -> anyhow::Result<AgentFlowConfig> {
    let content = std::fs::read_to_string(path)?;
    AgentFlowConfig::from_json(&content)
}

/// 保存配置到文件
pub fn save_config<P: AsRef<Path>>(config: &AgentFlowConfig, path: P) -> anyhow::Result<()> {
    let json = config.to_json()?;
    std::fs::write(path, json)?;
    Ok(())
}

/// 从环境变量加载配置
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

