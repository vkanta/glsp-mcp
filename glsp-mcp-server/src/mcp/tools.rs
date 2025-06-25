use crate::model::{DiagramModel, Node, Edge, Position};
use crate::mcp::protocol::{Tool, CallToolParams, CallToolResult, TextContent};
use serde_json::{json, Value};
use std::collections::HashMap;
use anyhow::Result;

pub struct DiagramTools {
    models: HashMap<String, DiagramModel>,
}

impl DiagramTools {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
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
                text: format!("Created diagram '{}' with ID: {}", diagram_type, diagram_id),
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
                text: format!("Created {} node with ID: {}", node_type, node_id),
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
                    text: format!("Source element {} not found", source_id),
                }],
                is_error: Some(true),
            });
        }

        if !diagram.elements.contains_key(target_id) {
            return Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Target element {} not found", target_id),
                }],
                is_error: Some(true),
            });
        }

        let edge = Edge::new(edge_type, source_id.to_string(), target_id.to_string(), label);
        let edge_id = edge.base.id.clone();
        
        diagram.add_element(edge.base);
        diagram.add_child_to_root(&edge_id);

        Ok(CallToolResult {
            content: vec![TextContent {
                content_type: "text".to_string(),
                text: format!("Created {} edge with ID: {}", edge_type, edge_id),
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
                    text: format!("Deleted element with ID: {}", element_id),
                }],
                is_error: None,
            }),
            None => Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Element {} not found", element_id),
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
                text: format!("Updated element with ID: {}", element_id),
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
                        text: format!("Layout algorithm '{}' not implemented yet", algorithm),
                    }],
                    is_error: Some(true),
                });
            }
        }

        Ok(CallToolResult {
            content: vec![TextContent {
                content_type: "text".to_string(),
                text: format!("Applied {} layout to diagram {}", algorithm, diagram_id),
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
                        text: format!("Exported diagram as JSON:\n{}", json_str),
                    }],
                    is_error: None,
                })
            }
            "svg" => {
                let svg = self.generate_svg(diagram);
                Ok(CallToolResult {
                    content: vec![TextContent {
                        content_type: "text".to_string(),
                        text: format!("Exported diagram as SVG:\n{}", svg),
                    }],
                    is_error: None,
                })
            }
            _ => Ok(CallToolResult {
                content: vec![TextContent {
                    content_type: "text".to_string(),
                    text: format!("Export format '{}' not supported yet", format),
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
        // Simple top-down hierarchical layout
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

    fn generate_svg(&self, diagram: &DiagramModel) -> String {
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

    pub fn get_diagram(&self, diagram_id: &str) -> Option<&DiagramModel> {
        self.models.get(diagram_id)
    }

    pub fn list_diagrams(&self) -> Vec<&DiagramModel> {
        self.models.values().collect()
    }
}