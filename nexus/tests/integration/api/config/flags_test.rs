use crate::common::create_test_server;
use serde_json::json;

#[tokio::test]
async fn test_create_feature_flag() {
    let server = create_test_server();

    let response = server
        .post("/api/config/flags")
        .json(&json!({
            "name": "test_flag",
            "status": "enabled"
        }))
        .await;

    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["name"], "test_flag");
    assert!(json_response["enabled"].as_bool().unwrap());
}

#[tokio::test]
async fn test_check_feature_flag() {
    let server = create_test_server();

    server
        .post("/api/config/flags")
        .json(&json!({
            "name": "check_flag",
            "status": "enabled"
        }))
        .await;

    let response = server.get("/api/config/flags/check_flag/check").await;

    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["name"], "check_flag");
    assert!(json_response["enabled"].is_boolean());
}

#[tokio::test]
async fn test_list_feature_flags() {
    let server = create_test_server();

    server
        .post("/api/config/flags")
        .json(&json!({
            "name": "flag1",
            "status": "enabled"
        }))
        .await;

    server
        .post("/api/config/flags")
        .json(&json!({
            "name": "flag2",
            "status": "disabled"
        }))
        .await;

    let response = server.get("/api/config/flags").await;

    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert!(json_response.is_array());
}
