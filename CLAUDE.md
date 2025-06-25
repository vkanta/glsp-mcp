# MCP-GLSP Development Notes

## Project Overview
This project implements a revolutionary AI-native graphical modeling platform using:
- **Backend**: Rust HTTP server implementing Model Context Protocol (MCP) over JSON-RPC
- **Frontend**: TypeScript web client with Canvas rendering
- **Protocol**: MCP-based communication enabling AI agents to create, modify, and analyze diagrams

## Architecture
- **MCP Resources** → Diagram model state (read-only views)
- **MCP Tools** → Diagram operations (create, modify, validate)  
- **MCP Prompts** → AI modeling workflows (templates for common tasks)

## Development Setup

### Backend (Rust)
```bash
cd glsp-mcp-server
cargo build
cargo run --bin server
# Server runs on http://127.0.0.1:3000
```

### Frontend (TypeScript)
**Package Manager Choice: npm** (selected for universal compatibility and zero-setup)
**Note: Requires Node.js and npm**
```bash
cd glsp-web-client
npm install
npm run dev
# Frontend runs on http://localhost:5173
```

### Dependencies Required
- **Rust**: Latest stable version
- **Node.js**: v18+ for frontend development
- **npm**: For frontend package management (chosen over yarn/pnpm/bun for reliability)

## Server Endpoints
- `POST /mcp/rpc` - Main MCP JSON-RPC endpoint
- `GET /health` - Server health check

## MCP Capabilities
- **Tools**: create_diagram, create_node, create_edge, delete_element, update_element, apply_layout, export_diagram
- **Resources**: diagram://model/{id}, diagram://validation/{id}, diagram://metadata/{id}, diagram://list
- **Prompts**: generate_workflow, optimize_layout, add_error_handling, analyze_diagram, create_subprocess, convert_diagram

## AI Integration
The server implements MCP protocol, enabling any MCP-compatible AI system to:
- Create diagrams from natural language descriptions
- Analyze existing diagrams for bottlenecks and improvements
- Apply layout optimizations
- Add error handling patterns
- Convert between diagram types

## Testing
Server is confirmed working and listening on port 3000. Frontend requires npm/yarn setup for testing.

## AI Integration Setup
- **Ollama**: Running on http://127.0.0.1:11434 (local LLM)
- **MCP-GLSP Server**: Running on http://127.0.0.1:3000 (diagram backend)
- **Web Client**: Running on http://localhost:5173 (frontend + AI agent)

## Current Status: 🎉 COMPLETE! 
1. ✅ Complete MCP-GLSP backend implementation (Rust)
2. ✅ Complete web frontend with Canvas rendering (TypeScript)
3. ✅ Complete AI agent integration (Ollama + MCP-GLSP)
4. ✅ Full natural language → AI → diagram creation workflow

## Ready to Test!
**Backend**: `cargo run --bin server` (http://127.0.0.1:3000)
**Frontend**: `npm run dev` (http://localhost:5173)
**AI**: Ollama running on http://127.0.0.1:11434

## Revolutionary Features Implemented
🤖 **Natural Language Diagram Creation**: "Create a workflow for order processing"
📊 **AI-Powered Analysis**: Intelligent diagram optimization and validation
🔧 **MCP Protocol**: Universal AI agent compatibility
🎨 **Interactive Canvas**: Real-time diagram editing and rendering