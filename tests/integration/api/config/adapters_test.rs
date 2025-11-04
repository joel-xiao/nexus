//! 适配器配置 API 集成测试

mod common;
use common::wait_for_adapters;

use axum_test::TestServer;
use nexus::create_test_app;

/// 测试配置端点 - 列出适配器
#[tokio::test]
async fn test_list_adapters() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    // 等待 mock 适配器注册完成
    wait_for_adapters().await;

    let response = server.get("/api/config/adapters").await;
    
    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["status"], "ok");
    assert!(json_response["adapters"].is_array());
}
