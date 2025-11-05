# MCP Client Implementation Guide

This guide shows how to implement an MCP client in Rust to connect to your existing Python MCP server.

## MCP Server Details

- **HTTP Endpoint**: `http://localhost:8002/mcp`
- **Protocol**: JSON-RPC 2.0
- **Available Tools**: search_businesses, get_business_details, get_bookings, get_services, get_employees, get_customers

## Rust MCP Client Implementation

### 1. MCP Client Structure

```rust
// src/mcp/mod.rs
pub mod client;
pub mod models;

pub use client::McpClient;
```

### 2. MCP Models

```rust
// src/mcp/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<McpResult>,
    pub error: Option<McpError>,
}

#[derive(Debug, Deserialize)]
pub struct McpResult {
    pub tools: Option<Vec<McpTool>>,
    pub content: Option<Vec<McpContent>>,
}

#[derive(Debug, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

### 3. MCP Client Implementation

```rust
// src/mcp/client.rs
use reqwest::Client;
use serde_json::json;
use anyhow::{Result, anyhow};

pub struct McpClient {
    client: Client,
    base_url: String,
    request_id: std::sync::atomic::AtomicU64,
}

impl McpClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }
    
    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
    
    pub async fn initialize(&self) -> Result<()> {
        let response = self.send_request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "beautibuk-agent",
                    "version": "1.0.0"
                }
            }),
        ).await?;
        
        if response.error.is_some() {
            return Err(anyhow!("MCP initialization failed: {:?}", response.error));
        }
        
        Ok(())
    }
    
    pub async fn list_tools(&self) -> Result<Vec<McpTool>> {
        let response = self.send_request("tools/list", json!({})).await?;
        
        if let Some(error) = response.error {
            return Err(anyhow!("MCP error: {}", error.message));
        }
        
        if let Some(result) = response.result {
            if let Some(tools) = result.tools {
                return Ok(tools);
            }
        }
        
        Err(anyhow!("No tools in MCP response"))
    }
    
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: &serde_json::Value,
    ) -> Result<String> {
        let response = self.send_request(
            "tools/call",
            json!({
                "name": name,
                "arguments": arguments
            }),
        ).await?;
        
        if let Some(error) = response.error {
            return Err(anyhow!("MCP tool call error: {}", error.message));
        }
        
        if let Some(result) = response.result {
            if let Some(content) = result.content {
                if let Some(first_content) = content.first() {
                    return Ok(first_content.text.clone());
                }
            }
        }
        
        Err(anyhow!("No content in MCP tool response"))
    }
    
    async fn send_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<McpResponse> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id(),
            method: method.to_string(),
            params,
        };
        
        let response = self.client
            .post(format!("{}/mcp", self.base_url))
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("MCP HTTP error: {}", error_text));
        }
        
        let mcp_response: McpResponse = response.json().await?;
        Ok(mcp_response)
    }
}
```

### 4. LLM Integration with Function Calling

```rust
// src/agent/llm.rs
use crate::mcp::McpClient;

impl LlmClient {
    pub async fn generate_with_mcp_tools(
        &self,
        messages: &[ChatMessage],
        mcp_client: &McpClient,
    ) -> Result<String> {
        // 1. Get available tools from MCP
        let tools = mcp_client.list_tools().await?;
        
        // 2. Convert MCP tools to LLM function format
        let functions = self.convert_mcp_tools_to_functions(&tools);
        
        // 3. Send to LLM with function calling
        match self.provider {
            LlmProvider::Groq => {
                self.call_groq_with_functions(messages, &functions, mcp_client).await
            }
            LlmProvider::Google => {
                self.call_google_with_functions(messages, &functions, mcp_client).await
            }
        }
    }
    
