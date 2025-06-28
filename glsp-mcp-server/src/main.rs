//! Main entry point for GLSP MCP Server using the MCP framework

use glsp_mcp_server::{GlspBackend, GlspConfig};
use mcp_server::McpBackend;
use mcp_transport::{Transport, stdio::StdioTransport, http::HttpTransport};
use mcp_protocol::*;
use std::sync::Arc;
use tracing::{info, error, Level};
use clap::Parser;
use std::path::Path;
use std::fs;
use serde_json::Value;

/// GLSP MCP Server - AI-native graphical modeling platform
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Transport type: 'stdio' or 'http' (default: http)
    #[arg(value_name = "TRANSPORT")]
    transport: Option<String>,

    /// Path to WebAssembly components directory
    #[arg(short, long, default_value = "../workspace/adas-wasm-components")]
    wasm_path: String,

    /// Path to diagrams storage directory
    #[arg(short, long, default_value = "../workspace/diagrams")]
    diagrams_path: String,

    /// HTTP server port (only used with http transport)
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Create directories if they don't exist
    #[arg(short, long)]
    force: bool,
}

// Type alias for request handler
type RequestHandler = Box<dyn Fn(Request) -> 
    std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Send + Sync>;

/// Helper function to serialize results or return an error response
fn serialize_result(value: impl serde::Serialize, request_id: Value) -> Response {
    match serde_json::to_value(value) {
        Ok(result) => Response {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            result: Some(result),
            error: None,
        },
        Err(e) => {
            error!("Failed to serialize response: {}", e);
            Response {
                jsonrpc: "2.0".to_string(),
                id: request_id,
                result: None,
                error: Some(Error::internal_error(format!("Serialization failed: {}", e))),
            }
        }
    }
}

fn create_mcp_handler(backend: Arc<GlspBackend>) -> RequestHandler {
    Box::new(move |request: Request| {
        let backend = backend.clone();
        Box::pin(async move {
            match request.method.as_str() {
                "initialize" => {
                    let server_info = backend.get_server_info();
                    Response {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(serde_json::json!({
                            "protocolVersion": server_info.protocol_version.to_string(),
                            "capabilities": server_info.capabilities,
                            "serverInfo": server_info.server_info,
                            "instructions": server_info.instructions
                        })),
                        error: None,
                    }
                }
                "tools/list" => {
                    match backend.list_tools(PaginatedRequestParam { cursor: None }).await {
                        Ok(result) => serialize_result(result, request.id),
                        Err(e) => Response {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(e.into()),
                        }
                    }
                }
                "tools/call" => {
                    match serde_json::from_value::<CallToolRequestParam>(request.params) {
                        Ok(params) => {
                            match backend.call_tool(params).await {
                                Ok(result) => serialize_result(result, request.id),
                                Err(e) => Response {
                                    jsonrpc: "2.0".to_string(),
                                    id: request.id,
                                    result: None,
                                    error: Some(e.into()),
                                }
                            }
                        }
                        Err(e) => Response {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(Error::invalid_params(format!("Invalid parameters: {e}"))),
                        }
                    }
                }
                "resources/list" => {
                    match backend.list_resources(PaginatedRequestParam { cursor: None }).await {
                        Ok(result) => serialize_result(result, request.id),
                        Err(e) => Response {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(e.into()),
                        }
                    }
                }
                "resources/read" => {
                    match serde_json::from_value::<ReadResourceRequestParam>(request.params) {
                        Ok(params) => {
                            match backend.read_resource(params).await {
                                Ok(result) => serialize_result(result, request.id),
                                Err(e) => Response {
                                    jsonrpc: "2.0".to_string(),
                                    id: request.id,
                                    result: None,
                                    error: Some(e.into()),
                                }
                            }
                        }
                        Err(e) => Response {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(Error::invalid_params(format!("Invalid parameters: {e}"))),
                        }
                    }
                }
                "prompts/list" => {
                    match backend.list_prompts(PaginatedRequestParam { cursor: None }).await {
                        Ok(result) => serialize_result(result, request.id),
                        Err(e) => Response {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(e.into()),
                        }
                    }
                }
                "initialized" => {
                    // This is a notification, not a request - just acknowledge it
                    Response {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(serde_json::json!({})),
                        error: None,
                    }
                }
                "ping" => {
                    // Handle ping requests
                    Response {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(serde_json::json!({"pong": true})),
                        error: None,
                    }
                }
                _ => {
                    Response {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(Error::method_not_found(format!("Unknown method: {}", request.method))),
                    }
                }
            }
        })
    })
}

