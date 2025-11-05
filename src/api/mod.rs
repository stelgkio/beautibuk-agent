pub mod handlers;
pub mod routes;

use crate::agent::Orchestrator;
use axum::Router;

pub fn create_router(orchestrator: Orchestrator) -> Router {
    routes::create_routes(orchestrator)
}
