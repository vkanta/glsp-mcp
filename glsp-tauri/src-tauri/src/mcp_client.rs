use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<McpError>,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolCall {
    pub name: String,
    pub arguments: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolResult {
    pub content: Vec<McpContent>,
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

/// Simple MCP client for communicating with the embedded GLSP server
#[derive(Debug)]
pub struct McpClient {
    base_url: std::sync::Arc<std::sync::Mutex<String>>,
    client: reqwest::Client,
    next_id: std::sync::atomic::AtomicU64,
    session_id: std::sync::Mutex<Option<String>>,
}

impl McpClient {
    pub fn new(server_port: u16) -> Self {
        Self {
            base_url: std::sync::Arc::new(std::sync::Mutex::new(format!(
                "http://localhost:{}/messages",
                server_port
            ))),
            client: reqwest::Client::new(),
            next_id: std::sync::atomic::AtomicU64::new(1),
            session_id: std::sync::Mutex::new(None),
        }
    }

    /// Update the server port if it changes
    pub fn update_port(&self, new_port: u16) {
        let mut url = self.base_url.lock().unwrap();
        *url = format!("http://localhost:{}/messages", new_port);
    }

    fn next_request_id(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// Initialize MCP session and get session ID
    pub async fn initialize(&self) -> Result<(), String> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "GLSP Tauri Client",
                    "version": "1.0.0"
                }
            }),
            id: self.next_request_id(),
        };

        let base_url = self.base_url.lock().unwrap().clone();
        let req_builder = self
            .client
            .post(&base_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&request);

        let response = req_builder
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let mcp_response: McpResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(error) = mcp_response.error {
            return Err(format!(
                "MCP error: {} (code: {})",
                error.message, error.code
            ));
        }

        // Extract session ID from response if available
        if let Some(result) = mcp_response.result {
            if let Some(session_id) = result.get("sessionId").and_then(|v| v.as_str()) {
                if let Ok(mut session_guard) = self.session_id.lock() {
                    *session_guard = Some(session_id.to_string());
                }
            }
        }

        Ok(())
    }

    /// Call an MCP tool on the server
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Option<Value>,
    ) -> Result<McpToolResult, String> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": tool_name,
                "arguments": arguments.unwrap_or(Value::Object(serde_json::Map::new()))
            }),
            id: self.next_request_id(),
        };

        debug!("Sending MCP request: {:?}", request);

        let base_url = self.base_url.lock().unwrap().clone();
        let mut req_builder = self
            .client
            .post(&base_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&request);

        // Add session ID header if available
        if let Ok(session_guard) = self.session_id.lock() {
            if let Some(ref session_id) = *session_guard {
                req_builder = req_builder.header("Mcp-Session-Id", session_id);
            }
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let mcp_response: McpResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        debug!("Received MCP response: {:?}", mcp_response);

        if let Some(error) = mcp_response.error {
            return Err(format!(
                "MCP error: {} (code: {})",
                error.message, error.code
            ));
        }

        let result = mcp_response.result.ok_or("No result in response")?;

        serde_json::from_value(result).map_err(|e| format!("Failed to parse tool result: {}", e))
    }

    /// Set workspace directory using MCP tool
    pub async fn set_workspace_directory(
        &self,
        workspace_path: &str,
        create_if_missing: bool,
    ) -> Result<String, String> {
        let result = self
            .call_tool(
                "set_workspace_directory",
                Some(serde_json::json!({
                    "workspace_path": workspace_path,
                    "create_if_missing": create_if_missing
                })),
            )
            .await?;

        if result.is_error.unwrap_or(false) {
            let error_msg = result
                .content
                .get(0)
                .and_then(|c| c.text.as_ref())
                .map_or("Unknown error", |s| s);
            return Err(error_msg.to_string());
        }

        let success_msg = result
            .content
            .get(0)
            .and_then(|c| c.text.as_ref())
            .map_or("Workspace directory set successfully", |s| s);

        Ok(success_msg.to_string())
    }

    /// Get current workspace information using MCP tool
    pub async fn get_workspace_info(&self) -> Result<Value, String> {
        let result = self.call_tool("get_workspace_info", None).await?;

        if result.is_error.unwrap_or(false) {
            let error_msg = result
                .content
                .get(0)
                .and_then(|c| c.text.as_ref())
                .map_or("Failed to get workspace info", |s| s);
            return Err(error_msg.to_string());
        }

        let workspace_text = result
            .content
            .get(0)
            .and_then(|c| c.text.as_ref())
            .ok_or("No workspace info in response")?;

        serde_json::from_str(workspace_text)
            .map_err(|e| format!("Failed to parse workspace info: {}", e))
    }

    /// Set WASM components path using MCP tool
    pub async fn set_wasm_components_path(&self, wasm_path: &str) -> Result<String, String> {
        let result = self
            .call_tool(
                "set_wasm_components_path",
                Some(serde_json::json!({
                    "wasm_path": wasm_path
                })),
            )
            .await?;

        if result.is_error.unwrap_or(false) {
            let error_msg = result
                .content
                .get(0)
                .and_then(|c| c.text.as_ref())
                .map_or("Unknown error", |s| s);
            return Err(error_msg.to_string());
        }

        let success_msg = result
            .content
            .get(0)
            .and_then(|c| c.text.as_ref())
            .map_or("WASM components path set successfully", |s| s);

        Ok(success_msg.to_string())
    }

    /// Set diagrams path using MCP tool
    pub async fn set_diagrams_path(&self, diagrams_path: &str) -> Result<String, String> {
        let result = self
            .call_tool(
                "set_diagrams_path",
                Some(serde_json::json!({
                    "diagrams_path": diagrams_path
                })),
            )
            .await?;

        if result.is_error.unwrap_or(false) {
            let error_msg = result
                .content
                .get(0)
                .and_then(|c| c.text.as_ref())
                .map_or("Unknown error", |s| s);
            return Err(error_msg.to_string());
        }

        let success_msg = result
            .content
            .get(0)
            .and_then(|c| c.text.as_ref())
            .map_or("Diagrams path set successfully", |s| s);

        Ok(success_msg.to_string())
    }

    /// Rescan workspace using MCP tool
    pub async fn rescan_workspace(&self) -> Result<String, String> {
        let result = self.call_tool("rescan_workspace", None).await?;

        if result.is_error.unwrap_or(false) {
            let error_msg = result
                .content
                .get(0)
                .and_then(|c| c.text.as_ref())
                .map_or("Unknown error", |s| s);
            return Err(error_msg.to_string());
        }

        let success_msg = result
            .content
            .get(0)
            .and_then(|c| c.text.as_ref())
            .map_or("Workspace rescanned successfully", |s| s);

        Ok(success_msg.to_string())
    }

    /// Validate workspace structure using MCP tool
    pub async fn validate_workspace(&self, workspace_path: &str) -> Result<Value, String> {
        let result = self
            .call_tool(
                "validate_workspace",
                Some(serde_json::json!({
                    "workspace_path": workspace_path
                })),
            )
            .await?;

        if result.is_error.unwrap_or(false) {
            let error_msg = result
                .content
                .get(0)
                .and_then(|c| c.text.as_ref())
                .map_or("Workspace validation failed", |s| s);
            return Err(error_msg.to_string());
        }

        let validation_text = result
            .content
            .get(0)
            .and_then(|c| c.text.as_ref())
            .ok_or("No validation result in response")?;

        serde_json::from_str(validation_text)
            .map_err(|e| format!("Failed to parse validation result: {}", e))
    }

    /// Create workspace structure using MCP tool
    pub async fn create_workspace_structure(&self, workspace_path: &str) -> Result<String, String> {
        let result = self
            .call_tool(
                "create_workspace_structure",
                Some(serde_json::json!({
                    "workspace_path": workspace_path
                })),
            )
            .await?;

        if result.is_error.unwrap_or(false) {
            let error_msg = result
                .content
                .get(0)
                .and_then(|c| c.text.as_ref())
                .map_or("Unknown error", |s| s);
            return Err(error_msg.to_string());
        }

        let success_msg = result
            .content
            .get(0)
            .and_then(|c| c.text.as_ref())
            .map_or("Workspace structure created successfully", |s| s);

        Ok(success_msg.to_string())
    }

    /// Check if the MCP server is healthy
    pub async fn health_check(&self) -> Result<bool, String> {
        let base_url = self.base_url.lock().unwrap().clone();
        // Extract port from base URL
        let port = base_url
            .split(':')
            .nth(2)
            .and_then(|s| s.split('/').next())
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(3000);

        let health_url = format!("http://localhost:{}/health", port);

        match self.client.get(&health_url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
