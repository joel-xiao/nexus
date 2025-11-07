//! 测试工具函数

/// 等待指定时间（用于测试中的时间控制）
pub async fn wait(ms: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
}

/// 等待适配器就绪（简化的等待函数）
pub async fn wait_for_ready() {
    wait(100).await;
}
