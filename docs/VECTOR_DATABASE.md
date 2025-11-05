# Vector Database with PostgreSQL + pgvector

This document describes how PostgreSQL with pgvector is used for semantic search and RAG (Retrieval Augmented Generation) in the booking agent.

**See also**: `VECTOR_DB_BENEFITS.md` for detailed benefits and business data storage strategies.

## Overview

PostgreSQL with the pgvector extension provides:
- **Semantic Search**: Find similar conversations based on meaning, not just keywords
- **RAG (Retrieval Augmented Generation)**: Retrieve relevant context from past conversations
- **Context Enhancement**: Improve agent responses by finding similar past interactions
- **Embedding Storage**: Store vector representations of conversations, messages, and entities

## Setup

### 1. Install PostgreSQL with pgvector

```bash
# Using Docker
docker run -d \
  --name postgres-vector \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=beautibuk_agent \
  -p 5432:5432 \
  pgvector/pgvector:pg16

# Or install locally
# See: https://github.com/pgvector/pgvector#installation
```

### 2. Enable pgvector Extension

```sql
CREATE EXTENSION IF NOT EXISTS vector;
```

### 3. Database Schema

```sql
-- Conversations table
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    messages JSONB NOT NULL,
    extracted_entities JSONB,
    last_search_results JSONB,
    pending_booking JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_conversations_session ON conversations(session_id);
CREATE INDEX idx_conversations_created ON conversations(created_at DESC);

-- Conversation embeddings for semantic search
CREATE TABLE conversation_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID REFERENCES conversations(id) ON DELETE CASCADE,
    message_index INTEGER NOT NULL,
    message_text TEXT NOT NULL,
    embedding vector(1536),  -- OpenAI text-embedding-ada-002 dimension
    intent TEXT,
    entities JSONB,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Index for similarity search (IVFFlat for fast approximate search)
CREATE INDEX idx_conversation_embeddings_vector 
    ON conversation_embeddings 
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100);

-- Business embeddings (optional - for semantic business matching)
CREATE TABLE business_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id TEXT NOT NULL UNIQUE,
    business_name TEXT NOT NULL,
    business_description TEXT,
    embedding vector(1536),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_business_embeddings_vector 
    ON business_embeddings 
    USING ivfflat (embedding vector_cosine_ops);

-- Service embeddings (optional - for semantic service matching)
CREATE TABLE service_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_id TEXT NOT NULL UNIQUE,
    service_name TEXT NOT NULL,
    service_description TEXT,
    embedding vector(1536),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_service_embeddings_vector 
    ON service_embeddings 
    USING ivfflat (embedding vector_cosine_ops);
```

## Usage Patterns

### 1. Store Conversation Embeddings

```rust
use sqlx::{PgPool, Postgres};
use pgvector::Vector;

pub struct VectorService {
    pool: PgPool,
}

impl VectorService {
    pub async fn store_conversation_embedding(
        &self,
        conversation_id: &str,
        message_index: i32,
        message_text: &str,
        embedding: Vec<f32>,
        intent: Option<&str>,
        entities: Option<serde_json::Value>,
    ) -> Result<(), sqlx::Error> {
        let embedding_vector = Vector::from(embedding);
        
        sqlx::query!(
            r#"
            INSERT INTO conversation_embeddings 
            (conversation_id, message_index, message_text, embedding, intent, entities)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            conversation_id::uuid,
            message_index,
            message_text,
            embedding_vector,
            intent,
            entities
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

### 2. Semantic Search for Similar Conversations

```rust
impl VectorService {
    pub async fn find_similar_conversations(
        &self,
        query_embedding: Vec<f32>,
        limit: i64,
        similarity_threshold: f64,
    ) -> Result<Vec<SimilarConversation>, sqlx::Error> {
        let query_vector = Vector::from(query_embedding);
        
        let results = sqlx::query_as!(
            SimilarConversation,
            r#"
            SELECT 
                ce.conversation_id,
                ce.message_text,
                ce.intent,
                ce.entities,
                1 - (ce.embedding <=> $1) as similarity
            FROM conversation_embeddings ce
            WHERE 1 - (ce.embedding <=> $1) > $2
            ORDER BY ce.embedding <=> $1
            LIMIT $3
            "#,
            query_vector,
            similarity_threshold,
            limit
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(results)
    }
}
```

### 3. RAG: Retrieve Relevant Context

```rust
impl VectorService {
    pub async fn retrieve_context_for_rag(
        &self,
        current_message: &str,
        current_embedding: Vec<f32>,
        limit: usize,
    ) -> Result<String, sqlx::Error> {
        // Find similar conversations
        let similar = self
            .find_similar_conversations(current_embedding, limit as i64, 0.7)
            .await?;
        
        // Build context from similar conversations
        let mut context = String::from("Relevant past conversations:\n");
        for conv in similar {
            context.push_str(&format!(
                "- Intent: {}\n  Message: {}\n  Entities: {}\n\n",
                conv.intent.unwrap_or("unknown"),
                conv.message_text,
                serde_json::to_string(&conv.entities).unwrap_or_default()
            ));
        }
        
        Ok(context)
    }
}
```

### 4. Generate Embeddings

```rust
use reqwest::Client;

pub struct EmbeddingService {
    client: Client,
    api_key: String,
}

impl EmbeddingService {
    pub async fn generate_embedding(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, anyhow::Error> {
        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": "text-embedding-ada-002",
                "input": text
            }))
            .send()
            .await?;
        
