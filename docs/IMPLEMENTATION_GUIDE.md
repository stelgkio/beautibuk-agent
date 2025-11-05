# Step-by-Step Implementation Guide

**Important**: This Rust agent integrates with the existing Python FastAPI backend (`beautibuk-back`). No database setup is needed - all data comes from the Python backend APIs.

## Phase 1: Project Setup and Foundation

### Step 1: Initialize Rust Project
```bash
cargo new beautibuk-agent --name beautibuk-agent
cd beautibuk-agent
```

### Step 2: Configure Cargo.toml
Add necessary dependencies:
- `axum` - Web framework
- `tokio` - Async runtime
- `serde` - Serialization (with `derive` feature)
- `reqwest` - HTTP client (for Python backend + LLM APIs)
- `anyhow` - Error handling
- `thiserror` - Custom error types
- `tracing` - Logging
- `chrono` - Date/time handling
- `uuid` - Session IDs
- `dotenv` - Environment variables
- `sqlx` - Async PostgreSQL driver (with `postgres`, `runtime-tokio-rustls`, `chrono`, `uuid` features)
- `pgvector` - PostgreSQL vector extension support

### Step 3: Project Structure
```
beautibuk-agent/
├── Cargo.toml
├── .env.example
├── src/
│   ├── main.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs
│   │   └── handlers.rs
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── orchestrator.rs
│   │   ├── llm.rs
│   │   └── prompt.rs
│   ├── backend/
│   │   ├── mod.rs
│   │   ├── client.rs          # Python backend API client
│   │   └── models.rs          # API request/response models
│   ├── models/
│   │   ├── mod.rs
│   │   ├── chat.rs            # Chat message models
│   │   └── conversation.rs    # Conversation state
│   ├── session/
│   │   ├── mod.rs
│   │   └── manager.rs         # Session/conversation state management
│   ├── database/
│   │   ├── mod.rs
│   │   ├── connection.rs       # PostgreSQL connection pool
│   │   └── migrations/         # SQL migrations
│   ├── vector/
│   │   ├── mod.rs
│   │   ├── service.rs          # Vector database operations
│   │   └── embeddings.rs      # Embedding generation
│   └── config/
│       ├── mod.rs
│       └── settings.rs
└── README.md
```

## Phase 2: MCP Client (Simpler than HTTP API Client!)

### Step 4: Create MCP Client
Implement MCP client for Python backend MCP server (see `MCP_CLIENT_IMPLEMENTATION.md`):
- `McpClient` struct with methods:
  - `initialize()` - Initialize MCP connection
  - `list_tools()` - Get available tools from MCP server
  - `call_tool(name, arguments)` - Call MCP tool
- Much simpler than HTTP API client - just JSON-RPC 2.0 calls

### Step 5: Define MCP Models
Create Rust structs for MCP protocol:
- `McpRequest`, `McpResponse`
- `McpTool` - Tool definition
- `McpError` - Error handling
- No need for business/booking models - MCP returns JSON directly

## Phase 3: Conversation State Management

### Step 6: Session Manager
Implement session/conversation state:
- `SessionManager` with HashMap storage
- Store conversation history per session
- Track extracted entities (location, date, time, service, business_id)
- Session timeout (default 30 minutes)

### Step 7: Conversation Context
Create conversation context model:
- `ConversationContext` struct:
  - `session_id: String`
  - `messages: Vec<ChatMessage>`
  - `extracted_entities: ExtractedEntities`
  - `last_search_results: Option<Vec<Business>>`
  - `pending_booking: Option<BookingDraft>`

### Step 8: Database Setup
Set up PostgreSQL with pgvector:
- Create database schema (see `VECTOR_DATABASE.md`)
- Set up SQLx connection pool
- Create migrations for:
  - `conversations` table
  - `conversation_embeddings` table
  - Vector indexes
- Run migrations on startup

