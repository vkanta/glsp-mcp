# Complete WASI-NN Setup Guide for ADAS Components

## Overview

This guide provides step-by-step instructions to build and run wasmtime with WASI-NN support for your ADAS WebAssembly components that use embedded ONNX models.

## Prerequisites

- macOS (Intel or Apple Silicon)
- Rust toolchain installed
- Git installed
- ~2GB free disk space for build

## Step 1: Clone and Prepare Wasmtime

```bash
# Clone wasmtime repository
git clone https://github.com/bytecodealliance/wasmtime.git
cd wasmtime

# Optional: checkout specific version (latest main has best support)
# git checkout v27.0.0
```

## Step 2: Build Wasmtime with ONNX Support

Wasmtime's WASI-NN implementation supports multiple backends. For ONNX (which your YOLOv5n model uses), we have two options:

### Option A: Build with ONNX Runtime (Recommended)

```bash
# Build with ONNX support and automatic runtime download
cargo build --release -p wasmtime-cli --features "wasi-nn,wasmtime-wasi-nn/onnx-download"
```

This will:
- Enable WASI-NN support
- Enable ONNX backend 
- Automatically download ONNX Runtime binaries

### Option B: Build with Manual ONNX Runtime

If you prefer to install ONNX Runtime manually:

```bash
# Install ONNX Runtime on macOS
brew install onnxruntime

# Build without auto-download
cargo build --release -p wasmtime-cli --features "wasi-nn,wasmtime-wasi-nn/onnx"
```

## Step 3: Verify Build

After building (takes 5-10 minutes):

```bash
# Check the built binary
./target/release/wasmtime --version

# Verify WASI-NN is available
./target/release/wasmtime run -S help | grep nn
# Should show: nn[=y|n] -- Enable support for WASI neural network API
```

## Step 4: Set Up Environment

```bash
# Add to your PATH (or use full path)
export PATH=$PWD/target/release:$PATH

# Go back to your ADAS project
cd /Users/r/git/glsp-rust/workspace/adas-wasm-components
```

## Step 5: Run Your ADAS Components

Now you can run your object detection component:

```bash
# Run with WASI-NN enabled
wasmtime run -S nn=y target/wasm32-wasip2/debug/adas_object_detection_ai.wasm
```

### Expected Behavior

When successfully running, the component will:

1. **Initialize**: Load the embedded YOLOv5n ONNX model (3.8MB)
2. **Create Graph**: Initialize WASI-NN graph for inference
3. **Process**: Wait for input or run test inference
4. **Output**: Return detection results

### Troubleshooting

#### Error: "wasi:nn not found"
- Ensure you built with `--features wasi-nn`
- Check you're using the correct wasmtime binary (not system one)

#### Error: "ONNX backend error"
- Verify ONNX Runtime is available
- Try building with `onnx-download` feature
- Check dynamic library paths:
  ```bash
  # macOS
  export DYLD_LIBRARY_PATH=/usr/local/lib:$DYLD_LIBRARY_PATH
  ```

#### Component Model Issues
If you see component model errors, also add:
```bash
cargo build --release -p wasmtime-cli --features "wasi-nn,wasmtime-wasi-nn/onnx-download,component-model"
```

## Step 6: Test with Debug Output

Create a test script to see what's happening:

```bash
# Enable logging to see WASI-NN operations
RUST_LOG=wasmtime_wasi_nn=debug wasmtime run -S nn=y \
  target/wasm32-wasip2/debug/adas_object_detection_ai.wasm
```

## Alternative: Docker Solution

If building from source is problematic, create a Docker container:

```dockerfile
# Dockerfile.wasi-nn
FROM rust:latest

# Install dependencies
RUN apt-get update && apt-get install -y \
    git \
    cmake \
    build-essential

# Clone and build wasmtime with WASI-NN
RUN git clone https://github.com/bytecodealliance/wasmtime.git && \
    cd wasmtime && \
    cargo build --release -p wasmtime-cli --features "wasi-nn,wasmtime-wasi-nn/onnx-download"

# Set up environment
ENV PATH="/wasmtime/target/release:${PATH}"

WORKDIR /app
```

Build and use:
```bash
docker build -t wasmtime-wasi-nn -f Dockerfile.wasi-nn .
docker run -v $(pwd):/app wasmtime-wasi-nn \
  wasmtime run -S nn=y /app/target/wasm32-wasip2/debug/adas_object_detection_ai.wasm
```

## Backend Options Summary

Wasmtime WASI-NN supports multiple backends:

| Backend | Feature Flag | Requirements | Best For |
|---------|-------------|--------------|----------|
| ONNX | `onnx` | ONNX Runtime | General models (YOLOv5n) |
| OpenVINO | `openvino` | Intel OpenVINO | Intel hardware |
| WinML | `winml` | Windows 10+ | Windows deployment |
| PyTorch | `pytorch` | LibTorch | PyTorch models |

## Next Steps

Once running:

1. **Process Real Video**: Modify component to accept video input
2. **Benchmark Performance**: Measure inference time vs native
3. **Deploy to ECU**: Use embedded WASM runtime with WASI-NN

## References

- [Wasmtime WASI-NN Documentation](https://github.com/bytecodealliance/wasmtime/tree/main/crates/wasi-nn)
- [WASI-NN Specification](https://github.com/WebAssembly/wasi-nn)
- [ORT (ONNX Runtime for Rust)](https://ort.pyke.io/)

## Support

If you encounter issues:
1. Check wasmtime GitHub issues for WASI-NN
2. Verify your ONNX model format
3. Test with simpler models first