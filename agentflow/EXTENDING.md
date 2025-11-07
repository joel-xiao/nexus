# 扩展 AgentFlow

AgentFlow 设计为可扩展的框架，支持自定义 LLM 提供者和 Agent 实现。

## 自定义 LLM 提供者

如果你不想使用 `llm-adapter`，或者需要使用其他 LLM 库，可以实现 `LLMProvider` trait：

```rust
use agentflow::{LLMProvider, LLMInvokeOptions};
use async_trait::async_trait;
use std::sync::Arc;

struct MyCustomLLMProvider {
    api_key: String,
    model: String,
}

#[async_trait]
impl LLMProvider for MyCustomLLMProvider {
    async fn invoke(&self, prompt: &str, options: &LLMInvokeOptions) -> anyhow::Result<String> {
        // 实现你的 LLM 调用逻辑
        // 可以使用任何 LLM SDK，如 openai-rs, anthropic-rs 等
        Ok(format!("Response to: {}", prompt))
    }

    fn name(&self) -> &str {
        "my_custom_llm"
    }
}

// 使用自定义提供者创建 Agent
use agentflow::{Agent, AgentConfig, AgentMessage, AgentResponse, AgentContext, MessageType};

struct MyAgent {
    config: AgentConfig,
    llm: Arc<dyn LLMProvider>,
}

#[async_trait]
impl Agent for MyAgent {
    fn config(&self) -> &AgentConfig {
        &self.config
    }

    async fn process(
        &self,
        message: AgentMessage,
        context: &mut AgentContext,
    ) -> anyhow::Result<AgentResponse> {
        let prompt = format!("{}\n{}", self.config.system_prompt, message.content);
        
        let options = LLMInvokeOptions {
            user_id: Some(message.sender_id.clone()),
            model: Some(self.config.metadata.get("model").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default()),
            temperature: self.config.temperature,
            max_tokens: None,
            metadata: self.config.metadata.clone(),
        };

        let result = self.llm.invoke(&prompt, &options).await?;

        let response_msg = AgentMessage::new(
            self.config.id.clone(),
            self.config.name.clone(),
            Some(message.sender_id.clone()),
            result,
            MessageType::Result,
        );

        Ok(AgentResponse::new(response_msg))
    }
}
```

## 与 llm-adapter 集成

如果你使用 `llm-adapter`，可以使用提供的桥接实现：

```rust
use agentflow::{LLMProvider, LLMInvokeOptions};
use llm_adapter::{Adapter, InvokeOptions};
use async_trait::async_trait;
use std::sync::Arc;

struct LLMAdapterBridge {
    adapter: Arc<dyn Adapter + Send + Sync>,
    name: String,
}

#[async_trait]
impl LLMProvider for LLMAdapterBridge {
    async fn invoke(&self, prompt: &str, options: &LLMInvokeOptions) -> anyhow::Result<String> {
        let invoke_options = InvokeOptions {
            user_id: options.user_id.clone(),
            model: options.model.clone(),
            temperature: options.temperature,
            max_tokens: options.max_tokens,
            metadata: options.metadata.clone(),
        };
        self.adapter.invoke_with_options(prompt, &invoke_options).await
    }

    fn name(&self) -> &str {
        &self.name
    }
}
```

## 自定义 Agent

除了使用 LLM，你还可以实现完全自定义的 Agent：

```rust
use agentflow::{Agent, AgentConfig, AgentMessage, AgentResponse, AgentContext, MessageType};
use async_trait::async_trait;

struct RuleBasedAgent {
    config: AgentConfig,
}

#[async_trait]
impl Agent for RuleBasedAgent {
    fn config(&self) -> &AgentConfig {
        &self.config
    }

    async fn process(
        &self,
        message: AgentMessage,
        _context: &mut AgentContext,
    ) -> anyhow::Result<AgentResponse> {
        // 基于规则的逻辑，不需要 LLM
        let response = if message.content.contains("hello") {
            "Hello! How can I help you?"
        } else {
            "I don't understand."
        }.to_string();

        let response_msg = AgentMessage::new(
            self.config.id.clone(),
            self.config.name.clone(),
            Some(message.sender_id.clone()),
            response,
            MessageType::Result,
        );

        Ok(AgentResponse::new(response_msg))
    }
}
```

## 扩展性优势

1. **不绑定特定 LLM 库** - 可以使用任何 LLM SDK
2. **灵活的实现** - 可以实现基于规则、数据库查询、API 调用等任何逻辑
3. **独立打包** - `agentflow` 不依赖任何具体的 LLM 实现，可以独立发布
4. **易于测试** - 可以轻松创建 Mock LLM 提供者进行测试

