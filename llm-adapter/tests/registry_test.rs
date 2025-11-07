use llm_adapter::providers::MockAdapter;
use llm_adapter::AdapterRegistry;
use std::sync::Arc;

#[tokio::test]
async fn test_registry_creation() {
    let registry = AdapterRegistry::new();
    let adapters = registry.list().await;
    assert_eq!(adapters.len(), 0);
}

#[tokio::test]
async fn test_register_adapter() {
    let registry = AdapterRegistry::new();
    let adapter = Arc::new(MockAdapter::new("test".to_string()));

    registry.register("test", adapter).await;

    let adapters = registry.list().await;
    assert_eq!(adapters.len(), 1);
    assert!(adapters.contains(&"test".to_string()));
}

#[tokio::test]
async fn test_get_adapter() {
    let registry = AdapterRegistry::new();
    let adapter = Arc::new(MockAdapter::new("test".to_string()));

    registry.register("test", adapter.clone()).await;

    let retrieved = registry.get("test").await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "test");
}

#[tokio::test]
async fn test_get_nonexistent_adapter() {
    let registry = AdapterRegistry::new();

    let retrieved = registry.get("nonexistent").await;
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_unregister_adapter() {
    let registry = AdapterRegistry::new();
    let adapter = Arc::new(MockAdapter::new("test".to_string()));

    registry.register("test", adapter).await;
    assert_eq!(registry.list().await.len(), 1);

    let removed = registry.unregister("test").await;
    assert!(removed);
    assert_eq!(registry.list().await.len(), 0);
}

#[tokio::test]
async fn test_unregister_nonexistent_adapter() {
    let registry = AdapterRegistry::new();

    let removed = registry.unregister("nonexistent").await;
    assert!(!removed);
}

#[tokio::test]
async fn test_list_adapters() {
    let registry = AdapterRegistry::new();

    registry
        .register(
            "adapter1",
            Arc::new(MockAdapter::new("adapter1".to_string())),
        )
        .await;
    registry
        .register(
            "adapter2",
            Arc::new(MockAdapter::new("adapter2".to_string())),
        )
        .await;

    let adapters = registry.list().await;
    assert_eq!(adapters.len(), 2);
    assert!(adapters.contains(&"adapter1".to_string()));
    assert!(adapters.contains(&"adapter2".to_string()));
}
