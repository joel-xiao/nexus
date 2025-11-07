use crate::common::fixtures::create_test_invoke_payload;
use crate::common::{create_test_server, get_test_mode, wait_for_adapters, TestMode};
use serde_json::json;

#[tokio::test]
async fn test_invoke_endpoint_success() {
    let server = create_test_server();
    wait_for_adapters().await;

    let (adapter_name, adapter_config) = match get_test_mode() {
        TestMode::Mock => ("mock".to_string(), None),
        TestMode::Real => {
            let name = std::env::var("NEXUS_TEST_ADAPTER_NAME")
                .unwrap_or_else(|_| "test-adapter".to_string());
            let config = Some(serde_json::json!({
                "name": name.clone(),
                "api_key": std::env::var("NEXUS_TEST_API_KEY").unwrap_or_else(|_| "test-key".to_string()),
                "model": std::env::var("NEXUS_TEST_MODEL").unwrap_or_else(|_| "test-model".to_string()),
                "base_url": std::env::var("NEXUS_TEST_BASE_URL").unwrap_or_else(|_| "https://api.test.com".to_string()),
                "enabled": true
            }));
            (name, config)
        }
    };

    if let Some(config) = adapter_config {
        let register_response = server
            .put(&format!("/api/config/reload/adapter"))
            .json(&config)
            .await;
        register_response.assert_status_ok();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    let payload = create_test_invoke_payload("Hello, world!", Some(&adapter_name));
    let response = server.post("/api/invoke").json(&payload).await;

    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["status"], "ok");
    assert!(json_response["data"]["result"].is_string());
    assert_eq!(json_response["data"]["adapter_used"], adapter_name.as_str());
    assert!(json_response["data"]["tasks"].is_array());
}

#[tokio::test]
async fn test_invoke_endpoint_no_adapter() {
    let server = create_test_server();

    let response = server
        .post("/api/invoke")
        .json(&json!({
            "input": "Hello, world!",
            "adapter": "nonexistent"
        }))
        .await;

    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["status"], "ok");
    assert!(json_response["data"]["result"].is_string());
}
