use crate::mcp::{McpClient, McpTool};
use crate::models::ChatMessage;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Clone)]
pub enum LlmProvider {
    Groq,
    Google,
}

pub struct LlmClient {
    provider: LlmProvider,
    api_key: String,
    model: String,
    client: Client,
    temperature: f32,
    max_tokens: u32,
}

impl LlmClient {
    pub fn new(
        provider: LlmProvider,
        api_key: String,
        model: String,
        temperature: f32,
        max_tokens: u32,
    ) -> Self {
        Self {
            provider,
            api_key,
            model,
            client: Client::new(),
            temperature,
            max_tokens,
        }
    }

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
                self.call_groq_with_functions(messages, &functions, mcp_client)
                    .await
            }
            LlmProvider::Google => {
                self.call_google_with_functions(messages, &functions, mcp_client)
                    .await
            }
        }
    }

    fn convert_mcp_tools_to_functions(&self, tools: &[McpTool]) -> Vec<serde_json::Value> {
        tools
            .iter()
            .map(|tool| {
                json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": tool.input_schema
                    }
                })
            })
            .collect()
    }

    async fn call_groq_with_functions(
        &self,
        messages: &[ChatMessage],
        functions: &[serde_json::Value],
        mcp_client: &McpClient,
    ) -> Result<String> {
        let mut current_messages = messages.to_vec();

        loop {
            let request = json!({
                "model": self.model,
                "messages": current_messages.iter().map(|m| {
                    json!({
                        "role": m.role,
                        "content": m.content
                    })
                }).collect::<Vec<_>>(),
                "tools": functions,
                "tool_choice": "auto",
                "temperature": self.temperature,
                "max_tokens": self.max_tokens,
            });

            let response = self
                .client
                .post("https://api.groq.com/openai/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(anyhow!("Groq API error: {}", error_text));
            }

            #[derive(Deserialize)]
            struct GroqResponse {
                choices: Vec<GroqChoice>,
            }

            #[derive(Deserialize)]
            struct GroqChoice {
                message: GroqMessage,
            }

            #[derive(Deserialize)]
            struct GroqMessage {
                content: Option<String>,
                tool_calls: Option<Vec<ToolCallResponse>>,
            }

            #[derive(Deserialize)]
            struct ToolCallResponse {
                id: String,
                r#type: String,
                function: FunctionCallResponse,
            }

            #[derive(Deserialize)]
            struct FunctionCallResponse {
                name: String,
                arguments: String,
            }

            let result: GroqResponse = response.json().await?;
            let message = &result.choices[0].message;

            // Check if LLM wants to call a tool
            if let Some(tool_calls) = &message.tool_calls {
                if !tool_calls.is_empty() {
                    // Add assistant message with tool calls
                    current_messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: message.content.clone().unwrap_or_default(),
                        tool_calls: Some(
                            tool_calls
                                .iter()
                                .map(|tc| crate::models::ToolCall {
                                    id: tc.id.clone(),
                                    r#type: tc.r#type.clone(),
                                    function: crate::models::FunctionCall {
                                        name: tc.function.name.clone(),
                                        arguments: serde_json::from_str(&tc.function.arguments)
                                            .unwrap_or_default(),
                                    },
                                })
                                .collect(),
                        ),
                    });

                    // Execute each tool call
                    for tool_call in tool_calls {
                        let arguments: serde_json::Value =
                            serde_json::from_str(&tool_call.function.arguments).unwrap_or_default();

                        let tool_result = mcp_client
                            .call_tool(&tool_call.function.name, &arguments)
                            .await?;

                        // Add tool result message
                        current_messages.push(ChatMessage {
                            role: "tool".to_string(),
                            content: tool_result,
                            tool_calls: None,
                        });
                    }
                    // Continue loop to process tool results
                    continue;
                }
            }

            // No tool calls, return the response
            return Ok(message.content.clone().unwrap_or_default());
        }
    }

    async fn call_google_with_functions(
        &self,
        messages: &[ChatMessage],
        functions: &[serde_json::Value],
        mcp_client: &McpClient,
    ) -> Result<String> {
        // Convert messages to Gemini format
        let mut contents: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                let role = match m.role.as_str() {
                    "user" => "user",
                    "assistant" => "model",
                    "tool" => "function",
                    _ => "user",
                };
                json!({
                    "role": role,
                    "parts": [{"text": m.content}]
                })
            })
            .collect();

        // Convert functions to Gemini format
        let function_declarations: Vec<serde_json::Value> = functions
            .iter()
            .map(|f| {
                let func = &f["function"];
                json!({
                    "name": func["name"],
                    "description": func["description"],
                    "parameters": func["parameters"]
                })
            })
            .collect();

        loop {
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

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(anyhow!("Google API error: {}", error_text));
            }

            #[derive(Deserialize)]
            struct GeminiResponse {
                candidates: Vec<GeminiCandidate>,
            }

            #[derive(Deserialize)]
            struct GeminiCandidate {
                content: GeminiContent,
            }

            #[derive(Deserialize)]
            struct GeminiContent {
                parts: Vec<serde_json::Value>,
            }

            let result: GeminiResponse = response.json().await?;

            // Check for function calls
            if let Some(candidate) = result.candidates.first() {
                let mut found_function_call = false;

                for part in &candidate.content.parts {
                    if let Some(function_call) = part.get("functionCall") {
                        found_function_call = true;
                        let func_name = function_call["name"].as_str().unwrap();
                        let func_args = &function_call["args"];

                        let tool_result = mcp_client.call_tool(func_name, func_args).await?;

                        // Add model response with function call
                        contents.push(json!({
                            "role": "model",
                            "parts": [{"functionCall": function_call}]
                        }));

                        // Add function response
                        contents.push(json!({
                            "role": "function",
                            "parts": [{
                                "functionResponse": {
                                    "name": func_name,
                                    "response": json!({"result": tool_result})
                                }
                            }]
                        }));

                        // Continue loop to process function result
                        break;
                    }
                }

                if !found_function_call {
                    // Return text response
                    if let Some(part) = candidate.content.parts.first() {
                        if let Some(text) = part.get("text") {
                            return Ok(text.as_str().unwrap().to_string());
                        }
                    }
                }
            } else {
                return Err(anyhow!("No candidates in Gemini response"));
            }
        }
    }

    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        match self.provider {
            LlmProvider::Google => self.generate_google_embedding(text).await,
            LlmProvider::Groq => Err(anyhow!(
                "Groq does not support embeddings. Use Google AI Studio for embeddings."
            )),
        }
    }

    async fn generate_google_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = json!({
            "model": "text-embedding-004",
            "content": {
                "parts": [{"text": text}]
            }
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/text-embedding-004:embedContent?key={}",
            self.api_key
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Google Embeddings API error: {}", error_text));
        }

        #[derive(Deserialize)]
        struct EmbeddingResponse {
            embedding: EmbeddingData,
        }

        #[derive(Deserialize)]
        struct EmbeddingData {
            values: Vec<f32>,
        }

        let result: EmbeddingResponse = response.json().await?;
        Ok(result.embedding.values)
    }
}
