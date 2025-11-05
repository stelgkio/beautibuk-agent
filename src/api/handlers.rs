use crate::agent::Orchestrator;
use crate::models::{ChatRequest, ChatResponse};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

pub async fn handle_health() -> StatusCode {
    StatusCode::OK
}

pub async fn handle_chat(
    State(orchestrator): State<Arc<Orchestrator>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<serde_json::Value>)> {
    let session_id = request
        .session_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    match orchestrator
        .process_message(request.message, session_id.clone())
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Error processing chat message: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to process message",
                    "message": e.to_string()
                })),
            ))
        }
    }
}
