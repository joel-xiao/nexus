pub mod agents;
pub mod api_doc;
pub mod common;
pub mod config;
pub mod handlers;
pub mod health;
pub mod invoke;

use axum::Router;

pub fn mod_routes() -> Router {
    Router::new()
        .merge(invoke::invoke_routes())
        .merge(agents::agents_routes())
        .nest("/config", config::config_routes())
}
