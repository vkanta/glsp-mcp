//! GLSP-MCP Server - A revolutionary AI-native graphical modeling platform
//!
//! This crate implements a Model Context Protocol (MCP) server that enables AI agents
//! to create, modify, and analyze diagrams through a standardized protocol. The server
//! combines WASM component execution with graphical diagram management.
//!
//! # Features
//!
//! - **MCP Protocol**: Full implementation of Model Context Protocol for AI integration
//! - **WASM Components**: Execute and manage WebAssembly components
//! - **Diagram Management**: Create, edit, and persist graphical diagrams
//! - **Real-time Collaboration**: Live diagram updates and validation
//! - **Sensor Data Integration**: Connect with sensor data streams
//! - **Security**: Sandboxed WASM execution with security analysis
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use glsp_mcp_server::{GlspBackend, GlspConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = GlspConfig::parse();
//!     let backend = GlspBackend::initialize(config).await?;
//!     // Server setup and execution...
//!     Ok(())
//! }
//! ```

/// Backend implementation and configuration
pub mod backend;
/// Database integration and sensor data management
pub mod database;
/// Model Context Protocol implementation
pub mod mcp;
/// Diagram model types and element definitions
pub mod model;
/// Diagram operations and transformations
pub mod operations;
/// Diagram persistence and file management
pub mod persistence;
/// Element selection and interaction management
pub mod selection;
/// Diagram validation and error checking
pub mod validation;
/// WebAssembly component execution and management
pub mod wasm;

// Re-export local MCP modules for easy access
pub use mcp::{prompts, protocol, resources, tools};

// Re-export core types for external users
pub use backend::*;
pub use model::*;
pub use pulseengine_mcp_protocol::{
    CallToolRequestParam, CallToolResult, Content, Error, Implementation, ListPromptsResult,
    ListResourcesResult, ListToolsResult, PaginatedRequestParam, Prompt, ProtocolVersion,
    ReadResourceRequestParam, Request, Resource, Response, ServerCapabilities, Tool,
};
