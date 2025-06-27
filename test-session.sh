#!/bin/bash

echo "Testing MCP Session ID handling..."

# Test initialize and capture headers
echo -e "\nSending initialize request..."
response=$(curl -i -X POST http://127.0.0.1:3000/mcp \
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
  }' 2>/dev/null)

echo "Full response:"
echo "$response" | head -20

# Extract session ID from headers
session_id=$(echo "$response" | grep -i "Mcp-Session-Id:" | cut -d' ' -f2 | tr -d '\r')

if [ -z "$session_id" ]; then
  echo "ERROR: No session ID found in response headers!"
else
  echo -e "\nSession ID found: $session_id"
  
  # Test using the session ID
  echo -e "\nSending initialized with session ID..."
  curl -X POST http://127.0.0.1:3000/mcp \
    -H "Content-Type: application/json" \
    -H "Mcp-Session-Id: $session_id" \
    -d '{
      "jsonrpc": "2.0",
      "id": 2,
      "method": "initialized",
      "params": {}
    }' | jq .
fi