//! 调用 API 集成测试

mod common;
use common::wait_for_adapters;

use axum_test::TestServer;
use serde_json::json;
use nexus::create_test_app;

/// 测试调用端点 - 成功场景
#[tokio::test]
async fn test_invoke_endpoint_success() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    // 等待 mock 适配器注册完成
    wait_for_adapters().await;

    let response = server
        .post("/api/invoke")
        .json(&json!({
            "input": "Hello, world!",
            "adapter": "mock"
        }))
        .await;

    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert!(json_response["result"].is_string());
    assert_eq!(json_response["adapter_used"], "mock");
    assert!(json_response["tasks"].is_array());
}

/// 测试调用端点 - 无适配器
#[tokio::test]
async fn test_invoke_endpoint_no_adapter() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/invoke")
        .json(&json!({
            "input": "Hello, world!",
            "adapter": "nonexistent"
        }))
        .await;

    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    // 应该返回错误信息或默认结果
    assert!(json_response["result"].is_string());
}
