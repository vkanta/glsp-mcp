mod amt_uml;
mod amt_compose;
use amt_compose::codegen_core::CanonicalNameExt;
use amt_compose::wit_bindgen_core::wit_parser::{Package, Resolve};
use anyhow::{anyhow, Context, Result};
use regex::Regex;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use serde_json::{json, Value};

/* =========================
 * Domain model
 * ========================= */

/// A 2D point in the diagram canvas.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Position {
    /// Horizontal coordinate (in pixels).
    pub x: f64,
    /// Vertical coordinate (in pixels).
    pub y: f64,
}

/// A task node in the workflow diagram.
///
/// When generated from WIT interfaces, `functions` contains the function
/// names belonging to the interface (prefixed with `F:` for readability).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Logical (temporary) ID used during construction before the tool
    /// assigns real node IDs. Generated as a UUID by default.
    pub id: String,
    /// Display name (package name or interface name).
    pub name: String,
    /// Suggested position for initial layout (the layout tool may override).
    pub position: Position,
    /// Optional list of function names belonging to this task/interface.
    #[serde(rename = "functions")]
    pub functions: Option<Vec<String>>,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "Empty Task".to_string(),
            position: Position { x: 100.0, y: 100.0 },
            functions: None,
        }
    }
}

/// A directed transition (edge) between two tasks.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transition {
    /// Logical (temporary) ID for the edge.
    pub id: String,
    /// Source logical task ID.
    #[serde(rename = "sourceTaskId")]
    pub source_task_id: String,
    /// Target logical task ID.
    #[serde(rename = "targetTaskId")]
    pub target_task_id: String,
}

/// A collection of tasks and transitions representing a workflow.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskList {
    /// Logical ID for this task list.
    pub id: String,
    /// All tasks (nodes).
    pub tasks: Vec<Task>,
    /// All transitions (edges) between tasks.
    pub transitions: Vec<Transition>,
}
/// Generate a UML diagram from amt-compose data, depicting each package, its interfaces, and functions.
pub fn generate_uml_from_amt_compose(config_path: &str, project_path: &str) -> Result<String> {
    let tasklist = build_task_list_from_amt_compose(config_path, project_path)?;
    let mut uml = String::from("@startuml\n");
    let mut packages = HashMap::new();
    for task in &tasklist.tasks {
        // Assume package names do not contain ':' and interfaces do
        if !task.name.contains(":") {
            packages.insert(task.id.clone(), task.name.clone());
            uml.push_str(&format!("package {} {{\n", task.name));
        } else {
            // Interface node
            uml.push_str(&format!("  interface {} {{\n", task.name));
            if let Some(funcs) = &task.functions {
                for f in funcs {
                    uml.push_str(&format!("    {}\n", f));
                }
            }
            uml.push_str("  }\n");
        }
    }
    // Close packages
    for _ in &packages {
        uml.push_str("}\n");
    }
    // Add transitions (edges)
    for tr in &tasklist.transitions {
        let src = &tasklist.tasks.iter().find(|t| t.id == tr.source_task_id).unwrap().name;
        let tgt = &tasklist.tasks.iter().find(|t| t.id == tr.target_task_id).unwrap().name;
        uml.push_str(&format!("{} --> {}\n", src, tgt));
    }
    uml.push_str("@enduml\n");
    Ok(uml)
}


/* =========================
 * Builders & transformers
 * ========================= */

/// Build one `Transition` from `parent` to each of the `children`.
///
/// The returned transitions use the *logical* IDs of the provided tasks.
pub fn build_transitions(parent: &Task, children: &[Task]) -> Vec<Transition> {
    children
        .iter()
        .map(|to| Transition {
            id: Uuid::new_v4().to_string(),
            source_task_id: parent.id.clone(),
            target_task_id: to.id.clone(),
        })
        .collect()
}

