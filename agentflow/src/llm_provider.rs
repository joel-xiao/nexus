use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LLMInvokeOptions {
    pub user_id: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for LLMInvokeOptions {
    fn default() -> Self {
        Self {
            user_id: None,
            model: None,
            temperature: None,
            max_tokens: None,
            metadata: HashMap::new(),
        }
    }
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn invoke(&self, prompt: &str, options: &LLMInvokeOptions) -> anyhow::Result<String>;
    
    fn name(&self) -> &str;
}

