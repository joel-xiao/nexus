//! 测试数据和夹具

/// 创建测试 JSON 负载
pub fn create_test_invoke_payload(input: &str, adapter: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "input": input,
        "adapter": adapter.unwrap_or("mock"),
        "user_id": Some("test_user")
    })
}

/// 创建测试配置 JSON
pub fn create_test_config() -> serde_json::Value {
    serde_json::json!({
        "version": "1.0.0",
        "adapters": {
            "test-adapter": {
                "name": "test-adapter",
                "api_key": "test-key",
                "model": "test-model",
                "base_url": "https://api.test.com",
                "enabled": true,
                "metadata": {
                    "endpoint_template": "/v1/chat",
                    "method": "POST",
                    "auth_type": "bearer",
                    "rate_limit_rps": 10
                }
            }
        },
        "prompts": {},
        "feature_flags": {},
        "routing_rules": []
    })
}
