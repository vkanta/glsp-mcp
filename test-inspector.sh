#!/bin/bash

echo "Testing MCP Inspector compatibility..."

# Test tools/list
echo -e "\n1. Testing tools/list..."
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }' | jq '.result.tools[] | .name'

# Test resources/list
echo -e "\n2. Testing resources/list..."
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "resources/list",
    "params": {}
  }' | jq '.result.resources[] | .name'

# Test prompts/list
echo -e "\n3. Testing prompts/list..."
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "prompts/list",
    "params": {}
  }' | jq '.result.prompts[] | .name'

echo -e "\nMCP Inspector should now be able to connect to http://127.0.0.1:3000/mcp"