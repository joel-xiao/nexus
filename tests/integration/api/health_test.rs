//! 健康检查 API 集成测试

use axum_test::TestServer;
use nexus::create_test_app;

/// 测试健康检查端点
#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let response = server.get("/health").await;
    
    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["status"], "healthy");
    assert_eq!(json_response["version"], "0.1.0");
}

/// 测试就绪检查端点
#[tokio::test]
async fn test_readiness_endpoint() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let response = server.get("/ready").await;
    
    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert!(json_response["checks"].is_object());
    assert!(json_response["checks"]["redis"].is_string());
    assert!(json_response["checks"]["adapters"].is_string());
}

/// 测试指标端点
#[tokio::test]
async fn test_metrics_endpoint() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let response = server.get("/metrics").await;
    
    response.assert_status_ok();
    let body = response.text();
    // Prometheus 指标格式应该包含一些指标或为空
    assert!(body.contains("http_requests_total") || body.is_empty() || body.contains("#"));
}