        let embedding_response: EmbeddingResponse = response.json().await?;
        Ok(embedding_response.data[0].embedding.clone())
    }
}
```

## Integration with Agent

### Enhanced Agent Flow with RAG

```rust
impl AgentOrchestrator {
    pub async fn process_message_with_rag(
        &self,
        message: &str,
        session_id: &str,
    ) -> Result<String, AgentError> {
        // 1. Generate embedding for current message
        let embedding = self.embedding_service.generate_embedding(message).await?;
        
        // 2. Retrieve similar conversations (RAG)
        let context = self
            .vector_service
            .retrieve_context_for_rag(message, embedding.clone(), 5)
            .await?;
        
        // 3. Load conversation history
        let conversation = self
            .conversation_service
            .get_conversation(session_id)
            .await?;
        
        // 4. Build enhanced prompt with RAG context
        let prompt = self.build_prompt_with_rag(
            message,
            &conversation.messages,
            &context,
        );
        
        // 5. Send to LLM
        let response = self.llm_client.generate_response(&prompt).await?;
        
        // 6. Store conversation and embedding
        self.store_conversation_turn(
            session_id,
            message,
            &response,
            embedding,
        ).await?;
        
        Ok(response)
    }
}
```

## Benefits

1. **Better Context Understanding**: Agent can learn from similar past conversations
2. **Improved Responses**: RAG provides relevant examples to guide response generation
3. **Semantic Matching**: Find conversations by meaning, not just keywords
4. **Scalability**: PostgreSQL handles large volumes of embeddings efficiently
5. **Flexibility**: Can store embeddings for businesses, services, and other entities

## Performance Considerations

1. **Index Type**: IVFFlat for approximate search (faster, slightly less accurate)
   - Use HNSW for better accuracy (PostgreSQL 17+)
   
2. **List Size**: Adjust `lists` parameter based on dataset size
   - Rule of thumb: `lists = rows / 1000` for IVFFlat

3. **Batch Operations**: Store embeddings in batches for efficiency

4. **Caching**: Cache frequently accessed embeddings

5. **Cleanup**: Remove old embeddings periodically to maintain performance

## Migration

```sql
-- Migration: Add pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Migration: Create tables
-- (See schema above)

-- Migration: Create indexes
-- (See indexes above)
```

## Rust Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"] }
pgvector = "0.2"
tokio-postgres = { version = "0.7", features = ["with-uuid-1"] }
```

## Example Query

```sql
-- Find conversations similar to "I need a haircut tomorrow"
SELECT 
    ce.message_text,
    ce.intent,
    1 - (ce.embedding <=> (SELECT embedding FROM conversation_embeddings 
                           WHERE message_text LIKE '%haircut%' LIMIT 1)) as similarity
FROM conversation_embeddings ce
WHERE 1 - (ce.embedding <=> (SELECT embedding FROM conversation_embeddings 
                             WHERE message_text LIKE '%haircut%' LIMIT 1)) > 0.7
ORDER BY similarity DESC
LIMIT 10;
```

