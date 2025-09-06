use reqwest::blocking::Client;
use serde_json::json;
use serde_json::Value;

/// Generates a UML interface/class diagram from WIT definitions using the MCP diagram tool API.
pub fn generate_uml_interface_class_diagram(tool_url: &str, diagram_name: &str) {
    let client = Client::new();
    // Step 1: Create UML diagram
    let create_diagram = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "create_diagram",
            "arguments": {
                "diagramType": "uml",
                "name": diagram_name
            }
        },
        "id": 1
    });
    let response = client
        .post(tool_url)
        .header("Content-Type", "application/json")
        .json(&create_diagram)
        .send()
        .expect("Failed to send create_diagram request");
    let result: Value = response.json().expect("Invalid JSON response");
    let text_msg = result
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.get(0))
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or_else(|| panic!("No diagram creation text found"));
    let diagram_id = text_msg.split("ID: ").nth(1).unwrap_or("").trim();
    println!("✅ UML Diagram ID: {}", diagram_id);

    // Step 2: Add UML interface node for 'math'
    let math_interface = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "add_uml_interface",
            "arguments": {
                "diagramId": diagram_id,
                "name": "math",
                "methods": [
                    {"name": "add", "signature": "(a: f32, b: f32) -> f32"},
                    {"name": "subtract", "signature": "(a: f32, b: f32) -> f32"},
                    {"name": "multiply", "signature": "(a: f32, b: f32) -> f32"},
                    {"name": "divide", "signature": "(a: f32, b: f32) -> result<f32, string>"}
                ],
                "position": {"x": 100.0, "y": 100.0}
            }
        },
        "id": 2
    });
    client.post(tool_url).header("Content-Type", "application/json").json(&math_interface).send().expect("Failed to add math interface");

    // Step 3: Add UML interface node for 'user'
    let user_interface = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "add_uml_interface",
            "arguments": {
                "diagramId": diagram_id,
                "name": "user",
                "methods": [
                    {"name": "get-user", "signature": "(id: u32) -> option<User>"},
                    {"name": "create-user", "signature": "(name: string) -> User"}
                ],
                "position": {"x": 350.0, "y": 100.0}
            }
        },
        "id": 3
    });
    client.post(tool_url).header("Content-Type", "application/json").json(&user_interface).send().expect("Failed to add user interface");

    // Step 4: Add UML class node for 'User'
    let user_class = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "add_uml_class",
            "arguments": {
                "diagramId": diagram_id,
                "name": "User",
                "attributes": [
                    {"name": "id", "type": "u32", "visibility": "public"},
                    {"name": "name", "type": "string", "visibility": "public"}
                ],
                "position": {"x": 225.0, "y": 250.0}
            }
        },
        "id": 4
    });
    client.post(tool_url).header("Content-Type", "application/json").json(&user_class).send().expect("Failed to add User class");

    println!("✅ UML interface/class diagram generated.");
}
