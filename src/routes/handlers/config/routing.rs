use axum::{Json, Extension};
use crate::state::AppState;
use crate::domain::config::routing::{RoutingRule, RoutingStrategy};
use std::sync::Arc;
use super::common::{ok_response, ok_response_with_message, error_response};
use crate::routes::config::routing::{CreateRuleRequest, UpdateRuleRequest};

fn parse_routing_strategy(strategy: &str) -> RoutingStrategy {
    match strategy {
        "round_robin" => RoutingStrategy::RoundRobin,
        "random" => RoutingStrategy::Random,
        "weighted" => RoutingStrategy::Weighted,
        "least_connections" => RoutingStrategy::LeastConnections,
        "user_based" => RoutingStrategy::UserBased,
        "hash_based" => RoutingStrategy::HashBased,
        _ => RoutingStrategy::RoundRobin,
    }
}

pub async fn create_routing_rule(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateRuleRequest>,
) -> Json<serde_json::Value> {
    let strategy = parse_routing_strategy(&payload.strategy);
    let rule = RoutingRule {
        name: payload.name.clone(),
        strategy,
        models: payload.models,
        condition: None,
        priority: payload.priority.unwrap_or(0),
    };

    state.config_manager.router().add_rule(rule).await;
    ok_response(serde_json::json!({ "rule": payload.name }))
}

pub async fn list_routing_rules(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<Vec<RoutingRule>> {
    let rules = state.config_manager.router().list_rules().await;
    Json(rules)
}

pub async fn get_routing_rule(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let rules = state.config_manager.router().list_rules().await;
    match rules.iter().find(|r| r.name == name) {
        Some(rule) => ok_response(serde_json::json!({ "rule": rule })),
        None => error_response(&format!("Routing rule {} not found", name))
    }
}

pub async fn update_routing_rule(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(payload): Json<UpdateRuleRequest>,
) -> Json<serde_json::Value> {
    let strategy = parse_routing_strategy(&payload.strategy);
    let rule = RoutingRule {
        name: name.clone(),
        strategy,
        models: payload.models,
        condition: None,
        priority: payload.priority.unwrap_or(0),
    };

    state.config_manager.router().update_rule(&name, rule).await;
    ok_response(serde_json::json!({ "rule": name }))
}

pub async fn delete_routing_rule(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    state.config_manager.router().remove_rule(&name).await;
    ok_response_with_message(&format!("Routing rule {} deleted", name), serde_json::json!({}))
}
