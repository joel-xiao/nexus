use crate::common::create_test_server;
use serde_json::json;

#[tokio::test]
async fn test_create_routing_rule() {
    let server = create_test_server();

    let response = server
        .post("/api/config/routing/rules")
        .json(&json!({
            "name": "test_rule",
            "strategy": "weighted",
            "priority": 10,
            "models": [
                {
                    "model_name": "model1",
                    "adapter_name": "mock",
                    "weight": 100,
                    "enabled": true
                }
            ]
        }))
        .await;

    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["status"], "ok");
}

#[tokio::test]
async fn test_list_routing_rules() {
    let server = create_test_server();

    let response = server.get("/api/config/routing/rules").await;

    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert!(json_response.is_array());
}
