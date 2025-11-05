# Communication Flow Diagrams (With MCP)

## Chat Request Flow (With MCP)

```
User                    Frontend              Rust Agent          LLM (with MCP)      MCP Server
 │                        │                      │                      │                  │
 │  Types message         │                      │                      │                  │
 │───────────────────────>│                      │                      │                  │
 │                        │                      │                      │                  │
 │                        │  POST /api/chat      │                      │                  │
 │                        │  {message: "..."}    │                      │                  │
 │                        │─────────────────────>│                      │                  │
 │                        │                      │                      │                  │
 │                        │                      │  Load conversation context              │
 │                        │                      │───────────────────────>│                │
 │                        │                      │                      │                  │
 │                        │                      │  Optional: RAG context                  │
 │                        │                      │───────────────────────>│                │
 │                        │                      │                      │                  │
 │                        │                      │  Get MCP tools list                     │
 │                        │                      │────────────────────────────────────────>│
 │                        │                      │                      │                  │
 │                        │                      │  List of tools        │                  │
 │                        │                      │<────────────────────────────────────────│
 │                        │                      │                      │                  │
 │                        │                      │  Send message + tools + context         │
 │                        │                      │───────────────────────>│                │
 │                        │                      │                      │                  │
 │                        │                      │                      │  LLM decides to call tool
 │                        │                      │                      │─────────────────>│
 │                        │                      │                      │                  │
 │                        │                      │                      │  Tool result     │
 │                        │                      │                      │<─────────────────│
 │                        │                      │                      │                  │
 │                        │                      │  Generate response    │                  │
 │                        │                      │<───────────────────────│                │
 │                        │                      │                      │                  │
 │                        │  Response JSON       │                      │                  │
 │                        │<─────────────────────│                      │                  │
 │                        │                      │                      │                  │
 │  Display message       │                      │                      │                  │
 │<───────────────────────│                      │                      │                  │
```

**Key Difference**: LLM autonomously handles intent extraction, entity extraction, and tool selection - no manual orchestrator logic needed!

## Booking Flow Example (With MCP)

### Step 1: User Initiates Booking

```
User: "I need a haircut near downtown tomorrow at 2pm"
  ↓
Frontend sends to /api/chat
  ↓
Agent loads conversation context
  ↓
Agent gets MCP tools from MCP server
  ↓
Agent sends to LLM with MCP tools available
  ↓
LLM autonomously decides to call:
  - search_businesses({query: "haircut", city: "downtown"})
  ↓
MCP server executes search, returns results
  ↓
LLM receives tool results, decides to call:
  - get_services({business_id: "salon_1"})
  ↓
MCP server returns services
  ↓
LLM generates response: "I found 3 salons near downtown. Here are your options..."
```

### Step 2: User Selects Salon

```
User: "I'll take the first one"
  ↓
Agent sends to LLM with conversation context
  ↓
LLM uses context from previous tool calls
  ↓
LLM autonomously decides to call:
  - create_booking({business_id: "salon_1", ...})
  ↓
MCP server creates booking
  ↓
LLM receives confirmation, generates response
  ↓
Agent returns: "Great! I've booked your haircut at [Salon Name] for tomorrow at 2pm. 
                 Your confirmation number is #12345"
```

## Component Interaction Details (With MCP)

### Agent Orchestrator Processing (SIMPLIFIED)

```
Message Received
    │
    ├─> Load Conversation Context
    │   └─> From PostgreSQL
    │
    ├─> Optional: RAG Context Enhancement
    │   ├─> Generate embedding
    │   ├─> Retrieve similar conversations
    │   └─> Build context
    │
    ├─> Get MCP Tools
    │   └─> Call MCP server: tools/list
    │
    ├─> Send to LLM with:
    │   ├─> Conversation history
    │   ├─> RAG context (if available)
    │   └─> Available MCP tools (function definitions)
    │
    ├─> LLM Autonomously:
    │   ├─> Understands intent (no manual extraction)
    │   ├─> Extracts entities (from tool schemas)
    │   ├─> Decides which tools to call
    │   ├─> Calls MCP tools as needed
    │   └─> Generates response
    │
    ├─> Store Conversation
    │   └─> Save to PostgreSQL with embedding
    │
    └─> Return Response to User
```

**Key Simplification**: No manual intent extraction, no entity extraction, no routing logic - LLM handles everything via MCP tools!

### Data Flow for Business Search (With MCP)

