use crate::registry::Adapter;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, error};

#[derive(Clone)]
pub struct DeepSeekAdapter {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct DeepSeekRequest {
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
struct DeepSeekResponse {
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
impl Adapter for DeepSeekAdapter {
    fn name(&self) -> &str {
        "deepseek"
    }

    async fn describe(&self) -> String {
        format!("DeepSeek {} 模型适配器", self.model)
    }

    async fn invoke(&self, prompt: &str) -> anyhow::Result<String> {
        info!("Calling DeepSeek with model: {}", self.model);
        
        let req = DeepSeekRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
            temperature: 0.7,
        };

        let response = self.client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("DeepSeek API error: {}", text);
            anyhow::bail!("DeepSeek API error: {}", status);
        }

        let result: DeepSeekResponse = response.json().await?;
        Ok(result.choices[0].message.content.clone())
    }

    async fn health(&self) -> bool {
        true
    }
}

impl DeepSeekAdapter {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "deepseek-chat".to_string()),
            base_url: "https://api.deepseek.com".to_string(),
            client: reqwest::Client::new(),
        }
    }
}

