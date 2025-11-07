use crate::registry::Adapter;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Clone)]
pub struct ZhipuAdapter {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct ZhipuRequest {
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
struct ZhipuResponse {
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
impl Adapter for ZhipuAdapter {
    fn name(&self) -> &str {
        "zhipu"
    }

    async fn describe(&self) -> String {
        format!("智谱AI {} 模型适配器", self.model)
    }

    async fn invoke(&self, prompt: &str) -> anyhow::Result<String> {
        info!("Calling Zhipu with model: {}", self.model);

        let req = ZhipuRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.7,
        };

        let response = self
            .client
            .post(format!("{}/v4/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("Zhipu API error: {}", text);
            anyhow::bail!("Zhipu API error: {}", status);
        }

        let result: ZhipuResponse = response.json().await?;
        Ok(result.choices[0].message.content.clone())
    }

    async fn health(&self) -> bool {
        true
    }
}

impl ZhipuAdapter {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "glm-4".to_string()),
            base_url: "https://open.bigmodel.cn/api/paas".to_string(),
            client: reqwest::Client::new(),
        }
    }
}
