pub mod health;
pub mod invoke;
pub mod config;
pub mod api_doc;
pub mod handlers;
pub mod common;

use axum::Router;

pub fn mod_routes() -> Router {
    Router::new()
        .merge(invoke::invoke_routes())
        .nest("/config", config::config_routes())
}



