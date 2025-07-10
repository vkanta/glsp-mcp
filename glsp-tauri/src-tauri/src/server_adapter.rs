use glsp_mcp_server::{run_server, GlspConfig};
use std::path::PathBuf;
use tracing::{info, warn};

/// Start the embedded MCP server for the Tauri application
pub async fn start_embedded_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Set environment variables to disable authentication storage
    std::env::set_var("PULSEENGINE_AUTH_DISABLED", "true");
    std::env::set_var("MCP_AUTH_STORAGE", "memory");
    std::env::set_var("MCP_DISABLE_AUTH", "true");
    
    start_embedded_server_with_workspace(None).await
}

/// Start the embedded MCP server with a specific workspace directory
pub async fn start_embedded_server_with_workspace(workspace_path: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting embedded MCP server for Tauri application");

    let config = if let Some(workspace) = workspace_path {
        info!("Using workspace directory: {}", workspace);
        GlspConfig {
            port: 3000,
            transport: "http-streaming".to_string(),
            wasm_path: format!("{}/wasm-components", workspace),
            diagrams_path: format!("{}/diagrams", workspace),
            force: true,
            database_backend: "mock".to_string(),
            database_host: "localhost".to_string(),
            database_port: 5432,
            database_name: "glsp_sensors".to_string(),
            database_user: None,
            enable_database: false,
            server_name: "glsp-desktop".to_string(),
            server_version: "1.0.0".to_string(),
        }
    } else {
        info!("Using default app data directory");
        GlspConfig {
            port: 3000,
            transport: "http-streaming".to_string(),
            wasm_path: get_app_dir("wasm-components"),
            diagrams_path: get_app_dir("diagrams"),
            force: true,
            database_backend: "mock".to_string(),
            database_host: "localhost".to_string(),
            database_port: 5432,
            database_name: "glsp_sensors".to_string(),
            database_user: None,
            enable_database: false,
            server_name: "glsp-desktop".to_string(),
            server_version: "1.0.0".to_string(),
        }
    };

    // Ensure directories exist
    create_app_directories(&config).await?;

    info!("Starting MCP server on port {}", config.port);
    run_server(config).await.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { 
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    })
}

/// Get application data directory path
fn get_app_dir(subdir: &str) -> String {
    let base = dirs::data_dir()
        .unwrap_or_else(|| {
            warn!("Could not get data directory, using current directory");
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        })
        .join("wasm-component-designer")
        .join(subdir);
    
    base.to_string_lossy().to_string()
}

/// Create necessary application directories
async fn create_app_directories(config: &GlspConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::fs;
    
    // Create WASM components directory
    if let Err(e) = fs::create_dir_all(&config.wasm_path) {
        warn!("Failed to create WASM directory {}: {}", config.wasm_path, e);
    } else {
        info!("Created WASM components directory: {}", config.wasm_path);
    }
    
    // Create diagrams directory
    if let Err(e) = fs::create_dir_all(&config.diagrams_path) {
        warn!("Failed to create diagrams directory {}: {}", config.diagrams_path, e);
    } else {
        info!("Created diagrams directory: {}", config.diagrams_path);
    }
    
    Ok(())
}

/// Get the app data directory for external use
pub fn get_app_data_directory() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        })
        .join("wasm-component-designer")
}