### Step 9: Vector Service
Implement vector database operations:
- `VectorService` struct with PostgreSQL connection
- Methods:
  - `store_conversation_embedding()` - Store message embedding
  - `find_similar_conversations()` - Semantic search
  - `retrieve_context_for_rag()` - Get relevant context for RAG

## Phase 4: LLM Integration

### Step 10: LLM Client
Implement LLM integration:
- `LlmClient` struct supporting:
  - OpenAI API (GPT-4, GPT-3.5-turbo)
  - Anthropic API (Claude)
  - Ollama (local LLM)
- Methods:
  - `send_message()` - Send message with conversation history
  - `extract_intent()` - Extract user intent
  - `extract_entities()` - Extract location, date, time, service
  - `generate_response()` - Generate natural language response
- Error handling and retry logic

### Step 11: Embedding Service
Implement embedding generation:
- `EmbeddingService` struct
- Methods:
  - `generate_embedding()` - Generate embedding from text
  - Support OpenAI embeddings API
  - Cache embeddings (optional)

### Step 12: Prompt Templates
Create prompt engineering:
- System prompt: Define agent role as booking assistant
- Context format: Include conversation history
- Tool descriptions: Available actions (search, book, etc.)
- Response format: Natural language with structured data extraction

## Phase 5: Agent Orchestrator (SIMPLIFIED with MCP)

### Step 13: Agent Orchestrator
Implement simplified agent logic:
- `AgentOrchestrator` struct:
  - `process_message()` - Main entry point
  - **Simplified flow**:
    1. Load conversation context
    2. Optional: RAG for context enhancement
    3. Get MCP tools from MCP server
    4. Send to LLM with MCP tools - **LLM handles everything!**
    5. LLM autonomously calls MCP tools as needed
    6. Store conversation and embedding
    7. Return response

**Key simplification**: No intent extraction, no entity extraction, no routing logic - LLM handles it all via MCP tools!

### Step 14: LLM Function Calling Integration
Implement LLM function calling support:
- **Groq**: OpenAI-compatible function calling
  - Convert MCP tools to Groq function format
  - Handle tool call responses
  - Continue conversation with tool results
- **Google Gemini**: Function calling API
  - Convert MCP tools to Gemini function declarations
  - Handle function call responses
  - Continue conversation with function results

**No manual intent handling needed** - LLM decides which tools to call based on user message!

Enhanced flow:
1. Generate embedding for current message
2. Retrieve similar conversations (RAG)
3. Build enhanced prompt with RAG context + MCP tools
4. LLM autonomously calls MCP tools and generates response
5. Store conversation and embedding

## Phase 6: API Layer

### Step 12: API Routes
Set up Axum routes:
- `POST /api/chat` - Main chat endpoint
- `GET /api/health` - Health check
- Optional: `GET /api/sessions/:id` - Get conversation history

### Step 13: Request Handlers
Implement handlers:
- `handle_chat()` - Main chat handler:
  - Extract session_id from request (or generate new)
  - Load conversation context
  - Call agent orchestrator
  - Return response
- Error handling and validation

### Step 14: Middleware
Add middleware for:
- CORS configuration (allow frontend origin)
- Request logging
- Error handling
- Rate limiting (optional)

## Phase 7: Configuration and Environment

### Step 15: Environment Configuration
Create `.env.example`:
```
# MCP Server Configuration (Python backend MCP server)
MCP_SERVER_URL=http://localhost:8002
MCP_TRANSPORT=http  # or stdio

# LLM Provider Configuration (choose one)
# Both support function calling for MCP tools
# Option 1: Groq (fast, free tier, recommended for development)
LLM_PROVIDER=groq
GROQ_API_KEY=gsk_your_key_here
LLM_MODEL=llama-3.1-8b-instant

# Option 2: Google AI Studio (quality, free tier, good for testing)
# LLM_PROVIDER=google
# GOOGLE_AI_API_KEY=your_key_here
# LLM_MODEL=gemini-2.0-flash-exp

# LLM Settings
LLM_TEMPERATURE=0.7
LLM_MAX_TOKENS=2000

# Embeddings (for vector database) - Use Google AI Studio
EMBEDDING_PROVIDER=google
EMBEDDING_MODEL=text-embedding-004
GOOGLE_AI_API_KEY=your_key_here  # Same key as LLM if using Google

DATABASE_URL=postgresql://user:password@localhost:5432/beautibuk_agent

AGENT_PORT=3000
SESSION_TIMEOUT_MINUTES=30
LOG_LEVEL=info
```

