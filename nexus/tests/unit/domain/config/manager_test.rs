use nexus::domain::config::manager::ConfigManager;

/// 测试配置管理器
#[tokio::test]
async fn test_config_manager() {
    let manager = ConfigManager::new();

    let config_json = r#"{
        "version": "1.0.0",
        "adapters": {},
        "prompts": {},
        "feature_flags": {},
        "routing_rules": []
    }"#;

    let result = manager.load_from_json(config_json).await;
    assert!(result.is_ok());

    let config = manager.get_config().await;
    assert_eq!(config.version, "1.0.0");
}
