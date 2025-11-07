//! 并发性能测试

mod common;
use common::wait_for_adapters;

use axum_test::TestServer;
use serde_json::json;
use nexus::create_test_app;
use std::time::Instant;

/// 测试并发请求处理
#[tokio::test]
async fn test_concurrent_requests() {
    // 等待 mock 适配器注册
    wait_for_adapters().await;

    let start = Instant::now();
    
    let mut handles = vec![];
    
    // 为每个并发任务创建独立的测试服务器
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let app = create_test_app();
            let server = TestServer::new(app).unwrap();
            
            for j in 0..10 {
                let response = server
                    .post("/api/invoke")
                    .json(&json!({
                        "input": format!("Request {}:{}", i, j),
                        "adapter": "mock"
                    }))
                    .await;
                
                response.assert_status_ok();
            }
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }
    
    let duration = start.elapsed();
    println!("100 concurrent requests completed in {:?}", duration);
    println!("Average: {:?} per request", duration / 100);
    
    // 100 个并发请求应该在合理时间内完成
    assert!(duration.as_secs() < 30);
}
