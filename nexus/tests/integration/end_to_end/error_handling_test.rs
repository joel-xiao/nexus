use crate::common::create_test_server;

#[tokio::test]
async fn test_404_not_found() {
    let server = create_test_server();
    let response = server.get("/nonexistent").await;

    response.assert_status_not_found();
}
