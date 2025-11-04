//! 提示存储单元测试

use nexus::application::prompt::PromptStore;

/// 测试提示存储
#[tokio::test]
async fn test_prompt_store() {
    let mut store = PromptStore::new();
    
    // 先注册一个模板
    store.register_template("test_template", "{{input}}").unwrap();
    
    // 测试渲染已注册的模板
    let result = store.render("test_template", &serde_json::json!({
        "input": "Hello"
    }));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello");
    
    // 测试渲染字符串模板（临时模板）
    let result = store.render_string("Hello {{name}}", &serde_json::json!({
        "name": "World"
    }));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello World");
}
