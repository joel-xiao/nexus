//! 功能标志单元测试

use nexus::domain::config::feature_flag::{FeatureFlag, FlagStatus, FeatureFlagStore};

/// 测试功能标志存储
#[tokio::test]
async fn test_feature_flags() {
    let store = FeatureFlagStore::new();
    
    // 创建并注册标志
    let flag = FeatureFlag::new("test_flag".to_string(), FlagStatus::Enabled);
    store.register(flag).await;
    
    // 验证标志状态
    let enabled = store.is_enabled("test_flag", Some("user1")).await;
    assert!(enabled);
}
