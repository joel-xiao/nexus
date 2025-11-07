use agentflow::{AgentConfig, AgentOrchestrator, AgentRole, OrchestrationConfig, SpeakerSelection};
use uuid;

#[tokio::test]
async fn test_orchestrator_creation() {
    let config = OrchestrationConfig {
        session_id: uuid::Uuid::new_v4().to_string(),
        max_rounds: 10,
        timeout_seconds: 30,
        auto_planning: false,
        save_history: true,
        speaker_selection: SpeakerSelection::RoundRobin,
        agent_order: Vec::new(),
        termination_condition: Some("TERMINATE".to_string()),
        metadata: std::collections::HashMap::new(),
    };

    let _orchestrator = AgentOrchestrator::new(config.clone());
    assert_eq!(config.max_rounds, 10);
}

#[tokio::test]
async fn test_orchestrator_with_agents() {
    let config = OrchestrationConfig {
        session_id: uuid::Uuid::new_v4().to_string(),
        max_rounds: 5,
        timeout_seconds: 10,
        auto_planning: false,
        save_history: true,
        speaker_selection: SpeakerSelection::RoundRobin,
        agent_order: Vec::new(),
        termination_condition: Some("TERMINATE".to_string()),
        metadata: std::collections::HashMap::new(),
    };

    let _orchestrator = AgentOrchestrator::new(config);

    let agent_config = AgentConfig::new(
        "agent1".to_string(),
        "Test Agent".to_string(),
        AgentRole::Assistant,
        "Test".to_string(),
        "You are helpful".to_string(),
        "mock".to_string(),
    );

    assert_eq!(agent_config.id, "agent1");
}
