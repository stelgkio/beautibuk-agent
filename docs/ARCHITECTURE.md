# Beauty Salon Booking Agent - Architecture Analysis

## System Overview

The booking agent system integrates with the existing **beautibuk-back** Python/FastAPI backend. The Rust agent acts as an intelligent middleware layer that adds conversational AI capabilities.

The system consists of:

1. **Frontend UI Layer** - Web interface with chat functionality
2. **Rust Agent Service** - AI-powered conversation agent (NEW)
3. **Python Backend API** - Existing FastAPI backend with all business logic (EXISTING)
4. **MongoDB Database** - Data storage (EXISTING)

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (Web UI)                         │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Chat Interface Component                               │  │
│  │  - Chat button                                          │  │
│  │  - Message input/output                                │  │
│  │  - Real-time updates                                   │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────┬────────────────────────────────────┘
                           │ HTTP
                           │
┌──────────────────────────▼────────────────────────────────────┐
│         Rust Agent Service (NEW)                             │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  REST API Endpoints                                    │  │
│  │  - POST /api/chat                                      │  │
│  │  - GET  /api/health                                    │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Agent Orchestrator                                    │  │
│  │  - Message routing                                     │  │
│  │  - Intent extraction                                   │  │
│  │  - Context management                                  │  │
│  │  - RAG (Retrieval Augmented Generation)               │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  LLM Integration                                       │  │
│  │  - OpenAI/Anthropic API                                │  │
│  │  - Local LLM (Ollama)                                  │  │
│  │  - Embeddings generation                               │  │
│  │  - Prompt engineering                                  │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  MCP Client                                            │  │
│  │  - HTTP client for MCP server (JSON-RPC 2.0)           │  │
│  │  - List available tools                                 │  │
│  │  - Call tools via MCP protocol                         │  │
│  │  - Error handling & retries                            │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Vector Database Service                               │  │
│  │  - Store conversation embeddings                       │  │
│  │  - Semantic search over conversations                  │  │
│  │  - Retrieve relevant context                           │  │
│  │  - Similar conversation matching                       │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           │ SQL (SQLx)
                           │
┌──────────────────────────▼────────────────────────────────────┐
│              PostgreSQL with pgvector                          │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Tables                                                │  │
│  │  - conversations (id, session_id, messages, ...)       │  │
│  │  - conversation_embeddings (id, embedding vector)      │  │
│  │  - business_embeddings (business_id, embedding)       │  │
│  │  - service_embeddings (service_id, embedding)         │  │
│  └────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
                           │
                           │
┌──────────────────────────▼────────────────────────────────────┐
                           │ HTTP (REST API)
                           │
┌──────────────────────────▼────────────────────────────────────┐
│      MCP Server (Python Backend) - Port 8002                  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  MCP Tools (JSON-RPC 2.0)                              │  │
│  │  - search_businesses                                   │  │
│  │  - get_business_details                                │  │
│  │  - get_bookings                                        │  │
│  │  - get_services                                        │  │
│  │  - get_employees                                       │  │
│  │  - get_customers                                       │  │
│  └────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Business Logic                                         │  │
│  │  - Business/Salon management                            │  │
│  │  - Booking management                                  │  │
│  │  - Service management                                  │  │
│  │  - Google Places integration                           │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────┬────────────────────────────────────┘
                           │
                           │ (Also available)
                           │
┌──────────────────────────▼────────────────────────────────────┐
│      Python Backend API (EXISTING - beautibuk-back)           │
│  FastAPI REST API - Port 8000                                │
│  (For direct HTTP access if needed)                           │
└────────────────────────────────────────────────────────────────┘
                           │
