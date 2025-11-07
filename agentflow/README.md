# AgentFlow

轻量级多代理协作和工作流编排框架

## 功能特性

- 🤖 **多代理管理** - 定义和管理多个智能体
- 🔄 **对话编排** - 协调代理之间的交互流程
- 📊 **工作流引擎** - 建模复杂的多步骤流程
- 💬 **消息协议** - 统一的消息格式和上下文管理
- ⚙️ **配置化** - 通过配置文件定义代理与工作流
- 🔌 **纯抽象** - 不绑定任何具体 LLM 实现

## 核心概念

### Agent (代理)
实现 `Agent` trait，用于处理消息并生成响应。

### Orchestrator (编排器)
负责调度多个代理、管理会话与协作流程。

### Workflow (工作流)
定义代理之间的执行顺序与条件，支持分支、循环、并行等模式。

## 快速开始

### 安装

```toml
[dependencies]
agentflow = "0.1"
```

### 实现自定义 Agent

```rust
use agentflow::{Agent, AgentConfig, AgentMessage, AgentResponse, AgentContext, MessageType};
use async_trait::async_trait;

struct MyAgent {
    config: AgentConfig,
}

#[async_trait]
impl Agent for MyAgent {
    fn config(&self) -> &AgentConfig {
        &self.config
    }

    async fn process(
        &self,
        message: AgentMessage,
        _context: &mut AgentContext,
    ) -> anyhow::Result<AgentResponse> {
        let response_msg = AgentMessage::new(
            self.id().to_string(),
            self.name().to_string(),
            message.sender_id.into(),
            format!("处理: {}", message.content),
            MessageType::Result,
        );
        Ok(AgentResponse::new(response_msg))
    }
}
```

### 使用编排器

```rust
use agentflow::{AgentOrchestrator, OrchestrationConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let orchestrator = AgentOrchestrator::new(OrchestrationConfig::default());

    let agent = Arc::new(MyAgent { config: AgentConfig::new(/* ... */) });
    orchestrator.register_agent(agent).await;

    let result = orchestrator.orchestrate(
        "请分析一下这个需求".to_string(),
        None,
    ).await?;

    println!("结果: {}", result.result);
    Ok(())
}
```

### 创建工作流

```rust
use agentflow::{Workflow, WorkflowConfig, WorkflowStep, StepType};

let config = WorkflowConfig {
    id: "workflow_01".to_string(),
    name: "多步骤分析".to_string(),
    ..Default::default()
};

let step1 = WorkflowStep::new_agent_execution(
    "plan".to_string(),
    "规划步骤".to_string(),
    "planner_agent".to_string(),
    "plan_output".to_string(),
);

let workflow = Workflow::new(config, vec![step1], "plan".to_string());
```

## 与其他工具集成

AgentFlow 可以与任意 LLM 调用库配合使用，例如将 `llm-adapter` 的 Adapter 包装为 Agent：

```rust
struct LLMAgent {
    config: AgentConfig,
    adapter: Arc<dyn llm_adapter::Adapter>,
}

#[async_trait]
impl Agent for LLMAgent {
    async fn process(
        &self,
        message: AgentMessage,
        _context: &mut AgentContext,
    ) -> anyhow::Result<AgentResponse> {
        let prompt = format!("{}\n{}",
            self.config().system_prompt,
            message.content
        );
        let result = self.adapter.invoke(&prompt).await?;
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
```

## 测试

```bash
# 运行 AgentFlow 单元 + 集成测试
./scripts/test/run_tests.sh

# 带性能测试（如已创建）
./scripts/test/run_tests.sh --performance
```

## License

MIT
