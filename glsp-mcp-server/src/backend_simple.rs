//! Simplified GLSP Backend implementation for MCP framework
//!
//! This is a simplified version to get the basic structure working first.

use mcp_protocol::*;
use tracing::{info, error};
use std::collections::HashMap;
use serde_json::json;
use crate::model::{DiagramModel, Node, Edge, Position, ElementType};
use crate::wasm::{WasmFileWatcher, FileSystemWatcher};
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
            diagrams_path: "../workspace/diagrams".to_string(),
            server_name: "GLSP MCP Server".to_string(),
            server_version: "0.1.0".to_string(),
        }
    }
}

/// Error type for GLSP backend operations
#[derive(Debug, thiserror::Error)]
pub enum GlspError {
    #[error("Tool execution failed: {0}")]
    ToolExecution(String),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<GlspError> for Error {
    fn from(err: GlspError) -> Self {
        match err {
            GlspError::ToolExecution(msg) => Error::internal_error(format!("Tool execution failed: {msg}")),
            GlspError::NotImplemented(msg) => Error::method_not_found(format!("Not implemented: {msg}")),
            GlspError::Io(e) => Error::internal_error(format!("IO error: {e}")),
            GlspError::Json(e) => Error::internal_error(format!("JSON error: {e}")),
        }
    }
}

/// GLSP Backend implementation
#[derive(Clone)]
pub struct GlspBackend {
    config: GlspConfig,
    models: std::sync::Arc<tokio::sync::Mutex<HashMap<String, DiagramModel>>>,
    wasm_watcher: std::sync::Arc<tokio::sync::Mutex<WasmFileWatcher>>,
    filesystem_watcher: std::sync::Arc<tokio::sync::RwLock<FileSystemWatcher>>,
    persistence: std::sync::Arc<PersistenceManager>,
}

impl GlspBackend {
    pub async fn initialize(config: GlspConfig) -> std::result::Result<Self, GlspError> {
        info!("Initializing GLSP backend with config: {:?}", config);
        
        let wasm_path = PathBuf::from(&config.wasm_path);
        let wasm_watcher = WasmFileWatcher::new(wasm_path.clone());
        let mut filesystem_watcher = FileSystemWatcher::new(wasm_path);
        
        // Start filesystem watching
        filesystem_watcher.start_watching().await
            .map_err(|e| GlspError::NotImplemented(format!("Failed to start filesystem watcher: {e}")))?;
        
        let diagrams_path = PathBuf::from(&config.diagrams_path);
        let persistence = PersistenceManager::new(diagrams_path);
        
        // Ensure storage directory exists
        persistence.ensure_storage_dir().await
            .map_err(|e| GlspError::NotImplemented(format!("Failed to create storage directory: {e}")))?;
        
        // Create backend instance
        let backend = Self {
            config,
            models: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            wasm_watcher: std::sync::Arc::new(tokio::sync::Mutex::new(wasm_watcher)),
            filesystem_watcher: std::sync::Arc::new(tokio::sync::RwLock::new(filesystem_watcher)),
            persistence: std::sync::Arc::new(persistence),
        };
        
        // Load existing diagrams from disk
        backend.load_all_diagrams().await?;
        
        // Perform initial WASM component scan with statistics
        info!("Performing initial WASM component scan...");
        {
            let mut wasm_watcher = backend.wasm_watcher.lock().await;
            if let Err(e) = wasm_watcher.scan_components().await {
                error!("Failed to perform initial WASM component scan: {}", e);
            }
        }
        
        Ok(backend)
    }

