use crate::routes::handlers::agents as handlers;
use axum::routing::{get, post};
use axum::Router;
use utoipa::OpenApi;

#[utoipa::path(
    post,
    path = "/api/agents/conversation",
    tag = "agents",
    request_body = handlers::ConversationRequest,
    responses(
        (status = 200, description = "对话结果", content_type = "application/json")
    )
)]
pub async fn start_conversation(
    axum::Extension(state): axum::Extension<std::sync::Arc<crate::state::AppState>>,
    axum::Json(payload): axum::Json<handlers::ConversationRequest>,
) -> axum::Json<serde_json::Value> {
    handlers::start_conversation(axum::Extension(state), axum::Json(payload)).await
}

#[utoipa::path(
    post,
    path = "/api/agents/orchestrate",
    tag = "agents",
    request_body = handlers::OrchestrateRequest,
    responses(
        (status = 200, description = "编排结果", content_type = "application/json")
    )
)]
pub async fn orchestrate_agents(
    axum::Extension(state): axum::Extension<std::sync::Arc<crate::state::AppState>>,
    axum::Json(payload): axum::Json<handlers::OrchestrateRequest>,
) -> axum::Json<serde_json::Value> {
    handlers::orchestrate_agents(axum::Extension(state), axum::Json(payload)).await
}

#[utoipa::path(
    get,
    path = "/api/agents",
    tag = "agents",
    responses(
        (status = 200, description = "代理列表", content_type = "application/json")
    )
)]
pub async fn list_agents(
    axum::Extension(state): axum::Extension<std::sync::Arc<crate::state::AppState>>,
) -> axum::Json<serde_json::Value> {
    handlers::list_agents(axum::Extension(state)).await
}

pub fn agents_routes() -> Router {
    Router::new()
        .route("/agents/conversation", post(start_conversation))
        .route("/agents/orchestrate", post(orchestrate_agents))
        .route("/agents", get(list_agents))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        start_conversation,
        orchestrate_agents,
        list_agents,
    ),
    components(schemas(
        handlers::ConversationRequest,
        handlers::OrchestrateRequest,
        handlers::ConversationResponse,
        crate::routes::common::ErrorResponse,
    )),
    tags(
        (name = "agents", description = "多智能体对话"),
    )
)]
pub struct AgentsApiDoc;

