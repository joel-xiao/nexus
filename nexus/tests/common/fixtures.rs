use crate::common::helpers::get_test_mode;

pub fn create_test_invoke_payload(input: &str, adapter: Option<&str>) -> serde_json::Value {
    let adapter_name: String =
        adapter
            .map(|s| s.to_string())
            .unwrap_or_else(|| match get_test_mode() {
                crate::common::helpers::TestMode::Mock => "mock".to_string(),
                crate::common::helpers::TestMode::Real => {
                    std::env::var("NEXUS_TEST_ADAPTER").unwrap_or_else(|_| "mock".to_string())
                }
            });

    serde_json::json!({
        "input": input,
        "adapter": adapter_name,
        "user_id": Some("test_user")
    })
}

pub fn create_real_adapter_config() -> serde_json::Value {
    serde_json::json!({
        "name": std::env::var("NEXUS_TEST_ADAPTER_NAME").unwrap_or_else(|_| "test-adapter".to_string()),
        "api_key": std::env::var("NEXUS_TEST_API_KEY").unwrap_or_else(|_| "test-key".to_string()),
        "model": std::env::var("NEXUS_TEST_MODEL").unwrap_or_else(|_| "test-model".to_string()),
        "base_url": std::env::var("NEXUS_TEST_BASE_URL").unwrap_or_else(|_| "https://api.test.com".to_string()),
        "enabled": true,
        "metadata": {
            "endpoint_template": "/v1/chat/completions",
            "method": "POST",
            "auth_type": "bearer",
            "rate_limit_rps": 10
        }
    })
}

pub fn create_test_config() -> serde_json::Value {
    let mut adapters = serde_json::json!({});

    match get_test_mode() {
        crate::common::helpers::TestMode::Mock => {
            adapters["mock"] = serde_json::json!({
                "name": "mock",
                "api_key": "mock-key",
                "model": "mock-model",
                "base_url": "http://mock.test",
                "enabled": true,
                "metadata": {}
            });
        }
        crate::common::helpers::TestMode::Real => {
            let real_config = create_real_adapter_config();
            let adapter_name = real_config["name"]
                .as_str()
                .unwrap_or("test-adapter")
                .to_string();
            adapters[adapter_name] = real_config;
        }
    }

    serde_json::json!({
        "version": "1.0.0",
        "adapters": adapters,
        "prompts": {},
        "feature_flags": {},
        "routing_rules": []
    })
}

pub fn create_qwen_adapter_config() -> serde_json::Value {
    serde_json::json!({
        "name": "sk-ede615cd70004d8ab45f8a73b50c42ee",
        "api_key": "sk-ede615cd70004d8ab45f8a73b50c42ee",
        "model": "qwen-turbo",
        "base_url": "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
        "enabled": true,
        "metadata": {
            "endpoint_template": "/compatible-mode/v1/chat/completions",
            "method": "POST",
            "auth_type": "bearer",
            "rate_limit_rps": 10
        }
    })
}