    pub fn get_server_info(&self) -> ServerInfo {
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

    pub async fn health_check(&self) -> std::result::Result<(), GlspError> {
        // Check if WASM components directory exists
        if !std::path::Path::new(&self.config.wasm_path).exists() {
            return Err(GlspError::ToolExecution(
                format!("WASM components directory not found: {}", self.config.wasm_path)
            ));
        }
        
        info!("GLSP backend health check passed");
        Ok(())
    }

    pub async fn list_tools(&self, _request: PaginatedRequestParam) -> std::result::Result<ListToolsResult, GlspError> {
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
            Tool {
                name: "refresh_wasm_interfaces".to_string(),
                description: "Refresh interface data for all WASM components in a diagram".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {"type": "string"}
                    },
                    "required": ["diagramId"]
                }),
            },
            Tool {
                name: "get_component_path".to_string(),
                description: "Get the file path for a WASM component".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "componentName": {
                            "type": "string",
                            "description": "Name of the WASM component"
                        }
                    },
                    "required": ["componentName"]
                }),
            },
            Tool {
                name: "get_component_wit_info".to_string(),
                description: "Get WIT interface information for a selected component in a diagram".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram containing the component"
                        },
                        "elementId": {
                            "type": "string",
                            "description": "ID of the component element to analyze"
                        }
                    },
                    "required": ["diagramId", "elementId"]
                }),
            },
            Tool {
                name: "debug_wit_analysis".to_string(),
                description: "Debug WIT interface analysis for a specific WASM component file".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "componentPath": {
                            "type": "string",
                            "description": "Full file path to the WASM component file to analyze"
                        }
                    },
                    "required": ["componentPath"]
                }),
            },
        ];
        
        Ok(ListToolsResult {
            tools,
            next_cursor: String::new(),
        })
    }

    pub async fn call_tool(&self, request: CallToolRequestParam) -> std::result::Result<CallToolResult, GlspError> {
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
            "refresh_wasm_interfaces" => self.refresh_wasm_interfaces(request.arguments).await,
            "get_component_path" => self.get_component_path(request.arguments).await,
            "get_component_wit_info" => self.get_component_wit_info(request.arguments).await,
            "debug_wit_analysis" => self.debug_wit_analysis(request.arguments).await,
            _ => Err(GlspError::NotImplemented(format!("Tool not implemented: {}", request.name)))
        }
    }

    pub async fn list_resources(&self, _request: PaginatedRequestParam) -> std::result::Result<ListResourcesResult, GlspError> {
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
                uri: format!("diagram://model/{id}"),
                name: diagram.name.clone(),
                description: Some(format!("{} diagram", diagram.diagram_type)),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                raw: None,
            });
            
            resources.push(Resource {
                uri: format!("diagram://validation/{id}"),
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

    pub async fn read_resource(&self, request: ReadResourceRequestParam) -> std::result::Result<ReadResourceResult, GlspError> {
        // Parse the URI to determine what resource is being requested
        if request.uri.starts_with("diagram://model/") {
            let diagram_id = request.uri.strip_prefix("diagram://model/").unwrap_or("");
            
            let models = self.models.lock().await;
            if let Some(model) = models.get(diagram_id) {
                // Return the diagram model as JSON
                let content = serde_json::to_string(model)
                    .map_err(|e| GlspError::NotImplemented(format!("Serialization error: {e}")))?;
                
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents {
                        uri: request.uri.clone(),
                        mime_type: Some("application/json".to_string()),
                        text: Some(content),
                        blob: None,
                    }],
                })
            } else {
                Err(GlspError::NotImplemented(format!("Diagram not found: {diagram_id}")))
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
            // Get WASM files from the filesystem watcher
            let filesystem_watcher = self.filesystem_watcher.read().await;
            let known_files = filesystem_watcher.get_known_files().await;
            
            let component_list: Vec<serde_json::Value> = known_files.iter()
                .filter_map(|path| {
                    // Extract component name from file path
                    let file_name = path.file_stem()?.to_str()?;
                    let component_name = file_name.replace('-', "_");
                    
                    Some(json!({
                        "name": component_name,
                        "path": path.to_string_lossy(),
                        "description": format!("WASM component: {}", component_name),
                        "status": "available",
                        "interfaces": 2, // Default interface count
                        "uri": format!("wasm://component/{}", component_name)
                    }))
                })
                .collect();

            let wasm_list = json!({
                "components": component_list,
                "total": component_list.len(),
                "available": component_list.len(),
                "missing": 0
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

    pub async fn list_prompts(&self, _request: PaginatedRequestParam) -> std::result::Result<ListPromptsResult, GlspError> {
        // Return empty prompts for now
        Ok(ListPromptsResult {
            prompts: vec![],
            next_cursor: None,
        })
    }

    pub async fn get_prompt(&self, request: GetPromptRequestParam) -> std::result::Result<GetPromptResult, GlspError> {
        Err(GlspError::NotImplemented(format!("Prompt not found: {}", request.name)))
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
            error!("Failed to save new diagram to disk: {e}");
        }

        Ok(CallToolResult {
            content: vec![Content::text(format!("Created diagram '{name}' with ID: {diagram_id}"))],
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
            return Err(GlspError::ToolExecution(format!("Diagram not found: {diagram_id}")));
        }
        
        // Delete from disk using persistence manager
        let name_for_deletion = diagram_name.as_deref().unwrap_or("Unknown");
        if let Err(e) = self.delete_diagram_files(name_for_deletion).await {
            error!("Failed to delete diagram files from disk: {e}");
            return Err(GlspError::ToolExecution(format!("Failed to delete diagram files: {e}")));
        }
        
        info!("Deleted diagram '{name_for_deletion}' (ID: {diagram_id})");
        
        Ok(CallToolResult {
            content: vec![Content::text(format!("Successfully deleted diagram '{name_for_deletion}' (ID: {diagram_id})"))],
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
            content: vec![Content::text(format!("Created {node_type} node with ID: {node_id}"))],
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
                content: vec![Content::text(format!("Source element {source_id} not found"))],
                is_error: Some(true),
            });
        }

        if !diagram.elements.contains_key(target_id) {
            return Ok(CallToolResult {
                content: vec![Content::text(format!("Target element {target_id} not found"))],
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
            content: vec![Content::text(format!("Created {edge_type} edge with ID: {edge_id}"))],
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
                    content: vec![Content::text(format!("Deleted element with ID: {element_id}"))],
                    is_error: Some(false),
                })
            },
            None => Ok(CallToolResult {
                content: vec![Content::text(format!("Element {element_id} not found"))],
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
            content: vec![Content::text(format!("Updated element with ID: {element_id}"))],
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
                    content: vec![Content::text(format!("Layout algorithm '{algorithm}' not implemented yet"))],
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
            content: vec![Content::text(format!("Applied {algorithm} layout to diagram {diagram_id}"))],
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
                    .map_err(|e| GlspError::ToolExecution(format!("JSON serialization failed: {e}")))?;
                Ok(CallToolResult {
                    content: vec![Content::text(format!("Exported diagram as JSON:\\n{json_str}"))],
                    is_error: Some(false),
                })
            }
            "svg" => {
                let svg = Self::generate_svg(diagram);
                Ok(CallToolResult {
                    content: vec![Content::text(format!("Exported diagram as SVG:\\n{svg}"))],
                    is_error: Some(false),
                })
            }
            _ => Ok(CallToolResult {
                content: vec![Content::text(format!("Export format '{format}' not supported yet"))],
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
        // Get current known files from the filesystem watcher
        let filesystem_watcher = self.filesystem_watcher.read().await;
        let known_files = filesystem_watcher.get_known_files().await;
        
        let wasm_files: Vec<_> = known_files.iter()
            .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("wasm"))
            .collect();
        
        let component_count = wasm_files.len();
        
        Ok(CallToolResult {
            content: vec![Content::text(format!(
                "WASM component scan completed.\\nFound {component_count} WASM files: {component_count} available, 0 missing"
            ))],
            is_error: Some(false),
        })
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
                        .map_err(|e| GlspError::ToolExecution(format!("JSON serialization failed: {e}")))?)],
                    is_error: Some(false),
                })
            }
            None => Ok(CallToolResult {
                content: vec![Content::text(format!("WASM component '{component_name}' not found"))],
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
                content: vec![Content::text(format!("Successfully saved diagram {diagram_id} to disk"))],
                is_error: Some(false),
            }),
            Err(e) => Ok(CallToolResult {
                content: vec![Content::text(format!("Failed to save diagram: {e}"))],
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
        // Use the flexible component finding method from WasmFileWatcher
        let component = {
            let wasm_watcher = self.wasm_watcher.lock().await;
            
            wasm_watcher.find_component_flexible(component_name)
                .cloned()
                .ok_or_else(|| GlspError::ToolExecution(
                    format!("WASM component '{component_name}' not found")
                ))?
        };

        if !component.file_exists {
            return Ok(CallToolResult {
                content: vec![Content::text(format!("Cannot load component '{component_name}': file is missing at {}", component.path))],
                is_error: Some(true),
            });
        }

        // Get the diagram
        let mut models = self.models.lock().await;
        let diagram = models.get_mut(diagram_id)
            .ok_or_else(|| GlspError::ToolExecution(format!("Diagram '{diagram_id}' not found")))?;

        // Create a WASM component node
        let mut node = Node::new("wasm-component", position, Some(component.name.clone()));
        
        // Add component-specific properties
        node.base.properties.insert("componentName".to_string(), json!(component.name));
        node.base.properties.insert("componentPath".to_string(), json!(component.path));
        node.base.properties.insert("description".to_string(), json!(component.description));
        
        // Add interface data for rendering
        node.base.properties.insert("interfaces".to_string(), json!(component.interfaces));
        node.base.properties.insert("status".to_string(), json!("available"));
        node.base.properties.insert("importsCount".to_string(), json!(
            component.interfaces.iter().filter(|i| i.interface_type == "import").count()
        ));
        node.base.properties.insert("exportsCount".to_string(), json!(
            component.interfaces.iter().filter(|i| i.interface_type == "export").count()
        ));
        node.base.properties.insert("totalFunctions".to_string(), json!(
            component.interfaces.iter().map(|i| i.functions.len()).sum::<usize>()
        ));

        let node_id = node.base.id.clone();
        diagram.add_element(node.base);
        diagram.add_child_to_root(&node_id);
        drop(models); // Release the lock before saving
        
        // Save to disk
        if let Err(e) = self.save_diagram(diagram_id).await {
            error!("Failed to save diagram after loading WASM component: {}", e);
        }

        Ok(CallToolResult {
            content: vec![Content::text(format!("Loaded WASM component '{component_name}' into diagram with ID: {node_id}"))],
            is_error: Some(false),
        })
    }

    async fn refresh_wasm_interfaces(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing diagramId".to_string()))?;

        let mut models = self.models.lock().await;
        let diagram = models.get_mut(diagram_id)
            .ok_or_else(|| GlspError::ToolExecution(format!("Diagram '{diagram_id}' not found")))?;

        let mut updated_count = 0;
        let wasm_watcher = self.wasm_watcher.lock().await;

        // Find all WASM components in the diagram
        let mut elements_to_update = Vec::new();
        for (element_id, element) in diagram.elements.iter() {
            // Check if this is a WASM component
            let is_wasm_component = match &element.element_type {
                ElementType::Component => true,
                ElementType::Custom(type_name) => type_name == "wasm-component",
                _ => false,
            };

            if is_wasm_component {
                // Try to get component name from properties or label
                if let Some(component_name) = element.properties.get("componentName")
                    .and_then(|v| v.as_str())
                    .or(element.label.as_deref()) {
                    
                    // Find the component using flexible name matching
                    if let Some(component) = wasm_watcher.find_component_flexible(component_name) {
                        elements_to_update.push((element_id.clone(), component.clone()));
                    }
                }
            }
        }
        drop(wasm_watcher); // Release the wasm_watcher lock

        // Update each WASM component with interface data
        for (element_id, component) in elements_to_update {
            if let Some(element) = diagram.elements.get_mut(&element_id) {
                // Add interface data for rendering
                element.properties.insert("interfaces".to_string(), json!(component.interfaces));
                element.properties.insert("status".to_string(), json!(if component.file_exists { "available" } else { "missing" }));
                element.properties.insert("importsCount".to_string(), json!(
                    component.interfaces.iter().filter(|i| i.interface_type == "import").count()
                ));
                element.properties.insert("exportsCount".to_string(), json!(
                    component.interfaces.iter().filter(|i| i.interface_type == "export").count()
                ));
                element.properties.insert("totalFunctions".to_string(), json!(
                    component.interfaces.iter().map(|i| i.functions.len()).sum::<usize>()
                ));
                
                updated_count += 1;
            }
        }

        drop(models); // Release the lock before saving
        
        // Save to disk if we updated any components
        if updated_count > 0 {
            if let Err(e) = self.save_diagram(diagram_id).await {
                error!("Failed to save diagram after refreshing interfaces: {}", e);
            }
        }

        Ok(CallToolResult {
            content: vec![Content::text(format!("Refreshed interface data for {updated_count} WASM components in diagram {diagram_id}"))],
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
            if element.element_type != ElementType::Graph && element.bounds.is_some() {
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
            if element.element_type != ElementType::Graph && element.bounds.is_some() {
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
        for element in diagram.elements.values() {
            if element.element_type != ElementType::Graph {
                if let Some(bounds) = &element.bounds {
                    if element.element_type.is_node_like() {
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
            .map_err(|e| GlspError::NotImplemented(format!("Failed to list diagrams: {e}")))?;
        
        let mut models = self.models.lock().await;
        
        for info in diagram_infos {
            match self.persistence.load_diagram(&info.file_name).await {
                Ok(diagram) => {
                    info!("Loaded diagram '{}' from disk", info.name);
                    models.insert(diagram.id.clone(), diagram);
                }
                Err(e) => {
                    error!("Failed to load diagram '{}': {e}", info.file_name);
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
                .map_err(|e| GlspError::NotImplemented(format!("Failed to save diagram: {e}")))?;
            info!("Saved diagram '{}' to disk", diagram.name);
            Ok(())
        } else {
            Err(GlspError::NotImplemented(format!("Diagram not found: {diagram_id}")))
        }
    }

    async fn delete_diagram_files(&self, diagram_name: &str) -> std::result::Result<(), GlspError> {
        self.persistence.delete_diagram(diagram_name).await
            .map_err(|e| GlspError::NotImplemented(format!("Failed to delete diagram files: {e}")))?;
        info!("Deleted diagram files for '{}'", diagram_name);
        Ok(())
    }
    
    pub fn get_filesystem_watcher(&self) -> std::sync::Arc<tokio::sync::RwLock<FileSystemWatcher>> {
        self.filesystem_watcher.clone()
    }

    pub fn get_wasm_watcher(&self) -> std::sync::Arc<tokio::sync::Mutex<WasmFileWatcher>> {
        self.wasm_watcher.clone()
    }

    async fn get_component_path(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        
        let component_name = args["componentName"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing componentName parameter".to_string()))?;
        
        // Look up the component using flexible name matching
        let wasm_watcher = self.wasm_watcher.lock().await;
        let component = wasm_watcher.find_component_flexible(component_name)
            .ok_or_else(|| GlspError::ToolExecution(format!("WASM component not found: {component_name}")))?;
        
        Ok(CallToolResult {
            content: vec![Content::text(component.path.clone())],
            is_error: Some(false),
        })
    }

    async fn get_component_wit_info(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        use crate::wasm::WitAnalyzer;
        use std::path::PathBuf;
        
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing diagramId parameter".to_string()))?;
        
        let element_id = args["elementId"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing elementId parameter".to_string()))?;
        
        // Get the diagram
        let models = self.models.lock().await;
        let diagram = models.get(diagram_id)
            .ok_or_else(|| GlspError::ToolExecution(format!("Diagram not found: {diagram_id}")))?;
        
        // Get the element
        let element = diagram.elements.get(element_id)
            .ok_or_else(|| GlspError::ToolExecution(format!("Element not found: {element_id}")))?;
        
        // Check if it's a WASM component
        let is_wasm_component = match &element.element_type {
            ElementType::Component => true,
            ElementType::Custom(type_name) => type_name == "wasm-component",
            _ => false,
        };
        
        if !is_wasm_component {
            return Ok(CallToolResult {
                content: vec![Content::text(format!("Element '{}' is not a WASM component (type: {})", element_id, element.element_type))],
                is_error: Some(true),
            });
        }
        
        // First, try to get the component file path from element properties
        let component_path = if let Some(path_value) = element.properties.get("componentPath") {
            path_value.as_str().map(|s| s.to_string())
        } else {
            None
        };
        
        // If we have a path, use it directly
        let component_file_path = if let Some(path) = component_path {
            PathBuf::from(path)
        } else {
            // Fallback: Try to find by component name
            let component_name = element.properties.get("componentName")
                .and_then(|v| v.as_str())
                .or(element.properties.get("name").and_then(|v| v.as_str()))
                .or(element.label.as_deref())
                .ok_or_else(|| GlspError::ToolExecution("Component name not found in element properties".to_string()))?;
            
            // Look up the component in the watcher using flexible name matching
            let wasm_watcher = self.wasm_watcher.lock().await;
            let component = wasm_watcher.find_component_flexible(component_name)
                .ok_or_else(|| GlspError::ToolExecution(format!("WASM component file not found: {component_name}")))?;
            
            PathBuf::from(&component.path)
        };
        
        // Verify the file exists
        if !component_file_path.exists() {
            return Ok(CallToolResult {
                content: vec![Content::text(format!("WASM component file not found at path: {}", component_file_path.display()))],
                is_error: Some(true),
            });
        }
        
        // Analyze the component
        match WitAnalyzer::analyze_component(&component_file_path).await {
            Ok(analysis) => {
                // Return structured WIT information suitable for properties panel
                let wit_info = json!({
                    "componentName": analysis.component_name,
                    "worldName": analysis.world_name,
                    "filePath": component_file_path.to_string_lossy(),
                    "imports": analysis.imports.iter().map(|interface| json!({
                        "name": interface.name,
                        "namespace": interface.namespace,
                        "package": interface.package,
                        "version": interface.version,
                        "functions": interface.functions.iter().map(|func| json!({
                            "name": func.name,
                            "params": func.params.iter().map(|p| json!({
                                "name": p.name,
                                "type": p.param_type.name
                            })).collect::<Vec<_>>(),
                            "results": func.results.iter().map(|r| json!({
                                "name": r.name,
                                "type": r.param_type.name
                            })).collect::<Vec<_>>(),
                            "isAsync": func.is_async
                        })).collect::<Vec<_>>(),
                        "types": interface.types.iter().map(|t| json!({
                            "name": t.name,
                            "definition": format!("{:?}", t.type_def)
                        })).collect::<Vec<_>>()
                    })).collect::<Vec<_>>(),
                    "exports": analysis.exports.iter().map(|interface| json!({
                        "name": interface.name,
                        "namespace": interface.namespace,
                        "package": interface.package,
                        "version": interface.version,
                        "functions": interface.functions.iter().map(|func| json!({
                            "name": func.name,
                            "params": func.params.iter().map(|p| json!({
                                "name": p.name,
                                "type": p.param_type.name
                            })).collect::<Vec<_>>(),
                            "results": func.results.iter().map(|r| json!({
                                "name": r.name,
                                "type": r.param_type.name
                            })).collect::<Vec<_>>(),
                            "isAsync": func.is_async
                        })).collect::<Vec<_>>(),
                        "types": interface.types.iter().map(|t| json!({
                            "name": t.name,
                            "definition": format!("{:?}", t.type_def)
                        })).collect::<Vec<_>>()
                    })).collect::<Vec<_>>(),
                    "dependencies": analysis.dependencies.iter().map(|dep| json!({
                        "package": dep.package,
                        "version": dep.version,
                        "interfaces": dep.interfaces
                    })).collect::<Vec<_>>(),
                    "summary": {
                        "importsCount": analysis.imports.len(),
                        "exportsCount": analysis.exports.len(),
                        "totalFunctions": analysis.imports.iter()
                            .chain(analysis.exports.iter())
                            .map(|i| i.functions.len())
                            .sum::<usize>(),
                        "typesCount": analysis.types.len(),
                        "dependenciesCount": analysis.dependencies.len()
                    }
                });
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&wit_info)
                        .map_err(|e| GlspError::ToolExecution(format!("Failed to serialize WIT info: {e}")))?)],
                    is_error: Some(false),
                })
            }
            Err(error) => {
                Ok(CallToolResult {
                    content: vec![Content::text(format!("Failed to analyze component WIT interfaces: {error}"))],
                    is_error: Some(true),
                })
            }
        }
    }

    /// Debug tool to analyze WIT interfaces for a specific component file
    async fn debug_wit_analysis(&self, args: Option<serde_json::Value>) -> std::result::Result<CallToolResult, GlspError> {
        use crate::wasm::WitAnalyzer;
        use std::path::PathBuf;
        
        let args = args.ok_or_else(|| GlspError::ToolExecution("Missing arguments".to_string()))?;
        
        let component_path = args["componentPath"].as_str()
            .ok_or_else(|| GlspError::ToolExecution("Missing componentPath parameter".to_string()))?;
        
        let path = PathBuf::from(component_path);
        
        if !path.exists() {
            return Ok(CallToolResult {
                content: vec![Content::text(format!("Component file not found: {component_path}"))],
                is_error: Some(true),
            });
        }
        
        // Analyze the component with the WIT analyzer
        match WitAnalyzer::analyze_component(&path).await {
            Ok(analysis) => {
                let debug_info = json!({
                    "analysis": "WIT Debug Analysis",
                    "componentName": analysis.component_name,
                    "worldName": analysis.world_name,
                    "filePath": component_path,
                    "imports": analysis.imports.iter().map(|interface| json!({
                        "name": interface.name,
                        "type": "import",
                        "functions": interface.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>(),
                        "functionCount": interface.functions.len(),
                        "types": interface.types.iter().map(|t| t.name.clone()).collect::<Vec<_>>(),
                        "typeCount": interface.types.len()
                    })).collect::<Vec<_>>(),
                    "exports": analysis.exports.iter().map(|interface| json!({
                        "name": interface.name,
                        "type": "export",
                        "functions": interface.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>(),
                        "functionCount": interface.functions.len(),
                        "types": interface.types.iter().map(|t| t.name.clone()).collect::<Vec<_>>(),
                        "typeCount": interface.types.len()
                    })).collect::<Vec<_>>(),
                    "summary": {
                        "totalImports": analysis.imports.len(),
                        "totalExports": analysis.exports.len(),
                        "totalInterfaces": analysis.imports.len() + analysis.exports.len(),
                        "totalTypes": analysis.types.len(),
                        "totalDependencies": analysis.dependencies.len(),
                        "hasRawWit": analysis.raw_wit.is_some()
                    },
                    "expectedForVideoAIPipeline": {
                        "expectedImports": ["video-decoder", "object-detection"],
                        "expectedExports": ["pipeline-control"],
                        "note": "If this is the video-ai-pipeline component, we should see exactly these interfaces"
                    }
                });
                
                Ok(CallToolResult {
                    content: vec![Content::text(serde_json::to_string_pretty(&debug_info)
                        .map_err(|e| GlspError::ToolExecution(format!("Failed to serialize debug info: {e}")))?)],
                    is_error: Some(false),
                })
            }
            Err(error) => {
                Ok(CallToolResult {
                    content: vec![Content::text(format!("WIT Analysis Failed: {error}\n\nThis might indicate:\n1. File is not a valid WASM component\n2. Component doesn't have WIT interfaces\n3. WIT parser error\n\nFalling back to basic file info..."))],
                    is_error: Some(true),
                })
            }
        }
    }
}