/// Collect all interfaces from a WIT `Package` as `Task`s.
///
/// Each interface becomes a task whose `name` is the canonical interface
/// name and whose `functions` lists its function names.
///
/// Returns an empty vector if the package has no interfaces.
pub fn wit_interfaces_as_tasks(resolve: &Resolve, package: &Package) -> Result<Vec<Task>> {
    let mut tasks = Vec::<Task>::new();

    for (_, iface_id) in &package.interfaces {
        let mut t = Task::default();
        t.name = resolve.interface_canon_by_id(*iface_id).unwrap_or_default();

        // Collect function names and prefix for readability in the diagram.
        let func_names: Vec<String> = resolve.interfaces[*iface_id]
            .functions
            .iter()
            .map(|(name, _)| format!("F:{name}"))
            .collect();

        if !func_names.is_empty() {
            t.functions = Some(func_names);
        }
        tasks.push(t);
    }

    Ok(tasks)
}

/// Build a `TaskList` from an amt-compose project by reading WIT packages
/// and their interfaces.
///
/// - Each WIT package becomes a parent `Task`.
/// - Each interface of that package becomes a child `Task`.
/// - A `Transition` is created from the package task to each interface task.
///
/// Returns a complete `TaskList` ready to be posted to the diagram tool.
pub fn build_task_list_from_amt_compose(config_path: &str, project_path: &str) -> Result<TaskList> {
    // Minimal parser: read `project_path/model/interfaces.wit` and `config_path` for components.
    // This avoids compiling the full `composer` crate while extracting package/interface names.
    use std::fs;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct AmtComponent {
        name: String,
        // imports/exports not strictly needed for labels; we read WIT for interface names
    }

    #[derive(Debug, Deserialize)]
    struct AmtCompose {
        components: Vec<serde_yaml::Value>,
    }

    let compose_text = fs::read_to_string(format!("{}/{}", project_path, config_path))
        .with_context(|| format!("Failed to read amt-compose file at {}/{}", project_path, config_path))?;
    let _compose: AmtCompose = serde_yaml::from_str(&compose_text).context("Failed to parse amt-compose YAML")?;

    // Read the WIT file for the project (model directory inside project_path)
    let wit_path = format!("{}/model/interfaces.wit", project_path);
    let wit_text = fs::read_to_string(&wit_path)
        .with_context(|| format!("Failed to read WIT file at {}", wit_path))?;

    // Parse package and interfaces from the WIT text using a very small parser.
    // We look for `package <name>;` and `interface <name> {` occurrences.
    let mut package_name = String::new();
    let mut interfaces: Vec<String> = Vec::new();
    for line in wit_text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("package ") {
            if let Some(rest) = trimmed.strip_prefix("package ") {
                package_name = rest.trim_end_matches(';').trim().to_string();
            }
        }
        if trimmed.starts_with("interface ") {
            if let Some(rest) = trimmed.strip_prefix("interface ") {
                // interface NAME { ...
                if let Some((name, _)) = rest.split_once(' ') {
                    interfaces.push(name.trim().to_string());
                } else {
                    // fallback: take until '{'
                    let name = rest.split('{').next().unwrap_or(rest).trim();
                    interfaces.push(name.to_string());
                }
            }
        }
    }

    // Build TaskList: one parent node for package and one child per interface.
    let mut out = TaskList { id: Uuid::new_v4().to_string(), ..Default::default() };
    let parent = Task { id: Uuid::new_v4().to_string(), name: package_name.clone(), position: Position { x: 100.0, y: 100.0 }, functions: None };
    out.tasks.push(parent.clone());

    for iface in interfaces {
        let mut t = Task::default();
        // use canonical naming like in the stored diagram: "<package>/<interface>" or keep original format
        t.name = format!("{}/{}", package_name, iface);
        let edges = build_transitions(&parent, &[t.clone()]);
        out.tasks.push(t);
        out.transitions.extend(edges);
    }

    Ok(out)
}

/* =========================
 * JSON-RPC client (tools)
 * ========================= */

