use crate::server_adapter;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::{command, State};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatus {
    running: bool,
    port: u16,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDialogResult {
    path: Option<String>,
    contents: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    path: String,
    name: String,
    last_used: String,
    diagrams_count: u32,
    wasm_components_count: u32,
}

/// Get the current status of the embedded MCP server
#[command]
pub async fn get_server_status(state: State<'_, AppState>) -> Result<ServerStatus, String> {
    let is_healthy = state.mcp_client.health_check().await.unwrap_or(false);
    let port = crate::server_adapter::get_allocated_server_port().await;
    let actual_port = if port > 0 { port } else { 3000 };

    Ok(ServerStatus {
        running: is_healthy,
        port: actual_port,
        url: format!("http://localhost:{}", actual_port),
    })
}

/// Open a local file using the native file dialog
#[command]
pub async fn open_local_file(extension: Option<String>) -> Result<FileDialogResult, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;

    let mut dialog = FileDialogBuilder::new();

    if let Some(ext) = extension {
        dialog = dialog.add_filter("WASM Component", &[&ext]);
    }

    match dialog.pick_file() {
        Some(path) => match std::fs::read(&path) {
            Ok(contents) => Ok(FileDialogResult {
                path: Some(path.to_string_lossy().to_string()),
                contents: Some(contents),
            }),
            Err(e) => Err(format!("Failed to read file: {}", e)),
        },
        None => Ok(FileDialogResult {
            path: None,
            contents: None,
        }),
    }
}

/// Save content to a local file using the native file dialog
#[command]
pub async fn save_to_file(
    content: String,
    default_name: Option<String>,
    extension: Option<String>,
) -> Result<Option<String>, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;

    let mut dialog = FileDialogBuilder::new();

    if let Some(name) = default_name {
        dialog = dialog.set_file_name(&name);
    }

    if let Some(ext) = extension {
        dialog = dialog.add_filter("Diagram File", &[&ext]);
    }

    match dialog.save_file() {
        Some(path) => match std::fs::write(&path, content) {
            Ok(_) => Ok(Some(path.to_string_lossy().to_string())),
            Err(e) => Err(format!("Failed to save file: {}", e)),
        },
        None => Ok(None),
    }
}

/// Get the application data directory path
#[command]
pub async fn get_app_data_dir() -> Result<String, String> {
    let path = server_adapter::get_app_data_directory();
    Ok(path.to_string_lossy().to_string())
}

/// Create a directory in the app data directory
#[command]
pub async fn create_directory(relative_path: String) -> Result<String, String> {
    let base = server_adapter::get_app_data_directory();
    let full_path = base.join(relative_path);

    match std::fs::create_dir_all(&full_path) {
        Ok(_) => Ok(full_path.to_string_lossy().to_string()),
        Err(e) => Err(format!("Failed to create directory: {}", e)),
    }
}

/// Select a workspace directory using the native folder dialog
#[command]
pub async fn select_workspace_directory() -> Result<Option<String>, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;

    match FileDialogBuilder::new()
        .set_title("Select Workspace Directory")
        .pick_folder()
    {
        Some(path) => {
            // Validate that the directory exists and is writable
            if !path.exists() {
                return Err("Selected directory does not exist".to_string());
            }

            if !path.is_dir() {
                return Err("Selected path is not a directory".to_string());
            }

            // Test write permission by attempting to create a test file
            let test_path = path.join(".glsp_test");
            match std::fs::write(&test_path, "test") {
                Ok(_) => {
                    // Clean up test file
                    let _ = std::fs::remove_file(&test_path);
                    Ok(Some(path.to_string_lossy().to_string()))
                }
                Err(e) => Err(format!("Directory is not writable: {}", e)),
            }
        }
        None => Ok(None),
    }
}

