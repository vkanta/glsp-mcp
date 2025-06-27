use crate::mcp::protocol::*;
use crate::mcp::tools::DiagramTools;
use crate::mcp::resources::DiagramResources;
use crate::mcp::prompts::DiagramPrompts;
use axum::{
    response::{Json, IntoResponse},
    routing::{post, get},
    Router,
    extract::Query,
    http::{header, HeaderMap},
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};
use uuid::Uuid;

pub type SharedMcpServer = Arc<Mutex<McpServer>>;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub client_info: Option<ClientInfo>,
}

pub struct McpServer {
    pub tools: DiagramTools,
    pub resources: DiagramResources,
    pub prompts: DiagramPrompts,
    pub capabilities: ServerCapabilities,
    pub sessions: HashMap<String, Session>,
}

impl McpServer {
    pub fn new() -> Self {
        let capabilities = ServerCapabilities {
            experimental: None,
            logging: Some(LoggingCapabilities {}),
            prompts: Some(PromptsCapabilities {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapabilities {
                subscribe: Some(false),
                list_changed: Some(true),
            }),
            tools: Some(ToolsCapabilities {
                list_changed: Some(true),
            }),
        };

        Self {
            tools: DiagramTools::new(),
            resources: DiagramResources::new(),
            prompts: DiagramPrompts::new(),
            capabilities,
            sessions: HashMap::new(),
        }
    }

    pub fn create_router() -> Router {
        let server = Arc::new(Mutex::new(McpServer::new()));
        
        Router::new()
            // Main MCP endpoint supporting both POST and GET
            .route("/mcp", post({
                let server = server.clone();
                move |headers: HeaderMap, Json(request): Json<JsonRpcRequest>| async move {
                    handle_post_request(server, headers, request).await
                }
            }))
            .route("/mcp", get({
                let server = server.clone();
                move |headers: HeaderMap, Query(params): Query<HashMap<String, String>>| async move {
                    handle_get_request(server, headers, params).await
                }
            }))
            // Legacy endpoint for compatibility
            .route("/mcp/rpc", post({
                let server = server.clone();
                move |headers: HeaderMap, Json(request): Json<JsonRpcRequest>| async move {
                    handle_post_request(server, headers, request).await
                }
            }))
            .route("/health", axum::routing::get(health_check))
            .layer(CorsLayer::permissive())
    }

    pub async fn handle_request(&mut self, request: JsonRpcRequest) -> (JsonRpcResponse, Option<String>) {
        info!("Handling MCP request: {}", request.method);

        match request.method.as_str() {
            "initialize" => {
                let (response, session_id) = self.handle_initialize(request).await;
                (response, Some(session_id))
            },
            "initialized" => (self.handle_initialized(request).await, None),
            "tools/list" => (self.handle_tools_list(request).await, None),
            "tools/call" => (self.handle_tools_call(request).await, None),
            "resources/list" => (self.handle_resources_list(request).await, None),
            "resources/read" => (self.handle_resources_read(request).await, None),
            "prompts/list" => (self.handle_prompts_list(request).await, None),
            "prompts/get" => (self.handle_prompts_get(request).await, None),
            _ => {
                warn!("Unknown method: {}", request.method);
                (JsonRpcResponse::error(request.id, JsonRpcError::method_not_found()), None)
            }
        }
    }

    async fn handle_initialize(&mut self, request: JsonRpcRequest) -> (JsonRpcResponse, String) {
        match request.params {
            Some(params) => {
                match serde_json::from_value::<InitializeParams>(params) {
                    Ok(init_params) => {
                        info!("Client initialized: {} v{}", 
                            init_params.client_info.name, 
                            init_params.client_info.version
                        );

                        // Create a new session
                        let session_id = Uuid::new_v4().to_string();
                        self.sessions.insert(session_id.clone(), Session {
                            id: session_id.clone(),
                            client_info: Some(init_params.client_info.clone()),
                        });

                        let result = InitializeResult {
                            protocol_version: "2025-03-26".to_string(),
                            capabilities: self.capabilities.clone(),
                            server_info: ServerInfo {
                                name: "MCP-GLSP Server".to_string(),
                                version: "0.1.0".to_string(),
                            },
                            session_id: Some(session_id.clone()),
                        };

                        (JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap()), session_id)
                    }
                    Err(e) => {
                        error!("Failed to parse initialize params: {}", e);
                        (JsonRpcResponse::error(request.id, JsonRpcError::invalid_params()), String::new())
                    }
                }
            }
            None => (JsonRpcResponse::error(request.id, JsonRpcError::invalid_params()), String::new()),
        }
    }

