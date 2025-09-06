Amt‑compose integration and build report

Summary

This document lists the system packages installed, workspace-level edits, and the small runtime verification I ran so the `tasklist_mcp_client` can build and generate UML diagrams without blocking on the (broken) `composer` crate.

1) System packages installed (OS / apt)

- libgtk-3-dev
- libsoup2.4-dev
- libjavascriptcoregtk-4.1-dev
- pkg-config adjustments / symlink for javascriptcoregtk .pc when needed
- exported PKG_CONFIG_PATH so pkg-config finds the .pc files (example: export PKG_CONFIG_PATH=/usr/lib/pkgconfig)

Notes: those packages are required by GTK/WebKit crates used in the workspace (tauri/glsp desktop bits). A small symlink was created in one iteration to bridge `javascriptcoregtk-4.1.pc` ↔ `javascriptcoregtk-4.0.pc` on this system when pkg-config version lookups mismatched.

2) Workspace / code changes made

Files added
- `tasklist_mcp_client/src/amt_compose.rs` — a small local stub that exposes a minimal API so `tasklist_mcp_client` can compile without building the `composer` crate. The stub provides `ProjectContext::new`, `ProjectContext::wit()`, and a simple `Resolve`/`interface_canon_by_id` surface.

Files edited
- `tasklist_mcp_client/Cargo.toml`
  - Fixed a malformed manifest and added necessary dependencies used by the client: chrono, reqwest (blocking + json), serde (derive), serde_json, uuid, regex, anyhow.

- `tasklist_mcp_client/src/main.rs`
  - Adjusted config/project path handling so it reliably finds `workspace/amt/simple/amt-compose.yaml` when run from `tasklist_mcp_client` folder.
  - Added chrono-based diagram naming: `amt uml_<YYYYMMDD_HHMM>`.
  - Fixed an argument type mismatch by passing `&resolve` to `wit_interfaces_as_tasks`.
  - Added `mod amt_compose;` and `mod amt_uml;` usage points (the latter wires UML generation code already present).

- `glsp-mcp-server/src/backend.rs` (run-time invocation)
  - No source edits required to change behavior, but server was started with explicit command-line arguments `--wasm-path` and `--diagrams-path` so the filesystem watcher uses `workspace/adas-wasm-components` and `workspace/diagrams` (relative to the repo root). This prevented a startup failure where the default path didn't exist in the process working directory.

- `glsp-tauri/src-tauri/src/mcp_client.rs`
  - Annotated previously-unused structs/methods with `#[allow(dead_code)]` to silence warnings while iterating.

Why these edits
- The real `composer` crate in `/composer` (which uses `wit_bindgen!` macro codegen) failed to compile in this environment — it blocked workspace builds. The local stub allowed progress: building, running the server, and exercising the client to generate UML diagrams.

3) Runtime verification performed

- Rebuilt the workspace: `cargo build --workspace` — succeeded (warnings only).
- Started the MCP server (background) with explicit paths:

```bash
cargo run --package glsp-mcp-server --bin server -- --wasm-path workspace/adas-wasm-components --diagrams-path workspace/diagrams
# or run detached (example)
nohup target/debug/server --wasm-path workspace/adas-wasm-components --diagrams-path workspace/diagrams > server.log 2>&1 &
```

- Ran the client:

```bash
cargo run --bin tasklist_mcp_client
```

Observed output (representative):
- Server logged that it was listening on 127.0.0.1:3000 (POST /messages, GET /sse).
- Client printed:
  - "✅ Diagram created with ID: <uuid>"
  - PlantUML block for the generated UML (empty in this minimal run due to stubbed interfaces)
  - "✅ UML Diagram ID: <uuid>"

4) Reproduction steps

From repo root (`/mnt/data/wspaces/glsp-mcp`):

- Install system packages (Ubuntu/Debian example):

```bash
sudo apt update
sudo apt install -y libgtk-3-dev libsoup2.4-dev libjavascriptcoregtk-4.1-dev pkg-config
# if pkg-config cannot find javascriptcoregtk version, one workaround used here was to create a symlink:
# sudo ln -s $(pkg-config --variable pcfiledir javascriptcoregtk-4.0)/javascriptcoregtk-4.0.pc /usr/lib/pkgconfig/javascriptcoregtk-4.1.pc
# Then export PKG_CONFIG_PATH if needed:
export PKG_CONFIG_PATH=/usr/lib/pkgconfig:$PKG_CONFIG_PATH
```

- Build workspace:

```bash
cargo build --workspace
```

- Start server (in separate terminal):

```bash
cargo run --package glsp-mcp-server --bin server -- --wasm-path workspace/adas-wasm-components --diagrams-path workspace/diagrams
```

- Run the client (once server is running):

```bash
cargo run --bin tasklist_mcp_client
```

5) Remaining work / recommended next steps

- Replace the local `amt_compose` stub with the real `composer` crate once the `composer` crate build issues are resolved (the root cause appears to be `wit_bindgen!` macro input mismatches in this environment). Fixing `composer` will likely restore full WIT parsing and richer UML output.
- Clean up warnings (unused imports) with `cargo fix` or targeted edits.
- Commit the client changes to the `uml_diagram` branch and open a PR describing the temporary stub and the steps needed to restore the real dependency.
- Optionally, expand the report with a diff/patch listing exact edits if you want to keep the human audit trail in the repo.

6) Contact / traceability

If you want, I can now:
- Commit these changes to a branch and push a PR with the stub clearly labeled.
- Attempt to fix the `composer` crate (investigate `composer/src/lib.rs` and the `wit_bindgen!` invocation errors) and try to re-integrate.

---
Generated on: 2025-09-07