┌──────────────────────────▼────────────────────────────────────┐
│                    MongoDB Database                            │
│  - Businesses/Salons                                          │
│  - Bookings                                                    │
│  - Services                                                    │
│  - Users                                                       │
└────────────────────────────────────────────────────────────────┘
```

## Technology Stack

### Rust Agent Service (NEW)
- **Web Framework**: Axum (async, high-performance)
- **HTTP Client**: Reqwest (for calling Python backend + LLM APIs)
- **JSON**: Serde (serialization)
- **Async Runtime**: Tokio
- **Error Handling**: anyhow + thiserror
- **Logging**: tracing + tracing-subscriber
- **Database**: PostgreSQL with pgvector extension (vector storage)
- **Database ORM**: SQLx (async PostgreSQL driver)
- **Session Storage**: PostgreSQL (conversation state + embeddings)

### Python Backend (EXISTING)
- **Framework**: FastAPI
- **Database**: MongoDB (via Beanie ODM)
- **Authentication**: JWT tokens
- **Features**: Google Places integration, Stripe subscriptions, Email notifications

### AI/LLM Integration
- **Options**:
  - OpenAI API (GPT-4, GPT-3.5-turbo)
  - Anthropic API (Claude)
  - Local: Ollama with Llama 2/Mistral

### Data Storage
- **Vector Database**: PostgreSQL with pgvector (for embeddings and semantic search)
  - Conversation embeddings (for RAG/context retrieval)
  - Business/service embeddings (for semantic matching)
  - Similar conversation search
- **Conversation State**: PostgreSQL (messages, context, extracted entities)
- **Business Data**: MongoDB (via Python backend APIs)
- **Bookings**: MongoDB (via Python backend APIs)

## Core Components

### 1. Rust Agent Service
- **Agent Orchestrator**: **Simplified** coordination logic (with MCP)
  - Context Management: Maintain conversation context
  - RAG: Retrieve similar conversations for context
  - **LLM with MCP Tools**: LLM autonomously handles intent, entities, and tool selection
  - Response Generation: Generate natural language responses

- **LLM Integration**: 
  - Send user messages to LLM with MCP tools
  - Generate embeddings for messages
  - **Function Calling**: LLM calls MCP tools directly
  - Handle tool call responses and continue conversation
  - Support for Groq and Google AI Studio function calling

- **MCP Client** (replaces Backend API Client):
  - HTTP client for MCP server (JSON-RPC 2.0)
  - Methods: `list_tools()`, `call_tool()`
  - Connects to Python MCP server at `http://localhost:8002/mcp`
  - Error handling and retries

**Note**: With MCP, the agent doesn't need complex intent/entity extraction or manual routing - the LLM handles everything via MCP tools!

- **Vector Database Service**:
  - Store conversation embeddings for RAG
  - Semantic search over conversation history
  - Retrieve similar conversations for context
  - Store business/service embeddings (for semantic business search)
  - **Note**: Embeddings only - full business data stays in MongoDB (Python backend)

- **Conversation State**:
  - Store conversation history in PostgreSQL
  - Track extracted entities (location, date, time, service)
  - Maintain pending booking state
  - Use embeddings for semantic context retrieval

### 2. Python Backend APIs (Existing)
- **Business/Salon APIs**:
  - `GET /api/businesses/search` - Search with filters (location, type, etc.)
  - `GET /api/businesses/{id}` - Get business details
  - `GET /api/businesses/search-autocomplete` - Autocomplete search

- **Service APIs**:
  - `GET /api/services/` - Get services (with business_id filter)
  - `GET /api/services/{id}` - Get service details

- **Booking APIs**:
  - `POST /api/bookings/` - Create booking
  - `GET /api/bookings/` - Get bookings (with filters)
  - `PUT /api/bookings/{id}/confirm` - Confirm booking
  - `PUT /api/bookings/{id}/cancel` - Cancel booking

### 3. API Layer (Rust)
- **Chat Endpoint**: `POST /api/chat` - Main entry point for chat messages
- **Health Check**: `GET /api/health` - Service health status

## Data Flow

### Chat Request Flow (With MCP)

1. User sends message via UI
2. Frontend sends POST to Rust agent `/api/chat`
3. Rust agent receives message, loads conversation context
4. Agent orchestrator processes message:
   - Loads conversation history from PostgreSQL
   - Optional: RAG retrieves similar conversations for context
   - Gets available MCP tools from MCP server
5. Agent sends to LLM with:
   - Conversation history
   - RAG context (if available)
   - Available MCP tools (function definitions)
6. **LLM autonomously decides**:
   - Which tools to call based on user message
   - What parameters to pass
   - Whether to chain multiple tools
7. LLM calls MCP tools via MCP client:
   - Example: Calls `search_businesses({query: "salon", city: "Athens"})`
   - MCP server executes tool and returns results
8. LLM receives tool results and generates response
9. Agent stores conversation and embeddings in PostgreSQL
10. Return natural language response to frontend
11. Frontend displays response

**Key Difference**: LLM handles intent extraction, entity extraction, and tool selection automatically - no manual orchestrator logic needed!

### Booking Flow (With MCP)

1. User: "I need a haircut near downtown tomorrow at 2pm"
2. Agent sends to LLM with MCP tools available
3. **LLM autonomously decides** to call: `search_businesses({query: "haircut", city: "downtown"})`
4. MCP client calls MCP server, receives salon list
5. LLM receives tool result, decides to call: `get_services({business_id: "salon_1", category: "hair"})`
6. LLM receives services, generates response: "I found 3 salons near downtown. Here are your options..."
7. User: "Book the first one for tomorrow at 2pm"
8. **LLM autonomously** uses context from previous tool calls, decides to call: `create_booking({...})`
   - Note: `create_booking` tool needs to be added to MCP server
