#!/bin/bash

echo "Testing MCP Server Endpoints..."

# Start the server in background
echo "Starting server..."
cd glsp-mcp-server
cargo run -- --port 3001 --transport http-streaming &
SERVER_PID=$!

# Wait for server to start
sleep 5

echo -e "\n=== Testing GET /sse endpoint ==="
curl -v http://localhost:3001/sse 2>&1 | grep -E "(< HTTP|< |404|200)"

echo -e "\n=== Testing POST /messages endpoint ==="
curl -X POST -v http://localhost:3001/messages \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' 2>&1 | grep -E "(< HTTP|< |404|200)"

echo -e "\n=== Testing GET / endpoint ==="
curl -v http://localhost:3001/ 2>&1 | grep -E "(< HTTP|< |404|200)"

echo -e "\n=== Testing POST /mcp/rpc endpoint ==="
curl -X POST -v http://localhost:3001/mcp/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' 2>&1 | grep -E "(< HTTP|< |404|200)"

# Kill the server
kill $SERVER_PID 2>/dev/null