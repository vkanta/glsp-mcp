# MCP-GLSP Desktop Application

This is the desktop version of the MCP-GLSP (WASM Component Designer) built with Tauri. It bundles the backend MCP server and frontend web client into a single native desktop application.

## Features

- **Single Executable**: No need to run separate backend and frontend processes
- **Native File System Access**: Full access to local files and directories
- **Embedded MCP Server**: Built-in Rust MCP server for diagram management
- **Cross-Platform**: Available for Windows, macOS, and Linux
- **Auto-Updates**: Built-in update mechanism (configurable)

## Quick Start

### Prerequisites

- Rust (latest stable)
- Node.js (18+)
- npm

### Development

1. **Install dependencies**:
   ```bash
   npm install
   ```

2. **Run in development mode**:
   ```bash
   npm run dev
   ```
   This will:
   - Start the frontend development server
   - Launch the Tauri desktop app
   - Enable hot reload for both frontend and backend

### Building for Distribution

1. **Build for current platform**:
   ```bash
   npm run build
   ```

2. **Build for specific platforms**:
   ```bash
   npm run build:win     # Windows
   npm run build:mac     # macOS
   npm run build:linux   # Linux
   ```

### Output Files

Built applications will be in `src-tauri/target/release/bundle/`:
- **Windows**: `.msi` installer
- **macOS**: `.dmg` disk image
- **Linux**: `.AppImage`, `.deb`, and `.rpm` packages

## Architecture

```
Desktop App
├── Tauri Frontend (WebView)
│   └── Uses built files from ../glsp-web-client/dist
├── Tauri Backend (Rust)
│   ├── Embedded MCP Server
│   ├── File System Access
│   └── Native OS Integration
└── Auto-Generated App Bundle
```

## Environment Detection

The frontend automatically detects when running in Tauri and adapts:
- Uses native file dialogs instead of web file inputs
- Accesses local file system directly
- Connects to embedded MCP server on localhost:3000

## Data Storage

- **Diagrams**: `~/Library/Application Support/wasm-component-designer/diagrams` (macOS)
- **WASM Components**: `~/Library/Application Support/wasm-component-designer/wasm-components` (macOS)
- **Logs**: Standard OS logging locations

## Development Notes

- The frontend is built from `../glsp-web-client` and served as static files
- The backend embeds the MCP server from `../glsp-mcp-server`
- Environment detection allows the same frontend code to work in both web and desktop
- Hot reload works for both frontend changes and Rust backend changes

## Troubleshooting

1. **Build fails**: Ensure all dependencies are installed and Rust is up to date
2. **App won't start**: Check that port 3000 is available for the embedded server
3. **File access issues**: Ensure the app has necessary permissions on macOS/Linux

## Contributing

This is part of the larger MCP-GLSP project. See the main README for contribution guidelines.