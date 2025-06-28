use axum::{
    routing::{get, post},
    Router,
    http::Method,
    Json,
    response::IntoResponse,
    extract::Path,
    http::StatusCode,
};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;
use serde_json::json;
use serde::{Deserialize, Serialize};

use crate::{GlspBackend, sse::wasm_changes_stream};
use crate::wasm::{WitAnalyzer, ComponentWitAnalysis};

/// Create the HTTP router with all endpoints
pub fn create_router(
    backend: Arc<GlspBackend>,
    mcp_handler: impl Fn(mcp_protocol::Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = mcp_protocol::Response> + Send>> + Send + Sync + 'static,
) -> Router {
    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // Get filesystem watcher
    let filesystem_watcher = backend.get_filesystem_watcher();

    // Create main router
    let app = Router::new()
        // MCP endpoint
        .route("/mcp/rpc", post({
            let handler = Arc::new(mcp_handler);
            move |Json(request): Json<mcp_protocol::Request>| {
                let handler = handler.clone();
                async move {
                    let response = handler(request).await;
                    Json(response)
                }
            }
        }))
        // Health check endpoint
        .route("/health", get(health_check))
        // WIT Analysis endpoints
        .route("/api/wasm/components", get({
            let backend = backend.clone();
            move || get_all_components(backend)
        }))
        .route("/api/wasm/components/:name/wit", get({
            let backend = backend.clone();
            move |path| get_component_wit_analysis(backend, path)
        }))
        .route("/api/wasm/analyze", post({
            let backend = backend.clone();
            move |json| analyze_wasm_file(backend, json)
        }))
        // Apply CORS to all routes
        .layer(cors);
    
    // Create a nested router for SSE with state
    let sse_router = Router::new()
        .route("/sse/wasm-changes", get(wasm_changes_stream))
        .with_state(filesystem_watcher);
    
    // Merge the routers
    app.merge(sse_router)
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "glsp-mcp-server",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Request structure for analyzing a WASM file
#[derive(Debug, Deserialize)]
struct AnalyzeWasmRequest {
    file_path: String,
}

/// Response structure for WIT analysis
#[derive(Debug, Serialize)]
struct WitAnalysisResponse {
    success: bool,
    analysis: Option<ComponentWitAnalysis>,
    error: Option<String>,
}

/// Get all available WASM components
async fn get_all_components(backend: Arc<GlspBackend>) -> impl IntoResponse {
    let wasm_watcher = backend.get_wasm_watcher();
    let watcher = wasm_watcher.lock().await;
    let components = watcher.get_components();
    
    let response = json!({
        "success": true,
        "components": components.iter().map(|c| {
            json!({
                "name": c.name,
                "path": c.path,
                "description": c.description,
                "file_exists": c.file_exists,
                "last_seen": c.last_seen,
                "interfaces_count": c.interfaces.len(),
                "has_wit_interfaces": c.wit_interfaces.is_some(),
                "dependencies_count": c.dependencies.len()
            })
        }).collect::<Vec<_>>(),
        "total_count": components.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Json(response)
}

/// Get WIT analysis for a specific component
async fn get_component_wit_analysis(
    backend: Arc<GlspBackend>, 
    Path(component_name): Path<String>
) -> impl IntoResponse {
    let wasm_watcher = backend.get_wasm_watcher();
    let watcher = wasm_watcher.lock().await;
    
    if let Some(component) = watcher.get_component(&component_name) {
        let response = json!({
            "success": true,
            "component_name": component.name,
            "wit_interfaces": component.wit_interfaces,
            "interfaces": component.interfaces,
            "dependencies": component.dependencies,
            "metadata": component.metadata,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        Json(response).into_response()
    } else {
        let response = json!({
            "success": false,
            "error": format!("Component '{}' not found", component_name),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        (StatusCode::NOT_FOUND, Json(response)).into_response()
    }
}

/// Analyze a WASM file and return WIT interfaces
async fn analyze_wasm_file(
    _backend: Arc<GlspBackend>,
    Json(request): Json<AnalyzeWasmRequest>
) -> impl IntoResponse {
    let file_path = std::path::Path::new(&request.file_path);
    
    if !file_path.exists() {
        let response = WitAnalysisResponse {
            success: false,
            analysis: None,
            error: Some(format!("File not found: {}", request.file_path)),
        };
        return (StatusCode::NOT_FOUND, Json(response)).into_response();
    }
    
    match WitAnalyzer::analyze_component(file_path).await {
        Ok(analysis) => {
            let response = WitAnalysisResponse {
                success: true,
                analysis: Some(analysis),
                error: None,
            };
            Json(response).into_response()
        }
        Err(error) => {
            let response = WitAnalysisResponse {
                success: false,
                analysis: None,
                error: Some(error.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}