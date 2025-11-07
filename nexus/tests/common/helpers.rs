use axum_test::TestServer;
use nexus::create_test_app;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestMode {
    Mock,
    Real,
}

pub fn get_test_mode() -> TestMode {
    match env::var("NEXUS_TEST_MODE").as_deref() {
        Ok("real") | Ok("REAL") => TestMode::Real,
        Ok("mock") | Ok("MOCK") | Ok(_) => TestMode::Mock,
        Err(_) => TestMode::Mock,
    }
}

pub fn is_real_test() -> bool {
    get_test_mode() == TestMode::Real
}

pub fn is_mock_test() -> bool {
    get_test_mode() == TestMode::Mock
}

pub fn create_test_server() -> TestServer {
    let app = create_test_app();
    TestServer::new(app).expect("Failed to create test server")
}

/// 等待适配器注册完成
/// 在 Mock 模式下等待时间较短，真实模式下等待时间较长
pub async fn wait_for_adapters() {
    let wait_time = if is_real_test() {
        1000 // 真实测试需要更长时间等待适配器初始化
    } else {
        200 // Mock 测试快速返回
    };
    tokio::time::sleep(tokio::time::Duration::from_millis(wait_time)).await;
}

/// 跳过真实测试的宏
/// 如果当前是真实测试模式，则跳过测试
#[macro_export]
macro_rules! skip_if_real {
    () => {
        if crate::tests::common::helpers::is_real_test() {
            return;
        }
    };
}

/// 跳过 Mock 测试的宏
/// 如果当前是 Mock 测试模式，则跳过测试
#[macro_export]
macro_rules! skip_if_mock {
    () => {
        if crate::tests::common::helpers::is_mock_test() {
            return;
        }
    };
}
