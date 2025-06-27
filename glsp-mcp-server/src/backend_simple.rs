//! Simplified GLSP Backend implementation for MCP framework
//!
//! This is a simplified version to get the basic structure working first.

use async_trait::async_trait;
use mcp_server::{McpBackend, BackendError};
use mcp_protocol::*;
use tracing::{info, error};
use std::collections::HashMap;
use serde_json::json;
use crate::model::{DiagramModel, Node, Edge, Position};
use crate::wasm::WasmFileWatcher;
use crate::persistence::PersistenceManager;
use std::path::PathBuf;

/// Configuration for the GLSP backend
#[derive(Debug, Clone)]
pub struct GlspConfig {
    /// Path to WASM components directory
    pub wasm_path: String,
    /// Path to diagrams storage directory
    pub diagrams_path: String,
    /// Server name and version information
    pub server_name: String,
    pub server_version: String,
}

impl Default for GlspConfig {
    fn default() -> Self {
        Self {
            wasm_path: "../workspace/adas-wasm-components".to_string(),
            diagrams_path: "./diagrams".to_string(),
            server_name: "GLSP MCP Server".to_string(),
            server_version: "0.1.0".to_string(),
        }
    }
}

/// Error type for GLSP backend operations
#[derive(Debug, thiserror::Error)]
pub enum GlspError {
    #[error("Backend error: {0}")]
    Backend(BackendError),
    
    #[error("Tool execution failed: {0}")]
    ToolExecution(String),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl From<GlspError> for Error {
    fn from(err: GlspError) -> Self {
        match err {
            GlspError::Backend(be) => be.into(),
            GlspError::ToolExecution(msg) => Error::internal_error(format!("Tool execution failed: {}", msg)),
            GlspError::NotImplemented(msg) => Error::method_not_found(format!("Not implemented: {}", msg)),
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
    models: std::sync::Arc<tokio::sync::Mutex<HashMap<String, DiagramModel>>>,
    wasm_watcher: std::sync::Arc<tokio::sync::Mutex<WasmFileWatcher>>,
    persistence: std::sync::Arc<PersistenceManager>,
}

#[async_trait]
impl McpBackend for GlspBackend {
    type Error = GlspError;
    type Config = GlspConfig;

    async fn initialize(config: Self::Config) -> std::result::Result<Self, Self::Error> {
        info!("Initializing GLSP backend with config: {:?}", config);
        
        let wasm_path = PathBuf::from(&config.wasm_path);
        let wasm_watcher = WasmFileWatcher::new(wasm_path);
        
        let diagrams_path = PathBuf::from(&config.diagrams_path);
        let persistence = PersistenceManager::new(diagrams_path);
        
        // Ensure storage directory exists
        persistence.ensure_storage_dir().await
            .map_err(|e| GlspError::NotImplemented(format!("Failed to create storage directory: {}", e)))?;
        
        // Create backend instance
        let backend = Self {
            config,
            models: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            wasm_watcher: std::sync::Arc::new(tokio::sync::Mutex::new(wasm_watcher)),
            persistence: std::sync::Arc::new(persistence),
        };
        
        // Load existing diagrams from disk
        backend.load_all_diagrams().await?;
        
        Ok(backend)
    }

    fn get_server_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: self.config.server_name.clone(),
                version: self.config.server_version.clone(),
            },
            instructions: Some("GLSP MCP Server for AI-native diagram modeling".to_string()),
        }
    }

    async fn health_check(&self) -> std::result::Result<(), Self::Error> {
        // Check if WASM components directory exists
        if !std::path::Path::new(&self.config.wasm_path).exists() {
            return Err(GlspError::ToolExecution(
                format!("WASM components directory not found: {}", self.config.wasm_path)
            ));
        }
        
        info!("GLSP backend health check passed");
        Ok(())
    }