9. LLM receives booking confirmation, generates: "Great! I've booked your haircut..."
10. Agent stores conversation and returns response to frontend

**Key Difference**: LLM chains multiple tool calls autonomously - no manual orchestration needed!

## Key Design Decisions

1. **Conversation State Management**
   - Store conversation context in memory HashMap (session-based)
   - Session ID from frontend or generate UUID
   - Use Redis for distributed systems (optional)
   - Maintain context window for LLM (last N messages)

2. **Backend API Integration**
   - Use existing Python backend APIs (no direct database access)
   - HTTP client with retry logic
   - Error handling for API failures
   - Type-safe request/response models matching Python schemas

3. **Data Models (Match Python Backend)**
   ```rust
   // Business/Salon (matches Python BusinessResponse)
   struct Business {
       id: String,
       name: String,
       address: String,
       city: String,
       state: String,
       latitude: Option<f64>,
       longitude: Option<f64>,
       business_type: String,
       google_rating: Option<f32>,
       // ... other fields
   }
   
   // Booking (matches Python BookingResponse)
   struct Booking {
       id: String,
       business_id: String,
       service_id: String,
       booking_datetime: DateTime<Utc>,
       status: String, // "pending", "confirmed", "cancelled"
       // ... other fields
   }
   ```

4. **Agent Prompt Structure**
   - System prompt: Define agent role and capabilities
   - Context: Include conversation history
   - Tools: Define available actions (search, book, etc.)
   - Response format: Structured JSON or natural language

## Security Considerations

- Input validation and sanitization
- Rate limiting on API endpoints
- Authentication for booking operations
- Secure storage of API keys (environment variables)
- CORS configuration for frontend access

## Scalability Considerations

- Stateless API design
- Connection pooling for database
- Caching for salon data (Redis)
- Async processing for LLM calls
- Load balancing for multiple instances

## Integration Points

### MCP Server Tools (Primary Integration)

The Rust agent uses **MCP tools** instead of direct HTTP API calls:

1. **`search_businesses`** - Search for businesses
   - Parameters: `query`, `business_type`, `city`, `state`, `limit`
   - Returns: List of businesses with details

2. **`get_business_details`** - Get business information
   - Parameters: `business_id` (required)
   - Returns: Full business details

3. **`get_services`** - Get services
   - Parameters: `business_id` (optional), `limit`
   - Returns: List of services

4. **`get_bookings`** - Get bookings
   - Parameters: `business_id`, `user_id`, `status`, `limit`
   - Returns: List of bookings

5. **`get_customers`** - Get customers for a business
   - Parameters: `business_id` (required), `limit`
   - Returns: List of customers

6. **`get_employees`** - Get employees
   - Parameters: `business_id` (optional), `limit`
   - Returns: List of employees

**Note**: Additional tools like `create_booking` can be added to MCP server as needed.

### Python Backend REST API (Alternative)

The REST API (port 8000) is still available for:
- Direct HTTP access if needed
- Frontend integration
- Other services

But the Rust agent primarily uses MCP (port 8002) for tool access.

### Configuration

- **MCP Server URL**: `MCP_SERVER_URL=http://localhost:8002` (HTTP mode)
- **MCP Transport**: `MCP_TRANSPORT=http` or `stdio`
- **LLM Provider**: `LLM_PROVIDER=groq` or `google`
- **LLM API Key**: `GROQ_API_KEY` or `GOOGLE_AI_API_KEY`
- **Database**: `DATABASE_URL=postgresql://user:pass@localhost:5432/beautibuk_agent`
- **Session Timeout**: Default 30 minutes
- **Embedding Model**: `text-embedding-004` (Google) for vector database

**Note**: Python backend REST API (port 8000) is still available but agent uses MCP (port 8002) for tool access.

## Deployment Strategy

- **Development**: 
  - Local Rust server on port 3000
  - Python backend on port 8000
  - PostgreSQL with pgvector on port 5432
  - Local PostgreSQL instance

- **Production**: 
  - Docker containerization for Rust agent
  - Deploy alongside Python backend
  - PostgreSQL with pgvector (managed service or container)
  - Environment-based configuration
  - Reverse proxy (Nginx) routing:
    - `/api/chat` → Rust agent
    - `/api/businesses/*` → Python backend
    - `/api/bookings/*` → Python backend

