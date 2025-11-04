//! 功能标志配置 API 集成测试

use axum_test::TestServer;
use serde_json::json;
use nexus::create_test_app;

/// 测试配置端点 - 创建功能标志
#[tokio::test]
async fn test_create_feature_flag() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

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

/// 测试配置端点 - 检查功能标志
#[tokio::test]
async fn test_check_feature_flag() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    // 先创建标志
    server
        .post("/api/config/flags")
        .json(&json!({
            "name": "check_flag",
            "status": "enabled"
        }))
        .await;

    // 然后检查
    let response = server.get("/api/config/flags/check_flag").await;
    
    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["name"], "check_flag");
}

/// 测试配置端点 - 列出功能标志
#[tokio::test]
async fn test_list_feature_flags() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    // 创建几个标志
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
    // list_flags 返回的是数组，不是对象
    assert!(json_response.is_array());
}
