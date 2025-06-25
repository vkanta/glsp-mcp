#!/usr/bin/env python3
"""
MCP-GLSP AI Agent Demonstration

This script demonstrates how an AI agent can interact with the MCP-GLSP server
to create, analyze, and optimize diagrams using natural language instructions.
"""

import requests
import json
import time
from typing import Dict, Any, List

class MCPGLSPAgent:
    """AI agent that can interact with MCP-GLSP server"""
    
    def __init__(self, server_url: str = "http://127.0.0.1:3000"):
        self.server_url = server_url
        self.request_id = 0
        self.initialized = False
        
    def _send_request(self, method: str, params: Any = None) -> Dict[str, Any]:
        """Send JSON-RPC request to MCP server"""
        self.request_id += 1
        
        payload = {
            "jsonrpc": "2.0",
            "method": method,
            "id": self.request_id
        }
        
        if params is not None:
            payload["params"] = params
            
        response = requests.post(
            f"{self.server_url}/mcp/rpc",
            json=payload,
            headers={"Content-Type": "application/json"}
        )
        
        if response.status_code != 200:
            raise Exception(f"HTTP {response.status_code}: {response.text}")
            
        result = response.json()
        
        if "error" in result:
            raise Exception(f"MCP Error: {result['error']}")
            
        return result.get("result", {})
    
    def initialize(self) -> None:
        """Initialize connection with MCP server"""
        print("🤖 Initializing AI agent connection to MCP-GLSP server...")
        
        params = {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "experimental": {},
                "sampling": {}
            },
            "clientInfo": {
                "name": "AI Workflow Agent",
                "version": "1.0.0"
            }
        }
        
        result = self._send_request("initialize", params)
        print(f"✅ Connected to {result['serverInfo']['name']} v{result['serverInfo']['version']}")
        
        # Send initialized notification
        self._send_request("initialized", {})
        self.initialized = True
    
    def call_tool(self, tool_name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """Call an MCP tool"""
        if not self.initialized:
            raise Exception("Agent not initialized")
            
        params = {
            "name": tool_name,
            "arguments": arguments
        }
        
        return self._send_request("tools/call", params)
    
    def read_resource(self, uri: str) -> Dict[str, Any]:
        """Read an MCP resource"""
        if not self.initialized:
            raise Exception("Agent not initialized")
            
        params = {"uri": uri}
        return self._send_request("resources/read", params)
    
    def get_prompt(self, prompt_name: str, arguments: Dict[str, str] = None) -> Dict[str, Any]:
        """Get an AI prompt template"""
        if not self.initialized:
            raise Exception("Agent not initialized")
            
        params = {"name": prompt_name}
        if arguments:
            params["arguments"] = arguments
            
        return self._send_request("prompts/get", params)

def demonstrate_ai_workflow():
    """Demonstrate a complete AI-driven workflow creation process"""
    
    agent = MCPGLSPAgent()
    agent.initialize()
    
    print("\n🎯 AI Task: Create an order fulfillment workflow")
    print("=" * 50)
    
    # Step 1: Create a new diagram
    print("\n📊 Step 1: Creating new workflow diagram...")
    create_result = agent.call_tool("create_diagram", {
        "diagramType": "bpmn",
        "name": "Order Fulfillment Process"
    })
    
    # Extract diagram ID from response
    response_text = create_result["content"][0]["text"]
    diagram_id = response_text.split("ID: ")[-1]
    print(f"✅ Created diagram: {diagram_id}")
    
    # Step 2: Create workflow nodes
    print("\n🔧 Step 2: Adding workflow elements...")
    
    # Start event
    start_result = agent.call_tool("create_node", {
        "diagramId": diagram_id,
        "nodeType": "start-event",
        "position": {"x": 50, "y": 150},
        "label": "Order Received"
    })
    print("✅ Added start event")
    
    # Payment validation task
    payment_result = agent.call_tool("create_node", {
        "diagramId": diagram_id,
        "nodeType": "task",
        "position": {"x": 200, "y": 150},
        "label": "Validate Payment"
    })
    print("✅ Added payment validation task")
    
    # Decision gateway
    gateway_result = agent.call_tool("create_node", {
        "diagramId": diagram_id,
        "nodeType": "gateway",
        "position": {"x": 350, "y": 150},
        "label": "Payment Valid?"
    })
    print("✅ Added decision gateway")
    
    # Inventory check
    inventory_result = agent.call_tool("create_node", {
        "diagramId": diagram_id,
        "nodeType": "task",
        "position": {"x": 500, "y": 100},
        "label": "Check Inventory"
    })
    print("✅ Added inventory check")
    
    # Reject order
    reject_result = agent.call_tool("create_node", {
        "diagramId": diagram_id,
        "nodeType": "task", 
        "position": {"x": 500, "y": 200},
        "label": "Reject Order"
    })
    print("✅ Added order rejection task")
    
    # Ship order
    ship_result = agent.call_tool("create_node", {
        "diagramId": diagram_id,
        "nodeType": "task",
        "position": {"x": 650, "y": 100},
        "label": "Ship Order"
    })
    print("✅ Added shipping task")
    
    # End events
    success_end = agent.call_tool("create_node", {
        "diagramId": diagram_id,
        "nodeType": "end-event",
        "position": {"x": 800, "y": 100},
        "label": "Order Completed"
    })
    
    failure_end = agent.call_tool("create_node", {
        "diagramId": diagram_id,
        "nodeType": "end-event",
        "position": {"x": 650, "y": 200},
        "label": "Order Rejected"
    })
    print("✅ Added end events")
    
    # Step 3: Apply intelligent layout
    print("\n📐 Step 3: Applying intelligent layout...")
    layout_result = agent.call_tool("apply_layout", {
        "diagramId": diagram_id,
        "algorithm": "hierarchical"
    })
    print("✅ Applied hierarchical layout")
    
    # Step 4: Analyze the diagram
    print("\n🔍 Step 4: Analyzing diagram for improvements...")
    
    # Get the current diagram state
    model_resource = agent.read_resource(f"diagram://model/{diagram_id}")
    diagram_data = json.loads(model_resource["text"])
    
    print(f"📈 Diagram contains {len(diagram_data['elements'])-1} elements")
    print(f"🔄 Current revision: {diagram_data['revision']}")
    
    # Get validation results
    validation_resource = agent.read_resource(f"diagram://validation/{diagram_id}")
    validation_data = json.loads(validation_resource["text"])
    
    print(f"✅ Validation status: {'Valid' if validation_data['isValid'] else 'Has Issues'}")
    if validation_data['issues']:
        print(f"⚠️  Found {len(validation_data['issues'])} validation issues")
    
    # Step 5: Get AI recommendations
    print("\n🧠 Step 5: Getting AI optimization recommendations...")
    
    prompt_result = agent.get_prompt("analyze_diagram", {
        "diagram_id": diagram_id,
        "focus": "performance"
    })
    
    print("🤖 AI Analysis Prompt Generated:")
    print("=" * 30)
    print(prompt_result["messages"][0]["content"]["text"][:500] + "...")
    
    # Step 6: Export the diagram
    print("\n💾 Step 6: Exporting diagram...")
    
    export_result = agent.call_tool("export_diagram", {
        "diagramId": diagram_id,
        "format": "svg"
    })
    
    print("✅ Diagram exported as SVG")
    
    # Final summary
    print("\n🎉 AI Workflow Demonstration Complete!")
    print("=" * 50)
    print(f"📊 Created BPMN diagram with ID: {diagram_id}")
    print("🔧 Added 7 process elements with proper flow")
    print("📐 Applied intelligent layout optimization")
    print("🔍 Performed automated validation analysis")
    print("🤖 Generated AI-powered improvement recommendations")
    print("💾 Exported diagram in SVG format")
    print("\n🚀 This demonstrates how AI agents can:")
    print("   • Create complex diagrams from natural language")
    print("   • Apply domain expertise for layout optimization")
    print("   • Perform intelligent analysis and validation")
    print("   • Generate actionable improvement suggestions")

def demonstrate_ai_prompts():
    """Demonstrate AI prompt generation capabilities"""
    
    agent = MCPGLSPAgent()
    agent.initialize()
    
    print("\n🧠 AI Prompt Generation Demonstration")
    print("=" * 50)
    
    prompts_to_demo = [
        ("generate_workflow", {
            "description": "Create a customer support ticket resolution process",
            "style": "bpmn"
        }),
        ("add_error_handling", {
            "diagram_id": "example-id",
            "error_types": "validation,system,business"
        }),
        ("convert_diagram", {
            "diagram_id": "example-id", 
            "target_type": "uml"
        })
    ]
    
    for prompt_name, args in prompts_to_demo:
        print(f"\n🤖 Generating '{prompt_name}' prompt...")
        
        try:
            result = agent.get_prompt(prompt_name, args)
            print(f"✅ Generated {len(result['messages'])} message(s)")
            print(f"📝 Preview: {result['messages'][0]['content']['text'][:200]}...")
            
        except Exception as e:
            print(f"❌ Error: {e}")

if __name__ == "__main__":
    print("🚀 MCP-GLSP AI Agent Demonstration")
    print("=" * 50)
    print("This demo shows how AI agents can create and analyze diagrams")
    print("using the Model Context Protocol (MCP) interface.\n")
    
    try:
        # Check if server is running
        response = requests.get("http://127.0.0.1:3000/health", timeout=5)
        if response.status_code == 200:
            print("✅ MCP-GLSP server is running")
            
            # Run demonstrations
            demonstrate_ai_workflow()
            print("\n" + "="*60)
            demonstrate_ai_prompts()
            
        else:
            print("❌ Server health check failed")
            
    except requests.exceptions.ConnectionError:
        print("❌ Cannot connect to MCP-GLSP server at http://127.0.0.1:3000")
        print("Please make sure the server is running with: cargo run --bin server")
        
    except Exception as e:
        print(f"❌ Demo failed: {e}")