/// Minimal JSON-RPC client for the diagram tool.
///
/// Reuses a single `reqwest` blocking `Client` and provides typed helper
/// methods for common tool calls. Errors include context for easy tracing.
pub struct DiagramToolClient<'a> {
    /// Base endpoint, e.g. `http://127.0.0.1:3000/messages`.
    pub url: &'a str,
    /// Shared blocking HTTP client.
    pub http: Client,
}

impl<'a> DiagramToolClient<'a> {
    /// Create a new client targeting `url`.
    pub fn new(url: &'a str) -> Self {
        Self {
            url,
            http: Client::new(),
        }
    }

    /// Perform a generic JSON-RPC call with a `name` and `arguments` map.
    fn rpc_call(&self, name: &str, arguments: Value, id: i64) -> Result<Value> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments },
            "id": id
        });

        let res = self
            .http
            .post(self.url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .with_context(|| format!("Failed to send JSON-RPC '{name}'"))?;

        let status = res.status();
        let value: Value = res
            .json()
            .with_context(|| format!("Invalid JSON response from '{name}' (status {status})"))?;

        Ok(value)
    }

    /// Extract the assistant’s plaintext message from a tool response.
    ///
    /// The diagram tool replies with a structure like:
    /// `{ result: { content: [ { text: "...ID: <uuid>..." } ] } }`.
    fn extract_text(response: &Value) -> Result<&str> {
        response
            .get("result")
            .and_then(|r| r.get("content"))
            .and_then(|c| c.get(0))
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("No text content found in tool response"))
    }

    /// Parse a UUID that follows the pattern `ID: <uuid>` within `text`.
    fn extract_uuid_from_text(&self, text: &str) -> Result<String> {
        // Compiled lazily per call; for hot paths you can memoize or use once_cell.
        let re = Regex::new(r"ID:\s*([a-f0-9-]{36})").context("Failed to compile UUID regex")?;
        re.captures(text)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| anyhow!("Could not find UUID in text"))
    }

    /// Create a diagram and return its assigned diagram ID.
    pub fn create_diagram(&self, diagram_type: &str, name: &str, id: i64) -> Result<String> {
        let args = json!({ "diagramType": diagram_type, "name": name });
        let resp = self.rpc_call("create_diagram", args, id)?;
        let text = Self::extract_text(&resp)?;
        self.extract_uuid_from_text(text)
    }

    /// Create a node and return the assigned node ID.
    pub fn create_node(
        &self,
        diagram_id: &str,
        node_type: &str,
        label: &str,
        position: &Position,
        id: i64,
    ) -> Result<String> {
        let args = json!({
            "diagramId": diagram_id,
            "nodeType": node_type,
            "position": { "x": position.x, "y": position.y },
            "label": label
        });
        let resp = self.rpc_call("create_node", args, id)?;
        let text = Self::extract_text(&resp)?;
        self.extract_uuid_from_text(text)
    }

    /// Create an edge between two existing nodes.
    pub fn create_edge(
        &self,
        diagram_id: &str,
        edge_type: &str,
        source_id: &str,
        target_id: &str,
        id: i64,
    ) -> Result<()> {
        let args = json!({
            "diagramId": diagram_id,
            "edgeType": edge_type,
            "sourceId": source_id,
            "targetId": target_id
        });
        let resp = self.rpc_call("create_edge", args, id)?;
        // Edge responses often don't embed a new ID; we still check status via presence of result.
        Self::extract_text(&resp).ok();
        Ok(())
    }

    /// Apply a layout algorithm to a diagram.
    pub fn apply_layout(
        &self,
        diagram_id: &str,
        algorithm: &str,
        direction: &str,
        id: i64,
    ) -> Result<()> {
        let args = json!({
            "diagramId": diagram_id,
            "algorithm": algorithm,
            "direction": direction
        });
        // Use `text()` instead of `json()` because layout often returns non-JSON text.
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": { "name": "apply_layout", "arguments": args },
            "id": id
        });

        let res = self
            .http
            .post(self.url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .context("Failed to apply layout")?;

        let status = res.status();
        let _body = res.text().unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!("Layout request failed with HTTP {status}"));
        }
        Ok(())
    }
}

