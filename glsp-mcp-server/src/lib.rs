pub mod mcp;
pub mod model;
pub mod operations;
pub mod validation;
pub mod selection;
pub mod wasm;
pub mod backend_simple;
pub mod persistence;

// Re-export local MCP modules
pub use mcp::{protocol, tools, resources, prompts};

// Re-export MCP framework types
pub use mcp_protocol::{
    Request, Response, Error, Content, 
    Tool, Resource, Prompt,
    CallToolRequestParam, CallToolResult,
    ListToolsResult, ListResourcesResult, ListPromptsResult,
    PaginatedRequestParam, ReadResourceRequestParam,
    Implementation, ServerCapabilities,
    ProtocolVersion
};
pub use model::*;
pub use backend_simple::*;
// pub use operations::*;
// pub use validation::*;