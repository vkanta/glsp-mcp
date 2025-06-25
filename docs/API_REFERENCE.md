# MCP-GLSP API Reference

## Overview
The MCP-GLSP server implements the Model Context Protocol (MCP) over HTTP with JSON-RPC 2.0, providing AI agents with powerful diagram creation and manipulation capabilities.

**Base URL**: `http://127.0.0.1:3000`

## Authentication
Currently no authentication required for local development.

## Endpoints

### Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "service": "MCP-GLSP Server", 
  "version": "0.1.0"
}
```

### MCP JSON-RPC Endpoint
```http
POST /mcp/rpc
Content-Type: application/json
```

All MCP communication happens through this single endpoint using JSON-RPC 2.0 protocol.

## MCP Protocol Methods

### 1. Initialize
Establishes connection with the MCP server.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "experimental": {},
      "sampling": {}
    },
    "clientInfo": {
      "name": "Your Client Name",
      "version": "1.0.0"
    }
  },
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {"listChanged": true},
      "resources": {"listChanged": true, "subscribe": false},
      "prompts": {"listChanged": true}
    },
    "serverInfo": {
      "name": "MCP-GLSP Server",
      "version": "0.1.0"
    }
  },
  "id": 1
}
```

### 2. Tools

#### List Available Tools
```json
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "id": 2
}
```

#### Call Tool - Create Diagram
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_diagram",
    "arguments": {
      "diagramType": "workflow",
      "name": "My Process"
    }
  },
  "id": 3
}
```

#### Call Tool - Create Node
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_node",
    "arguments": {
      "diagramId": "diagram-uuid",
      "nodeType": "task",
      "position": {"x": 100, "y": 200},
      "label": "Process Order"
    }
  },
  "id": 4
}
```

#### Call Tool - Create Edge
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_edge",
    "arguments": {
      "diagramId": "diagram-uuid",
      "edgeType": "sequence-flow",
      "sourceId": "node-1-uuid",
      "targetId": "node-2-uuid"
    }
  },
  "id": 5
}
```

#### Call Tool - Apply Layout
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "apply_layout",
    "arguments": {
      "diagramId": "diagram-uuid",
      "algorithm": "hierarchical",
      "direction": "left-right"
    }
  },
  "id": 6
}
```

### 3. Resources

#### List Available Resources
```json
{
  "jsonrpc": "2.0",
  "method": "resources/list",
  "id": 7
}
```

#### Read Diagram Model
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "diagram://model/{diagram-id}"
  },
  "id": 8
}
```

#### Read Validation Results
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "diagram://validation/{diagram-id}"
  },
  "id": 9
}
```

### 4. Prompts

#### List Available Prompts
```json
{
  "jsonrpc": "2.0",
  "method": "prompts/list",
  "id": 10
}
```

#### Get AI Workflow Generation Prompt
```json
{
  "jsonrpc": "2.0",
  "method": "prompts/get",
  "params": {
    "name": "generate_workflow",
    "arguments": {
      "description": "Create an order fulfillment process",
      "style": "bpmn"
    }
  },
  "id": 11
}
```

## Available Tools

| Tool Name | Description | Key Parameters |
|-----------|-------------|----------------|
| `create_diagram` | Create new diagram | `diagramType`, `name` |
| `create_node` | Add node to diagram | `diagramId`, `nodeType`, `position`, `label` |
| `create_edge` | Connect two nodes | `diagramId`, `sourceId`, `targetId`, `edgeType` |
| `delete_element` | Remove element | `diagramId`, `elementId` |
| `update_element` | Modify element properties | `diagramId`, `elementId`, `properties` |
| `apply_layout` | Auto-arrange elements | `diagramId`, `algorithm` |
| `export_diagram` | Export in various formats | `diagramId`, `format` |

## Available Resources

| Resource URI Pattern | Description |
|---------------------|-------------|
| `diagram://list` | List all diagrams |
| `diagram://model/{id}` | Complete diagram model |
| `diagram://elements/{id}` | All elements in diagram |
| `diagram://metadata/{id}` | Diagram statistics |
| `diagram://validation/{id}` | Validation results |
| `diagram://schemas/model` | Diagram model JSON schema |

## Available Prompts

| Prompt Name | Description | Parameters |
|-------------|-------------|------------|
| `generate_workflow` | Create workflow from description | `description`, `style` |
| `optimize_layout` | Improve diagram layout | `diagram_id`, `criteria` |
| `add_error_handling` | Add error handling patterns | `diagram_id`, `error_types` |
| `analyze_diagram` | Analyze for improvements | `diagram_id`, `focus` |
| `create_subprocess` | Break down complex task | `task_description`, `detail_level` |
| `convert_diagram` | Convert between formats | `diagram_id`, `target_type` |

## Node Types

- `start-event` - Process start point
- `end-event` - Process completion 
- `task` - Work activity
- `user-task` - Manual user activity
- `service-task` - Automated system activity
- `gateway` - Decision point or parallel split
- `intermediate-event` - Milestone or wait point

## Edge Types

- `sequence-flow` - Normal process flow
- `conditional-flow` - Decision branch
- `message-flow` - Communication between processes
- `association` - Non-flow relationship

## Error Handling

All responses follow JSON-RPC 2.0 error format:

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "Internal error",
    "data": "Additional error details"
  },
  "id": null
}
```

## Example Workflow

1. **Initialize**: Connect to MCP server
2. **Create Diagram**: Use `create_diagram` tool
3. **Add Elements**: Use `create_node` and `create_edge` tools
4. **Apply Layout**: Use `apply_layout` tool for organization
5. **Validate**: Read `diagram://validation/{id}` resource
6. **Export**: Use `export_diagram` tool

## AI Integration

This API is designed for AI agents. Use the prompts to get detailed instructions for complex operations, then execute the recommended tool sequences to accomplish sophisticated modeling tasks.