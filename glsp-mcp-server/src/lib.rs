pub mod backend;
pub mod mcp;
pub mod model;
pub mod operations;
pub mod persistence;
pub mod selection;
pub mod validation;
pub mod wasm;

// Re-export local MCP modules
pub use mcp::{prompts, protocol, resources, tools};

// Re-export MCP framework types
pub use backend::*;
pub use model::*;
pub use pulseengine_mcp_protocol::{
    CallToolRequestParam, CallToolResult, Content, Error, Implementation, ListPromptsResult,
    ListResourcesResult, ListToolsResult, PaginatedRequestParam, Prompt, ProtocolVersion,
    ReadResourceRequestParam, Request, Resource, Response, ServerCapabilities, Tool,
};
// pub use operations::*;
// pub use validation::*;
