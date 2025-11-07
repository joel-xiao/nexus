use crate::registry::Adapter;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Clone)]
pub struct QianwenAdapter {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct QianwenRequest {
    model: String,
    input: QianwenInput,
    parameters: QianwenParameters,
}

#[derive(Serialize)]
struct QianwenInput {
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct QianwenParameters {
    temperature: f32,
    top_p: f32,
}

#[derive(Deserialize)]
struct QianwenResponse {
    output: QianwenOutput,
}

#[derive(Deserialize)]
struct QianwenOutput {
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
impl Adapter for QianwenAdapter {
    fn name(&self) -> &str {
        "qianwen"
    }

    async fn describe(&self) -> String {
        format!("通义千问 {} 模型适配器", self.model)
    }

    async fn invoke(&self, prompt: &str) -> anyhow::Result<String> {
        info!("Calling Qianwen with model: {}", self.model);

        let req = QianwenRequest {
            model: self.model.clone(),
            input: QianwenInput {
                messages: vec![Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }],
            },
            parameters: QianwenParameters {
                temperature: 0.7,
                top_p: 0.9,
            },
        };

        let response = self
            .client
            .post(format!(
                "{}/v1/services/aigc/text-generation/generation",
                self.base_url
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("Qianwen API error: {}", text);
            anyhow::bail!("Qianwen API error: {}", status);
        }

        let result: QianwenResponse = response.json().await?;
        Ok(result.output.choices[0].message.content.clone())
    }

    async fn health(&self) -> bool {
        true
    }
}

impl QianwenAdapter {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "qwen-turbo".to_string()),
            base_url: "https://dashscope.aliyuncs.com/api".to_string(),
            client: reqwest::Client::new(),
        }
    }
}
