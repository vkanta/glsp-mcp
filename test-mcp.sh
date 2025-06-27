#!/bin/bash

echo "Testing MCP Server endpoints..."

# Test the new /mcp endpoint with POST
echo -e "\n1. Testing POST /mcp (initialize):"
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-03-26",
      "capabilities": {},
      "clientInfo": {
        "name": "Test Client",
        "version": "1.0"
      }
    }
  }' | jq .

# Test GET /mcp endpoint
echo -e "\n2. Testing GET /mcp:"
curl -X GET http://127.0.0.1:3000/mcp \
  -H "Accept: application/json" | jq .

# Test health check
echo -e "\n3. Testing health check:"
curl http://127.0.0.1:3000/health | jq .