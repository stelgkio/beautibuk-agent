# Quick Reference Guide

## Project Structure Template

```
beautibuk-agent/
├── Cargo.toml
├── .env.example
├── README.md
├── ARCHITECTURE.md
├── IMPLEMENTATION_GUIDE.md
├── API_INTEGRATION.md
├── VECTOR_DATABASE.md
├── TESTING_LLM_OPTIONS.md
├── src/
│   ├── main.rs                 # Application entry point
│   │
│   ├── api/                    # API layer
│   │   ├── mod.rs
│   │   ├── routes.rs           # Route definitions
│   │   ├── handlers.rs         # Request handlers
│   │   └── middleware.rs       # CORS, logging, etc.
│   │
│   ├── agent/                  # AI Agent system
│   │   ├── mod.rs
│   │   ├── orchestrator.rs     # Simplified agent logic (MCP)
│   │   ├── llm.rs              # LLM API integration with function calling
│   │   └── embeddings.rs       # Embedding generation
│   │
│   ├── mcp/                    # MCP client (replaces backend API client)
│   │   ├── mod.rs
│   │   ├── client.rs           # MCP client (JSON-RPC 2.0)
│   │   └── models.rs           # MCP protocol models
│   │
│   ├── vector/                 # Vector database operations
│   │   ├── mod.rs
│   │   ├── service.rs          # Vector DB operations
│   │   └── embeddings.rs       # Embedding generation
│   │
│   ├── session/                 # Session/conversation management
│   │   ├── mod.rs
│   │   └── manager.rs          # Session state management
│   │
│   ├── models/                 # Data models
│   │   ├── mod.rs
│   │   ├── chat.rs             # Chat message models
│   │   └── conversation.rs     # Conversation state
│   │
│   ├── database/               # Database layer
│   │   ├── mod.rs
│   │   ├── connection.rs       # PostgreSQL connection pool
│   │   └── migrations/         # SQL migrations
│   │
│   └── config/                 # Configuration
│       ├── mod.rs
│       └── settings.rs         # App settings
│
└── migrations/                 # Database migrations
    └── 001_initial.sql
```

## Cargo.toml Template

```toml
[package]
name = "beautibuk-agent"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database (PostgreSQL with pgvector)
sqlx = { version = "0.7", features = [
    "runtime-tokio-rustls",
    "postgres",
    "chrono",
    "uuid",
    "json"
] }
pgvector = "0.2"

# HTTP client
reqwest = { version = "0.11", features = ["json"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Environment variables
dotenv = "0.15"

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.0", features = ["v4", "serde"] }
```

## Key Code Patterns

### 1. Main Application Structure

```rust
// src/main.rs
use axum::{Router, routing::post};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Initialize database
    let db_pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await.unwrap();
    
    // Initialize services
    let mcp_client = McpClient::new(
        std::env::var("MCP_SERVER_URL").unwrap()
    );
    let llm_client = LlmClient::new(/* config */);
    let vector_service = VectorService::new(db_pool.clone());
    let orchestrator = AgentOrchestrator::new(
        llm_client,
        mcp_client,
        vector_service,
    );
    
    // Build application
    let app = Router::new()
        .route("/api/chat", post(handle_chat))
        .route("/api/health", get(handle_health))
        .with_state(AppState { orchestrator });
    
    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server running on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

### 2. Chat Handler

```rust
// src/api/handlers.rs
use axum::{Json, extract::State};
use crate::agent::Orchestrator;
use crate::models::chat::ChatRequest;

pub async fn handle_chat(
    State(orchestrator): State<Orchestrator>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let response = orchestrator
        .process_message(
            request.message,
            request.session_id,
        )
        .await?;
    
    Ok(Json(response))
}
```

### 3. Agent Orchestrator (SIMPLIFIED with MCP)

```rust
// src/agent/orchestrator.rs
pub struct Orchestrator {
    llm_client: LlmClient,
    mcp_client: McpClient,
    vector_service: VectorService,
    session_manager: SessionManager,
}

