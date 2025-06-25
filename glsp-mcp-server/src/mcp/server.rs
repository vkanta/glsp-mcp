use crate::mcp::protocol::*;
use crate::mcp::tools::DiagramTools;
use crate::mcp::resources::DiagramResources;
use crate::mcp::prompts::DiagramPrompts;
use axum::{
    response::Json,
    routing::post,
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};

pub type SharedMcpServer = Arc<Mutex<McpServer>>;

pub struct McpServer {
    pub tools: DiagramTools,
    pub resources: DiagramResources,
    pub prompts: DiagramPrompts,
    pub capabilities: ServerCapabilities,
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
        }
    }

    pub fn create_router() -> Router {
        let server = Arc::new(Mutex::new(McpServer::new()));
        
        Router::new()
            .route("/mcp/rpc", post({
                let server = server.clone();
move |Json(request): Json<JsonRpcRequest>| async move {
                    let response = {
                        let mut server = server.lock().await;
                        server.handle_request(request).await
                    };
                    Json(response)
                }
            }))
            .route("/health", axum::routing::get(health_check))
            .layer(CorsLayer::permissive())
    }

    pub async fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling MCP request: {}", request.method);

        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "initialized" => self.handle_initialized(request).await,
            "tools/list" => self.handle_tools_list(request).await,
            "tools/call" => self.handle_tools_call(request).await,
            "resources/list" => self.handle_resources_list(request).await,
            "resources/read" => self.handle_resources_read(request).await,
            "prompts/list" => self.handle_prompts_list(request).await,
            "prompts/get" => self.handle_prompts_get(request).await,
            _ => {
                warn!("Unknown method: {}", request.method);
                JsonRpcResponse::error(request.id, JsonRpcError::method_not_found())
            }
        }
    }

    async fn handle_initialize(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.params {
            Some(params) => {
                match serde_json::from_value::<InitializeParams>(params) {
                    Ok(init_params) => {
                        info!("Client initialized: {} v{}", 
                            init_params.client_info.name, 
                            init_params.client_info.version
                        );

                        let result = InitializeResult {
                            protocol_version: "2024-11-05".to_string(),
                            capabilities: self.capabilities.clone(),
                            server_info: ServerInfo {
                                name: "MCP-GLSP Server".to_string(),
                                version: "0.1.0".to_string(),
                            },
                        };

                        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
                    }
                    Err(e) => {
                        error!("Failed to parse initialize params: {}", e);
                        JsonRpcResponse::error(request.id, JsonRpcError::invalid_params())
                    }
                }
            }
            None => JsonRpcResponse::error(request.id, JsonRpcError::invalid_params()),
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