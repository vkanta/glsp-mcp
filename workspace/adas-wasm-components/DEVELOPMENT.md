# ADAS WebAssembly Development Setup

## 🎯 WASI Preview 2 Only

**Important**: This project uses **WASI Preview 2** exclusively. WASI Preview 1 is obsolete and not supported.

## 🛠️ Required Setup

### 1. Install Rust with WASI Preview 2

```bash
# Ensure you have the latest Rust
rustup update

# Install WASI Preview 2 target (the only supported target)
rustup target add wasm32-wasip2

# Verify installation
rustup target list | grep wasip2
# Should show: wasm32-wasip2 (installed)
```

### 2. Install Component Model Tools

```bash
# Install wasm-tools for component manipulation
cargo install wasm-tools

# Install wit-bindgen CLI for WIT file processing
cargo install wit-bindgen-cli

# Install wasmtime with component model support
curl https://wasmtime.dev/install.sh -sSf | bash
```

### 3. Configure Build Environment

```bash
# Set default target for all builds
echo '[build]
target = "wasm32-wasip2"' > ~/.cargo/config.toml
```

## 🚀 Building Components

### Standard Build Command

```bash
# Always use wasm32-wasip2 target
cargo build --target wasm32-wasip2 --release
```

### Component Validation

```bash
# Validate the component model
wasm-tools validate --features component-model target/wasm32-wasip2/release/component.wasm

# Inspect component interfaces
wasm-tools component wit target/wasm32-wasip2/release/component.wasm
```

## 📋 Key Differences from Preview 1

### Component Model
- **Preview 2**: Full component model with WIT interfaces ✅
- **Preview 1**: Module-only, no components ❌

### Imports/Exports
- **Preview 2**: Rich interface types via WIT ✅
- **Preview 1**: Basic WASI functions only ❌

### Resource Management
- **Preview 2**: Proper resource types with ownership ✅
- **Preview 1**: Manual handle management ❌

### Async Support
- **Preview 2**: Native async/futures support ✅
- **Preview 1**: Blocking I/O only ❌

## 🔧 Toolchain Requirements

| Tool | Minimum Version | Purpose |
|------|----------------|----------|
| Rust | 1.75+ | WASI Preview 2 support |
| cargo-component | 0.7+ | Component building |
| wit-bindgen | 0.33+ | WIT code generation |
| wasm-tools | 1.0+ | Component validation |
| wasmtime | 24.0+ | Runtime with component model |

## ⚠️ Common Issues

### "target wasm32-wasip1 not found"
✅ **Solution**: You're using the correct setup! This project doesn't support Preview 1.

### "component model not supported"
✅ **Solution**: Update your toolchain - all modern versions support Preview 2.

### Build Performance
✅ **Tip**: Use `--release` for smaller components and better runtime performance.

## 🎯 Why Preview 2 Only?

1. **Component Model**: Essential for multi-component architectures
2. **WIT Interfaces**: Type-safe communication between components
3. **Resource Types**: Proper handle management for automotive safety
4. **Future-Proof**: Preview 1 is deprecated, Preview 2 is the standard
5. **Better Performance**: Optimized for real-world use cases

## 📚 Resources

- [WASI Preview 2 Spec](https://github.com/WebAssembly/WASI/blob/main/preview2/README.md)
- [Component Model Docs](https://component-model.bytecodealliance.org/)
- [WIT Format Guide](https://component-model.bytecodealliance.org/design/wit.html)
- [Wasmtime Component Guide](https://docs.wasmtime.dev/api/wasmtime/component/index.html)