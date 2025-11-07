use llm_adapter::providers::MockAdapter;
use llm_adapter::Adapter;

#[tokio::test]
async fn test_mock_adapter_name() {
    let adapter = MockAdapter::new("test_adapter".to_string());
    assert_eq!(adapter.name(), "test_adapter");
}

#[tokio::test]
async fn test_mock_adapter_invoke() {
    let adapter = MockAdapter::new("test".to_string());
    let result = adapter.invoke("Hello").await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("Mock response to: Hello"));
}

#[tokio::test]
async fn test_mock_adapter_describe() {
    let adapter = MockAdapter::new("test_adapter".to_string());
    let description = adapter.describe().await;

    assert!(description.contains("Mock adapter"));
    assert!(description.contains("test_adapter"));
}

#[tokio::test]
async fn test_mock_adapter_health() {
    let adapter = MockAdapter::new("test".to_string());
    assert!(adapter.health().await);
}
