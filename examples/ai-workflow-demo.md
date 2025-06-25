# MCP-GLSP AI Workflow Demo

This example demonstrates how AI agents can interact with the MCP-GLSP server to create, analyze, and optimize diagrams using the Model Context Protocol.

## Example AI Interaction Flow

### 1. AI Agent Creates a Workflow

**AI Agent Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "prompts/get",
  "params": {
    "name": "generate_workflow",
    "arguments": {
      "description": "Create an order fulfillment process with payment validation, inventory check, and shipping",
      "style": "bpmn"
    }
  },
  "id": 1
}
```

**Server Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "description": "Generate a workflow diagram from a description",
    "messages": [
      {
        "role": "user",
        "content": {
          "type": "text",
          "text": "You are an expert diagram designer. Create a workflow diagram based on this description: ..."
        }
      }
    ]
  },
  "id": 1
}
```

### 2. AI Agent Executes the Workflow Creation

The AI agent would then execute the following sequence of tool calls:

```json
// Create diagram
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_diagram",
    "arguments": {
      "diagramType": "bpmn",
      "name": "Order Fulfillment Process"
    }
  },
  "id": 2
}

// Create start event
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_node",
    "arguments": {
      "diagramId": "diagram-123",
      "nodeType": "start-event",
      "position": {"x": 50, "y": 100},
      "label": "Order Received"
    }
  },
  "id": 3
}

// Create payment validation task
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_node",
    "arguments": {
      "diagramId": "diagram-123",
      "nodeType": "task",
      "position": {"x": 200, "y": 100},
      "label": "Validate Payment"
    }
  },
  "id": 4
}

// Create payment gateway
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_node",
    "arguments": {
      "diagramId": "diagram-123",
      "nodeType": "gateway",
      "position": {"x": 350, "y": 100},
      "label": "Payment Valid?"
    }
  },
  "id": 5
}

// Continue with inventory check, shipping, etc...
```

### 3. AI Agent Analyzes the Diagram

```json
{
  "jsonrpc": "2.0",
  "method": "prompts/get",
  "params": {
    "name": "analyze_diagram",
    "arguments": {
      "diagram_id": "diagram-123",
      "focus": "performance"
    }
  },
  "id": 10
}
```

The AI agent can then read diagram resources to understand the current state:

```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "diagram://model/diagram-123"
  },
  "id": 11
}

{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "diagram://validation/diagram-123"
  },
  "id": 12
}
```

### 4. AI Agent Optimizes Layout

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "apply_layout",
    "arguments": {
      "diagramId": "diagram-123",
      "algorithm": "hierarchical",
      "direction": "left-right"
    }
  },
  "id": 13
}
```

### 5. AI Agent Adds Error Handling

```json
{
  "jsonrpc": "2.0",
  "method": "prompts/get",
  "params": {
    "name": "add_error_handling",
    "arguments": {
      "diagram_id": "diagram-123",
      "error_types": "validation,system,business"
    }
  },
  "id": 14
}
```

## Natural Language Interaction Examples

### Example 1: Creating a Diagram from Description

**User:** "Create a BPMN process for handling customer support tickets"

**AI Agent Actions:**
1. Uses `generate_workflow` prompt
2. Calls `create_diagram` with type "bpmn"
3. Creates nodes for: ticket creation, triage, assignment, resolution, closure
4. Adds gateways for priority and complexity decisions
5. Applies hierarchical layout
6. Returns summary of created process

### Example 2: Optimizing an Existing Diagram

**User:** "Make this workflow more efficient and add error handling"

**AI Agent Actions:**
1. Reads current diagram state via resources
2. Uses `analyze_diagram` prompt to identify bottlenecks
3. Uses `add_error_handling` prompt to enhance robustness
4. Applies layout optimization
5. Provides recommendations for process improvements

### Example 3: Converting Between Diagram Types

**User:** "Convert this flowchart to a UML activity diagram"

**AI Agent Actions:**
1. Reads source diagram structure
2. Uses `convert_diagram` prompt with target type "uml"
3. Maps flowchart elements to UML equivalents
4. Creates new diagram with UML conventions
5. Validates the conversion result

## Integration with AI Platforms

### Claude Desktop Integration

```typescript
// Claude Desktop MCP integration
const mcpClient = new McpClient('http://localhost:3000');

// Claude can directly call GLSP tools
await mcpClient.callTool('create_diagram', {
  diagramType: 'workflow',
  name: 'User-requested process'
});
```

### Custom AI Agent Integration

```python
# Python AI agent using MCP
import requests

class GLSPAgent:
    def __init__(self, server_url="http://localhost:3000"):
        self.server_url = server_url
        self.request_id = 0
    
    def call_tool(self, tool_name, arguments):
        self.request_id += 1
        payload = {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            },
            "id": self.request_id
        }
        
        response = requests.post(f"{self.server_url}/mcp/rpc", json=payload)
        return response.json()
    
    def create_workflow_from_description(self, description):
        # Use AI to parse description and create diagram
        prompt_result = self.get_prompt("generate_workflow", {
            "description": description
        })
        
        # Execute the workflow creation steps
        # (AI agent would implement the actual workflow creation logic)
        pass
```

## Benefits of MCP-GLSP for AI

1. **Standardized Interface**: AI agents can use the same protocol regardless of the underlying implementation
2. **Rich Context**: Resources provide comprehensive diagram state information
3. **Guided Operations**: Prompts give AI agents templates for complex modeling tasks
4. **Type Safety**: WIT-based protocol ensures correct parameter types
5. **Extensibility**: New diagram types and operations can be added without breaking existing AI integrations

## Use Cases

- **Automated Documentation**: AI generates process diagrams from code or requirements
- **Process Optimization**: AI analyzes existing workflows and suggests improvements
- **Model Translation**: AI converts between different diagram notations
- **Interactive Modeling**: AI assists users in real-time diagram creation
- **Compliance Checking**: AI validates diagrams against regulatory requirements
- **Training Data**: AI generates diverse diagram examples for training purposes