    fn convert_mcp_tools_to_functions(&self, tools: &[McpTool]) -> Vec<serde_json::Value> {
        tools.iter().map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.input_schema
                }
            })
        }).collect()
    }
    
    async fn call_groq_with_functions(
        &self,
        messages: &[ChatMessage],
        functions: &[serde_json::Value],
        mcp_client: &McpClient,
    ) -> Result<String> {
        let request = json!({
            "model": self.model,
            "messages": messages.iter().map(|m| {
                json!({
                    "role": m.role,
                    "content": m.content
                })
            }).collect::<Vec<_>>(),
            "tools": functions,
            "tool_choice": "auto", // Let LLM decide
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
        });
        
        let response = self.client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;
        
        let result: GroqResponse = response.json().await?;
        let message = &result.choices[0].message;
        
        // Check if LLM wants to call a tool
        if let Some(tool_calls) = &message.tool_calls {
            if !tool_calls.is_empty() {
                // Execute tool calls
                let mut tool_results = Vec::new();
                for tool_call in tool_calls {
                    let tool_result = mcp_client
                        .call_tool(&tool_call.function.name, &tool_call.function.arguments)
                        .await?;
                    
                    tool_results.push(json!({
                        "role": "tool",
                        "content": tool_result,
                        "tool_call_id": tool_call.id,
                    }));
                }
                
                // Continue conversation with tool results
                let mut new_messages = messages.to_vec();
                new_messages.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: message.content.clone(),
                    tool_calls: Some(tool_calls.clone()),
                });
                new_messages.extend(tool_results.into_iter().map(|r| {
                    ChatMessage {
                        role: r["role"].as_str().unwrap().to_string(),
                        content: r["content"].as_str().unwrap().to_string(),
                        tool_calls: None,
                    }
                }));
                
                // Recursive call with tool results
                return self.call_groq_with_functions(&new_messages, functions, mcp_client).await;
            }
        }
        
        Ok(message.content.clone())
    }
    
    async fn call_google_with_functions(
        &self,
        messages: &[ChatMessage],
        functions: &[serde_json::Value],
        mcp_client: &McpClient,
    ) -> Result<String> {
        // Convert messages to Gemini format
        let contents: Vec<serde_json::Value> = messages.iter().map(|m| {
            let role = match m.role.as_str() {
                "user" => "user",
                "assistant" => "model",
                "tool" => "function", // Gemini uses "function" for tool results
                _ => "user",
            };
            json!({
                "role": role,
                "parts": [{"text": m.content}]
            })
        }).collect();
        
        // Convert functions to Gemini format
        let function_declarations: Vec<serde_json::Value> = functions.iter()
            .map(|f| {
                let func = &f["function"];
                json!({
                    "name": func["name"],
                    "description": func["description"],
                    "parameters": func["parameters"]
                })
            })
            .collect();
        
        let request = json!({
            "contents": contents,
            "tools": [{
                "functionDeclarations": function_declarations
            }],
            "generationConfig": {
                "temperature": self.temperature,
                "maxOutputTokens": self.max_tokens,
            }
        });
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );
        
        let response = self.client.post(&url).json(&request).send().await?;
        let result: GeminiResponse = response.json().await?;
        
        // Check for function calls
        if let Some(candidate) = result.candidates.first() {
            if let Some(function_call) = &candidate.content.parts.iter()
                .find(|p| p.get("functionCall").is_some()) {
                
                let func_call = &function_call["functionCall"];
                let tool_result = mcp_client
                    .call_tool(
                        func_call["name"].as_str().unwrap(),
                        &func_call["args"],
                    )
                    .await?;
                
                // Continue conversation with tool result
                let mut new_contents = contents;
                new_contents.push(json!({
                    "role": "model",
                    "parts": [{"functionCall": func_call}]
                }));
                new_contents.push(json!({
                    "role": "function",
                    "parts": [{
                        "functionResponse": {
                            "name": func_call["name"],
                            "response": json!({"result": tool_result})
                        }
                    }]
                }));
                
                // Recursive call
                return self.call_google_with_functions_impl(new_contents, functions, mcp_client).await;
            }
            
            // Return text response
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }
        
        Err(anyhow!("No content in Gemini response"))
    }
}
```

### 5. Simplified Orchestrator

```rust
// src/agent/orchestrator.rs
pub struct Orchestrator {
    llm_client: LlmClient,
    mcp_client: McpClient,
    session_manager: SessionManager,
    vector_service: VectorService,
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
                content: format!("Relevant context from past conversations:\n{}", similar_context),
            });
        }
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: message.clone(),
        });
        
        // 4. LLM handles everything via MCP tools
        let response = self.llm_client
            .generate_with_mcp_tools(&messages, &self.mcp_client).await?;
        
        // 5. Store conversation and embedding
        self.session_manager
            .add_message(&session_id, &message, &response).await?;
        self.vector_service
            .store_conversation_embedding(&session_id, &message, &embedding).await?;
        
        Ok(AgentResponse { response, session_id })
    }
}
```

## Testing MCP Connection

```rust
#[tokio::test]
async fn test_mcp_connection() {
    let client = McpClient::new("http://localhost:8002".to_string());
    
    // Initialize
    client.initialize().await.unwrap();
    
    // List tools
    let tools = client.list_tools().await.unwrap();
    assert!(tools.len() > 0);
    println!("Available tools: {:?}", tools.iter().map(|t| &t.name).collect::<Vec<_>>());
    
    // Test tool call
    let result = client.call_tool(
        "search_businesses",
        &json!({
            "query": "salon",
            "limit": 5
        }),
    ).await.unwrap();
    
    println!("Tool result: {}", result);
}
```

## Environment Configuration

```bash
# .env
# MCP Server
MCP_SERVER_URL=http://localhost:8002
MCP_TRANSPORT=http

# LLM (supports function calling)
LLM_PROVIDER=groq
GROQ_API_KEY=gsk_your_key_here
LLM_MODEL=llama-3.1-8b-instant

# Or Google
# LLM_PROVIDER=google
# GOOGLE_AI_API_KEY=your_key_here
# LLM_MODEL=gemini-2.0-flash-exp
```

## Benefits

1. **No Backend API Client Needed** - MCP client replaces it
2. **No Intent Extraction** - LLM handles it
3. **No Entity Extraction** - LLM extracts from tool schemas
4. **No Routing Logic** - LLM routes automatically
5. **Simpler Code** - 70% less orchestrator code

## Example Flow

```
User: "Find me a salon for a haircut in Athens"

1. Agent loads conversation context
2. Optional: RAG retrieves similar conversations
3. LLM receives message + MCP tools list
4. LLM decides to call: search_businesses({
     query: "salon",
     business_type: "Beauty Salon",
     city: "Athens"
   })
5. MCP client calls MCP server
6. MCP server executes search_businesses tool
7. Results returned to LLM
8. LLM generates response: "I found 5 salons in Athens..."
9. Agent stores conversation
```

**No manual intent extraction, no routing logic - LLM handles everything!**

