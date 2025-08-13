use amt_compose::codegen_core::CanonicalNameExt;
use amt_compose::wit_bindgen_core::wit_parser::{Package, Resolve};
use anyhow::Error;
use regex::Regex;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub position: Position,
    #[serde(rename = "functions")]
    pub functions: Option<Vec<String>>,
}
impl Default for Task {
    fn default() -> Self {
        self::Task {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Empty Task".to_string(),
            position: Position { x: 100.0, y: 100.0 },
            functions: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transition {
    pub id: String,
    #[serde(rename = "sourceTaskId")]
    pub source_task_id: String,
    #[serde(rename = "targetTaskId")]
    pub target_task_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskList {
    pub id: String,
    pub tasks: Vec<Task>,
    pub transitions: Vec<Transition>,
}
fn build_transitions(parent: &Task, children: &Vec<Task>) -> Vec<Transition> {
    let edges: Vec<Transition> = children
        .iter()
        .map(|to| Transition {
            source_task_id: parent.id.clone(),
            target_task_id: to.id.clone(),
            id: Uuid::new_v4().to_string(),
        })
        .collect();
    dbg!(edges)
}

fn get_wit_interfaces(r: Resolve, p: &Package) -> Result<Vec<Task>, Error> {
    let mut the_interfaces = Vec::<Task>::new();
    for (_, id) in &p.interfaces {
        let mut i_task = Task::default(); // this task contains the interface data
        i_task.name = r.interface_canon_by_id(*id).unwrap_or_default();
        let functions = &r.interfaces[*id].functions;
        let func_names: Vec<String> = functions
            .into_iter()
            .map(|(name, _)| "F:".to_owned() + name)
            .collect();
        i_task.functions = Some(func_names); // add the function names to the interface
        the_interfaces.push(i_task);
    }
    Ok(the_interfaces) //Return the interfaces collection
}

/// this function collects data from the amt compose
/// returns a tasklist to be displayed
fn amt_compose_task_list() -> Result<TaskList, Error> {
    use amt_compose::{core::LogTracker, project::ProjectContext};

    let config_path = "amt-compose.yaml"; // relative (to project_path) or absolute
    let project_path = "/home/vkanta/wspaces/glsp-mcp/workspace/amt/simple";
    // relative (to current directory) or absolute
    let logger = LogTracker::new_boxed(); // forward messages to the `log` facade
    let quiet = true;

    let project = ProjectContext::new(config_path, project_path, quiet, logger);
    let binding = project?;
    let resolve = binding.wit().resolve();
    // let packages = wp::get_packages(resolve);
    let packages = resolve.packages.clone();
    //create the task list to store everything.
    let mut updated_task_list = TaskList::default();
    for package in packages {
        let mut task = Task::default();
        task.name = package.1.name.to_string();
        let parrent_task = task.clone();
        updated_task_list.tasks.push(task.clone());
        let interfaces = get_wit_interfaces(resolve.clone(), &package.1).unwrap();
        updated_task_list.tasks.extend(interfaces.clone());
        updated_task_list.transitions = build_transitions(&parrent_task, &interfaces);
    }

    Ok(updated_task_list)
}
//----
///
fn main() {
    let client = Client::new();
    let url = "http://127.0.0.1:3000/messages";

    // Step 1: Create diagram
    let create_diagram = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "create_diagram",
            "arguments": {
                "diagramType": "workflow",
                "name": "Amt-Test-Diagram-5"
            }
        },
        "id": 1
    });

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&create_diagram)
        .send()
        .expect("Failed to send create_diagram request");

    let result: Value = response.json().expect("Invalid JSON response");
    println!("Full response from create_diagram:\n{:#}", result);

    let text_msg = result
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.get(0))
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or_else(|| panic!("No diagram creation text found"));

    let diagram_id = extract_diagram_id(text_msg)
        .unwrap_or_else(|| panic!("Failed to extract diagramId from text"));

    println!("✅ Extracted diagram ID: {}", diagram_id);

    // Step 2: Add tasks and collect real node IDs
    // let tasklist = sample_tasklist();
    let tasklist = amt_compose_task_list().unwrap();
    let mut id_counter = 2;
    let mut task_id_map: HashMap<String, String> = HashMap::new();

    for task in tasklist.tasks {
        let create_node = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "create_node",
                "arguments": {
                    "diagramId": diagram_id,
                    "nodeType": "task",
                    "position": {
                        "x": task.position.x,
                        "y": task.position.y
                    },
                    "label": task.name
                }
            },
            "id": id_counter
        });

        id_counter += 1;

        let res = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&create_node)
            .send()
            .expect("Failed to send create_node");

        let result: Value = res.json().expect("Invalid JSON response");

        println!("Node response: {:#}", result);

        let text_msg = result
            .get("result")
            .and_then(|r| r.get("content"))
            .and_then(|c| c.get(0))
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or_else(|| panic!("No create_node response text"));

        let assigned_id = extract_node_id(text_msg)
            .unwrap_or_else(|| panic!("Could not extract assigned node ID"));

        task_id_map.insert(task.id.clone(), assigned_id);
    }

    // Step 3: Create edges using the real assigned node IDs
    for transition in tasklist.transitions {
        let source_id = task_id_map
            .get(&transition.source_task_id)
            .unwrap_or_else(|| panic!("Missing source_task_id mapping"));
        let target_id = task_id_map
            .get(&transition.target_task_id)
            .unwrap_or_else(|| panic!("Missing target_task_id mapping"));

        let create_edge = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "create_edge",
                "arguments": {
                    "diagramId": diagram_id,
                    "edgeType": "sequence-flow",
                    "sourceId": source_id,
                    "targetId":target_id
                }
            },
            "id": id_counter
        });

        id_counter += 1;

        let res = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&create_edge)
            .send()
            .expect("Failed to send create_edge");

        let status = res.status();
        let body = res.text().unwrap_or_default();

        println!("Edge response: (status {}) {}", status, body);
    }
    //Step 4 aply layout
    let apply_layout = json!({
      "jsonrpc": "2.0",
      "method": "tools/call",
      "params": {
        "name": "apply_layout",
        "arguments": {
          "diagramId": diagram_id,
          "algorithm": "hierarchical",
          "direction": "left-right"
        }
      },
      "id": 6
    });

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&apply_layout)
        .send()
        .expect("Failed to apply layout");

    let status = res.status();
    let body = res.text().unwrap_or_default();

    println!("Apply layout response: (status {}) {}", status, body);
}

fn extract_diagram_id(text: &str) -> Option<String> {
    let re = Regex::new(r"ID: ([a-f0-9-]{36})").ok()?;
    re.captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

fn extract_node_id(text: &str) -> Option<String> {
    let re = Regex::new(r"ID: ([a-f0-9-]{36})").ok()?;
    re.captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}
#[warn(dead_code)]
fn sample_tasklist() -> TaskList {
    TaskList {
        id: Uuid::new_v4().to_string(),
        tasks: vec![
            Task {
                id: "node-1".into(),
                name: "Start".into(),
                position: Position { x: 100.0, y: 100.0 },
                functions: None,
            },
            Task {
                id: "node-2".into(),
                name: "Process".into(),
                position: Position { x: 300.0, y: 200.0 },
                functions: None,
            },
        ],
        transitions: vec![Transition {
            id: "edge-1".into(),
            source_task_id: "node-1".into(),
            target_task_id: "node-2".into(),
        }],
    }
}
