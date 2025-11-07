use crate::registry::Adapter;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Clone)]
pub struct OpenAIAdapter {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}

#[async_trait]
impl Adapter for OpenAIAdapter {
    fn name(&self) -> &str {
        "openai"
    }

    async fn describe(&self) -> String {
        format!("OpenAI {} 模型适配器", self.model)
    }

    async fn invoke(&self, prompt: &str) -> anyhow::Result<String> {
        info!("Calling OpenAI with model: {}", self.model);

        let req = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.7,
        };

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("OpenAI API error: {}", text);
            anyhow::bail!("OpenAI API error: {}", status);
        }

        let result: OpenAIResponse = response.json().await?;
        Ok(result.choices[0].message.content.clone())
    }

    async fn health(&self) -> bool {
        true // TODO: 实现健康检查
    }
}

impl OpenAIAdapter {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "gpt-4o-mini".to_string()),
            base_url: "https://api.openai.com".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn new_with_base(api_key: String, model: String, base_url: String) -> Self {
        Self {
            api_key,
            model,
            base_url,
            client: reqwest::Client::new(),
        }
    }
}
