//! GLSP Backend implementation for MCP framework
//!
//! This module implements the McpBackend trait for our GLSP diagram server,
//! providing tools, resources, and prompts for diagram manipulation and WASM component analysis.

use async_trait::async_trait;
use mcp_server::{McpBackend, BackendError};
use mcp_protocol::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

use crate::mcp::tools::DiagramTools;
use crate::mcp::resources::DiagramResources;
use crate::mcp::prompts::DiagramPrompts;

/// Configuration for the GLSP backend
#[derive(Debug, Clone)]
pub struct GlspConfig {
    /// Path to WASM components directory
    pub wasm_path: String,
    /// Server name and version information
    pub server_name: String,
    pub server_version: String,
}

impl Default for GlspConfig {
    fn default() -> Self {
        Self {
            wasm_path: "../workspace/adas-wasm-components".to_string(),
            server_name: "GLSP MCP Server".to_string(),
            server_version: "0.1.0".to_string(),
        }
    }
}

/// Error type for GLSP backend operations
#[derive(Debug, thiserror::Error)]
pub enum GlspError {
    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),
    
    #[error("Tool execution failed: {0}")]
    ToolExecution(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Prompt not found: {0}")]
    PromptNotFound(String),
    
    #[error("WASM processing error: {0}")]
    WasmProcessing(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl From<GlspError> for Error {
    fn from(err: GlspError) -> Self {
        match err {
            GlspError::Backend(be) => be.into(),
            GlspError::ToolExecution(msg) => Error::internal_error(format!("Tool execution failed: {}", msg)),
            GlspError::ResourceNotFound(msg) => Error::invalid_params(format!("Resource not found: {}", msg)),
            GlspError::PromptNotFound(msg) => Error::invalid_params(format!("Prompt not found: {}", msg)),
            GlspError::WasmProcessing(msg) => Error::internal_error(format!("WASM processing error: {}", msg)),
            GlspError::Serialization(err) => Error::internal_error(format!("Serialization error: {}", err)),
        }
    }
}

impl From<BackendError> for GlspError {
    fn from(err: BackendError) -> Self {
        Self::Backend(err)
    }
}

/// GLSP Backend implementation
#[derive(Clone)]
pub struct GlspBackend {
    config: GlspConfig,
    tools: Arc<Mutex<DiagramTools>>,
    resources: Arc<Mutex<DiagramResources>>,
    prompts: Arc<Mutex<DiagramPrompts>>,
}

#[async_trait]
impl McpBackend for GlspBackend {
    type Error = GlspError;
    type Config = GlspConfig;

    async fn initialize(config: Self::Config) -> std::result::Result<Self, Self::Error> {
        info!("Initializing GLSP backend with config: {:?}", config);
        
        let tools = Arc::new(Mutex::new(DiagramTools::new()));
        let resources = Arc::new(Mutex::new(DiagramResources::new()));
        let prompts = Arc::new(Mutex::new(DiagramPrompts::new()));
        
        Ok(Self {
            config,
            tools,
            resources,
            prompts,
        })
    }

    fn get_server_info(&self) -> ServerInfo {
        ServerInfo {
            name: self.config.server_name.clone(),
            version: self.config.server_version.clone(),
        }
    }

    async fn health_check(&self) -> std::result::Result<(), Self::Error> {
        // Check if WASM components directory exists
        if !std::path::Path::new(&self.config.wasm_path).exists() {
            return Err(GlspError::WasmProcessing(
                format!("WASM components directory not found: {}", self.config.wasm_path)
            ));
        }
        
        info!("GLSP backend health check passed");
        Ok(())
    }

    async fn list_tools(&self, request: PaginatedRequestParam) -> std::result::Result<ListToolsResult, Self::Error> {
        let tools = self.tools.lock().await;
        let available_tools = tools.get_available_tools();
        
        // Convert to MCP protocol format
        let mcp_tools: Vec<Tool> = available_tools.into_iter().map(|tool| {
            Tool {
                name: tool.name,
                description: tool.description,
                input_schema: tool.input_schema,
            }
        }).collect();
        
        // Handle pagination - cursor is a string that we parse to usize
        let cursor = match request.cursor {
            Some(c) => c.parse::<usize>().unwrap_or(0),
            None => 0,
        };
        
        let page_size = 50; // Default page size
        let end = std::cmp::min(cursor + page_size, mcp_tools.len());
        let page_tools = mcp_tools[cursor..end].to_vec();
        
        let next_cursor = if end < mcp_tools.len() {
            Some(end.to_string())
        } else {
            None
        };
        
        Ok(ListToolsResult {
            tools: page_tools,
            next_cursor,
        })
    }

    async fn call_tool(&self, request: CallToolRequestParam) -> Result<CallToolResult, Self::Error> {
        let mut tools = self.tools.lock().await;
        
        let call_params = crate::mcp::protocol::CallToolParams {
            name: request.name,
            arguments: request.arguments,
        };
        
        match tools.call_tool(call_params).await {
            Ok(result) => Ok(CallToolResult {
                content: result.content,
                is_error: result.is_error,
            }),
            Err(e) => {
                error!("Tool call failed: {}", e);
                Err(GlspError::ToolExecution(e.to_string()))
            }
        }
    }

    async fn list_resources(&self, request: PaginatedRequestParam) -> Result<ListResourcesResult, Self::Error> {
        let resources = self.resources.lock().await;
        let available_resources = resources.get_available_resources(&*self.tools.lock().await);
        
        // Convert to MCP protocol format
        let mcp_resources: Vec<Resource> = available_resources.into_iter().map(|resource| {
            Resource {
                uri: resource.uri,
                name: resource.name,
                description: resource.description,
                mime_type: resource.mime_type,
            }
        }).collect();
        
        // Handle pagination
        let cursor = request.cursor.unwrap_or(0);
        let page_size = 50;
        let end = std::cmp::min(cursor + page_size, mcp_resources.len());
        let page_resources = mcp_resources[cursor..end].to_vec();
        
        let next_cursor = if end < mcp_resources.len() {
            Some(end)
        } else {
            None
        };
        
        Ok(ListResourcesResult {
            resources: page_resources,
            next_cursor: next_cursor.map(|c| c as i64),
        })
    }

    async fn read_resource(&self, request: ReadResourceRequestParam) -> Result<ReadResourceResult, Self::Error> {
        let resources = self.resources.lock().await;
        
        match resources.read_resource(&request.uri, &*self.tools.lock().await).await {
            Ok(content) => Ok(ReadResourceResult {
                contents: vec![content],
            }),
            Err(e) => {
                error!("Resource read failed: {}", e);
                Err(GlspError::ResourceNotFound(request.uri))
            }
        }
    }

    async fn list_prompts(&self, request: PaginatedRequestParam) -> Result<ListPromptsResult, Self::Error> {
        let prompts = self.prompts.lock().await;
        let available_prompts = prompts.get_available_prompts();
        
        // Convert to MCP protocol format
        let mcp_prompts: Vec<Prompt> = available_prompts.into_iter().map(|prompt| {
            Prompt {
                name: prompt.name,
                description: prompt.description,
                arguments: prompt.arguments.map(|args| {
                    args.into_iter().map(|arg| PromptArgument {
                        name: arg.name,
                        description: arg.description,
                        required: arg.required,
                    }).collect()
                }),
            }
        }).collect();
        
        // Handle pagination
        let cursor = request.cursor.unwrap_or(0);
        let page_size = 50;
        let end = std::cmp::min(cursor + page_size, mcp_prompts.len());
        let page_prompts = mcp_prompts[cursor..end].to_vec();
        
        let next_cursor = if end < mcp_prompts.len() {
            Some(end)
        } else {
            None
        };
        
        Ok(ListPromptsResult {
            prompts: page_prompts,
            next_cursor: next_cursor.map(|c| c as i64),
        })
    }

    async fn get_prompt(&self, request: GetPromptRequestParam) -> Result<GetPromptResult, Self::Error> {
        let prompts = self.prompts.lock().await;
        
        let prompt_params = crate::mcp::protocol::GetPromptParams {
            name: request.name.clone(),
            arguments: request.arguments,
        };
        
        match prompts.get_prompt(prompt_params).await {
            Ok(result) => Ok(GetPromptResult {
                description: result.description,
                messages: result.messages.into_iter().map(|msg| PromptMessage {
                    role: msg.role,
                    content: msg.content,
                }).collect(),
            }),
            Err(e) => {
                error!("Prompt get failed: {}", e);
                Err(GlspError::PromptNotFound(request.name))
            }
        }
    }

    async fn on_startup(&self) -> Result<(), Self::Error> {
        info!("GLSP backend starting up");
        // Any startup initialization can go here
        Ok(())
    }

    async fn on_shutdown(&self) -> Result<(), Self::Error> {
        info!("GLSP backend shutting down");
        // Any cleanup can go here
        Ok(())
    }

    async fn on_client_connect(&self, client_info: &Implementation) -> Result<(), Self::Error> {
        info!("Client connected: {} v{}", client_info.name, client_info.version);
        Ok(())
    }

    async fn on_client_disconnect(&self, client_info: &Implementation) -> Result<(), Self::Error> {
        info!("Client disconnected: {} v{}", client_info.name, client_info.version);
        Ok(())
    }
}