use crate::agent::{EmbeddingService, LlmClient};
use crate::mcp::McpClient;
use crate::models::{ChatMessage, ChatResponse, ConversationContext};
use crate::session::SessionManager;
use crate::vector::VectorService;
use anyhow::Result;
use uuid::Uuid;

pub struct Orchestrator {
    llm_client: LlmClient,
    mcp_client: McpClient,
    session_manager: SessionManager,
    vector_service: VectorService,
    embedding_service: EmbeddingService,
}

impl Orchestrator {
    pub fn new(
        llm_client: LlmClient,
        mcp_client: McpClient,
        session_manager: SessionManager,
        vector_service: VectorService,
        embedding_service: EmbeddingService,
    ) -> Self {
        Self {
            llm_client,
            mcp_client,
            session_manager,
            vector_service,
            embedding_service,
        }
    }

    pub async fn process_message(
        &self,
        message: String,
        session_id: String,
    ) -> Result<ChatResponse> {
        // 1. Load conversation context
        let context = self
            .session_manager
            .get_or_create_session(&session_id)
            .await?;

        // 2. Optional: RAG for context enhancement
        let embedding = self
            .embedding_service
            .generate_embedding(&message)
            .await?;
        let similar_context = self
            .vector_service
            .retrieve_context_for_rag(&embedding, 5)
            .await?;

        // 3. Build messages with context
        let mut messages = context.messages.clone();
        if !similar_context.is_empty() {
            messages.insert(
                0,
                ChatMessage {
                    role: "system".to_string(),
                    content: format!(
                        "Relevant context from past conversations:\n{}",
                        similar_context.join("\n")
                    ),
                    tool_calls: None,
                },
            );
        }
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: message.clone(),
            tool_calls: None,
        });

        // 4. LLM handles everything via MCP tools - no manual routing!
        let response = self
            .llm_client
            .generate_with_mcp_tools(&messages, &self.mcp_client)
            .await?;

        // 5. Store conversation
        self.session_manager
            .add_message(&session_id, &message, &response)
            .await?;

        // 6. Store embedding
        self.vector_service
            .store_conversation_embedding(&session_id, &message, &embedding)
            .await?;

        Ok(ChatResponse {
            response,
            session_id,
        })
    }
}

