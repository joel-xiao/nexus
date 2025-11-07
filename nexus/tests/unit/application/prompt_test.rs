use nexus::application::prompt::PromptStore;

/// 测试提示存储
#[tokio::test]
async fn test_prompt_store() {
    let mut store = PromptStore::new();

    store
        .register_template("test_template", "{{input}}")
        .unwrap();

    let result = store.render(
        "test_template",
        &serde_json::json!({
            "input": "Hello"
        }),
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello");

    let result = store.render_string(
        "Hello {{name}}",
        &serde_json::json!({
            "name": "World"
        }),
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello World");
}
