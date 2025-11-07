use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub name: String,

    #[serde(default)]
    pub api_key: Option<String>,

    #[serde(default)]
    pub model: Option<String>,

    #[serde(default)]
    pub base_url: Option<String>,

    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}

impl AdapterConfig {
    pub fn new(name: String) -> Self {
        Self {
            name,
            api_key: None,
            model: None,
            base_url: None,
            enabled: true,
            metadata: HashMap::new(),
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}
