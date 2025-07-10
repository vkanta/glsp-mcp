use glsp_mcp_server::{run_server, GlspConfig};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

// Global state to track the allocated server port
static SERVER_PORT: std::sync::OnceLock<Arc<RwLock<u16>>> = std::sync::OnceLock::new();

/// Try to find an available port starting from the preferred port
fn find_available_port(preferred_port: u16, fallback_ports: &[u16]) -> Result<u16, std::io::Error> {
    // First try the preferred port
    if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", preferred_port)) {
        drop(listener);
        return Ok(preferred_port);
    }

    // Try fallback ports
    for &port in fallback_ports {
        if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
            drop(listener);
            return Ok(port);
        }
    }

    // If all specified ports fail, let the OS assign a random available port
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

/// Get the server port (reading from environment variable or using default)
fn get_server_port() -> u16 {
    // Check environment variable first
    if let Ok(port_str) = std::env::var("GLSP_SERVER_PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            return port;
        }
    }

    // Try to find an available port
    let preferred_port = 3000;
    let fallback_ports = [3001, 3002, 3003, 8080, 8081, 8082, 9000, 9001];

    match find_available_port(preferred_port, &fallback_ports) {
        Ok(port) => {
            info!("Found available port: {}", port);
            port
        }
        Err(e) => {
            warn!("Failed to find available port: {}, using default 3000", e);
            preferred_port
        }
    }
}

/// Get the currently allocated server port
pub async fn get_allocated_server_port() -> u16 {
    let port_lock = SERVER_PORT.get_or_init(|| Arc::new(RwLock::new(0)));
    let port = port_lock.read().await;
    *port
}

/// Start the embedded MCP server for the Tauri application
pub async fn start_embedded_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Set environment variables to disable authentication storage
    std::env::set_var("PULSEENGINE_AUTH_DISABLED", "true");
    std::env::set_var("MCP_AUTH_STORAGE", "memory");
    std::env::set_var("MCP_DISABLE_AUTH", "true");

    start_embedded_server_with_workspace(None).await
}

/// Start the embedded MCP server with a specific workspace directory
pub async fn start_embedded_server_with_workspace(
    workspace_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting embedded MCP server for Tauri application");

    // Allocate port dynamically
    let port = get_server_port();

    // Store the allocated port for later use
    let port_lock = SERVER_PORT.get_or_init(|| Arc::new(RwLock::new(0)));
    *port_lock.write().await = port;

    let config = if let Some(workspace) = workspace_path {
        info!("Using workspace directory: {}", workspace);
        GlspConfig {
            port,
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
            port,
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
    run_server(config)
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
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
async fn create_app_directories(
    config: &GlspConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::fs;

    // Create WASM components directory
    if let Err(e) = fs::create_dir_all(&config.wasm_path) {
        warn!(
            "Failed to create WASM directory {}: {}",
            config.wasm_path, e
        );
    } else {
        info!("Created WASM components directory: {}", config.wasm_path);
    }

    // Create diagrams directory
    if let Err(e) = fs::create_dir_all(&config.diagrams_path) {
        warn!(
            "Failed to create diagrams directory {}: {}",
            config.diagrams_path, e
        );
    } else {
        info!("Created diagrams directory: {}", config.diagrams_path);
    }

    Ok(())
}

/// Get the app data directory for external use
pub fn get_app_data_directory() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("wasm-component-designer")
}
