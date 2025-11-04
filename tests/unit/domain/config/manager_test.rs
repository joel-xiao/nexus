//! 配置管理器单元测试

use nexus::domain::config::manager::ConfigManager;

/// 测试配置管理器
#[tokio::test]
async fn test_config_manager() {
    let manager = ConfigManager::new();
    
    // 测试加载 JSON 配置
    let config_json = r#"{
        "version": "1.0.0",
        "adapters": {},
        "prompts": {},
        "feature_flags": {},
        "routing_rules": []
    }"#;
    
    let result = manager.load_from_json(config_json).await;
    assert!(result.is_ok());
    
    // 验证配置已更新
    let config = manager.get_config().await;
    assert_eq!(config.version, "1.0.0");
}
