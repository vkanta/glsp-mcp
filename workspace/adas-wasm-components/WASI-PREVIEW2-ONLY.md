# ⚠️ WASI Preview 2 Only - Preview 1 is Obsolete

## This Project Uses WASI Preview 2 Exclusively

### 🚫 No Preview 1 Support

WASI Preview 1 is **obsolete** and **not supported** in this project. All components require:

- **Target**: `wasm32-wasip2` 
- **Runtime**: Component Model support
- **Interfaces**: WIT-based definitions

### ✅ Correct Setup

```bash
# ONLY install Preview 2
rustup target add wasm32-wasip2

# Verify NO Preview 1 is installed
rustup target list | grep wasi
# Should show ONLY: wasm32-wasip2 (installed)
```

### ❌ Wrong Setup

```bash
# DO NOT install Preview 1
rustup target add wasm32-wasip1  # ❌ OBSOLETE
```

### 🔧 If You Have Preview 1 Installed

```bash
# Remove the obsolete target
rustup target remove wasm32-wasip1
```

## Why Preview 2?

1. **Component Model**: Required for multi-component architectures
2. **WIT Interfaces**: Type-safe component communication
3. **Resource Types**: Proper handle management
4. **Async Support**: Native async/await capabilities
5. **Future Standard**: Preview 1 is deprecated

## Build Commands

```bash
# Always use Preview 2
cargo build --target wasm32-wasip2

# Never use Preview 1
cargo build --target wasm32-wasip1  # ❌ WILL NOT WORK
```

## Runtime Requirements

- **Wasmtime 24.0+**: With component model enabled
- **WasmEdge**: Latest version with component support
- **WAMR**: With WASI Preview 2 support

## Component Validation

```bash
# Validate Preview 2 component
wasm-tools validate --features component-model component.wasm

# Check component interfaces
wasm-tools component wit component.wasm
```