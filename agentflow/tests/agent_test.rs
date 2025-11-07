use agentflow::{AgentConfig, AgentContext, AgentMessage, AgentRole, MessageType};

#[test]
fn test_agent_config_creation() {
    let config = AgentConfig::new(
        "agent1".to_string(),
        "Test Agent".to_string(),
        AgentRole::Assistant,
        "A test agent".to_string(),
        "You are a helpful assistant".to_string(),
        "mock".to_string(),
    );

    assert_eq!(config.id, "agent1");
    assert_eq!(config.name, "Test Agent");
    assert_eq!(config.role, AgentRole::Assistant);
    assert_eq!(config.system_prompt, "You are a helpful assistant");
}

#[test]
fn test_agent_message_creation() {
    let message = AgentMessage::new(
        "sender1".to_string(),
        "Sender".to_string(),
        Some("receiver1".to_string()),
        "Hello".to_string(),
        MessageType::Text,
    );

    assert_eq!(message.sender_id, "sender1");
    assert_eq!(message.content, "Hello");
    assert_eq!(message.message_type, MessageType::Text);
}

#[test]
fn test_agent_context_creation() {
    let context = AgentContext::new();

    assert_eq!(context.conversation_history.len(), 0);
    assert_eq!(context.shared_state.len(), 0);
    assert_eq!(context.local_state.len(), 0);
}

#[tokio::test]
async fn test_agent_context_add_message() {
    let mut context = AgentContext::new();
    let message = AgentMessage::new(
        "sender1".to_string(),
        "Sender".to_string(),
        None,
        "Hello".to_string(),
        MessageType::Text,
    );

    context.add_message(message);
    assert_eq!(context.conversation_history.len(), 1);
}

#[tokio::test]
async fn test_agent_context_shared_state() {
    let mut context = AgentContext::new();

    context.set_shared("key1".to_string(), serde_json::json!("value1"));
    assert_eq!(
        context.get_shared("key1"),
        Some(&serde_json::json!("value1"))
    );

    assert_eq!(context.get_shared("nonexistent"), None);
}

#[tokio::test]
async fn test_agent_context_local_state() {
    let mut context = AgentContext::new();

    context.set_local("key1".to_string(), serde_json::json!("value1"));
    assert_eq!(
        context.get_local("key1"),
        Some(&serde_json::json!("value1"))
    );

    assert_eq!(context.get_local("nonexistent"), None);
}

#[tokio::test]
async fn test_agent_context_get_last_n_messages() {
    let mut context = AgentContext::new();

    for i in 0..10 {
        let message = AgentMessage::new(
            "sender".to_string(),
            "Sender".to_string(),
            None,
            format!("Message {}", i),
            MessageType::Text,
        );
        context.add_message(message);
    }

    let recent = context.get_last_n_messages(5);
    assert_eq!(recent.len(), 5);
    assert_eq!(recent[0].content, "Message 5");
    assert_eq!(recent[4].content, "Message 9");
}