```
1. User: "Find me a salon for a haircut"
   │
2. Agent loads conversation context
   │
3. Agent gets MCP tools: [search_businesses, get_business_details, ...]
   │
4. Agent sends to LLM with tools
   │
5. LLM autonomously decides:
   │   └─> Call search_businesses({query: "salon", service_type: "haircut"})
   │
6. MCP Client calls MCP Server:
   │   └─> tools/call with search_businesses
   │
7. MCP Server executes search, returns results
   │
8. LLM receives tool results, generates response:
   │   "I found 5 salons near you:
   │    1. Hair Studio (0.5 miles, 4.8★)
   │    2. Style Salon (0.8 miles, 4.6★)
   │    ..."
   │
9. Agent stores conversation and embedding
   │
10. Response sent to frontend
```

### Booking Creation Flow (With MCP)

```
1. User: "Book the first one for tomorrow at 2pm"
   │
2. Agent loads conversation context (includes previous search results)
   │
3. Agent sends to LLM with MCP tools + context
   │
4. LLM autonomously:
   │   ├─> Uses context to identify "first one" (salon_1 from previous results)
   │   ├─> Parses "tomorrow at 2pm" → DateTime
   │   └─> Decides to call: create_booking({...})
   │
5. MCP Client calls MCP Server:
   │   └─> tools/call with create_booking
   │
6. MCP Server validates and creates booking
   │
7. LLM receives confirmation, generates response:
   │   "Booking confirmed! ..."
   │
8. Agent stores conversation
   │
9. Response sent to frontend
```

## Context Management (With MCP)

### Conversation Context Structure

```rust
struct ConversationContext {
    session_id: String,
    messages: Vec<ChatMessage>,  // Full conversation history
    // No need for extracted_entities or current_intent
    // LLM handles these via MCP tools
}
```

### Context Flow (With MCP)

```
Message 1: "Find salons near me"
  │
  ├─> Context: messages=[user: "Find salons near me"]
  ├─> LLM calls: search_businesses({...})
  ├─> Store search results in conversation context
  └─> Response: List of salons

Message 2: "Book the first one"
  │
  ├─> Context: messages=[...previous message..., LLM tool calls, results]
  ├─> LLM uses context to identify "first one"
  ├─> LLM calls: create_booking({...})
  └─> Response: Booking confirmation

Message 3: "Cancel my booking"
  │
  ├─> Context: messages=[...full conversation...]
  ├─> LLM calls: get_bookings({user_id: ...}) if needed
  ├─> LLM calls: cancel_booking({...})
  └─> Response: Cancellation confirmation
```

**Key Difference**: Context is managed through conversation history, not manual entity extraction. LLM understands context from the full conversation.

## Error Handling Flow (With MCP)

```
Request
  │
  ├─> Validation Error
  │   └─> Return: "I didn't understand. Could you clarify..."
  │
  ├─> MCP Connection Error
  │   ├─> Retry MCP connection
  │   └─> Fallback: "I'm having trouble connecting. Please try again."
  │
  ├─> LLM API Error
  │   ├─> Retry (with backoff)
  │   └─> Fallback: "I'm having trouble processing that. Please try again."
  │
  ├─> MCP Tool Error (No businesses found)
  │   └─> LLM generates: "I couldn't find any salons matching your criteria. 
  │                       Would you like to try a different area?"
  │
  ├─> MCP Tool Error (Booking unavailable)
  │   └─> LLM generates: "That time slot is unavailable. Here are available times..."
  │
  └─> Success
      └─> Return agent response
```

## Comparison: REST API vs MCP

### REST API Approach (Old/Alternative)
- Complex orchestrator with intent extraction
- Manual entity extraction
- Hardcoded routing logic
- Multiple LLM calls per message
- ~100+ lines of orchestrator code

### MCP Approach (Current/Recommended)
- Simple orchestrator (just pass to LLM)
- LLM handles intent/entity extraction
- LLM autonomously selects tools
- Single LLM call with tool calls
- ~20 lines of orchestrator code

**Result**: 70% less code, simpler architecture, easier maintenance!

## WebSocket vs REST Comparison

### REST API (Recommended for MVP)
- Simple HTTP POST for each message
- Stateless, easier to implement
- Frontend polls or uses long polling
- Good for initial implementation

### WebSocket (Advanced)
- Persistent connection
- Real-time bidirectional communication
- Better for streaming responses
- More complex to implement

**Recommendation**: Start with REST API, upgrade to WebSocket later if needed.
