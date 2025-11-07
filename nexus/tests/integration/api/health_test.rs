use crate::common::create_test_server;

#[tokio::test]
async fn test_health_endpoint() {
    let server = create_test_server();
    let response = server.get("/health").await;

    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["status"], "ok");
    assert_eq!(json_response["data"]["status"], "healthy");
}

#[tokio::test]
async fn test_readiness_endpoint() {
    let server = create_test_server();
    let response = server.get("/ready").await;

    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["status"], "ok");
    assert!(json_response["data"]["ready"].is_boolean());
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let server = create_test_server();
    let response = server.get("/metrics").await;

    response.assert_status_ok();
    let body = response.text();
    assert!(body.contains("http_requests_total") || body.is_empty() || body.contains("#"));
}
