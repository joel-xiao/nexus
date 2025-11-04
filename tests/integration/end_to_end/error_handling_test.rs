//! 错误处理端到端测试

use axum_test::TestServer;
use nexus::create_test_app;

/// 测试错误处理 - 404 端点
#[tokio::test]
async fn test_404_not_found() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let response = server.get("/nonexistent").await;
    
    response.assert_status_not_found();
}