/// Validate and optionally create directories
fn validate_directories(wasm_path: &str, diagrams_path: &str, force: bool) -> anyhow::Result<()> {
    let wasm_dir = Path::new(wasm_path);
    let diagrams_dir = Path::new(diagrams_path);

    // Check WASM components directory
    if !wasm_dir.exists() {
        if force {
            info!("Creating WASM components directory: {}", wasm_path);
            fs::create_dir_all(wasm_dir)
                .map_err(|e| anyhow::anyhow!("Failed to create WASM directory '{}': {}", wasm_path, e))?;
        } else {
            return Err(anyhow::anyhow!(
                "WASM components directory '{}' does not exist. Use --force to create it.", 
                wasm_path
            ));
        }
    } else if !wasm_dir.is_dir() {
        return Err(anyhow::anyhow!(
            "WASM components path '{}' exists but is not a directory", 
            wasm_path
        ));
    }

    // Check diagrams directory
    if !diagrams_dir.exists() {
        if force {
            info!("Creating diagrams directory: {}", diagrams_path);
            fs::create_dir_all(diagrams_dir)
                .map_err(|e| anyhow::anyhow!("Failed to create diagrams directory '{}': {}", diagrams_path, e))?;
        } else {
            return Err(anyhow::anyhow!(
                "Diagrams directory '{}' does not exist. Use --force to create it.", 
                diagrams_path
            ));
        }
    } else if !diagrams_dir.is_dir() {
        return Err(anyhow::anyhow!(
            "Diagrams path '{}' exists but is not a directory", 
            diagrams_path
        ));
    }

    info!("Directory validation successful");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting GLSP MCP Server with framework");
    info!("WASM components path: {}", args.wasm_path);
    info!("Diagrams storage path: {}", args.diagrams_path);

    // Validate directories before proceeding
    validate_directories(&args.wasm_path, &args.diagrams_path, args.force)?;

    // Create GLSP backend configuration with CLI-provided paths
    let glsp_config = GlspConfig {
        wasm_path: args.wasm_path,
        diagrams_path: args.diagrams_path,
        server_name: "GLSP MCP Server".to_string(),
        server_version: "0.1.0".to_string(),
    };
    
    // Initialize the GLSP backend
    let backend = Arc::new(GlspBackend::initialize(glsp_config).await?);
    
    // Create handler
    let handler = create_mcp_handler(backend.clone());
    
    // Determine transport type from CLI args
    let transport_type = args.transport.as_deref().unwrap_or("http");
    
    if transport_type == "stdio" {
        info!("Starting stdio transport");
        let mut transport = StdioTransport::new();
        transport.start(handler).await?;
    } else {
        info!("Starting HTTP transport on port {}", args.port);
        let mut transport = HttpTransport::new(args.port);
        
        // Start the transport and keep it running
        match transport.start(handler).await {
            Ok(_) => {
                info!("HTTP transport started successfully");
                // Keep the server running indefinitely
                tokio::signal::ctrl_c().await?;
                info!("Received shutdown signal, stopping server");
            }
            Err(e) => {
                error!("Failed to start HTTP transport: {}", e);
                return Err(e.into());
            }
        }
    }
    
    Ok(())
}