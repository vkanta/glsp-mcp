use crate::model::{DiagramModel, Node, Edge, Position, ElementType};
use crate::mcp::protocol::{Tool, CallToolParams, CallToolResult, TextContent};
use crate::selection::SelectionMode;
use crate::wasm::{WasmFileWatcher, WasmComponent};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use anyhow::Result;

pub struct DiagramTools {
    models: HashMap<String, DiagramModel>,
    wasm_watcher: WasmFileWatcher,
}

impl Default for DiagramTools {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagramTools {
    pub fn new() -> Self {
        // Default WASM watch path - can be configured via CLI args
        let wasm_path = std::env::args()
            .find(|arg| arg.starts_with("--wasm-path="))
            .map(|arg| PathBuf::from(arg.strip_prefix("--wasm-path=").unwrap()))
            .or_else(|| std::env::var("WASM_WATCH_PATH").ok().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("../workspace/adas-wasm-components"));

        println!("WASM watch path: {wasm_path:?}");

        Self {
            models: HashMap::new(),
            wasm_watcher: WasmFileWatcher::new(wasm_path),
        }
    }

    pub fn get_available_tools(&self) -> Vec<Tool> {
        vec![
            Tool {
                name: "create_diagram".to_string(),
                description: Some("Create a new diagram model".to_string()),
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
                description: Some("Delete a diagram and its associated files".to_string()),
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
                description: Some("Create a new node in the diagram".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram to add the node to"
                        },
                        "nodeType": {
                            "type": "string",
                            "description": "Type of node (e.g., 'task', 'gateway', 'event', 'decision')"
                        },
                        "position": {
                            "type": "object",
                            "properties": {
                                "x": {"type": "number"},
                                "y": {"type": "number"}
                            },
                            "required": ["x", "y"]
                        },
                        "label": {
                            "type": "string",
                            "description": "Label text for the node"
                        },
                        "properties": {
                            "type": "object",
                            "description": "Additional properties for the node"
                        }
                    },
                    "required": ["diagramId", "nodeType", "position"]
                }),
            },
            Tool {
                name: "create_edge".to_string(),
                description: Some("Create a new edge connecting two nodes".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram to add the edge to"
                        },
                        "edgeType": {
                            "type": "string",
                            "description": "Type of edge (e.g., 'flow', 'association', 'dependency')"
                        },
                        "sourceId": {
                            "type": "string",
                            "description": "ID of the source node"
                        },
                        "targetId": {
                            "type": "string",
                            "description": "ID of the target node"
                        },
                        "label": {
                            "type": "string",
                            "description": "Label text for the edge"
                        },
                        "properties": {
                            "type": "object",
                            "description": "Additional properties for the edge"
                        }
                    },
                    "required": ["diagramId", "edgeType", "sourceId", "targetId"]
                }),
            },
            Tool {
                name: "delete_element".to_string(),
                description: Some("Delete an element from the diagram".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram"
                        },
                        "elementId": {
                            "type": "string",
                            "description": "ID of the element to delete"
                        }
                    },
                    "required": ["diagramId", "elementId"]
                }),
            },
            Tool {
                name: "update_element".to_string(),
                description: Some("Update properties of an existing element".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram"
                        },
                        "elementId": {
                            "type": "string",
                            "description": "ID of the element to update"
                        },
                        "properties": {
                            "type": "object",
                            "description": "Properties to update"
                        },
                        "position": {
                            "type": "object",
                            "properties": {
                                "x": {"type": "number"},
                                "y": {"type": "number"}
                            },
                            "description": "New position for the element"
                        }
                    },
                    "required": ["diagramId", "elementId"]
                }),
            },
            Tool {
                name: "apply_layout".to_string(),
                description: Some("Apply automatic layout to the diagram".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram"
                        },
                        "algorithm": {
                            "type": "string",
                            "enum": ["hierarchical", "force", "circular", "grid"],
                            "description": "Layout algorithm to apply"
                        },
                        "direction": {
                            "type": "string",
                            "enum": ["top-bottom", "left-right", "bottom-top", "right-left"],
                            "description": "Direction for hierarchical layout"
                        }
                    },
                    "required": ["diagramId", "algorithm"]
                }),
            },
            Tool {
                name: "export_diagram".to_string(),
                description: Some("Export diagram in various formats".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram to export"
                        },
                        "format": {
                            "type": "string",
                            "enum": ["svg", "png", "json", "dot"],
                            "description": "Export format"
                        }
                    },
                    "required": ["diagramId", "format"]
                }),
            },
            // Selection tools
            Tool {
                name: "select_elements".to_string(),
                description: Some("Select one or more elements in the diagram".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram"
                        },
                        "elementIds": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "IDs of elements to select"
                        },
                        "mode": {
                            "type": "string",
                            "enum": ["single", "multiple", "range"],
                            "description": "Selection mode",
                            "default": "single"
                        },
                        "append": {
                            "type": "boolean",
                            "description": "Whether to append to current selection",
                            "default": false
                        }
                    },
                    "required": ["diagramId", "elementIds"]
                }),
            },
            Tool {
                name: "select_all".to_string(),
                description: Some("Select all elements in the diagram".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram"
                        }
                    },
                    "required": ["diagramId"]
                }),
            },
            Tool {
                name: "clear_selection".to_string(),
                description: Some("Clear the current selection".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram"
                        }
                    },
                    "required": ["diagramId"]
                }),
            },
            Tool {
                name: "get_selection".to_string(),
                description: Some("Get the currently selected elements".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram"
                        }
                    },
                    "required": ["diagramId"]
                }),
            },
            Tool {
                name: "hover_element".to_string(),
                description: Some("Set the hover state for an element".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram"
                        },
                        "elementId": {
                            "type": "string",
                            "description": "ID of element to hover (null to clear)",
                            "nullable": true
                        }
                    },
                    "required": ["diagramId"]
                }),
            },
            Tool {
                name: "get_element_at_position".to_string(),
                description: Some("Find element at given canvas position".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram"
                        },
                        "x": {
                            "type": "number",
                            "description": "X coordinate"
                        },
                        "y": {
                            "type": "number",
                            "description": "Y coordinate"
                        },
                        "tolerance": {
                            "type": "number",
                            "description": "Hit detection tolerance in pixels",
                            "default": 5.0
                        },
                        "includeEdges": {
                            "type": "boolean",
                            "description": "Whether to include edges in hit detection",
                            "default": true
                        }
                    },
                    "required": ["diagramId", "x", "y"]
                }),
            },
            // WASM component tools
            Tool {
                name: "scan_wasm_components".to_string(),
                description: Some("Scan for WASM components in the watch directory".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
            },
            Tool {
                name: "check_wasm_component_status".to_string(),
                description: Some("Check the status of a specific WASM component".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "componentName": {
                            "type": "string",
                            "description": "Name of the WASM component to check"
                        }
                    },
                    "required": ["componentName"]
                }),
            },
            Tool {
                name: "remove_missing_component".to_string(),
                description: Some("Permanently remove a missing WASM component from tracking".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "componentName": {
                            "type": "string",
                            "description": "Name of the missing component to remove"
                        }
                    },
                    "required": ["componentName"]
                }),
            },
            Tool {
                name: "load_wasm_component".to_string(),
                description: Some("Load a WASM component into a diagram".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "diagramId": {
                            "type": "string",
                            "description": "ID of the diagram to load the component into"
                        },
                        "componentName": {
                            "type": "string",
                            "description": "Name of the WASM component to load"
                        },
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
        ]
    }

    pub async fn call_tool(&mut self, params: CallToolParams) -> Result<CallToolResult> {
        match params.name.as_str() {
            "create_diagram" => self.create_diagram(params.arguments).await,
            "create_node" => self.create_node(params.arguments).await,
            "create_edge" => self.create_edge(params.arguments).await,
            "delete_element" => self.delete_element(params.arguments).await,
            "update_element" => self.update_element(params.arguments).await,
            "apply_layout" => self.apply_layout(params.arguments).await,
            "export_diagram" => self.export_diagram(params.arguments).await,
            // Selection tools
            "select_elements" => self.select_elements(params.arguments).await,
            "select_all" => self.select_all(params.arguments).await,
            "clear_selection" => self.clear_selection(params.arguments).await,
            "get_selection" => self.get_selection(params.arguments).await,
            "hover_element" => self.hover_element(params.arguments).await,
            "get_element_at_position" => self.get_element_at_position(params.arguments).await,
            // WASM component tools
            "scan_wasm_components" => self.scan_wasm_components().await,
            "check_wasm_component_status" => self.check_wasm_component_status(params.arguments).await,
            "remove_missing_component" => self.remove_missing_component(params.arguments).await,
            "load_wasm_component" => self.load_wasm_component(params.arguments).await,
            _ => Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Unknown tool: {}", params.name),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn create_diagram(&mut self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_type = args["diagramType"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramType"))?;
        
        let diagram = DiagramModel::new(diagram_type);
        let diagram_id = diagram.id.clone();
        self.models.insert(diagram_id.clone(), diagram);

        Ok(CallToolResult {
            content: vec![TextContent {
                content_type: "text".to_string(),
                text: format!("Created diagram '{diagram_type}' with ID: {diagram_id}"),
            }],
            is_error: None,
        })
    }

    async fn create_node(&mut self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;
        let node_type = args["nodeType"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing nodeType"))?;
        
        let position = Position {
            x: args["position"]["x"].as_f64()
                .ok_or_else(|| anyhow::anyhow!("Missing position.x"))?,
            y: args["position"]["y"].as_f64()
                .ok_or_else(|| anyhow::anyhow!("Missing position.y"))?,
        };

        let label = args["label"].as_str().map(|s| s.to_string());

        let diagram = self.models.get_mut(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found"))?;

        let node = Node::new(node_type, position, label);
        let node_id = node.base.id.clone();
        
        diagram.add_element(node.base);
        diagram.add_child_to_root(&node_id);

        Ok(CallToolResult {
            content: vec![TextContent {
                content_type: "text".to_string(),
                text: format!("Created {node_type} node with ID: {node_id}"),
            }],
            is_error: None,
        })
    }

    async fn create_edge(&mut self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;
        let edge_type = args["edgeType"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing edgeType"))?;
        let source_id = args["sourceId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing sourceId"))?;
        let target_id = args["targetId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing targetId"))?;

        let label = args["label"].as_str().map(|s| s.to_string());

        let diagram = self.models.get_mut(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found"))?;

        // Verify source and target exist
        if !diagram.elements.contains_key(source_id) {
            return Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Source element {source_id} not found"),
                }],
                is_error: Some(true),
            });
        }

        if !diagram.elements.contains_key(target_id) {
            return Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Target element {target_id} not found"),
                }],
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

        Ok(CallToolResult {
            content: vec![TextContent {
                content_type: "text".to_string(),
                text: format!("Created {edge_type} edge with ID: {edge_id}"),
            }],
            is_error: None,
        })
    }

    async fn delete_element(&mut self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;
        let element_id = args["elementId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing elementId"))?;

        let diagram = self.models.get_mut(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found"))?;

        match diagram.remove_element(element_id) {
            Some(_) => Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Deleted element with ID: {element_id}"),
                }],
                is_error: None,
            }),
            None => Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Element {element_id} not found"),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn update_element(&mut self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;
        let element_id = args["elementId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing elementId"))?;

        let diagram = self.models.get_mut(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found"))?;

        let element = diagram.get_element_mut(element_id)
            .ok_or_else(|| anyhow::anyhow!("Element not found"))?;

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

        Ok(CallToolResult {
            content: vec![TextContent {
                content_type: "text".to_string(),
                text: format!("Updated element with ID: {element_id}"),
            }],
            is_error: None,
        })
    }

    async fn apply_layout(&mut self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;
        let algorithm = args["algorithm"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing algorithm"))?;

        let diagram = self.models.get_mut(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found"))?;

        // Simple layout implementation - in practice, this would use a proper layout engine
        match algorithm {
            "grid" => Self::apply_grid_layout(diagram),
            "hierarchical" => Self::apply_hierarchical_layout(diagram),
            _ => {
                return Ok(CallToolResult {
                    content: vec![TextContent {
                        content_type: "text".to_string(),
                        text: format!("Layout algorithm '{algorithm}' not implemented yet"),
                    }],
                    is_error: Some(true),
                });
            }
        }

        Ok(CallToolResult {
            content: vec![TextContent {
                content_type: "text".to_string(),
                text: format!("Applied {algorithm} layout to diagram {diagram_id}"),
            }],
            is_error: None,
        })
    }

    async fn export_diagram(&self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;
        let format = args["format"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing format"))?;

        let diagram = self.models.get(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found"))?;

        match format {
            "json" => {
                let json_str = serde_json::to_string_pretty(diagram)?;
                Ok(CallToolResult {
                    content: vec![TextContent {
                        content_type: "text".to_string(),
                        text: format!("Exported diagram as JSON:\n{json_str}"),
                    }],
                    is_error: None,
                })
            }
            "svg" => {
                let svg = self.generate_svg(diagram);
                Ok(CallToolResult {
                    content: vec![TextContent {
                        content_type: "text".to_string(),
                        text: format!("Exported diagram as SVG:\n{svg}"),
                    }],
                    is_error: None,
                })
            }
            _ => Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Export format '{format}' not supported yet"),
                }],
                is_error: Some(true),
            }),
        }
    }

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
        // Simple top-down hierarchical layout
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

    fn generate_svg(&self, diagram: &DiagramModel) -> String {
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

    pub fn get_diagram(&self, diagram_id: &str) -> Option<&DiagramModel> {
        self.models.get(diagram_id)
    }

    pub fn list_diagrams(&self) -> Vec<&DiagramModel> {
        self.models.values().collect()
    }

    pub fn get_diagrams_path(&self) -> &std::path::Path {
        std::path::Path::new("./diagrams") // Default path
    }

    // Selection tool implementations
    async fn select_elements(&mut self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;
        let element_ids = args["elementIds"].as_array()
            .ok_or_else(|| anyhow::anyhow!("Missing elementIds"))?;
        let mode = args["mode"].as_str().unwrap_or("single");
        let append = args["append"].as_bool().unwrap_or(false);

        let diagram = self.models.get_mut(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found"))?;

        if let Some(selection) = &mut diagram.selection {
            let ids: Vec<String> = element_ids
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();

            let mode = match mode {
                "multiple" => SelectionMode::Multiple,
                "range" => SelectionMode::Range,
                _ => SelectionMode::Single,
            };

            if ids.len() == 1 && !append {
                selection.select_element(ids[0].clone(), mode);
            } else {
                selection.select_multiple(ids, append);
            }

            let selected_count = selection.get_selected_count();
            let selected_ids = selection.get_selected_ids();

            Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Selected {selected_count} element(s): {selected_ids:?}"),
                }],
                is_error: None,
            })
        } else {
            Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: "Selection state not initialized".to_string(),
                }],
                is_error: Some(true),
            })
        }
    }

    async fn select_all(&mut self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;

        let diagram = self.models.get_mut(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found"))?;

        let all_ids = diagram.get_all_element_ids();
        let count = all_ids.len();

        if let Some(selection) = &mut diagram.selection {
            selection.select_all(all_ids);

            Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Selected all {count} elements"),
                }],
                is_error: None,
            })
        } else {
            Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: "Selection state not initialized".to_string(),
                }],
                is_error: Some(true),
            })
        }
    }

    async fn clear_selection(&mut self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;

        let diagram = self.models.get_mut(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found"))?;

        if let Some(selection) = &mut diagram.selection {
            selection.clear_selection();

            Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: "Selection cleared".to_string(),
                }],
                is_error: None,
            })
        } else {
            Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: "Selection state not initialized".to_string(),
                }],
                is_error: Some(true),
            })
        }
    }

    async fn get_selection(&self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;

        let diagram = self.models.get(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found"))?;

        if let Some(selection) = &diagram.selection {
            let selected_ids = selection.get_selected_ids();
            let response = json!({
                "selectedElements": selected_ids,
                "count": selected_ids.len(),
                "hoveredElement": selection.hovered_element,
                "lastSelected": selection.last_selected,
                "selectionMode": selection.selection_mode
            });

            Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: serde_json::to_string_pretty(&response)?,
                }],
                is_error: None,
            })
        } else {
            Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: "Selection state not initialized".to_string(),
                }],
                is_error: Some(true),
            })
        }
    }

    async fn hover_element(&mut self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;
        let element_id = args["elementId"].as_str().map(|s| s.to_string());

        let diagram = self.models.get_mut(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found"))?;

        if let Some(selection) = &mut diagram.selection {
            selection.set_hover(element_id.clone());

            Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: match element_id {
                        Some(id) => format!("Hovering element: {id}"),
                        None => "Hover cleared".to_string(),
                    },
                }],
                is_error: None,
            })
        } else {
            Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: "Selection state not initialized".to_string(),
                }],
                is_error: Some(true),
            })
        }
    }

    async fn get_element_at_position(&self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;
        let x = args["x"].as_f64()
            .ok_or_else(|| anyhow::anyhow!("Missing x coordinate"))?;
        let y = args["y"].as_f64()
            .ok_or_else(|| anyhow::anyhow!("Missing y coordinate"))?;
        let tolerance = args["tolerance"].as_f64().unwrap_or(5.0);

        let diagram = self.models.get(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found"))?;

        match diagram.get_element_at_position(x, y, tolerance) {
            Some(element_id) => {
                let element = diagram.get_element(&element_id);
                let response = json!({
                    "found": true,
                    "elementId": element_id,
                    "elementType": element.map(|e| &e.element_type),
                    "position": { "x": x, "y": y }
                });

                Ok(CallToolResult {
                    content: vec![TextContent {
                        content_type: "text".to_string(),
                        text: serde_json::to_string_pretty(&response)?,
                    }],
                    is_error: None,
                })
            }
            None => {
                let response = json!({
                    "found": false,
                    "position": { "x": x, "y": y }
                });

                Ok(CallToolResult {
                    content: vec![TextContent {
                        content_type: "text".to_string(),
                        text: serde_json::to_string_pretty(&response)?,
                    }],
                    is_error: None,
                })
            }
        }
    }

    // WASM component methods
    pub fn get_wasm_components(&self) -> Vec<&WasmComponent> {
        self.wasm_watcher.get_components()
    }

    pub fn get_wasm_component(&self, name: &str) -> Option<&WasmComponent> {
        self.wasm_watcher.get_component(name)
    }

    pub fn get_wasm_watch_path(&self) -> String {
        self.wasm_watcher.get_watch_path().to_string_lossy().to_string()
    }

    pub fn get_last_wasm_scan_time(&self) -> DateTime<Utc> {
        self.wasm_watcher.get_last_scan_time()
    }

    pub async fn scan_wasm_components_internal(&mut self) -> Result<()> {
        self.wasm_watcher.scan_components().await
    }

    pub fn remove_missing_wasm_component(&mut self, name: &str) -> bool {
        self.wasm_watcher.remove_missing_component(name)
    }

    // WASM tool handlers
    async fn scan_wasm_components(&mut self) -> Result<CallToolResult> {
        match self.wasm_watcher.scan_components().await {
            Ok(()) => {
                let components = self.wasm_watcher.get_components();
                let available = components.iter().filter(|c| c.file_exists).count();
                let missing = components.len() - available;

                Ok(CallToolResult {
                    content: vec![TextContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "WASM component scan completed.\nFound {} components: {} available, {} missing",
                            components.len(), available, missing
                        ),
                    }],
                    is_error: None,
                })
            }
            Err(err) => Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Failed to scan WASM components: {err}"),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn check_wasm_component_status(&self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let component_name = args["componentName"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing componentName"))?;

        match self.wasm_watcher.get_component(component_name) {
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
                    content: vec![TextContent {
                        content_type: "text".to_string(),
                        text: serde_json::to_string_pretty(&status)?,
                    }],
                    is_error: None,
                })
            }
            None => Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("WASM component '{component_name}' not found"),
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn remove_missing_component(&mut self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let component_name = args["componentName"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing componentName"))?;

        if self.wasm_watcher.remove_missing_component(component_name) {
            Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Successfully removed missing component: {component_name}"),
                }],
                is_error: None,
            })
        } else {
            Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Component '{component_name}' not found or not missing"),
                }],
                is_error: Some(true),
            })
        }
    }

    async fn load_wasm_component(&mut self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        let diagram_id = args["diagramId"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing diagramId"))?;
        let component_name = args["componentName"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing componentName"))?;
        
        let position = Position {
            x: args["position"]["x"].as_f64().unwrap_or(100.0),
            y: args["position"]["y"].as_f64().unwrap_or(100.0),
        };

        // Check if component exists and is available
        let component = self.wasm_watcher.get_component(component_name)
            .ok_or_else(|| anyhow::anyhow!("WASM component '{}' not found", component_name))?;

        if !component.file_exists {
            return Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Cannot load component '{component_name}': file is missing at {}", component.path),
                }],
                is_error: Some(true),
            });
        }

        // Get the diagram
        let diagram = self.models.get_mut(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram '{}' not found", diagram_id))?;

        // Create a WASM component node with appropriate size for metadata and interfaces
        let pos_x = position.x;
        let pos_y = position.y;
        let mut node = Node::new("wasm-component", position, Some(component.name.clone()));
        
        // Calculate dynamic size based on content
        let interface_count = component.interfaces.len() as f64;
        let description_lines = component.description.len() as f64 / 50.0; // Rough estimate
        
        // Base size + space for interfaces + metadata
        let width = 220.0_f64.max(component.name.len() as f64 * 8.0); // Minimum width based on name
        let height = 120.0_f64.max(80.0 + (interface_count * 25.0) + (description_lines * 15.0));
        
        // Update the bounds and size for proper rendering
        node.base.bounds = Some(crate::model::Bounds {
            x: pos_x,
            y: pos_y,
            width,
            height,
        });
        node.size = Some(crate::model::Size { width, height });
        
        // Add component-specific properties
        node.base.properties.insert("componentName".to_string(), json!(component.name));
        node.base.properties.insert("componentPath".to_string(), json!(component.path));
        node.base.properties.insert("description".to_string(), json!(component.description));
        node.base.properties.insert("interfaces".to_string(), json!(component.interfaces));

        let node_id = node.base.id.clone();
        diagram.add_element(node.base);
        diagram.add_child_to_root(&node_id);

        Ok(CallToolResult {
            content: vec![TextContent {
                content_type: "text".to_string(),
                text: format!("Loaded WASM component '{component_name}' into diagram with ID: {node_id}"),
            }],
            is_error: None,
        })
    }
}