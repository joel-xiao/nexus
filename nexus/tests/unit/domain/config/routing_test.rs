use nexus::domain::config::routing::{ModelRouter, ModelWeight, RoutingRule, RoutingStrategy};

/// 测试路由规则
#[tokio::test]
async fn test_routing_rules() {
    let router = ModelRouter::new();

    let rule = RoutingRule {
        name: "test_rule".to_string(),
        strategy: RoutingStrategy::Weighted,
        models: vec![ModelWeight {
            model_name: "model1".to_string(),
            adapter_name: "mock".to_string(),
            weight: 100,
            enabled: true,
        }],
        condition: None,
        priority: 10,
    };

    router.add_rule(rule).await;

    let _result = router.select_model(Some("user1"), None).await;
}
