/// 集成层 - 桥接 llm-adapter 和 agentflow
/// 
/// 这一层负责将独立的工具集成到 nexus 业务中

pub mod llm_agent;

pub use llm_agent::LLMAgent;

