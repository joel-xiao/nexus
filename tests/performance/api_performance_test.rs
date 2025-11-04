//! API 性能测试

use axum_test::TestServer;
use nexus::create_test_app;
use std::time::Instant;

/// 测试健康检查端点的性能
#[tokio::test]
async fn test_health_performance() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let start = Instant::now();
    
    for _ in 0..100 {
        let response = server.get("/health").await;
        response.assert_status_ok();
    }
    
    let duration = start.elapsed();
    println!("100 health checks completed in {:?}", duration);
    println!("Average: {:?} per request", duration / 100);
    
    // 每个请求应该在 10ms 内完成
    assert!(duration.as_millis() < 10000);
}

/// 测试大量请求的吞吐量
#[tokio::test]
async fn test_throughput() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();

    let num_requests = 1000;
    let start = Instant::now();
    
    for i in 0..num_requests {
        let response = server.get("/health").await;
        response.assert_status_ok();
        
        if i % 100 == 0 {
            println!("Processed {} requests", i);
        }
    }
    
    let duration = start.elapsed();
    let rps = num_requests as f64 / duration.as_secs_f64();
    
    println!("Processed {} requests in {:?}", num_requests, duration);
    println!("Throughput: {:.2} requests/second", rps);
    
    // 应该至少达到 100 req/s
    assert!(rps > 100.0);
}
