use crate::domain::adapters::registry::Adapter;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, error};

/// 通用 HTTP Adapter - 支持通过配置动态创建
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
    /// 请求路径模板，支持 {model} 变量
    pub endpoint_template: String,
    /// 请求体模板（JSON），支持变量替换
    pub body_template: Option<Value>,
    /// HTTP 方法（POST, GET 等）
    pub method: String,
    /// 认证方式
    pub auth_type: AuthType,
    /// 认证 header 名称（如 Authorization）
    pub auth_header: Option<String>,
    /// 模型字段名（不同 API 可能不同）
    pub model_field: String,
    /// 消息字段名
    pub message_field: String,
    /// 响应解析路径（JSONPath 风格，简化版）
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
                let header_name = self.request_config.auth_header.as_deref().unwrap_or("Authorization");
                let value = format!("Bearer {}", self.api_key);
                headers.insert(
                    header_name.parse().unwrap_or(reqwest::header::AUTHORIZATION),
                    value.parse().unwrap(),
                );
            },
            AuthType::Header(name) => {
                if let (Ok(header_name), Ok(header_value)) = (name.parse::<reqwest::header::HeaderName>(), self.api_key.parse::<reqwest::header::HeaderValue>()) {
                    headers.insert(header_name, header_value);
                }
            },
            AuthType::Query(_) => {
                // Query 参数在 URL 中处理
            },
            AuthType::None => {},
        }

        headers
    }

    fn build_body(&self, prompt: &str) -> anyhow::Result<Value> {
        if let Some(ref template) = self.request_config.body_template {
            let mut body = template.clone();
            
            // Use proper JSON manipulation instead of string replacement
            // This avoids control character issues
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
            
            // 如果模板中有嵌套结构，尝试设置模型和消息字段
            if let Some(obj) = body.as_object_mut() {
                // 设置模型字段
                if !obj.contains_key(&self.request_config.model_field) {
                    obj.insert(self.request_config.model_field.clone(), Value::String(self.model.clone()));
                }
                
                // 设置消息字段
                if let Some(msg_field) = obj.get_mut(&self.request_config.message_field) {
                    match msg_field {
                        Value::Array(arr) => {
                            // 如果已经是数组，添加消息
                            arr.push(Value::Object({
                                let mut msg = serde_json::Map::new();
                                msg.insert("role".to_string(), Value::String("user".to_string()));
                                msg.insert("content".to_string(), Value::String(prompt.to_string()));
                                msg
                            }));
                        },
                        _ => {
                            // 否则替换
                            *msg_field = Value::String(prompt.to_string());
                        },
                    }
                } else {
                    // 创建默认消息结构
                    let mut messages = serde_json::Map::new();
                    messages.insert("role".to_string(), Value::String("user".to_string()));
                    messages.insert("content".to_string(), Value::String(prompt.to_string()));
                    obj.insert(self.request_config.message_field.clone(), Value::Array(vec![Value::Object(messages)]));
                }
            }
            
            Ok(body)
        } else {
            // 默认请求体
            let mut body = serde_json::Map::new();
            body.insert(self.request_config.model_field.clone(), serde_json::Value::String(self.model.clone()));
            body.insert(self.request_config.message_field.clone(), serde_json::Value::String(prompt.to_string()));
            Ok(serde_json::Value::Object(body))
        }
    }

    fn extract_response(&self, response: Value) -> anyhow::Result<String> {
        // 简单的 JSONPath 解析（简化版）
        let path_parts: Vec<&str> = self.request_config.response_path.split('.').collect();
        let mut current = &response;
        
        for part in path_parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)
                        .ok_or_else(|| anyhow::anyhow!("Path {} not found", part))?;
                },
                Value::Array(arr) => {
                    let idx: usize = part.parse()
                        .map_err(|_| anyhow::anyhow!("Invalid array index: {}", part))?;
                    current = arr.get(idx)
                        .ok_or_else(|| anyhow::anyhow!("Array index {} out of bounds", idx))?;
                },
                _ => {
                    return Err(anyhow::anyhow!("Cannot navigate path at: {}", part));
                },
            }
        }
        
        match current {
            Value::String(s) => Ok(s.clone()),
            _ => {
                // Use serde_json to properly serialize non-string values
                serde_json::to_string(current)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize response value: {}", e))
            }
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
        
        // 处理 Query 参数认证
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

        // Get response text first for better error handling
        let response_text = response.text().await?;
        
        // Try to parse as JSON
        let result: Value = serde_json::from_str(&response_text)
            .map_err(|e| {
                error!("{} JSON parse error: {}", self.name, e);
                error!("Response preview (first 200 chars): {}", 
                    &response_text.chars().take(200).collect::<String>());
                anyhow::anyhow!("Failed to parse response as JSON: {}", e)
            })?;
        
        self.extract_response(result)
    }

    async fn health(&self) -> bool {
        // 简单的健康检查：尝试发送一个最小请求
        // 在实际实现中，可以发送一个轻量级的请求
        true
    }
}