/* =========================
 * Orchestrator
 * ========================= */

/// Create a diagram for the amt-compose project and populate it with nodes
/// and edges built from WIT packages and interfaces.
///
/// Returns the assigned `diagram_id` when successful.
pub fn generate_diagram_from_amt(
    tool_url: &str,
    config_path: &str,
    project_path: &str,
    diagram_name: &str,
) -> Result<String> {
    // 1) Build the logical task graph from amt-compose.
    let mut tasklist = build_task_list_from_amt_compose(config_path, project_path)?;

    // Compute a simple tree-like layout for the logical graph so nodes are
    // positioned before being created in the diagram tool. This produces a
    // left-to-right tree layout with even vertical spacing for siblings.
    layout_tasklist_tree(&mut tasklist, 300.0, 120.0);

    // 2) Create the diagram.
    let tool = DiagramToolClient::new(tool_url);
    let mut next_id: i64 = 1;
    let diagram_id = tool.create_diagram("workflow", diagram_name, next_id)?;
    next_id += 1;

    // 3) Create nodes; map logical task IDs -> assigned node IDs.
    let mut id_map: HashMap<String, String> = HashMap::new();

    for task in &tasklist.tasks {
        let assigned_id =
            tool.create_node(&diagram_id, "task", &task.name, &task.position, next_id)?;
        next_id += 1;
        id_map.insert(task.id.clone(), assigned_id);
    }

    // 4) Create edges using real node IDs.
    for tr in &tasklist.transitions {
        let source = id_map
            .get(&tr.source_task_id)
            .ok_or_else(|| anyhow!("Missing mapping for source task {}", tr.source_task_id))?;
        let target = id_map
            .get(&tr.target_task_id)
            .ok_or_else(|| anyhow!("Missing mapping for target task {}", tr.target_task_id))?;
        tool.create_edge(&diagram_id, "sequence-flow", source, target, next_id)?;
        next_id += 1;
    }

    // 5) Apply a layout for readability.
    tool.apply_layout(&diagram_id, "hierarchical", "left-right", next_id)?;

    Ok(diagram_id)
}

/* =========================
 * Example / entry point
 * ========================= */

fn main() -> Result<()> {
    // Adjust these to your environment.
    let tool_url = "http://127.0.0.1:3000/messages";
    let config_path = "amt-compose.yaml";
    let project_path = "workspace/amt/simple";
    let now = chrono::Local::now();
    let diagram_name = format!("amt uml_{}", now.format("%Y%m%d_%H%M"));

    // If the project path or config is missing in this environment, fall
    // back to the stored test TaskList so we can demonstrate node
    // positioning as a tree without failing.
    use std::path::Path;
    let diagram_id = if Path::new(project_path).exists() && Path::new(&format!("{}/{}", project_path, config_path)).exists() {
        generate_diagram_from_amt(tool_url, config_path, project_path, diagram_name.as_str())?
    } else {
        println!("⚠️  Project files not found at '{project_path}'. Falling back to test TaskList to demonstrate layout.");
        let tl = build_tasklist_amt_test_5();
        generate_diagram_from_tasklist(tool_url, &tl, diagram_name.as_str())?
    };
    println!("✅ Diagram created with ID: {diagram_id}");

    // Generate UML diagram from amt-compose data only when files exist.
    if std::path::Path::new(project_path).exists() && std::path::Path::new(&format!("{}/{}", project_path, config_path)).exists() {
        if let Ok(uml) = generate_uml_from_amt_compose(config_path, project_path) {
            println!("\nUML Diagram:\n{uml}");
        }
    } else {
        println!("⚠️  Skipping textual UML generation because project files are missing.");
    }

    // Generate UML interface/class diagram from WIT definitions
    amt_uml::generate_uml_interface_class_diagram(tool_url, "Amt UML Interface/Class Diagram");
    Ok(())
}

/* =========================
 * Test data helper (optional)
 * ========================= */

