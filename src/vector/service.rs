use anyhow::Result;
use sqlx::PgPool;

pub struct VectorService {
    pool: PgPool,
}

impl VectorService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_conversation_embedding(
        &self,
        conversation_id: &str,
        message_text: &str,
        embedding: &[f32],
    ) -> Result<()> {
        // Convert f32 slice to pgvector format
        let embedding_str = format!(
            "[{}]",
            embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        sqlx::query(
            r#"
            INSERT INTO conversation_embeddings (conversation_id, message_text, embedding)
            VALUES (
                (SELECT id FROM conversations WHERE session_id::text = $1 LIMIT 1),
                $2,
                $3::vector
            )
            "#,
        )
        .bind(conversation_id)
        .bind(message_text)
        .bind(embedding_str)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn retrieve_context_for_rag(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<String>> {
        let embedding_str = format!(
            "[{}]",
            query_embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let rows = sqlx::query_as::<_, (String, f64)>(
            r#"
            SELECT message_text, 
                   1 - (embedding <=> $1::vector) as similarity
            FROM conversation_embeddings
            ORDER BY embedding <=> $1::vector
            LIMIT $2
            "#,
        )
        .bind(embedding_str)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(text, _)| text).collect())
    }
}