    async fn handle_initialized(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        info!("Client initialization completed");
        JsonRpcResponse::success(request.id, json!({}))
    }

    async fn handle_tools_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let tools = self.tools.get_available_tools();
        JsonRpcResponse::success(request.id, json!({ "tools": tools }))
    }

    async fn handle_tools_call(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.params {
            Some(params) => {
                match serde_json::from_value::<CallToolParams>(params) {
                    Ok(tool_params) => {
                        match self.tools.call_tool(tool_params).await {
                            Ok(result) => JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap()),
                            Err(e) => {
                                error!("Tool call failed: {}", e);
                                JsonRpcResponse::error(request.id, JsonRpcError::internal_error())
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse tool params: {}", e);
                        JsonRpcResponse::error(request.id, JsonRpcError::invalid_params())
                    }
                }
            }
            None => JsonRpcResponse::error(request.id, JsonRpcError::invalid_params()),
        }
    }

    async fn handle_resources_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let resources = self.resources.get_available_resources(&self.tools);
        JsonRpcResponse::success(request.id, json!({ "resources": resources }))
    }

    async fn handle_resources_read(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.params {
            Some(params) => {
                if let Some(uri) = params["uri"].as_str() {
                    match self.resources.read_resource(uri, &self.tools).await {
                        Ok(content) => JsonRpcResponse::success(request.id, serde_json::to_value(content).unwrap()),
                        Err(e) => {
                            error!("Failed to read resource {}: {}", uri, e);
                            JsonRpcResponse::error(request.id, JsonRpcError::internal_error())
                        }
                    }
                } else {
                    JsonRpcResponse::error(request.id, JsonRpcError::invalid_params())
                }
            }
            None => JsonRpcResponse::error(request.id, JsonRpcError::invalid_params()),
        }
    }

    async fn handle_prompts_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let prompts = self.prompts.get_available_prompts();
        JsonRpcResponse::success(request.id, json!({ "prompts": prompts }))
    }

    async fn handle_prompts_get(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.params {
            Some(params) => {
                match serde_json::from_value::<GetPromptParams>(params) {
                    Ok(prompt_params) => {
                        match self.prompts.get_prompt(prompt_params).await {
                            Ok(result) => JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap()),
                            Err(e) => {
                                error!("Failed to get prompt: {}", e);
                                JsonRpcResponse::error(request.id, JsonRpcError::internal_error())
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse prompt params: {}", e);
                        JsonRpcResponse::error(request.id, JsonRpcError::invalid_params())
                    }
                }
            }
            None => JsonRpcResponse::error(request.id, JsonRpcError::invalid_params()),
        }
    }
}


async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "MCP-GLSP Server",
        "version": "0.1.0"
    }))
}

async fn handle_post_request(
    server: SharedMcpServer,
    headers: HeaderMap,
    request: JsonRpcRequest,
) -> impl IntoResponse {
    info!("Received {} request", request.method);
    
    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    info!("Session ID from headers: {:?}", session_id);

    let (response, new_session_id) = {
        let mut server = server.lock().await;
        
        // Validate session if provided - allow operation without session IDs for now
        // The MCP spec says session IDs are optional (MAY use, not MUST)
        if let Some(ref sid) = session_id {
            if !server.sessions.contains_key(sid) {
                info!("Session ID {} not found, but continuing anyway", sid);
            }
        }
        
        server.handle_request(request).await
    };

    // Return with session ID header - either existing or newly created
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    
    if let Some(sid) = new_session_id {
        // New session from initialize
        info!("Returning new session ID in headers: {}", sid);
        headers.insert("Mcp-Session-Id", sid.parse().unwrap());
    } else if let Some(sid) = session_id {
        // Existing session
        headers.insert("Mcp-Session-Id", sid.parse().unwrap());
    }
    
    (headers, Json(response)).into_response()
}

async fn handle_get_request(
    _server: SharedMcpServer,
    headers: HeaderMap,
    _params: HashMap<String, String>,
) -> impl IntoResponse {
    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // For GET requests, we'll implement streaming later
    // For now, return a simple JSON response indicating the endpoint is ready
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    
    if let Some(sid) = session_id {
        response_headers.insert("Mcp-Session-Id", sid.parse().unwrap());
    }

    (
        response_headers,
        Json(json!({
            "status": "ready",
            "transport": "streamable-http",
            "streaming": false
        }))
    ).into_response()
}