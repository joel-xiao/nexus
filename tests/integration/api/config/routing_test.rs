//! 路由规则配置 API 集成测试

use axum_test::TestServer;
use serde_json::json;
use nexus::create_test_app;

/// 测试配置端点 - 创建路由规则
#[tokio::test]
async fn test_create_routing_rule() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

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

/// 测试配置端点 - 列出路由规则
#[tokio::test]
async fn test_list_routing_rules() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/config/routing/rules").await;
    
    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    // list_routing_rules 返回的是数组，不是对象
    assert!(json_response.is_array());
}
