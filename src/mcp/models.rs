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
    #[allow(dead_code)]
    pub jsonrpc: String,
    #[allow(dead_code)]
    pub id: u64,
    pub result: Option<McpResult>,
    pub error: Option<McpError>,
}

#[derive(Debug, Deserialize)]
pub struct McpResult {
    pub tools: Option<Vec<McpTool>>,
    pub content: Option<Vec<McpContent>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct McpError {
    #[allow(dead_code)]
    pub code: i32,
    pub message: String,
    #[allow(dead_code)]
    pub data: Option<serde_json::Value>,
}