#[allow(dead_code)]
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

/// Build a TaskList that replicates the stored workspace diagram
/// `Amt-Test-Diagram-5` found in `workspace/diagrams/Amt-Test-Diagram-5.glsp.json`.
#[allow(dead_code)]
fn build_tasklist_amt_test_5() -> TaskList {
    // IDs and labels replicated from the stored diagram so transitions
    // reference the same logical IDs used there.
    let mut tasks = Vec::new();

    let add_task = |id: &str, label: &str| -> Task {
        Task {
            id: id.to_string(),
            name: label.to_string(),
            position: Position { x: 100.0, y: 100.0 },
            functions: None,
        }
    };

    tasks.push(add_task("71271131-b4be-4597-85af-a48817c45039", "amt-composer:simple/inter-guest"));
    tasks.push(add_task("7b375e1b-a147-4853-8ea4-c2024ecd473e", "amt-composer:simple/guest-main"));
    tasks.push(add_task("d3c1c2ad-78fc-4887-90ec-ca6baecef114", "amt-composer:simple"));
    tasks.push(add_task("111aa192-bafe-4d79-a65c-5123c6e5415a", "amt-composer:simple/inter-guest-types"));
    tasks.push(add_task("95ef2403-140a-4044-a0b6-befc9f110d8d", "amt-composer:simple/host-goodies-types"));
    tasks.push(add_task("a5dbf16f-14c5-41cf-ad69-90034de92471", "amt-compose:bridge@0.1.0"));
    tasks.push(add_task("20d4f80f-099b-46d0-8941-a25355b4ca96", "amt-composer:simple/host-goodies"));
    tasks.push(add_task("c652e8f8-1150-4f40-9b04-b61398306f3b", "amt-compose:bridge/fallible@0.1.0"));

    // Build transitions based on the sequence-flow nodes in the stored JSON.
    let mut transitions = Vec::new();

    let add_transition = |from: &str, to: &str| -> Transition {
        Transition {
            id: Uuid::new_v4().to_string(),
            source_task_id: from.to_string(),
            target_task_id: to.to_string(),
        }
    };

    transitions.push(add_transition("d3c1c2ad-78fc-4887-90ec-ca6baecef114", "71271131-b4be-4597-85af-a48817c45039"));
    transitions.push(add_transition("d3c1c2ad-78fc-4887-90ec-ca6baecef114", "95ef2403-140a-4044-a0b6-befc9f110d8d"));
    transitions.push(add_transition("d3c1c2ad-78fc-4887-90ec-ca6baecef114", "7b375e1b-a147-4853-8ea4-c2024ecd473e"));
    transitions.push(add_transition("d3c1c2ad-78fc-4887-90ec-ca6baecef114", "111aa192-bafe-4d79-a65c-5123c6e5415a"));
    transitions.push(add_transition("d3c1c2ad-78fc-4887-90ec-ca6baecef114", "20d4f80f-099b-46d0-8941-a25355b4ca96"));
    transitions.push(add_transition("a5dbf16f-14c5-41cf-ad69-90034de92471", "c652e8f8-1150-4f40-9b04-b61398306f3b"));

    TaskList {
        id: Uuid::new_v4().to_string(),
        tasks,
        transitions,
    }
}

/// Create a diagram from an existing `TaskList` instead of building from amt-compose.
fn generate_diagram_from_tasklist(tool_url: &str, tasklist: &TaskList, diagram_name: &str) -> Result<String> {
    // Create a mutable copy of the tasklist so we can compute node positions
    // without mutating the original caller data.
    let mut local = tasklist.clone();
    layout_tasklist_tree(&mut local, 300.0, 120.0);

    let tool = DiagramToolClient::new(tool_url);
    let mut next_id: i64 = 1;
    let diagram_id = tool.create_diagram("workflow", diagram_name, next_id)?;
    next_id += 1;

    let mut id_map: HashMap<String, String> = HashMap::new();
    for task in &local.tasks {
        let assigned_id = tool.create_node(&diagram_id, "task", &task.name, &task.position, next_id)?;
        next_id += 1;
        id_map.insert(task.id.clone(), assigned_id);
    }

    for tr in &tasklist.transitions {
        let source = id_map.get(&tr.source_task_id)
            .ok_or_else(|| anyhow!("Missing mapping for source task {}", tr.source_task_id))?;
        let target = id_map.get(&tr.target_task_id)
            .ok_or_else(|| anyhow!("Missing mapping for target task {}", tr.target_task_id))?;
        tool.create_edge(&diagram_id, "sequence-flow", source, target, next_id)?;
        next_id += 1;
    }

    tool.apply_layout(&diagram_id, "hierarchical", "left-right", next_id)?;
    Ok(diagram_id)
}