impl Orchestrator {
    pub async fn process_message(
        &self,
        message: String,
        session_id: String,
    ) -> Result<AgentResponse> {
        // 1. Load conversation context
        let context = self.session_manager
            .get_or_create_session(&session_id).await?;
        
        // 2. Optional: RAG for context enhancement
        let embedding = self.llm_client.generate_embedding(&message).await?;
        let similar_context = self.vector_service
            .retrieve_context_for_rag(&embedding, 5).await?;
        
        // 3. Build messages with context
        let mut messages = context.messages.clone();
        if !similar_context.is_empty() {
            messages.insert(0, ChatMessage {
                role: "system".to_string(),
                content: format!("Context: {}", similar_context),
            });
        }
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: message.clone(),
        });
        
        // 4. LLM handles everything via MCP tools - no manual routing!
        let response = self.llm_client
            .generate_with_mcp_tools(&messages, &self.mcp_client).await?;
        
        // 5. Store conversation
        self.session_manager.add_message(&session_id, &message, &response).await?;
        self.vector_service.store_embedding(&session_id, &message, &embedding).await?;
        
        Ok(AgentResponse { response, session_id })
    }
}
```

### 4. MCP Client (Replaces Backend API Client)

```rust
// src/mcp/client.rs
use reqwest::Client;
use serde_json::json;

pub struct McpClient {
    client: Client,
    base_url: String,
}

impl McpClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
    
    pub async fn list_tools(&self) -> Result<Vec<McpTool>> {
        let response = self.client
            .post(format!("{}/mcp", self.base_url))
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list",
                "params": {}
            }))
            .send()
            .await?;
        
        let result: McpResponse = response.json().await?;
        Ok(result.result.unwrap().tools.unwrap())
    }
    
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: &serde_json::Value,
    ) -> Result<String> {
        let response = self.client
            .post(format!("{}/mcp", self.base_url))
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": name,
                    "arguments": arguments
                }
            }))
            .send()
            .await?;
        
        let result: McpResponse = response.json().await?;
        Ok(result.result.unwrap().content.unwrap()[0].text.clone())
    }
}
```

### 5. LLM Client (Groq & Google)

```rust
// src/agent/llm.rs
pub struct LlmClient {
    provider: LlmProvider,
    api_key: String,
    model: String,
    client: Client,
}

impl LlmClient {
    pub async fn generate_response(
        &self,
        messages: &[ChatMessage],
    ) -> Result<String> {
        match self.provider {
            LlmProvider::Groq => self.call_groq(messages).await,
            LlmProvider::Google => self.call_google(messages).await,
        }
    }
    
    async fn call_groq(&self, messages: &[ChatMessage]) -> Result<String> {
        let response = self.client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": self.model,
                "messages": messages,
                "temperature": 0.7,
            }))
            .send()
            .await?;
        
        let result: GroqResponse = response.json().await?;
        Ok(result.choices[0].message.content.clone())
    }
    
    async fn call_google(&self, messages: &[ChatMessage]) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );
        
        let response = self.client
            .post(&url)
            .json(&json!({
                "contents": messages,
            }))
            .send()
            .await?;
        
        let result: GeminiResponse = response.json().await?;
        Ok(result.candidates[0].content.parts[0].text.clone())
    }
    
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Only Google supports embeddings
        match self.provider {
            LlmProvider::Google => {
                // Call Google embeddings API
                // ...
            }
            _ => Err(anyhow::anyhow!("Embeddings only supported by Google")),
        }
    }
}
```

### 6. Vector Service

```rust
// src/vector/service.rs
pub struct VectorService {
    pool: PgPool,
}

