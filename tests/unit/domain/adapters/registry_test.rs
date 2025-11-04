//! 适配器注册表单元测试

use nexus::domain::adapters::registry::AdapterRegistry;
use nexus::domain::adapters::implementations::mock::MockAdapter;
use nexus::domain::adapters::registry::Adapter;
use std::sync::Arc;

/// 测试适配器注册表
#[tokio::test]
async fn test_adapter_registry() {
    let registry = AdapterRegistry::new();
    
    // 注册一个适配器
    let adapter = Arc::new(MockAdapter::new("test_adapter".to_string()));
    registry.register("test_adapter", adapter.clone()).await;
    
    // 验证可以获取
    let retrieved = registry.get("test_adapter").await;
    assert!(retrieved.is_some());
    
    // 验证列表包含该适配器
    let list = registry.list().await;
    assert!(list.iter().any(|name| name == "test_adapter"));
}
