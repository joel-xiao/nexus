use crate::common::create_test_server;
use std::time::Instant;

#[tokio::test]
async fn test_health_performance() {
    let server = create_test_server();

    let start = Instant::now();

    for _ in 0..100 {
        let response = server.get("/health").await;
        response.assert_status_ok();
    }

    let duration = start.elapsed();
    println!("100 health checks completed in {:?}", duration);
    println!("Average: {:?} per request", duration / 100);

    assert!(duration.as_millis() < 10000);
}

#[tokio::test]
async fn test_throughput() {
    let server = create_test_server();

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

    assert!(rps > 100.0);
}
