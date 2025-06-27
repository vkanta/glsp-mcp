//! Main entry point for GLSP MCP Server using the MCP framework

use glsp_mcp_server::{GlspBackend, GlspConfig};
use mcp_server::McpBackend;
use mcp_transport::{Transport, stdio::StdioTransport, http::HttpTransport};
use mcp_protocol::*;
use std::sync::Arc;
use tracing::{info, error, Level};
use tracing_subscriber;

// Type alias for request handler
type RequestHandler = Box<dyn Fn(Request) -> 
    std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Send + Sync>;

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
                        Ok(result) => Response {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: Some(serde_json::to_value(result).unwrap()),
                            error: None,
                        },
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
                                Ok(result) => Response {
                                    jsonrpc: "2.0".to_string(),
                                    id: request.id,
                                    result: Some(serde_json::to_value(result).unwrap()),
                                    error: None,
                                },
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
                            error: Some(Error::invalid_params(format!("Invalid parameters: {}", e))),
                        }
                    }
                }
                "resources/list" => {
                    match backend.list_resources(PaginatedRequestParam { cursor: None }).await {
                        Ok(result) => Response {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: Some(serde_json::to_value(result).unwrap()),
                            error: None,
                        },
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
                                Ok(result) => Response {
                                    jsonrpc: "2.0".to_string(),
                                    id: request.id,
                                    result: Some(serde_json::to_value(result).unwrap()),
                                    error: None,
                                },
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
                            error: Some(Error::invalid_params(format!("Invalid parameters: {}", e))),
                        }
                    }
                }
                "prompts/list" => {
                    match backend.list_prompts(PaginatedRequestParam { cursor: None }).await {
                        Ok(result) => Response {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: Some(serde_json::to_value(result).unwrap()),
                            error: None,
                        },
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting GLSP MCP Server with framework");

    // Create GLSP backend configuration
    let glsp_config = GlspConfig::default();
    
    // Initialize the GLSP backend
    let backend = Arc::new(GlspBackend::initialize(glsp_config).await?);
    
    // Create handler
    let handler = create_mcp_handler(backend.clone());
    
    // Determine transport type from command line args
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "stdio" {
        info!("Starting stdio transport");
        let mut transport = StdioTransport::new();
        transport.start(handler).await?;
    } else {
        info!("Starting HTTP transport on port 3000");
        let mut transport = HttpTransport::new(3000);
        
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