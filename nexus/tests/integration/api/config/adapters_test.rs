use crate::common::{create_test_server, wait_for_adapters};

#[tokio::test]
async fn test_list_adapters() {
    let server = create_test_server();
    wait_for_adapters().await;

    let response = server.get("/api/config/adapters").await;

    response.assert_status_ok();
    let json_response: serde_json::Value = response.json();
    assert_eq!(json_response["status"], "ok");
    assert!(json_response["data"]["adapters"].is_array());
}
