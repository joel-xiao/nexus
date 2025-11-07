use crate::registry::Adapter;
use async_trait::async_trait;
use tracing::info;

pub struct MockAdapter {
    name: String,
}

#[async_trait]
impl Adapter for MockAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    async fn describe(&self) -> String {
        format!("Mock adapter for testing: {}", self.name)
    }

    async fn invoke(&self, prompt: &str) -> anyhow::Result<String> {
        info!("Mock adapter processing: {}", prompt);
        Ok(format!("Mock response to: {}", prompt))
    }

    async fn health(&self) -> bool {
        true
    }
}

impl MockAdapter {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
