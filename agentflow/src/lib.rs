/// AgentFlow - 轻量级多代理协作和工作流编排框架
/// 
/// 这是一个独立的多代理协作框架，不涉及任何具体的 LLM 调用实现
/// Agent trait 是纯抽象，需要用户实现具体的 Agent

pub mod agent;
pub mod orchestrator;
pub mod workflow;
pub mod config;

// 重新导出核心类型
pub use agent::{
    AgentFlowAgent as Agent,
    AgentConfig,
    AgentRole,
    AgentCapability,
    AgentMessage,
    AgentResponse,
    AgentContext,
    MessageType,
};

pub use orchestrator::{
    AgentOrchestrator,
    OrchestrationConfig,
    OrchestrationResult,
};

pub use workflow::{
    Workflow,
    WorkflowConfig,
    WorkflowStep,
    WorkflowEngine,
    WorkflowResult,
    StepType,
};

pub use config::{
    AgentFlowConfig,
    GlobalConfig,
    load_config,
    save_config,
};
