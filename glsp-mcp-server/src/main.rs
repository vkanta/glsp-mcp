use glsp_mcp_server::mcp::server::McpServer;
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting MCP-GLSP Server");

    // Create the server router
    let app = McpServer::create_router();

    // Bind to address
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    info!("Server listening on http://127.0.0.1:3000");

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}