/// Get workspace information - use MCP if no path provided, else use local calculation
#[command]
pub async fn get_workspace_info(
    workspace_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<WorkspaceInfo, String> {
    match workspace_path {
        Some(path) => {
            // Local calculation for a specific path (for validation purposes)
            get_workspace_info_local(&path).await
        }
        None => {
            // Get current workspace info from MCP server
            let workspace_info = state.mcp_client.get_workspace_info().await?;

            // Parse the MCP response into our WorkspaceInfo format
            let workspace_root = workspace_info
                .get("workspace_root")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let _wasm_path = workspace_info
                .get("wasm_components_path")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let _diagrams_path = workspace_info
                .get("diagrams_path")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let wasm_count = workspace_info
                .get("wasm_components_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            let diagrams_count = workspace_info
                .get("diagrams_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;

            let name = if workspace_root != "Unknown" {
                std::path::Path::new(workspace_root)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Current Workspace")
                    .to_string()
            } else {
                "Current Workspace".to_string()
            };

            Ok(WorkspaceInfo {
                path: workspace_root.to_string(),
                name,
                last_used: "Active".to_string(),
                diagrams_count,
                wasm_components_count: wasm_count,
            })
        }
    }
}

/// Local workspace info calculation (helper function)
async fn get_workspace_info_local(workspace_path: &str) -> Result<WorkspaceInfo, String> {
    let path = std::path::Path::new(workspace_path);

    if !path.exists() {
        return Err("Workspace directory does not exist".to_string());
    }

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Count diagrams
    let diagrams_path = path.join("diagrams");
    let diagrams_count = if diagrams_path.exists() {
        std::fs::read_dir(&diagrams_path)
            .map(|entries| entries.filter_map(|e| e.ok()).count() as u32)
            .unwrap_or(0)
    } else {
        0
    };

    // Count WASM components
    let wasm_path = path.join("wasm-components");
    let wasm_components_count = if wasm_path.exists() {
        std::fs::read_dir(&wasm_path)
            .map(|entries| entries.filter_map(|e| e.ok()).count() as u32)
            .unwrap_or(0)
    } else {
        0
    };

    let last_used = std::fs::metadata(&path)
        .and_then(|m| m.accessed())
        .map(|t| format!("{:?}", t))
        .unwrap_or_else(|_| "Unknown".to_string());

    Ok(WorkspaceInfo {
        path: workspace_path.to_string(),
        name,
        last_used,
        diagrams_count,
        wasm_components_count,
    })
}

/// Get list of recently used workspaces
#[command]
pub async fn get_recent_workspaces() -> Result<Vec<WorkspaceInfo>, String> {
    let settings_path = server_adapter::get_app_data_directory().join("settings.json");

    if !settings_path.exists() {
        return Ok(Vec::new());
    }

    let settings_content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings: {}", e))?;

    let settings: serde_json::Value = serde_json::from_str(&settings_content)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;

    let recent_workspaces = settings
        .get("recent_workspaces")
        .and_then(|rw| rw.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut workspace_infos = Vec::new();

    for workspace_path in recent_workspaces {
        if let Ok(info) = get_workspace_info_local(&workspace_path).await {
            workspace_infos.push(info);
        }
    }

    Ok(workspace_infos)
}

/// Add a workspace to the recent workspaces list
#[command]
pub async fn add_recent_workspace(workspace_path: String) -> Result<(), String> {
    let settings_path = server_adapter::get_app_data_directory().join("settings.json");

    // Read existing settings or create new ones
    let mut settings = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))?
    } else {
        serde_json::json!({})
    };

    // Get or create recent workspaces array
    let recent_workspaces = settings
        .get_mut("recent_workspaces")
        .and_then(|rw| rw.as_array_mut())
        .map(|arr| {
            // Remove if already exists
            arr.retain(|v| v.as_str() != Some(&workspace_path));
            // Add to front
            arr.insert(0, serde_json::Value::String(workspace_path.clone()));
            // Keep only last 10
            arr.truncate(10);
            arr.clone()
        })
        .unwrap_or_else(|| vec![serde_json::Value::String(workspace_path.clone())]);

    settings["recent_workspaces"] = serde_json::Value::Array(recent_workspaces);

    // Write back to file
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

/// Set workspace directory using MCP (no server restart required)
#[command]
pub async fn set_workspace_directory(
    workspace_path: String,
    create_if_missing: Option<bool>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Setting workspace directory to: {}", workspace_path);

    let create = create_if_missing.unwrap_or(true);
    let result = state
        .mcp_client
        .set_workspace_directory(&workspace_path, create)
        .await?;

    // Update current workspace in app state
    {
        let mut current = state.current_workspace.lock().await;
        *current = Some(workspace_path.clone());
    }

    // Add the workspace to recent workspaces
    add_recent_workspace(workspace_path).await?;

    Ok(result)
}

/// Validate workspace structure using MCP
#[command]
pub async fn validate_workspace(
    workspace_path: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    info!("Validating workspace: {}", workspace_path);
    state.mcp_client.validate_workspace(&workspace_path).await
}

/// Create workspace structure using MCP
#[command]
pub async fn create_workspace_structure(
    workspace_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Creating workspace structure at: {}", workspace_path);
    state
        .mcp_client
        .create_workspace_structure(&workspace_path)
        .await
}
