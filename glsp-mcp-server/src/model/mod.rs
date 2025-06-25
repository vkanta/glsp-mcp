use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Core diagram model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramModel {
    pub id: String,
    pub diagram_type: String,
    pub revision: u32,
    pub root: ModelElement,
    pub elements: HashMap<String, ModelElement>,
}

/// Base model element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelElement {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<Bounds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_options: Option<HashMap<String, serde_json::Value>>,
    #[serde(flatten)]
    pub properties: HashMap<String, serde_json::Value>,
}

/// Element bounds/position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Position coordinate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Node element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    #[serde(flatten)]
    pub base: ModelElement,
    pub position: Position,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<Size>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Size dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

/// Edge element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    #[serde(flatten)]
    pub base: ModelElement,
    #[serde(rename = "sourceId")]
    pub source_id: String,
    #[serde(rename = "targetId")]
    pub target_id: String,
    #[serde(rename = "routingPoints", skip_serializing_if = "Option::is_none")]
    pub routing_points: Option<Vec<Position>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Validation marker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marker {
    pub label: String,
    pub description: String,
    #[serde(rename = "elementId")]
    pub element_id: String,
    pub kind: String,
    pub severity: MarkerSeverity,
}

/// Marker severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkerSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl DiagramModel {
    pub fn new(diagram_type: &str) -> Self {
        let id = Uuid::new_v4().to_string();
        let root_id = format!("{}_root", id);
        
        let root = ModelElement {
            id: root_id.clone(),
            element_type: "graph".to_string(),
            children: Some(Vec::new()),
            bounds: None,
            layout_options: None,
            properties: HashMap::new(),
        };

        let mut elements = HashMap::new();
        elements.insert(root_id.clone(), root.clone());

        Self {
            id,
            diagram_type: diagram_type.to_string(),
            revision: 0,
            root,
            elements,
        }
    }

    pub fn add_element(&mut self, element: ModelElement) {
        self.elements.insert(element.id.clone(), element);
        self.revision += 1;
    }

    pub fn remove_element(&mut self, element_id: &str) -> Option<ModelElement> {
        let removed = self.elements.remove(element_id);
        if removed.is_some() {
            self.revision += 1;
        }
        removed
    }

    pub fn get_element(&self, element_id: &str) -> Option<&ModelElement> {
        self.elements.get(element_id)
    }

    pub fn get_element_mut(&mut self, element_id: &str) -> Option<&mut ModelElement> {
        self.elements.get_mut(element_id)
    }

    pub fn add_child_to_root(&mut self, child_id: &str) {
        if let Some(children) = &mut self.root.children {
            if !children.contains(&child_id.to_string()) {
                children.push(child_id.to_string());
            }
        } else {
            self.root.children = Some(vec![child_id.to_string()]);
        }
        self.revision += 1;
    }
}

impl Node {
    pub fn new(node_type: &str, position: Position, label: Option<String>) -> Self {
        let id = Uuid::new_v4().to_string();
        let mut properties = HashMap::new();
        
        if let Some(ref label_text) = label {
            properties.insert("label".to_string(), serde_json::Value::String(label_text.clone()));
        }

        Self {
            base: ModelElement {
                id: id.clone(),
                element_type: node_type.to_string(),
                children: None,
                bounds: Some(Bounds {
                    x: position.x,
                    y: position.y,
                    width: 100.0,
                    height: 50.0,
                }),
                layout_options: None,
                properties,
            },
            position,
            size: Some(Size {
                width: 100.0,
                height: 50.0,
            }),
            label,
        }
    }
}

impl Edge {
    pub fn new(edge_type: &str, source_id: String, target_id: String, label: Option<String>) -> Self {
        let id = Uuid::new_v4().to_string();
        let mut properties = HashMap::new();
        
        if let Some(ref label_text) = label {
            properties.insert("label".to_string(), serde_json::Value::String(label_text.clone()));
        }

        Self {
            base: ModelElement {
                id: id.clone(),
                element_type: edge_type.to_string(),
                children: None,
                bounds: None,
                layout_options: None,
                properties,
            },
            source_id,
            target_id,
            routing_points: None,
            label,
        }
    }
}