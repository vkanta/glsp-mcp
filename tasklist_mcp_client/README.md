# 🛠 Tasklist MCP Client

This Rust client connects to an MCP-compatible server via JSON-RPC to:

- Create a new diagram
- Add task nodes
- Connect them with transitions (edges)

It works with the [WASM Component Designer](https://github.com/eclipse-glsp/glsp-mcp) or any GLSP-based frontend that supports MCP.

---

## 🚀 Features

- ✅ Creates workflow diagrams via MCP JSON-RPC
- ✅ Dynamically maps server-assigned node IDs
- ✅ Visualizes task nodes and connections
- ✅ Easy to extend or adapt to new diagram types

---

## 🧪 Screenshot

The image below shows the generated diagram rendered in the WASM Component Designer:

![WASM Component Designer Screenshot](Screenshot_20250806_164929.png)

---

## 🔧 Usage

### Prerequisites

- Rust installed (`cargo`)
- MCP server running at `http://127.0.0.1:3000/messages`
- WASM Component Designer frontend running (e.g., on `http://localhost:5173`)

### Run the client

```bash
cargo run


📦 Dependencies
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.5", features = ["v4"] }
reqwest = { version = "0.11", features = ["blocking", "json"] }
regex = "1.10"


📁 Project Structure

src/
├── main.rs         # Sends JSON-RPC calls to create the diagram
├── tasklist.rs     # (optional) TaskList data model if modularized
e9a74450-277e-4662-a28b-511a8eecaa84.png  # Screenshot image

What changed & why
DRY JSON-RPC: DiagramToolClient centralizes request/response handling, text extraction, and UUID parsing, replacing repeated blocks.

Unified UUID parsing: extract_uuid_from_text replaces the two nearly identical extract_*_id functions.

Borrow instead of clone: wit_interfaces_as_tasks(&Resolve, &Package) and iteration over resolve.packages.iter() avoid unnecessary moves.

Smaller, focused functions: build_task_list_from_amt_compose builds the graph; generate_diagram_from_amt drives the whole flow.

Error handling: Removed unwrap/expect; added anyhow::Context for traceable failures.

Docs: Added /// comments for all public structs and functions.
