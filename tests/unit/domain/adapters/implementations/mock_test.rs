//! Mock 适配器单元测试

use nexus::domain::adapters::implementations::mock::MockAdapter;
use nexus::domain::adapters::registry::Adapter;
use std::sync::Arc;

/// 测试 Mock 适配器
#[tokio::test]
async fn test_mock_adapter() {
    let adapter = Arc::new(MockAdapter::new("test".to_string()));
    
    assert_eq!(adapter.name(), "test");
    
    let result = adapter.invoke("Hello").await;
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
    
    let description = adapter.describe().await;
    assert!(description.contains("Mock") || description.contains("test"));
    
    assert!(adapter.health().await);
}