impl VectorService {
    pub async fn store_conversation_embedding(
        &self,
        conversation_id: &str,
        message_text: &str,
        embedding: Vec<f32>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO conversation_embeddings 
            (conversation_id, message_text, embedding)
            VALUES ($1, $2, $3)
            "#,
            conversation_id,
            message_text,
            pgvector::Vector::from(embedding)
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn find_similar_conversations(
        &self,
        query_embedding: Vec<f32>,
        limit: i64,
    ) -> Result<Vec<SimilarConversation>> {
        let query_vector = pgvector::Vector::from(query_embedding);
        
        let results = sqlx::query_as!(
            SimilarConversation,
            r#"
            SELECT 
                conversation_id,
                message_text,
                1 - (embedding <=> $1) as similarity
            FROM conversation_embeddings
            WHERE 1 - (embedding <=> $1) > 0.7
            ORDER BY embedding <=> $1
            LIMIT $2
            "#,
            query_vector,
            limit
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(results)
    }
}
```

## Environment Variables (.env)

```bash
# Python Backend
BACKEND_API_URL=http://localhost:8000/api

# LLM Provider (choose one)
# Option 1: Groq (fast, free tier)
LLM_PROVIDER=groq
GROQ_API_KEY=gsk_your_key_here
LLM_MODEL=llama-3.1-8b-instant

# Option 2: Google AI Studio (quality, embeddings)
# LLM_PROVIDER=google
# GOOGLE_AI_API_KEY=your_key_here
# LLM_MODEL=gemini-2.0-flash-exp

# LLM Settings
LLM_TEMPERATURE=0.7
LLM_MAX_TOKENS=2000

# Embeddings (for vector database)
EMBEDDING_PROVIDER=google
EMBEDDING_MODEL=text-embedding-004
GOOGLE_AI_API_KEY=your_key_here

# Database (PostgreSQL with pgvector)
DATABASE_URL=postgresql://user:password@localhost:5432/beautibuk_agent

# Server
AGENT_PORT=3000
SESSION_TIMEOUT_MINUTES=30
LOG_LEVEL=info

# CORS
ALLOWED_ORIGINS=http://localhost:8080
```

## Frontend Template (Vanilla JS)

```html
<!-- frontend/index.html -->
<!DOCTYPE html>
<html>
<head>
    <title>Beautibuk Agent</title>
    <link rel="stylesheet" href="css/style.css">
</head>
<body>
    <div class="chat-container">
        <div class="chat-header">
            <h1>Beautibuk Booking Agent</h1>
        </div>
        <div class="chat-messages" id="messages"></div>
        <div class="chat-input">
            <input type="text" id="messageInput" placeholder="Type your message...">
            <button id="sendButton">Send</button>
        </div>
    </div>
    <script src="js/app.js"></script>
</body>
</html>
```

```javascript
// frontend/js/app.js
const API_URL = 'http://localhost:3000/api';

let sessionId = localStorage.getItem('sessionId') || generateSessionId();

function generateSessionId() {
    const id = crypto.randomUUID();
    localStorage.setItem('sessionId', id);
    return id;
}

async function sendMessage(message) {
    const response = await fetch(`${API_URL}/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
            message,
            session_id: sessionId 
        }),
    });
    
    const data = await response.json();
    displayMessage(data.response, 'agent');
}

function displayMessage(text, sender) {
    const messagesDiv = document.getElementById('messages');
    const messageDiv = document.createElement('div');
    messageDiv.className = `message ${sender}`;
    messageDiv.textContent = text;
    messagesDiv.appendChild(messageDiv);
    messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

document.getElementById('sendButton').addEventListener('click', () => {
    const input = document.getElementById('messageInput');
    const message = input.value.trim();
    if (message) {
        displayMessage(message, 'user');
        sendMessage(message);
        input.value = '';
    }
});
```

## Database Schema (PostgreSQL with pgvector)

```sql
-- migrations/001_initial.sql

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

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

-- Conversation embeddings for semantic search (RAG)
CREATE TABLE conversation_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID REFERENCES conversations(id) ON DELETE CASCADE,
    message_index INTEGER NOT NULL,
    message_text TEXT NOT NULL,
    embedding vector(1536),  -- OpenAI/Gemini embedding dimension
    intent TEXT,
    entities JSONB,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Index for similarity search
CREATE INDEX idx_conversation_embeddings_vector 
    ON conversation_embeddings 
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100);

-- Business embeddings (optional - for semantic business search)
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
```

## Testing Checklist

- [ ] Python backend is running on http://localhost:8000
- [ ] PostgreSQL with pgvector is set up
- [ ] Database migrations run successfully
- [ ] Groq API key configured and working
- [ ] Google AI Studio API key configured (for embeddings)
- [ ] Agent understands search requests
- [ ] Agent extracts location correctly
- [ ] Business search via Python backend returns results
- [ ] Semantic search in vector DB works
- [ ] Booking creation via Python backend works
- [ ] Conversation context is stored in PostgreSQL
- [ ] Embeddings are generated and stored
- [ ] RAG retrieves similar conversations
- [ ] Error messages are user-friendly
- [ ] Frontend displays messages correctly
- [ ] API handles concurrent requests
- [ ] Session management works correctly

## Quick Setup Commands

```bash
# 1. Install PostgreSQL with pgvector
docker run -d \
  --name postgres-vector \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=beautibuk_agent \
  -p 5432:5432 \
  pgvector/pgvector:pg16

# 2. Get API keys
# Groq: https://console.groq.com
# Google: https://aistudio.google.com

# 3. Configure .env (see above)

# 4. Run migrations
sqlx migrate run

# 5. Start Python backend
cd ../beautibuk-back
python run.py

# 6. Start Rust agent
cargo run
```

## Next Steps After MVP

1. Add user authentication
2. Implement booking history
3. Add payment integration
4. Email/SMS confirmations
5. Admin dashboard
6. Analytics and reporting
7. Mobile app
8. Multi-language support
9. Real-time WebSocket updates
10. Advanced RAG improvements
