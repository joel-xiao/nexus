use llm_adapter::config::AdapterConfig;

#[test]
fn test_adapter_config_creation() {
    let config = AdapterConfig::new("test".to_string());

    assert_eq!(config.name, "test");
    assert!(config.api_key.is_none());
    assert!(config.model.is_none());
    assert!(config.base_url.is_none());
    assert!(config.enabled);
}

#[test]
fn test_adapter_config_with_api_key() {
    let mut config = AdapterConfig::new("test".to_string());
    config.api_key = Some("sk-test123".to_string());

    assert_eq!(config.api_key, Some("sk-test123".to_string()));
}

#[test]
fn test_adapter_config_with_model() {
    let mut config = AdapterConfig::new("test".to_string());
    config.model = Some("gpt-4".to_string());

    assert_eq!(config.model, Some("gpt-4".to_string()));
}

#[test]
fn test_adapter_config_serialization() {
    let mut config = AdapterConfig::new("test".to_string());
    config.api_key = Some("sk-test123".to_string());
    config.model = Some("gpt-4".to_string());
    config.enabled = true;

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("sk-test123"));
    assert!(json.contains("gpt-4"));
}

#[test]
fn test_adapter_config_deserialization() {
    let json = r#"{
        "name": "test",
        "api_key": "sk-test123",
        "model": "gpt-4",
        "enabled": true
    }"#;

    let config: AdapterConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.name, "test");
    assert_eq!(config.api_key, Some("sk-test123".to_string()));
    assert_eq!(config.model, Some("gpt-4".to_string()));
    assert!(config.enabled);
}
