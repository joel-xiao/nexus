use agentflow::{LLMInvokeOptions, LLMProvider};
use async_trait::async_trait;
use llm_adapter::Adapter;
use std::sync::Arc;

pub struct LLMAdapterProvider {
    adapter: Arc<dyn Adapter + Send + Sync>,
    name: String,
}

impl LLMAdapterProvider {
    pub fn new(name: String, adapter: Arc<dyn Adapter + Send + Sync>) -> Self {
        Self { adapter, name }
    }
}

#[async_trait]
impl LLMProvider for LLMAdapterProvider {
    async fn invoke(&self, prompt: &str, options: &LLMInvokeOptions) -> anyhow::Result<String> {
        use llm_adapter::InvokeOptions;
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

