use crate::domain::adapters::registry::Adapter;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, error};

#[derive(Clone)]
pub struct DoubaoAdapter {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct DoubaoRequest {
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
struct DoubaoResponse {
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
impl Adapter for DoubaoAdapter {
    fn name(&self) -> &str {
        "doubao"
    }

    async fn describe(&self) -> String {
        format!("豆包 {} 模型适配器", self.model)
    }

    async fn invoke(&self, prompt: &str) -> anyhow::Result<String> {
        info!("Calling Doubao with model: {}", self.model);
        
        let req = DoubaoRequest {
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
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("Doubao API error: {}", text);
            anyhow::bail!("Doubao API error: {}", status);
        }

        let result: DoubaoResponse = response.json().await?;
        Ok(result.choices[0].message.content.clone())
    }

    async fn health(&self) -> bool {
        true
    }
}

impl DoubaoAdapter {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "doubao-pro-4k".to_string()),
            base_url: "https://ark.cn-beijing.volces.com/api/v3".to_string(),
            client: reqwest::Client::new(),
        }
    }
}

