-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Conversations table
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    messages JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_conversations_session ON conversations(session_id);
CREATE INDEX idx_conversations_created ON conversations(created_at DESC);

-- Conversation embeddings for RAG
CREATE TABLE conversation_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID REFERENCES conversations(id) ON DELETE CASCADE,
    message_text TEXT NOT NULL,
    embedding vector(768),  -- Google text-embedding-004 dimension (768)
    created_at TIMESTAMP DEFAULT NOW()
);

-- Index for similarity search (IVFFlat for fast approximate search)
CREATE INDEX idx_conversation_embeddings_vector 
    ON conversation_embeddings 
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100);

-- Business embeddings (optional - for semantic search)
CREATE TABLE business_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id TEXT NOT NULL UNIQUE,
    business_name TEXT NOT NULL,
    embedding vector(768),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_business_embeddings_vector 
    ON business_embeddings 
    USING ivfflat (embedding vector_cosine_ops);

