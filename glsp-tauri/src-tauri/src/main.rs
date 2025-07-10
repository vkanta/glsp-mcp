#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod server_adapter;
mod mcp_client;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;
use tauri::Manager;

/// Simplified AppState that only tracks essential application state
#[derive(Debug)]
struct AppState {
    mcp_client: Arc<mcp_client::McpClient>,
    current_workspace: Arc<Mutex<Option<String>>>,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("glsp_desktop=info,glsp_mcp_server=info")
        }))
        .init();

    // Start embedded MCP server in background thread
    tokio::spawn(async move {
        if let Err(e) = server_adapter::start_embedded_server().await {
            eprintln!("Failed to start MCP server: {}", e);
        }
    });

    // Wait a moment for server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Create MCP client and app state
    let mcp_client = Arc::new(mcp_client::McpClient::new(3000));
    
    // Initialize MCP client session
    if let Err(e) = mcp_client.initialize().await {
        eprintln!("Failed to initialize MCP client: {}", e);
    }
    
    let app_state = AppState {
        mcp_client,
        current_workspace: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_server_status,
            commands::open_local_file,
            commands::save_to_file,
            commands::get_app_data_dir,
            commands::create_directory,
            commands::select_workspace_directory,
            commands::get_workspace_info,
            commands::get_recent_workspaces,
            commands::add_recent_workspace,
            commands::set_workspace_directory,
            commands::validate_workspace,
            commands::create_workspace_structure,
        ])
        .setup(|app| {
            // Wait for MCP server to initialize
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            // Optional: Open developer tools in debug mode
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}