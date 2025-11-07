//! 通用测试辅助函数

use nexus::create_test_app;
use axum_test::TestServer;

/// 创建测试服务器实例
pub fn create_test_server() -> TestServer {
    let app = create_test_app();
    TestServer::new(app).expect("Failed to create test server")
}

/// 等待适配器注册完成
pub async fn wait_for_adapters() {
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
}
