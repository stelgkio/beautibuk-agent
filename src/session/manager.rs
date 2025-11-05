use crate::models::{ChatMessage, ConversationContext};
use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct SessionManager {
    pool: PgPool,
}

impl SessionManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_or_create_session(&self, session_id: &str) -> Result<ConversationContext> {
        let session_uuid = Uuid::parse_str(session_id).unwrap_or_else(|_| Uuid::new_v4());

        let row = sqlx::query!(
            r#"
            SELECT 
                session_id::text as session_id,
                messages::jsonb as messages
            FROM conversations
            WHERE session_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            session_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let messages: Vec<ChatMessage> = serde_json::from_value(row.messages)?;
            Ok(ConversationContext {
                session_id: row.session_id,
                messages,
            })
        } else {
            Ok(ConversationContext::new(session_id.to_string()))
        }
    }

    pub async fn add_message(
        &self,
        session_id: &str,
        user_message: &str,
        assistant_message: &str,
    ) -> Result<()> {
        let session_uuid = Uuid::parse_str(session_id).unwrap_or_else(|_| Uuid::new_v4());

        let mut context = self.get_or_create_session(session_id).await?;

        context.add_message(ChatMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
            tool_calls: None,
        });

        context.add_message(ChatMessage {
            role: "assistant".to_string(),
            content: assistant_message.to_string(),
            tool_calls: None,
        });

        let messages_json = serde_json::to_value(&context.messages)?;

        sqlx::query!(
            r#"
            INSERT INTO conversations (session_id, messages, updated_at)
            VALUES ($1, $2, $3)
            "#,
            session_uuid,
            messages_json,
            Utc::now()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
