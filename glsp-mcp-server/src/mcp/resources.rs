use crate::mcp::protocol::{Resource, ResourceContent};
use crate::mcp::tools::DiagramTools;
use serde_json::{json, Value};
use anyhow::Result;

pub struct DiagramResources;

impl DiagramResources {
    pub fn new() -> Self {
        Self
    }

    pub fn get_available_resources(&self, tools: &DiagramTools) -> Vec<Resource> {
        let mut resources = vec![
            Resource {
                uri: "diagram://schemas/model".to_string(),
                name: "Diagram Model Schema".to_string(),
                description: Some("JSON schema for diagram model structure".to_string()),
                mime_type: Some("application/schema+json".to_string()),
            },
            Resource {
                uri: "diagram://schemas/node".to_string(),
                name: "Node Schema".to_string(),
                description: Some("JSON schema for node elements".to_string()),
                mime_type: Some("application/schema+json".to_string()),
            },
            Resource {
                uri: "diagram://schemas/edge".to_string(),
                name: "Edge Schema".to_string(),
                description: Some("JSON schema for edge elements".to_string()),
                mime_type: Some("application/schema+json".to_string()),
            },
            Resource {
                uri: "diagram://list".to_string(),
                name: "Diagram List".to_string(),
                description: Some("List of all available diagrams".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            // WASM component resources
            Resource {
                uri: "wasm://components/list".to_string(),
                name: "WASM Components List".to_string(),
                description: Some("List of all available WASM components".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "wasm://components/missing".to_string(),
                name: "Missing WASM Components".to_string(),
                description: Some("List of components with missing files".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "wasm://components/status".to_string(),
                name: "WASM Components Status".to_string(),
                description: Some("Overall status of WASM components".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ];

        // Add resources for individual WASM components
        // TODO: Get actual WASM components from the file watcher
        let wasm_components = tools.get_wasm_components(); // This method needs to be added
        for component in wasm_components {
            resources.push(Resource {
                uri: format!("wasm://component/{}", component.name),
                name: format!("WASM Component: {}", component.name),
                description: Some(format!("Details for {} component", component.name)),
                mime_type: Some("application/json".to_string()),
            });
        }

        // Add resources for each diagram
        for diagram in tools.list_diagrams() {
            resources.push(Resource {
                uri: format!("diagram://model/{}", diagram.id),
                name: format!("Diagram: {}", diagram.diagram_type),
                description: Some(format!("Current state of {} diagram", diagram.diagram_type)),
                mime_type: Some("application/vnd.glsp-model+json".to_string()),
            });

            resources.push(Resource {
                uri: format!("diagram://elements/{}", diagram.id),
                name: format!("Elements: {}", diagram.diagram_type),
                description: Some(format!("All elements in {} diagram", diagram.diagram_type)),
                mime_type: Some("application/json".to_string()),
            });

            resources.push(Resource {
                uri: format!("diagram://metadata/{}", diagram.id),
                name: format!("Metadata: {}", diagram.diagram_type),
                description: Some(format!("Metadata for {} diagram", diagram.diagram_type)),
                mime_type: Some("application/json".to_string()),
            });

            resources.push(Resource {
                uri: format!("diagram://validation/{}", diagram.id),
                name: format!("Validation: {}", diagram.diagram_type),
                description: Some(format!("Validation results for {} diagram", diagram.diagram_type)),
                mime_type: Some("application/json".to_string()),
            });
        }

        resources
    }

    pub async fn read_resource(&self, uri: &str, tools: &DiagramTools) -> Result<ResourceContent> {
        match uri {
            "diagram://schemas/model" => Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: Some("application/schema+json".to_string()),
                text: Some(self.get_model_schema()),
                blob: None,
            }),
            "diagram://schemas/node" => Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: Some("application/schema+json".to_string()),
                text: Some(self.get_node_schema()),
                blob: None,
            }),
            "diagram://schemas/edge" => Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: Some("application/schema+json".to_string()),
                text: Some(self.get_edge_schema()),
                blob: None,
            }),
            "diagram://list" => Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(self.get_diagram_list(tools)),
                blob: None,
            }),
            "wasm://components/list" => Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(self.get_wasm_components_list(tools)),
                blob: None,
            }),
            "wasm://components/missing" => Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(self.get_missing_components(tools)),
                blob: None,
            }),
            "wasm://components/status" => Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(self.get_wasm_status(tools)),
                blob: None,
            }),
            _ => {
                if uri.starts_with("diagram://model/") {
                    let diagram_id = uri.strip_prefix("diagram://model/").unwrap();
                    self.get_diagram_model(diagram_id, tools)
                } else if uri.starts_with("diagram://elements/") {
                    let diagram_id = uri.strip_prefix("diagram://elements/").unwrap();
                    self.get_diagram_elements(diagram_id, tools)
                } else if uri.starts_with("diagram://metadata/") {
                    let diagram_id = uri.strip_prefix("diagram://metadata/").unwrap();
                    self.get_diagram_metadata(diagram_id, tools)
                } else if uri.starts_with("diagram://validation/") {
                    let diagram_id = uri.strip_prefix("diagram://validation/").unwrap();
                    self.get_validation_results(diagram_id, tools)
                } else if uri.starts_with("wasm://component/") {
                    let component_name = uri.strip_prefix("wasm://component/").unwrap();
                    self.get_wasm_component_details(component_name, tools)
                } else {
                    Err(anyhow::anyhow!("Resource not found: {}", uri))
                }
            }
        }
    }

    fn get_model_schema(&self) -> String {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "title": "Diagram Model",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "Unique identifier for the diagram"
                },
                "diagramType": {
                    "type": "string",
                    "description": "Type of diagram (e.g., 'workflow', 'bpmn', 'uml')"
                },
                "revision": {
                    "type": "integer",
                    "description": "Current revision number of the model"
                },
                "root": {
                    "$ref": "#/definitions/ModelElement"
                },
                "elements": {
                    "type": "object",
                    "additionalProperties": {
                        "$ref": "#/definitions/ModelElement"
                    }
                }
            },
            "definitions": {
                "ModelElement": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "type": {"type": "string"},
                        "children": {
                            "type": "array",
                            "items": {"type": "string"}
                        },
                        "bounds": {
                            "$ref": "#/definitions/Bounds"
                        },
                        "layoutOptions": {"type": "object"},
                        "properties": {"type": "object"}
                    },
                    "required": ["id", "type"]
                },
                "Bounds": {
                    "type": "object",
                    "properties": {
                        "x": {"type": "number"},
                        "y": {"type": "number"},
                        "width": {"type": "number"},
                        "height": {"type": "number"}
                    },
                    "required": ["x", "y", "width", "height"]
                }
            },
            "required": ["id", "diagramType", "revision", "root", "elements"]
        }).to_string()
    }

    fn get_node_schema(&self) -> String {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "title": "Node Element",
            "properties": {
                "id": {"type": "string"},
                "type": {"type": "string"},
                "position": {
                    "type": "object",
                    "properties": {
                        "x": {"type": "number"},
                        "y": {"type": "number"}
                    },
                    "required": ["x", "y"]
                },
                "size": {
                    "type": "object",
                    "properties": {
                        "width": {"type": "number"},
                        "height": {"type": "number"}
                    }
                },
                "label": {"type": "string"},
                "properties": {"type": "object"}
            },
            "required": ["id", "type", "position"]
        }).to_string()
    }

    fn get_edge_schema(&self) -> String {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "title": "Edge Element",
            "properties": {
                "id": {"type": "string"},
                "type": {"type": "string"},
                "sourceId": {"type": "string"},
                "targetId": {"type": "string"},
                "routingPoints": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "x": {"type": "number"},
                            "y": {"type": "number"}
                        },
                        "required": ["x", "y"]
                    }
                },
                "label": {"type": "string"},
                "properties": {"type": "object"}
            },
            "required": ["id", "type", "sourceId", "targetId"]
        }).to_string()
    }

    fn get_diagram_list(&self, tools: &DiagramTools) -> String {
        let diagrams: Vec<Value> = tools.list_diagrams()
            .iter()
            .map(|diagram| json!({
                "id": diagram.id,
                "type": diagram.diagram_type,
                "revision": diagram.revision,
                "elementCount": diagram.elements.len(),
                "uri": format!("diagram://model/{}", diagram.id)
            }))
            .collect();

        json!({
            "diagrams": diagrams,
            "total": diagrams.len()
        }).to_string()
    }

    fn get_diagram_model(&self, diagram_id: &str, tools: &DiagramTools) -> Result<ResourceContent> {
        let diagram = tools.get_diagram(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found: {}", diagram_id))?;

        Ok(ResourceContent {
            uri: format!("diagram://model/{}", diagram_id),
            mime_type: Some("application/vnd.glsp-model+json".to_string()),
            text: Some(serde_json::to_string_pretty(diagram)?),
            blob: None,
        })
    }

    fn get_diagram_elements(&self, diagram_id: &str, tools: &DiagramTools) -> Result<ResourceContent> {
        let diagram = tools.get_diagram(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found: {}", diagram_id))?;

        let elements: Vec<Value> = diagram.elements.values()
            .filter(|element| element.element_type != "graph")
            .map(|element| {
                let mut elem_json = json!({
                    "id": element.id,
                    "type": element.element_type,
                    "properties": element.properties
                });

                if let Some(bounds) = &element.bounds {
                    elem_json["bounds"] = json!({
                        "x": bounds.x,
                        "y": bounds.y,
                        "width": bounds.width,
                        "height": bounds.height
                    });
                }

                if let Some(children) = &element.children {
                    elem_json["children"] = json!(children);
                }

                elem_json
            })
            .collect();

        Ok(ResourceContent {
            uri: format!("diagram://elements/{}", diagram_id),
            mime_type: Some("application/json".to_string()),
            text: Some(json!({
                "elements": elements,
                "count": elements.len()
            }).to_string()),
            blob: None,
        })
    }

    fn get_diagram_metadata(&self, diagram_id: &str, tools: &DiagramTools) -> Result<ResourceContent> {
        let diagram = tools.get_diagram(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found: {}", diagram_id))?;

        let node_count = diagram.elements.values()
            .filter(|e| e.element_type != "graph" && !e.element_type.contains("edge"))
            .count();
        
        let edge_count = diagram.elements.values()
            .filter(|e| e.element_type.contains("edge"))
            .count();

        let element_types: std::collections::HashMap<String, usize> = diagram.elements.values()
            .filter(|e| e.element_type != "graph")
            .fold(std::collections::HashMap::new(), |mut acc, element| {
                *acc.entry(element.element_type.clone()).or_insert(0) += 1;
                acc
            });

        Ok(ResourceContent {
            uri: format!("diagram://metadata/{}", diagram_id),
            mime_type: Some("application/json".to_string()),
            text: Some(json!({
                "id": diagram.id,
                "type": diagram.diagram_type,
                "revision": diagram.revision,
                "statistics": {
                    "totalElements": diagram.elements.len() - 1, // exclude root
                    "nodes": node_count,
                    "edges": edge_count,
                    "elementTypes": element_types
                },
                "lastModified": chrono::Utc::now().to_rfc3339()
            }).to_string()),
            blob: None,
        })
    }

    fn get_validation_results(&self, diagram_id: &str, tools: &DiagramTools) -> Result<ResourceContent> {
        let diagram = tools.get_diagram(diagram_id)
            .ok_or_else(|| anyhow::anyhow!("Diagram not found: {}", diagram_id))?;

        // Simple validation - check for disconnected nodes
        let mut issues = Vec::new();
        
        for (element_id, element) in &diagram.elements {
            if element.element_type != "graph" && !element.element_type.contains("edge") {
                // Check if node has any connections
                let has_connections = diagram.elements.values()
                    .any(|e| e.element_type.contains("edge") && 
                        e.properties.get("sourceId").and_then(|v| v.as_str()) == Some(element_id) ||
                        e.properties.get("targetId").and_then(|v| v.as_str()) == Some(element_id));

                if !has_connections && diagram.elements.len() > 2 { // more than just root and this element
                    issues.push(json!({
                        "elementId": element_id,
                        "severity": "warning",
                        "message": "Node has no connections",
                        "description": "This node is not connected to any other elements in the diagram"
                    }));
                }
            }
        }

        Ok(ResourceContent {
            uri: format!("diagram://validation/{}", diagram_id),
            mime_type: Some("application/json".to_string()),
            text: Some(json!({
                "diagramId": diagram_id,
                "isValid": issues.is_empty(),
                "issues": issues,
                "summary": {
                    "errors": issues.iter().filter(|i| i["severity"] == "error").count(),
                    "warnings": issues.iter().filter(|i| i["severity"] == "warning").count(),
                    "info": issues.iter().filter(|i| i["severity"] == "info").count()
                }
            }).to_string()),
            blob: None,
        })
    }

    fn get_wasm_components_list(&self, tools: &DiagramTools) -> String {
        // TODO: Get actual WASM components from file watcher
        let components = tools.get_wasm_components();
        
        let component_list: Vec<Value> = components.iter()
            .map(|component| json!({
                "name": component.name,
                "path": component.path,
                "description": component.description,
                "status": if component.file_exists { "available" } else { "missing" },
                "interfaces": component.interfaces.len(),
                "uri": format!("wasm://component/{}", component.name)
            }))
            .collect();

        json!({
            "components": component_list,
            "total": component_list.len(),
            "available": components.iter().filter(|c| c.file_exists).count(),
            "missing": components.iter().filter(|c| !c.file_exists).count()
        }).to_string()
    }

    fn get_missing_components(&self, tools: &DiagramTools) -> String {
        let components = tools.get_wasm_components();
        let missing: Vec<Value> = components.iter()
            .filter(|c| !c.file_exists)
            .map(|component| json!({
                "name": component.name,
                "path": component.path,
                "description": component.description,
                "lastSeen": component.last_seen,
                "removedAt": component.removed_at
            }))
            .collect();

        json!({
            "missingComponents": missing,
            "count": missing.len()
        }).to_string()
    }

    fn get_wasm_status(&self, tools: &DiagramTools) -> String {
        let components = tools.get_wasm_components();
        let total = components.len();
        let available = components.iter().filter(|c| c.file_exists).count();
        let missing = total - available;

        json!({
            "watchPath": tools.get_wasm_watch_path(),
            "totalComponents": total,
            "availableComponents": available,
            "missingComponents": missing,
            "healthPercentage": if total > 0 { (available as f64 / total as f64) * 100.0 } else { 100.0 },
            "lastScan": tools.get_last_wasm_scan_time(),
            "status": if missing == 0 { "healthy" } else if missing < total / 2 { "degraded" } else { "critical" }
        }).to_string()
    }

    fn get_wasm_component_details(&self, component_name: &str, tools: &DiagramTools) -> Result<ResourceContent> {
        let component = tools.get_wasm_component(component_name)
            .ok_or_else(|| anyhow::anyhow!("WASM component not found: {}", component_name))?;

        Ok(ResourceContent {
            uri: format!("wasm://component/{}", component_name),
            mime_type: Some("application/json".to_string()),
            text: Some(json!({
                "name": component.name,
                "path": component.path,
                "description": component.description,
                "fileExists": component.file_exists,
                "lastSeen": component.last_seen,
                "removedAt": component.removed_at,
                "interfaces": component.interfaces,
                "metadata": component.metadata,
                "witInterfaces": component.wit_interfaces,
                "dependencies": component.dependencies
            }).to_string()),
            blob: None,
        })
    }
}