use crate::mcp::models::*;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct McpClient {
    client: Client,
    base_url: String,
    request_id: AtomicU64,
}

impl McpClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            request_id: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    pub async fn initialize(&self) -> Result<()> {
        let response = self
            .send_request(
                "initialize",
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "beautibuk-agent",
                        "version": "1.0.0"
                    }
                }),
            )
            .await?;

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

    pub async fn call_tool(&self, name: &str, arguments: &serde_json::Value) -> Result<String> {
        let response = self
            .send_request(
                "tools/call",
                json!({
                    "name": name,
                    "arguments": arguments
                }),
            )
            .await?;

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

    async fn send_request(&self, method: &str, params: serde_json::Value) -> Result<McpResponse> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id(),
            method: method.to_string(),
            params,
        };

        let response = self
            .client
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