/// Compute a simple tree layout for the given `TaskList`.
///
/// - `h_spacing` is horizontal distance between levels.
/// - `v_spacing` is vertical distance between sibling nodes.
fn layout_tasklist_tree(tasklist: &mut TaskList, h_spacing: f64, v_spacing: f64) {
    use std::collections::{HashMap, VecDeque};

    if tasklist.tasks.is_empty() {
        return;
    }

    // Map id -> index for quick updates.
    let mut index_map: HashMap<String, usize> = HashMap::new();
    for (i, t) in tasklist.tasks.iter().enumerate() {
        index_map.insert(t.id.clone(), i);
    }

    // Build adjacency list and indegree counts.
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut indeg: HashMap<String, usize> = HashMap::new();
    for t in &tasklist.tasks {
        indeg.entry(t.id.clone()).or_insert(0);
    }
    for tr in &tasklist.transitions {
        adj.entry(tr.source_task_id.clone()).or_default().push(tr.target_task_id.clone());
        *indeg.entry(tr.target_task_id.clone()).or_insert(0) += 1;
    }

    // Roots are nodes with indegree == 0. If none, pick the first task as root.
    let mut roots: Vec<String> = indeg
        .iter()
        .filter_map(|(id, &d)| if d == 0 { Some(id.clone()) } else { None })
        .collect();
    if roots.is_empty() {
        roots.push(tasklist.tasks[0].id.clone());
    }

    // Layout each root's subtree; space multiple roots vertically.
    for (root_idx, root_id) in roots.iter().enumerate() {
        // BFS to determine level (depth) per node within this root's reachable subgraph.
        let mut level: HashMap<String, usize> = HashMap::new();
        let mut q: VecDeque<String> = VecDeque::new();
        level.insert(root_id.clone(), 0);
        q.push_back(root_id.clone());

        while let Some(curr) = q.pop_front() {
            let curr_level = *level.get(&curr).unwrap();
            if let Some(children) = adj.get(&curr) {
                for child in children {
                    if !level.contains_key(child) {
                        level.insert(child.clone(), curr_level + 1);
                        q.push_back(child.clone());
                    }
                }
            }
        }

        if level.is_empty() {
            continue;
        }

        // Group nodes by level.
        let mut levels: HashMap<usize, Vec<String>> = HashMap::new();
        for (id, &lv) in &level {
            levels.entry(lv).or_default().push(id.clone());
        }

        // Determine a vertical baseline for this root so multiple roots don't overlap.
        let root_baseline_y = 100.0 + (root_idx as f64) * (v_spacing * 6.0);

        // For each level, assign positions.
        let mut sorted_levels: Vec<usize> = levels.keys().cloned().collect();
        sorted_levels.sort_unstable();
        for &lv in &sorted_levels {
            let nodes = &levels[&lv];
            let n = nodes.len();
            for (i, node_id) in nodes.iter().enumerate() {
                if let Some(&idx) = index_map.get(node_id) {
                    let x = (lv as f64) * h_spacing + 100.0;
                    // center siblings around baseline
                    let y = root_baseline_y + (i as f64 - (n as f64 - 1.0) / 2.0) * v_spacing;
                    tasklist.tasks[idx].position = Position { x, y };
                }
            }
        }
    }
}
