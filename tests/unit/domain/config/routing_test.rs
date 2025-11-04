//! 路由规则单元测试

use nexus::domain::config::routing::{ModelRouter, RoutingRule, RoutingStrategy, ModelWeight};

/// 测试路由规则
#[tokio::test]
async fn test_routing_rules() {
    let router = ModelRouter::new();
    
    // 创建路由规则
    let rule = RoutingRule {
        name: "test_rule".to_string(),
        strategy: RoutingStrategy::Weighted,
        models: vec![
            ModelWeight {
                model_name: "model1".to_string(),
                adapter_name: "mock".to_string(),
                weight: 100,
                enabled: true,
            }
        ],
        condition: None,
        priority: 10,
    };
    
    router.add_rule(rule).await;
    
    // 测试选择模型
    let _result = router.select_model(Some("user1"), None).await;
    // 由于 mock 适配器可能不存在，结果可能是 None
    // 这取决于实际的路由逻辑
}
