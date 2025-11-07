use crate::registry::Adapter;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info};

#[derive(Clone)]
pub struct GenericAdapter {
    name: String,
    api_key: String,
    model: String,
    base_url: String,
    endpoint: String,
    client: reqwest::Client,
    request_config: RequestConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestConfig {
    pub endpoint_template: String,
    pub body_template: Option<Value>,
    pub method: String,
    pub auth_type: AuthType,
    pub auth_header: Option<String>,
    pub model_field: String,
    pub message_field: String,
    pub response_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthType {
    Bearer,
    Header(String), // 自定义 header
    Query(String),  // Query 参数
    None,
}

impl GenericAdapter {
    pub fn new(
        name: String,
        api_key: String,
        model: String,
        base_url: String,
        request_config: RequestConfig,
    ) -> Self {
        Self {
            name,
            api_key,
            model,
            base_url,
            endpoint: request_config.endpoint_template.clone(),
            client: reqwest::Client::new(),
            request_config,
        }
    }

    fn build_url(&self) -> String {
        let endpoint = self.endpoint.replace("{model}", &self.model);
        format!("{}{}", self.base_url, endpoint)
    }

    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        match &self.request_config.auth_type {
            AuthType::Bearer => {
                let header_name = self
                    .request_config
                    .auth_header
                    .as_deref()
                    .unwrap_or("Authorization");
                let value = format!("Bearer {}", self.api_key);
                headers.insert(
                    header_name
                        .parse()
                        .unwrap_or(reqwest::header::AUTHORIZATION),
                    value.parse().unwrap(),
                );
            }
            AuthType::Header(name) => {
                if let (Ok(header_name), Ok(header_value)) = (
                    name.parse::<reqwest::header::HeaderName>(),
                    self.api_key.parse::<reqwest::header::HeaderValue>(),
                ) {
                    headers.insert(header_name, header_value);
                }
            }
            AuthType::Query(_) => {}
            AuthType::None => {}
        }

        headers
    }

    fn build_body(&self, prompt: &str) -> anyhow::Result<Value> {
        if let Some(ref template) = self.request_config.body_template {
            let mut body = template.clone();

            fn replace_in_value(value: &mut Value, model: &str, prompt: &str) {
                match value {
                    Value::String(s) => {
                        if s == "{model}" {
                            *s = model.to_string();
                        } else if s == "{prompt}" || s == "{message}" {
                            *s = prompt.to_string();
                        }
                    }
                    Value::Array(arr) => {
                        for item in arr {
                            replace_in_value(item, model, prompt);
                        }
                    }
                    Value::Object(obj) => {
                        for (_key, val) in obj {
                            replace_in_value(val, model, prompt);
                        }
                    }
                    _ => {}
                }
            }

            replace_in_value(&mut body, &self.model, prompt);

            if let Some(obj) = body.as_object_mut() {
                if !obj.contains_key(&self.request_config.model_field) {
                    obj.insert(
                        self.request_config.model_field.clone(),
                        Value::String(self.model.clone()),
                    );
                }

                if let Some(msg_field) = obj.get_mut(&self.request_config.message_field) {
                    match msg_field {
                        Value::Array(arr) => {
                            arr.push(Value::Object({
                                let mut msg = serde_json::Map::new();
                                msg.insert("role".to_string(), Value::String("user".to_string()));
                                msg.insert(
                                    "content".to_string(),
                                    Value::String(prompt.to_string()),
                                );
                                msg
                            }));
                        }
                        _ => {
                            *msg_field = Value::String(prompt.to_string());
                        }
                    }
                } else {
                    let mut messages = serde_json::Map::new();
                    messages.insert("role".to_string(), Value::String("user".to_string()));
                    messages.insert("content".to_string(), Value::String(prompt.to_string()));
                    obj.insert(
                        self.request_config.message_field.clone(),
                        Value::Array(vec![Value::Object(messages)]),
                    );
                }
            }

            Ok(body)
        } else {
            let mut body = serde_json::Map::new();
            body.insert(
                self.request_config.model_field.clone(),
                serde_json::Value::String(self.model.clone()),
            );
            body.insert(
                self.request_config.message_field.clone(),
                serde_json::Value::String(prompt.to_string()),
            );
            Ok(serde_json::Value::Object(body))
        }
    }

    fn extract_response(&self, response: Value) -> anyhow::Result<String> {
        let path_parts: Vec<&str> = self.request_config.response_path.split('.').collect();
        let mut current = &response;

        for part in path_parts {
            match current {
                Value::Object(map) => {
                    current = map
                        .get(part)
                        .ok_or_else(|| anyhow::anyhow!("Path {} not found", part))?;
                }
                Value::Array(arr) => {
                    let idx: usize = part
                        .parse()
                        .map_err(|_| anyhow::anyhow!("Invalid array index: {}", part))?;
                    current = arr
                        .get(idx)
                        .ok_or_else(|| anyhow::anyhow!("Array index {} out of bounds", idx))?;
                }
                _ => {
                    return Err(anyhow::anyhow!("Cannot navigate path at: {}", part));
                }
            }
        }

        match current {
            Value::String(s) => Ok(s.clone()),
            _ => serde_json::to_string(current)
                .map_err(|e| anyhow::anyhow!("Failed to serialize response value: {}", e)),
        }
    }
}

#[async_trait]
impl Adapter for GenericAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    async fn describe(&self) -> String {
        format!("Generic {} adapter for model {}", self.name, self.model)
    }

    async fn invoke(&self, prompt: &str) -> anyhow::Result<String> {
        info!("Calling {} with model: {}", self.name, self.model);

        let url = self.build_url();
        let headers = self.build_headers();
        let body = self.build_body(prompt)?;

        let mut request = match self.request_config.method.as_str() {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            _ => anyhow::bail!("Unsupported HTTP method: {}", self.request_config.method),
        };

        request = request.headers(headers);

        if let AuthType::Query(param_name) = &self.request_config.auth_type {
            request = request.query(&[(param_name, &self.api_key)]);
        }

        if self.request_config.method != "GET" {
            request = request.json(&body);
        }

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("{} API error ({}): {}", self.name, status, text);
            anyhow::bail!("{} API error: {}", self.name, status);
        }

        let response_text = response.text().await?;

        let result: Value = serde_json::from_str(&response_text).map_err(|e| {
            error!("{} JSON parse error: {}", self.name, e);
            error!(
                "Response preview (first 200 chars): {}",
                &response_text.chars().take(200).collect::<String>()
            );
            anyhow::anyhow!("Failed to parse response as JSON: {}", e)
        })?;

        self.extract_response(result)
    }

    async fn health(&self) -> bool {
        true
    }
}