**See `TESTING_LLM_OPTIONS.md` for detailed setup instructions for Groq and Google AI Studio.**

### Step 16: Settings Module
Implement configuration:
- Load from environment variables
- Default values for development
- Validation of required settings

## Phase 8: Integration and Testing

### Step 17: End-to-End Testing
Test complete flows with Python backend running:
1. Start Python backend: `cd beautibuk-back && python run.py`
2. Start Rust agent: `cargo run`
3. Test chat flow:
   - User: "Find salons near me"
   - Agent searches via Python backend, returns results
   - User: "Book the first one for tomorrow"
   - Agent creates booking via Python backend, confirms

### Step 18: Error Handling
Implement comprehensive error handling:
- Invalid inputs
- LLM API failures
- Python backend API failures (network, 404, 500, etc.)
- Session not found
- User-friendly error messages

### Step 19: Logging and Monitoring
Add logging for:
- Chat requests/responses
- Python backend API calls
- LLM API calls
- Agent decisions
- Errors and exceptions
- Performance metrics

## Phase 9: Optimization and Polish

### Step 20: Performance Optimization
- Connection pooling for HTTP client
- Caching for Python backend responses (optional)
- Async processing improvements
- Response time optimization
- Session cleanup (remove old sessions)

### Step 21: User Experience Enhancements
- Better error messages
- Loading states in responses
- Conversation context preservation
- Support for follow-up questions

### Step 22: Documentation
- Code comments
- README updates
- API integration guide (see `API_INTEGRATION.md`)
- Deployment guide

## Phase 10: Deployment Preparation

### Step 23: Docker Setup
- Create Dockerfile for Rust agent
- Docker Compose including:
  - Rust agent service
  - Python backend service (reference)
  - Optional: Redis for session storage
- Production configuration

### Step 24: Environment Configuration
- Production environment variables
- Secrets management
- Health check endpoints
- Monitoring setup

### Step 25: CI/CD (Optional)
- GitHub Actions workflow
- Automated testing
- Docker image builds
- Deployment pipeline

## Implementation Priority

**High Priority (MVP)**:
1. Steps 1-3: Project setup and structure
2. Steps 4-5: MCP client (much simpler than HTTP API client!)
3. Steps 6-7: Session/conversation management
4. Steps 8-9: Database setup and vector service
5. Steps 10-12: LLM integration with function calling + embeddings
6. Steps 13-14: Simplified agent orchestrator (LLM handles tool selection)
7. Steps 15-16: API layer and configuration

**Note**: With MCP, the agent is much simpler - no intent/entity extraction needed!

**Medium Priority**:
8. Steps 17-19: Testing and error handling
9. Steps 20-21: Optimization and UX

**Low Priority (Nice to have)**:
10. Steps 22-25: Documentation and deployment

## Quick Start Checklist

- [ ] Initialize Rust project
- [ ] Set up dependencies (include sqlx, pgvector)
- [ ] Create project structure
- [ ] Set up PostgreSQL with pgvector
- [ ] Create database schema and migrations
- [ ] Implement MCP client (see `MCP_CLIENT_IMPLEMENTATION.md`)
- [ ] Implement session manager with PostgreSQL
- [ ] Implement vector service (see `VECTOR_DATABASE.md`)
- [ ] Set up embedding generation
- [ ] Set up LLM integration with function calling
- [ ] Create simplified agent orchestrator (LLM handles tool selection)
- [ ] Create chat API endpoint
- [ ] Test MCP connection to Python backend
- [ ] Test with LLM calling MCP tools
- [ ] Add error handling
- [ ] Configure environment variables

