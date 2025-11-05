use crate::agent::Orchestrator;
use axum::{routing::post, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use super::handlers;

pub fn create_routes(orchestrator: Orchestrator) -> Router {
    Router::new()
        .route("/api/chat", post(handlers::handle_chat))
        .route("/api/health", axum::routing::get(handlers::handle_health))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(orchestrator))
}

