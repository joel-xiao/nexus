use llm_adapter::providers::MockAdapter;
use std::sync::Arc;

pub fn create_mock_adapter(name: impl Into<String>) -> Arc<MockAdapter> {
    Arc::new(MockAdapter::new(name.into()))
}
