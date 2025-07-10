# Building WASM Component Designer Desktop App

This guide covers building and bundling the GLSP Tauri desktop application for different platforms.

## Prerequisites

- **Node.js** (v18+)
- **npm** (comes with Node.js)
- **Rust** (latest stable)
- **Xcode Command Line Tools** (macOS only)

## Quick Build

### macOS

```bash
# Use the build script
./build-macos.sh

# Or manually:
cd ../glsp-web-client && npx vite build
cd ../glsp-tauri && npx tauri build
```

### Windows

```bash
# Build for Windows
npm run build:win
```

### Linux

```bash
# Build for Linux
npm run build:linux
```

## Build Outputs

After a successful build, you'll find:

### macOS
- **App Bundle**: `target/release/bundle/macos/WASM Component Designer.app`
- **DMG Installer**: `target/release/bundle/dmg/WASM Component Designer_1.0.0_aarch64.dmg`

### Windows
- **MSI Installer**: `target/release/bundle/msi/*.msi`
- **Executable**: `target/release/glsp-desktop.exe`

### Linux
- **AppImage**: `target/release/bundle/appimage/*.AppImage`
- **Debian Package**: `target/release/bundle/deb/*.deb`

## Configuration

### App Metadata

Edit `src-tauri/tauri.conf.json` to modify:
- Application name
- Version
- Bundle identifier
- Window settings
- Icons

### Icons

The app uses the following icon files:
- `src-tauri/icons/32x32.png`
- `src-tauri/icons/128x128.png`
- `src-tauri/icons/128x128@2x.png`
- `src-tauri/icons/icon.icns` (macOS)
- `src-tauri/icons/icon.ico` (Windows)

## Troubleshooting

### TypeScript Errors

If you encounter TypeScript errors during the web client build:
```bash
# Build with Vite directly (bypasses TypeScript checking)
cd ../glsp-web-client
npx vite build
```

### Missing Rust Targets

If building for a different architecture:
```bash
# Add the target
rustup target add x86_64-apple-darwin  # For Intel Macs
rustup target add aarch64-apple-darwin  # For Apple Silicon
rustup target add x86_64-pc-windows-msvc  # For Windows
```

### Port Conflicts

The embedded MCP server uses dynamic port allocation. If you need a specific port:
```bash
export GLSP_SERVER_PORT=8080
./build-macos.sh
```

## Development Build

For development with hot reload:
```bash
npm run dev
```

## Code Signing (macOS)

For distribution, you'll need to code sign the app:

1. Get an Apple Developer certificate
2. Update `tauri.conf.json` with your signing identity
3. Build with signing enabled

## Creating a Universal Binary (macOS)

To create a universal binary that runs on both Intel and Apple Silicon:

```bash
npm run build:mac  # This creates a universal binary
```

Note: This requires both `x86_64-apple-darwin` and `aarch64-apple-darwin` targets installed.

## Environment Variables

- `GLSP_SERVER_PORT`: Override the default MCP server port
- `RUST_LOG`: Set logging level (e.g., `debug`, `info`, `warn`, `error`)

## Performance Optimization

For production builds, the following optimizations are applied:
- Release mode compilation with full optimizations
- Minified JavaScript and CSS
- Compressed assets
- Tree-shaking for unused code removal

## Security

The desktop app includes:
- Sandboxed WASM execution
- Restricted file system access
- Secure IPC between renderer and main process
- No external network access by default