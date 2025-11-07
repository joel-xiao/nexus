use nexus::domain::config::feature_flag::{FeatureFlag, FeatureFlagStore, FlagStatus};

/// 测试功能标志存储
#[tokio::test]
async fn test_feature_flags() {
    let store = FeatureFlagStore::new();

    let flag = FeatureFlag::new("test_flag".to_string(), FlagStatus::Enabled);
    store.register(flag).await;

    let enabled = store.is_enabled("test_flag", Some("user1")).await;
    assert!(enabled);
}
