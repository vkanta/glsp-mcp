//! Main entry point for GLSP MCP Server using the MCP framework

use glsp_mcp_server::{GlspBackend, GlspConfig};
use tracing::info;
use tracing_subscriber::EnvFilter;
use clap::Parser;
use std::path::Path;
use std::fs;

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

    /// Force create directories if they don't exist
    #[arg(short, long)]
    force: bool,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("glsp_mcp_server=debug"))
        )
        .init();

    info!("Starting GLSP MCP Server...");

    // Create directories if they don't exist (when force flag is used)
    if args.force {
        if !Path::new(&args.wasm_path).exists() {
            info!("Creating WASM components directory: {}", args.wasm_path);
            fs::create_dir_all(&args.wasm_path)?;
        }
        if !Path::new(&args.diagrams_path).exists() {
            info!("Creating diagrams directory: {}", args.diagrams_path);
            fs::create_dir_all(&args.diagrams_path)?;
        }
    } else {
        // Verify paths exist when not forcing
        if !Path::new(&args.wasm_path).exists() {
            eprintln!("Warning: WASM components directory does not exist: {}", args.wasm_path);
            eprintln!("Use --force flag to create it automatically");
        }
        if !Path::new(&args.diagrams_path).exists() {
            eprintln!("Warning: Diagrams directory does not exist: {}", args.diagrams_path);
            eprintln!("Use --force flag to create it automatically");
        }
    }

    // Create backend configuration
    let backend_config = GlspConfig {
        wasm_path: args.wasm_path,
        diagrams_path: args.diagrams_path,
        server_name: "GLSP MCP Server".to_string(),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    // Initialize backend
    info!("Initializing GLSP backend...");
    let backend = GlspBackend::initialize(backend_config).await?;

    // Determine transport type
    let transport_type = args.transport.as_deref().unwrap_or("http");
    
    info!("Starting GLSP MCP Server with {} transport on port {}...", transport_type, args.port);
    
    // For now, we're using a simple HTTP server implementation
    // The full framework integration will be completed when dependencies are resolved
    use axum::{
        routing::post,
        Router,
        extract::Json,
        response::Json as JsonResponse,
        http::{StatusCode, Method, header},
    };
    use mcp_protocol::{Request, Response};
    use std::sync::Arc;
    use tower_http::cors::{CorsLayer, Any};
    
    let backend = Arc::new(backend);
    let backend_clone = backend.clone();
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT]);
    
    let app = Router::new()
        .route("/messages", post(move |Json(request): Json<Request>| {
            let backend = backend_clone.clone();
            async move {
                match handle_mcp_request(backend, request).await {
                    Ok(response) => (StatusCode::OK, JsonResponse(response)),
                    Err(e) => {
                        let error_response = Response {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(mcp_protocol::Error::internal_error(format!("Request handling failed: {e}"))),
                            id: serde_json::Value::Null,
                        };
                        (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(error_response))
                    }
                }
            }
        }))
        .route("/health", axum::routing::get(|| async { "OK" }))
        .layer(cors);
    
    let addr = format!("127.0.0.1:{}", args.port).parse::<std::net::SocketAddr>()?;
    
    info!("GLSP MCP Server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    info!("GLSP MCP Server shutdown complete");
    Ok(())
}

async fn handle_mcp_request(
    backend: std::sync::Arc<GlspBackend>,
    request: mcp_protocol::Request,
) -> Result<mcp_protocol::Response, Box<dyn std::error::Error>> {
    use mcp_protocol::{CallToolRequestParam, PaginatedRequestParam, ReadResourceRequestParam, GetPromptRequestParam};
    
    match request.method.as_str() {
        "initialize" => {
            let server_info = backend.get_server_info();
            Ok(mcp_protocol::Response {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(server_info)?),
                error: None,
                id: request.id.clone(),
            })
        }
        "tools/list" => {
            let params: PaginatedRequestParam = if request.params.is_null() {
                PaginatedRequestParam { cursor: None }
            } else {
                serde_json::from_value(request.params)?
            };
            let result = backend.list_tools(params).await?;
            Ok(mcp_protocol::Response {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(result)?),
                error: None,
                id: request.id.clone(),
            })
        }
        "tools/call" => {
            let params: CallToolRequestParam = if request.params.is_null() {
                return Err("Missing parameters".into());
            } else {
                serde_json::from_value(request.params)?
            };
            let result = backend.call_tool(params).await?;
            Ok(mcp_protocol::Response {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(result)?),
                error: None,
                id: request.id.clone(),
            })
        }
        "resources/list" => {
            let params: PaginatedRequestParam = if request.params.is_null() {
                PaginatedRequestParam { cursor: None }
            } else {
                serde_json::from_value(request.params)?
            };
            let result = backend.list_resources(params).await?;
            Ok(mcp_protocol::Response {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(result)?),
                error: None,
                id: request.id.clone(),
            })
        }
        "resources/read" => {
            let params: ReadResourceRequestParam = if request.params.is_null() {
                return Err("Missing parameters".into());
            } else {
                serde_json::from_value(request.params)?
            };
            let result = backend.read_resource(params).await?;
            Ok(mcp_protocol::Response {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(result)?),
                error: None,
                id: request.id.clone(),
            })
        }
        "prompts/list" => {
            let params: PaginatedRequestParam = if request.params.is_null() {
                PaginatedRequestParam { cursor: None }
            } else {
                serde_json::from_value(request.params)?
            };
            let result = backend.list_prompts(params).await?;
            Ok(mcp_protocol::Response {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(result)?),
                error: None,
                id: request.id.clone(),
            })
        }
        "prompts/get" => {
            let params: GetPromptRequestParam = if request.params.is_null() {
                return Err("Missing parameters".into());
            } else {
                serde_json::from_value(request.params)?
            };
            let result = backend.get_prompt(params).await?;
            Ok(mcp_protocol::Response {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(result)?),
                error: None,
                id: request.id.clone(),
            })
        }
        method => {
            Ok(mcp_protocol::Response {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(mcp_protocol::Error::method_not_found(format!("Unknown method: {method}"))),
                id: request.id.clone(),
            })
        }
    }
}