    async fn list_tools(&self, _request: PaginatedRequestParam) -> std::result::Result<ListToolsResult, Self::Error> {
        let tools = vec![
            // Core diagram tools
            Tool {
                name: "create_diagram".to_string(),
                description: "Create a new diagram model".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramType": {
                            "type": "string",
                            "description": "Type of diagram to create (e.g., 'workflow', 'bpmn', 'uml')"
                        },
                        "name": {
                            "type": "string",
                            "description": "Name for the new diagram"
                        }
                    },
                    "required": ["diagramType"]
                }),
            },
            Tool {
                name: "delete_diagram".to_string(),
                description: "Delete a diagram and its associated files".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram to delete"
                        }
                    },
                    "required": ["diagramId"]
                }),
            },
            Tool {
                name: "create_node".to_string(),
                description: "Create a new node in the diagram".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"},
                        "nodeType": {"type": "string"},
                        "position": {
                            "type": "object",
                            "properties": {
                                "x": {"type": "number"},
                                "y": {"type": "number"}
                            },
                            "required": ["x", "y"]
                        },
                        "label": {"type": "string"}
                    },
                    "required": ["diagramId", "nodeType", "position"]
                }),
            },
            Tool {
                name: "create_edge".to_string(),
                description: "Create a new edge connecting two nodes".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"},
                        "edgeType": {"type": "string"},
                        "sourceId": {"type": "string"},
                        "targetId": {"type": "string"},
                        "label": {"type": "string"}
                    },
                    "required": ["diagramId", "edgeType", "sourceId", "targetId"]
                }),
            },
            Tool {
                name: "delete_element".to_string(),
                description: "Delete an element from the diagram".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"},
                        "elementId": {"type": "string"}
                    },
                    "required": ["diagramId", "elementId"]
                }),
            },
            Tool {
                name: "update_element".to_string(),
                description: "Update properties of an existing element".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"},
                        "elementId": {"type": "string"},
                        "properties": {"type": "object"},
                        "position": {
                            "type": "object",
                            "properties": {
                                "x": {"type": "number"},
                                "y": {"type": "number"}
                            }
                        }
                    },
                    "required": ["diagramId", "elementId"]
                }),
            },
            Tool {
                name: "apply_layout".to_string(),
                description: "Apply automatic layout to the diagram".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"},
                        "algorithm": {
                            "type": "string",
                            "enum": ["hierarchical", "force", "circular", "grid"]
                        },
                        "direction": {
                            "type": "string",
                            "enum": ["top-bottom", "left-right", "bottom-top", "right-left"]
                        }
                    },
                    "required": ["diagramId", "algorithm"]
                }),
            },
            Tool {
                name: "export_diagram".to_string(),
                description: "Export diagram in various formats".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"},
                        "format": {
                            "type": "string",
                            "enum": ["svg", "png", "json", "dot"]
                        }
                    },
                    "required": ["diagramId", "format"]
                }),
            },
            Tool {
                name: "save_diagram".to_string(),
                description: "Save a diagram to disk (creates both content and layout files)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"}
                    },
                    "required": ["diagramId"]
                }),
            },
            // Selection tools
            Tool {
                name: "select_elements".to_string(),
                description: "Select one or more elements in the diagram".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"},
                        "elementIds": {
                            "type": "array",
                            "items": {"type": "string"}
                        },
                        "mode": {
                            "type": "string",
                            "enum": ["single", "multiple", "range"],
                            "default": "single"
                        },
                        "append": {"type": "boolean", "default": false}
                    },
                    "required": ["diagramId", "elementIds"]
                }),
            },
            Tool {
                name: "select_all".to_string(),
                description: "Select all elements in the diagram".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"}
                    },
                    "required": ["diagramId"]
                }),
            },
            Tool {
                name: "clear_selection".to_string(),
                description: "Clear the current selection".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"}
                    },
                    "required": ["diagramId"]
                }),
            },
            Tool {
                name: "get_selection".to_string(),
                description: "Get the currently selected elements".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"}
                    },
                    "required": ["diagramId"]
                }),
            },
            // WASM component tools
            Tool {
                name: "scan_wasm_components".to_string(),
                description: "Scan for WASM components in the watch directory".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "check_wasm_component_status".to_string(),
                description: "Check the status of a specific WASM component".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "componentName": {"type": "string"}
                    },
                    "required": ["componentName"]
                }),
            },
            Tool {
                name: "load_wasm_component".to_string(),
                description: "Load a WASM component into a diagram".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"},
                        "componentName": {"type": "string"},
                        "position": {
                            "type": "object",
                            "properties": {
                                "x": {"type": "number"},
                                "y": {"type": "number"}
                            },
                            "required": ["x", "y"]
                        }
                    },
                    "required": ["diagramId", "componentName", "position"]
                }),
            },
        ];
        
        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(&self, request: CallToolRequestParam) -> std::result::Result<CallToolResult, Self::Error> {
        match request.name.as_str() {
            "create_diagram" => self.create_diagram(request.arguments).await,
            "delete_diagram" => self.delete_diagram(request.arguments).await,
            "create_node" => self.create_node(request.arguments).await,
            "create_edge" => self.create_edge(request.arguments).await,
            "delete_element" => self.delete_element(request.arguments).await,
            "update_element" => self.update_element(request.arguments).await,
            "apply_layout" => self.apply_layout(request.arguments).await,
            "export_diagram" => self.export_diagram(request.arguments).await,
            "save_diagram" => self.save_diagram_tool(request.arguments).await,
            "select_elements" => self.select_elements(request.arguments).await,
            "select_all" => self.select_all(request.arguments).await,
            "clear_selection" => self.clear_selection(request.arguments).await,
            "get_selection" => self.get_selection(request.arguments).await,
            "scan_wasm_components" => self.scan_wasm_components().await,
            "check_wasm_component_status" => self.check_wasm_component_status(request.arguments).await,
            "load_wasm_component" => self.load_wasm_component(request.arguments).await,
            _ => Err(GlspError::NotImplemented(format!("Tool not implemented: {}", request.name)))
        }
    }

    async fn list_resources(&self, _request: PaginatedRequestParam) -> std::result::Result<ListResourcesResult, Self::Error> {
        let models = self.models.lock().await;
        let mut resources = vec![
            Resource {
                uri: "diagram://list".to_string(),
                name: "Diagram List".to_string(),
                description: Some("List of all available diagrams".to_string()),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                raw: None,
            },
            Resource {
                uri: "wasm://components/list".to_string(),
                name: "WASM Components List".to_string(),
                description: Some("List of all available WASM components".to_string()),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                raw: None,
            }
        ];
        
        // Add resources for each loaded diagram
        for (id, diagram) in models.iter() {
            resources.push(Resource {
                uri: format!("diagram://model/{}", id),
                name: diagram.name.clone(),
                description: Some(format!("{} diagram", diagram.diagram_type)),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                raw: None,
            });
            
            resources.push(Resource {
                uri: format!("diagram://validation/{}", id),
                name: format!("{} Validation", diagram.name),
                description: Some("Validation results for the diagram".to_string()),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                raw: None,
            });
        }
        
        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
        })
    }

    async fn read_resource(&self, request: ReadResourceRequestParam) -> std::result::Result<ReadResourceResult, Self::Error> {
        // Parse the URI to determine what resource is being requested
        if request.uri.starts_with("diagram://model/") {
            let diagram_id = request.uri.strip_prefix("diagram://model/").unwrap_or("");
            
            let models = self.models.lock().await;
            if let Some(model) = models.get(diagram_id) {
                // Return the diagram model as JSON
                let content = serde_json::to_string(model)
                    .map_err(|e| GlspError::NotImplemented(format!("Serialization error: {}", e)))?;
                
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents {
                        uri: request.uri.clone(),
                        mime_type: Some("application/json".to_string()),
                        text: Some(content),
                        blob: None,
                    }],
                })
            } else {
                Err(GlspError::NotImplemented(format!("Diagram not found: {}", diagram_id)))
            }
        } else if request.uri.starts_with("diagram://validation/") {
            // Return a simple validation result
            let _diagram_id = request.uri.strip_prefix("diagram://validation/").unwrap_or("");
            let validation = json!({
                "isValid": true,
                "issues": []
            });
            
            Ok(ReadResourceResult {
                contents: vec![ResourceContents {
                    uri: request.uri.clone(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(validation.to_string()),
                    blob: None,
                }],
            })
        } else if request.uri == "diagram://list" {
            // Return list of diagrams from both memory and disk
            let models = self.models.lock().await;
            let mut diagram_infos = Vec::new();
            
            // Add loaded diagrams with their info
            for (id, diagram) in models.iter() {
                diagram_infos.push(json!({
                    "id": id,
                    "name": diagram.name,
                    "diagramType": diagram.diagram_type,
                    "createdAt": diagram.created_at,
                    "updatedAt": diagram.updated_at,
                }));
            }
            
            let list = json!({
                "diagrams": diagram_infos
            });
            
            Ok(ReadResourceResult {
                contents: vec![ResourceContents {
                    uri: request.uri.clone(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(list.to_string()),
                    blob: None,
                }],
            })
        } else if request.uri == "wasm://components/list" {
            // Get WASM components from the file watcher
            let wasm_watcher = self.wasm_watcher.lock().await;
            let components = wasm_watcher.get_components();
            
            let component_list: Vec<serde_json::Value> = components.iter()
                .map(|component| json!({
                    "name": component.name,
                    "path": component.path,
                    "description": component.description,
                    "status": if component.file_exists { "available" } else { "missing" },
                    "interfaces": component.interfaces.len(),
                    "uri": format!("wasm://component/{}", component.name)
                }))
                .collect();

            let wasm_list = json!({
                "components": component_list,
                "total": component_list.len(),
                "available": components.iter().filter(|c| c.file_exists).count(),
                "missing": components.iter().filter(|c| !c.file_exists).count()
            });
            
            Ok(ReadResourceResult {
                contents: vec![ResourceContents {
                    uri: request.uri.clone(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(wasm_list.to_string()),
                    blob: None,
                }],
            })
        } else {
            Err(GlspError::NotImplemented(format!("Resource type not supported: {}", request.uri)))
        }
    }

    async fn list_prompts(&self, _request: PaginatedRequestParam) -> std::result::Result<ListPromptsResult, Self::Error> {
        // Return empty prompts for now
        Ok(ListPromptsResult {
            prompts: vec![],
            next_cursor: None,
        })
    }

    async fn get_prompt(&self, request: GetPromptRequestParam) -> std::result::Result<GetPromptResult, Self::Error> {
        Err(GlspError::NotImplemented(format!("Prompt not found: {}", request.name)))
    }

    async fn on_startup(&self) -> std::result::Result<(), Self::Error> {
        info!("GLSP backend starting up");
        Ok(())
    }

    async fn on_shutdown(&self) -> std::result::Result<(), Self::Error> {
        info!("GLSP backend shutting down");
        Ok(())
    }

    async fn on_client_connect(&self, client_info: &Implementation) -> std::result::Result<(), Self::Error> {
        info!("Client connected: {} v{}", client_info.name, client_info.version);
        Ok(())
    }

    async fn on_client_disconnect(&self, client_info: &Implementation) -> std::result::Result<(), Self::Error> {
        info!("Client disconnected: {} v{}", client_info.name, client_info.version);
        Ok(())
    }
}

impl GlspBackend {
    // Tool implementation methods
    async fn create_diagram(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        let diagram_type = args["diagramType"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing diagramType".to_string()))?;
        let name = args["name"].as_str()
            .unwrap_or("Untitled Diagram");
        
        let mut diagram = DiagramModel::new(diagram_type);
        diagram.name = name.to_string();
        let diagram_id = diagram.id.clone();
        
        // Save to memory
        let mut models = self.models.lock().await;
        models.insert(diagram_id.clone(), diagram.clone());
        drop(models); // Release the lock before saving to disk
        
        // Save to disk
        if let Err(e) = self.save_diagram(&diagram_id).await {
            error!("Failed to save new diagram to disk: {}", e);
        }

        Ok(CallToolResult {
            content: vec![Content::text(format!("Created diagram '{}' with ID: {}", name, diagram_id))],
            is_error: Some(false),
        })
    }

    async fn delete_diagram(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing diagramId".to_string()))?;
        
        // Get the diagram name before deletion for the response
        let diagram_name = {
            let models = self.models.lock().await;
            models.get(diagram_id).map(|d| d.name.clone())
        };
        
        // Remove from memory
        let mut models = self.models.lock().await;
        let removed = models.remove(diagram_id);
        drop(models); // Release the lock before filesystem operations
        
        if removed.is_none() {
            return Err(GlspError::ToolExecution(format!("Diagram not found: {}", diagram_id)));
        }
        
        // Delete from disk using persistence manager
        let name_for_deletion = diagram_name.as_deref().unwrap_or("Unknown");
        if let Err(e) = self.delete_diagram_files(name_for_deletion).await {
            error!("Failed to delete diagram files from disk: {}", e);
            return Err(GlspError::ToolExecution(format!("Failed to delete diagram files: {}", e)));
        }
        
        info!("Deleted diagram '{}' (ID: {})", name_for_deletion, diagram_id);
        
        Ok(CallToolResult {
            content: vec![Content::text(format!("Successfully deleted diagram '{}' (ID: {})", 
                name_for_deletion, diagram_id))],
            is_error: Some(false),
        })
    }

    async fn create_node(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing diagramId".to_string()))?;
        let node_type = args["nodeType"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing nodeType".to_string()))?;
        
        let position = Position {
            x: args["position"]["x"].as_f64()
                .ok_or_else(|| GlspError::ToolExecution("Missing position.x".to_string()))?,
            y: args["position"]["y"].as_f64()
                .ok_or_else(|| GlspError::ToolExecution("Missing position.y".to_string()))?,
        };

        let label = args["label"].as_str().map(|s| s.to_string());

        let mut models = self.models.lock().await;
        let diagram = models.get_mut(diagram_id)
            .ok_or_else(|| GlspError::ToolExecution("Diagram not found".to_string()))?;

        let node = Node::new(node_type, position, label);
        let node_id = node.base.id.clone();
        
        diagram.add_element(node.base);
        diagram.add_child_to_root(&node_id);
        drop(models); // Release the lock before saving
        
        // Save to disk
        if let Err(e) = self.save_diagram(diagram_id).await {
            error!("Failed to save diagram after creating node: {}", e);
        }

        Ok(CallToolResult {
            content: vec![Content::text(format!("Created {} node with ID: {}", node_type, node_id))],
            is_error: Some(false),
        })
    }

    async fn create_edge(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing diagramId".to_string()))?;
        let edge_type = args["edgeType"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing edgeType".to_string()))?;
        let source_id = args["sourceId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing sourceId".to_string()))?;
        let target_id = args["targetId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing targetId".to_string()))?;

        let label = args["label"].as_str().map(|s| s.to_string());

        let mut models = self.models.lock().await;
        let diagram = models.get_mut(diagram_id)
            .ok_or_else(|| GlspError::ToolExecution("Diagram not found".to_string()))?;

        // Verify source and target exist
        if !diagram.elements.contains_key(source_id) {
            return Ok(CallToolResult {
                content: vec![Content::text(format!("Source element {} not found", source_id))],
                is_error: Some(true),
            });
        }

        if !diagram.elements.contains_key(target_id) {
            return Ok(CallToolResult {
                content: vec![Content::text(format!("Target element {} not found", target_id))],
                is_error: Some(true),
            });
        }

        let edge = Edge::new(edge_type, source_id.to_string(), target_id.to_string(), label);
        let edge_id = edge.base.id.clone();
        
        // Convert Edge to ModelElement with sourceId and targetId in properties
        let mut edge_element = edge.base;
        edge_element.properties.insert("sourceId".to_string(), serde_json::Value::String(source_id.to_string()));
        edge_element.properties.insert("targetId".to_string(), serde_json::Value::String(target_id.to_string()));
        
        diagram.add_element(edge_element);
        diagram.add_child_to_root(&edge_id);
        drop(models); // Release the lock before saving
        
        // Save to disk
        if let Err(e) = self.save_diagram(diagram_id).await {
            error!("Failed to save diagram after creating edge: {}", e);
        }

        Ok(CallToolResult {
            content: vec![Content::text(format!("Created {} edge with ID: {}", edge_type, edge_id))],
            is_error: Some(false),
        })
    }

    async fn delete_element(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing diagramId".to_string()))?;
        let element_id = args["elementId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing elementId".to_string()))?;

        let mut models = self.models.lock().await;
        let diagram = models.get_mut(diagram_id)
            .ok_or_else(|| GlspError::ToolExecution("Diagram not found".to_string()))?;

        match diagram.remove_element(element_id) {
            Some(_) => {
                drop(models); // Release the lock before saving
                
                // Save to disk
                if let Err(e) = self.save_diagram(diagram_id).await {
                    error!("Failed to save diagram after deleting element: {}", e);
                }
                
                Ok(CallToolResult {
                    content: vec![Content::text(format!("Deleted element with ID: {}", element_id))],
                    is_error: Some(false),
                })
            },
            None => Ok(CallToolResult {
                content: vec![Content::text(format!("Element {} not found", element_id))],
                is_error: Some(true),
            }),
        }
    }

    async fn update_element(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing diagramId".to_string()))?;
        let element_id = args["elementId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing elementId".to_string()))?;

        let mut models = self.models.lock().await;
        let diagram = models.get_mut(diagram_id)
            .ok_or_else(|| GlspError::ToolExecution("Diagram not found".to_string()))?;

        let element = diagram.get_element_mut(element_id)
            .ok_or_else(|| GlspError::ToolExecution("Element not found".to_string()))?;

        if let Some(properties) = args["properties"].as_object() {
            for (key, value) in properties {
                element.properties.insert(key.clone(), value.clone());
            }
        }

        if let Some(position) = args["position"].as_object() {
            if let (Some(x), Some(y)) = (position["x"].as_f64(), position["y"].as_f64()) {
                if let Some(bounds) = &mut element.bounds {
                    bounds.x = x;
                    bounds.y = y;
                }
            }
        }

        drop(models); // Release the lock before saving
        
        // Save to disk
        if let Err(e) = self.save_diagram(diagram_id).await {
            error!("Failed to save diagram after updating element: {}", e);
        }

        Ok(CallToolResult {
            content: vec![Content::text(format!("Updated element with ID: {}", element_id))],
            is_error: Some(false),
        })
    }

    async fn apply_layout(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing diagramId".to_string()))?;
        let algorithm = args["algorithm"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing algorithm".to_string()))?;

        let mut models = self.models.lock().await;
        let diagram = models.get_mut(diagram_id)
            .ok_or_else(|| GlspError::ToolExecution("Diagram not found".to_string()))?;

        // Simple layout implementation
        match algorithm {
            "grid" => Self::apply_grid_layout(diagram),
            "hierarchical" => Self::apply_hierarchical_layout(diagram),
            _ => {
                return Ok(CallToolResult {
                    content: vec![Content::text(format!("Layout algorithm '{}' not implemented yet", algorithm))],
                    is_error: Some(true),
                });
            }
        }

        drop(models); // Release the lock before saving
        
        // Save to disk
        if let Err(e) = self.save_diagram(diagram_id).await {
            error!("Failed to save diagram after applying layout: {}", e);
        }

        Ok(CallToolResult {
            content: vec![Content::text(format!("Applied {} layout to diagram {}", algorithm, diagram_id))],
            is_error: Some(false),
        })
    }

    async fn export_diagram(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing diagramId".to_string()))?;
        let format = args["format"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing format".to_string()))?;

        let models = self.models.lock().await;
        let diagram = models.get(diagram_id)
            .ok_or_else(|| GlspError::ToolExecution("Diagram not found".to_string()))?;

        match format {
            "json" => {
                let json_str = serde_json::to_string_pretty(diagram)
                    .map_err(|e| GlspError::ToolExecution(format!("JSON serialization failed: {}", e)))?;
                Ok(CallToolResult {
                    content: vec![Content::text(format!("Exported diagram as JSON:\\n{}", json_str))],
                    is_error: Some(false),
                })
            }
            "svg" => {
                let svg = Self::generate_svg(diagram);
                Ok(CallToolResult {
                    content: vec![Content::text(format!("Exported diagram as SVG:\\n{}", svg))],
                    is_error: Some(false),
                })
            }
            _ => Ok(CallToolResult {
                content: vec![Content::text(format!("Export format '{}' not supported yet", format))],
                is_error: Some(true),
            }),
        }
    }

    // Selection tool implementations - simplified for now
    async fn select_elements(&self, _args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        Ok(CallToolResult {
            content: vec![Content::text("Selection functionality not yet implemented")],
            is_error: Some(false),
        })
    }

    async fn select_all(&self, _args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        Ok(CallToolResult {
            content: vec![Content::text("Select all functionality not yet implemented")],
            is_error: Some(false),
        })
    }

    async fn clear_selection(&self, _args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        Ok(CallToolResult {
            content: vec![Content::text("Clear selection functionality not yet implemented")],
            is_error: Some(false),
        })
    }

    async fn get_selection(&self, _args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        Ok(CallToolResult {
            content: vec![Content::text("Get selection functionality not yet implemented")],
            is_error: Some(false),
        })
    }

    // WASM component implementations
    async fn scan_wasm_components(&self) -> std::result::Result<CallToolResult, GlspError> {
        let mut wasm_watcher = self.wasm_watcher.lock().await;
        
        match wasm_watcher.scan_components().await {
            Ok(()) => {
                let components = wasm_watcher.get_components();
                let available = components.iter().filter(|c| c.file_exists).count();
                let missing = components.len() - available;

                Ok(CallToolResult {
                    content: vec![Content::text(format!(
                        "WASM component scan completed.\\nFound {} components: {} available, {} missing",
                        components.len(), available, missing
                    ))],
                    is_error: Some(false),
                })
            }
            Err(err) => Ok(CallToolResult {
                content: vec![Content::text(format!("Failed to scan WASM components: {}", err))],
                is_error: Some(true),
            }),
        }
    }

    async fn check_wasm_component_status(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        let component_name = args["componentName"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing componentName".to_string()))?;

        let wasm_watcher = self.wasm_watcher.lock().await;
        
        match wasm_watcher.get_component(component_name) {
            Some(component) => {
                let status = json!({
                    "name": component.name,
                    "path": component.path,
                    "fileExists": component.file_exists,
                    "status": if component.file_exists { "available" } else { "missing" },
                    "lastSeen": component.last_seen,
                    "removedAt": component.removed_at,
                    "interfaces": component.interfaces.len(),
                    "description": component.description
                });

                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&status)
                        .map_err(|e| GlspError::ToolExecution(format!("JSON serialization failed: {}", e)))?)],
                    is_error: Some(false),
                })
            }
            None => Ok(CallToolResult {
                content: vec![Content::text(format!("WASM component '{}' not found", component_name))],
                is_error: Some(true),
            }),
        }
    }

    async fn save_diagram_tool(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing diagramId".to_string()))?;
        
        match self.save_diagram(diagram_id).await {
            Ok(_) => Ok(CallToolResult {
                content: vec![Content::text(format!("Successfully saved diagram {} to disk", diagram_id))],
                is_error: Some(false),
            }),
            Err(e) => Ok(CallToolResult {
                content: vec![Content::text(format!("Failed to save diagram: {}", e))],
                is_error: Some(true),
            }),
        }
    }

    async fn load_wasm_component(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing diagramId".to_string()))?;
        let component_name = args["componentName"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing componentName".to_string()))?;
        
        let position = Position {
            x: args["position"]["x"].as_f64().unwrap_or(100.0),
            y: args["position"]["y"].as_f64().unwrap_or(100.0),
        };

        // Check if component exists and is available
        let component = {
            let wasm_watcher = self.wasm_watcher.lock().await;
            wasm_watcher.get_component(component_name)
                .ok_or_else(|| GlspError::ToolExecution(format!("WASM component '{}' not found", component_name)))?
                .clone()
        };

        if !component.file_exists {
            return Ok(CallToolResult {
                content: vec![Content::text(format!("Cannot load component '{}': file is missing at {}", component_name, component.path))],
                is_error: Some(true),
            });
        }

        // Get the diagram
        let mut models = self.models.lock().await;
        let diagram = models.get_mut(diagram_id)
            .ok_or_else(|| GlspError::ToolExecution(format!("Diagram '{}' not found", diagram_id)))?;

        // Create a WASM component node
        let mut node = Node::new("wasm-component", position, Some(component.name.clone()));
        
        // Add component-specific properties
        node.base.properties.insert("componentName".to_string(), json!(component.name));
        node.base.properties.insert("componentPath".to_string(), json!(component.path));
        node.base.properties.insert("description".to_string(), json!(component.description));

        let node_id = node.base.id.clone();
        diagram.add_element(node.base);
        diagram.add_child_to_root(&node_id);

        Ok(CallToolResult {
            content: vec![Content::text(format!("Loaded WASM component '{}' into diagram with ID: {}", component_name, node_id))],
            is_error: Some(false),
        })
    }

    // Layout helpers
    fn apply_grid_layout(diagram: &mut DiagramModel) {
        let mut x = 50.0;
        let mut y = 50.0;
        let spacing_x = 150.0;
        let _spacing_y = 100.0;
        let cols = 4;
        let mut col = 0;

        for (_, element) in diagram.elements.iter_mut() {
            if element.element_type != "graph" && element.bounds.is_some() {
                if let Some(bounds) = &mut element.bounds {
                    bounds.x = x;
                    bounds.y = y;
                }

                col += 1;
                if col >= cols {
                    col = 0;
                    x = 50.0;
                    y += 100.0;
                } else {
                    x += spacing_x;
                }
            }
        }
        diagram.revision += 1;
    }

    fn apply_hierarchical_layout(diagram: &mut DiagramModel) {
        let y = 50.0;
        let mut x = 50.0;
        let spacing_x = 150.0;

        for (_, element) in diagram.elements.iter_mut() {
            if element.element_type != "graph" && element.bounds.is_some() {
                if let Some(bounds) = &mut element.bounds {
                    bounds.x = x;
                    bounds.y = y;
                }
                x += spacing_x;
            }
        }
        diagram.revision += 1;
    }

    fn generate_svg(diagram: &DiagramModel) -> String {
        let mut svg = String::from(r#"<svg width="800" height="600" xmlns="http://www.w3.org/2000/svg">"#);
        
        // Add elements
        for (_, element) in &diagram.elements {
            if element.element_type != "graph" {
                if let Some(bounds) = &element.bounds {
                    if element.element_type.contains("node") || element.element_type == "task" {
                        svg.push_str(&format!(
                            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="lightblue" stroke="black" stroke-width="1"/>"#,
                            bounds.x, bounds.y, bounds.width, bounds.height
                        ));
                        
                        if let Some(label) = element.properties.get("label") {
                            if let Some(label_text) = label.as_str() {
                                svg.push_str(&format!(
                                    r#"<text x="{}" y="{}" text-anchor="middle" dominant-baseline="middle">{}</text>"#,
                                    bounds.x + bounds.width / 2.0,
                                    bounds.y + bounds.height / 2.0,
                                    label_text
                                ));
                            }
                        }
                    }
                }
            }
        }
        
        svg.push_str("</svg>");
        svg
    }

    // Persistence helper methods
    async fn load_all_diagrams(&self) -> std::result::Result<(), GlspError> {
        let diagram_infos = self.persistence.list_diagrams().await
            .map_err(|e| GlspError::NotImplemented(format!("Failed to list diagrams: {}", e)))?;
        
        let mut models = self.models.lock().await;
        
        for info in diagram_infos {
            match self.persistence.load_diagram(&info.file_name).await {
                Ok(diagram) => {
                    info!("Loaded diagram '{}' from disk", info.name);
                    models.insert(diagram.id.clone(), diagram);
                }
                Err(e) => {
                    error!("Failed to load diagram '{}': {}", info.file_name, e);
                }
            }
        }
        
        info!("Loaded {} diagrams from disk", models.len());
        Ok(())
    }

    async fn save_diagram(&self, diagram_id: &str) -> std::result::Result<(), GlspError> {
        let models = self.models.lock().await;
        
        if let Some(diagram) = models.get(diagram_id) {
            self.persistence.save_diagram(diagram).await
                .map_err(|e| GlspError::NotImplemented(format!("Failed to save diagram: {}", e)))?;
            info!("Saved diagram '{}' to disk", diagram.name);
            Ok(())
        } else {
            Err(GlspError::NotImplemented(format!("Diagram not found: {}", diagram_id)))
        }
    }

    async fn delete_diagram_files(&self, diagram_name: &str) -> std::result::Result<(), GlspError> {
        self.persistence.delete_diagram(diagram_name).await
            .map_err(|e| GlspError::NotImplemented(format!("Failed to delete diagram files: {}", e)))?;
        info!("Deleted diagram files for '{}'", diagram_name);
        Ok(())
    }
}