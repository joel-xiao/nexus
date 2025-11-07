use crate::common::{create_test_server, wait_for_adapters};
use serde_json::json;
use std::time::Instant;

#[tokio::test]
async fn test_concurrent_requests() {
    wait_for_adapters().await;

    let start = Instant::now();

    let mut handles = vec![];

    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let server = create_test_server();

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

    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start.elapsed();
    println!("100 concurrent requests completed in {:?}", duration);
    println!("Average: {:?} per request", duration / 100);

    assert!(duration.as_secs() < 30);
}
