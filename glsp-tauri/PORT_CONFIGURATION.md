# Port Configuration for GLSP Tauri Desktop

The GLSP Tauri desktop application now supports flexible port allocation to avoid conflicts with other applications.

## Default Behavior

By default, the application will:
1. Try to use port 3000 (the traditional default)
2. If port 3000 is unavailable, try fallback ports: 3001, 3002, 3003, 8080, 8081, 8082, 9000, 9001
3. If all predefined ports are unavailable, let the OS assign a random available port

## Configuration Options

### Environment Variable

You can specify a preferred port using the `GLSP_SERVER_PORT` environment variable:

```bash
# macOS/Linux
export GLSP_SERVER_PORT=8080
./glsp-desktop

# Windows
set GLSP_SERVER_PORT=8080
glsp-desktop.exe
```

### Checking the Allocated Port

The application will log the allocated port on startup:
```
INFO glsp_mcp_server: Found available port: 3001
INFO glsp_mcp_server: GLSP MCP Server listening on port 3001
```

### From the Application

You can check the current server status (including port) from within the application:
- The server status is available through the Tauri commands
- The workspace selector and other components automatically use the correct port

## Troubleshooting

If you're having port-related issues:

1. **Check if another application is using the port:**
   ```bash
   # macOS/Linux
   lsof -i :3000
   
   # Windows
   netstat -ano | findstr :3000
   ```

2. **Try a different port:**
   ```bash
   export GLSP_SERVER_PORT=9090
   ```

3. **Let the OS choose:**
   If you don't care which port is used, just run the application normally. It will find an available port automatically.

## Security Note

The server only binds to `127.0.0.1` (localhost) for security. It is not accessible from other machines on the network.