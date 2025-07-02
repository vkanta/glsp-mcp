#!/bin/bash

# Build and run ADAS components with WASI-NN support
# This script automates the wasmtime build process with ONNX backend

set -e

echo "🚗 ADAS WASI-NN Setup and Execution"
echo "===================================="

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Configuration
WASMTIME_DIR="wasmtime-wasi-nn-build"
WASMTIME_BIN="$WASMTIME_DIR/target/release/wasmtime"

# Step 1: Check if wasmtime with WASI-NN is already built
if [ -f "$WASMTIME_BIN" ]; then
    echo -e "${GREEN}✅ Found existing wasmtime build with WASI-NN${NC}"
    WASMTIME_VERSION=$("$WASMTIME_BIN" --version)
    echo "Version: $WASMTIME_VERSION"
else
    echo -e "${BLUE}🔧 Building wasmtime with WASI-NN ONNX support...${NC}"
    
    # Clone wasmtime if needed
    if [ ! -d "$WASMTIME_DIR" ]; then
        echo "Cloning wasmtime repository..."
        git clone https://github.com/bytecodealliance/wasmtime.git "$WASMTIME_DIR"
    fi
    
    cd "$WASMTIME_DIR"
    
    # Build with ONNX support and auto-download
    echo -e "${BLUE}Building wasmtime (this will take 5-10 minutes)...${NC}"
    cargo build --release -p wasmtime-cli --features "wasi-nn,wasmtime-wasi-nn/onnx-download"
    
    cd ..
    echo -e "${GREEN}✅ Build complete!${NC}"
fi

# Step 2: Verify WASI-NN support
echo -e "\n${BLUE}🔍 Verifying WASI-NN support...${NC}"
if "$WASMTIME_BIN" run -S help | grep -q "nn\[=y|n\]"; then
    echo -e "${GREEN}✅ WASI-NN support confirmed${NC}"
else
    echo -e "${RED}❌ WASI-NN support not found in build${NC}"
    exit 1
fi

# Step 3: Check for ADAS components
echo -e "\n${BLUE}📦 Checking ADAS components...${NC}"
OBJECT_DETECTION_WASM="target/wasm32-wasip2/debug/adas_object_detection_ai.wasm"

if [ ! -f "$OBJECT_DETECTION_WASM" ]; then
    echo -e "${YELLOW}⚠️  Components not built. Building now...${NC}"
    ./build.sh
fi

if [ -f "$OBJECT_DETECTION_WASM" ]; then
    echo -e "${GREEN}✅ Found object detection component${NC}"
    echo "Size: $(ls -lh "$OBJECT_DETECTION_WASM" | awk '{print $5}')"
else
    echo -e "${RED}❌ Object detection component not found${NC}"
    exit 1
fi

# Step 4: Run the object detection component
echo -e "\n${BLUE}🚀 Running Object Detection AI Component${NC}"
echo "========================================"
echo "Component: $OBJECT_DETECTION_WASM"
echo "Model: Embedded YOLOv5n ONNX (3.8MB)"
echo ""

# Run with debug logging to see what's happening
echo -e "${BLUE}Executing with WASI-NN...${NC}"
echo "Command: $WASMTIME_BIN run -S nn=y $OBJECT_DETECTION_WASM"
echo ""

# Set up environment for better debugging
export RUST_LOG=wasmtime_wasi_nn=debug

# Run the component
"$WASMTIME_BIN" run -S nn=y "$OBJECT_DETECTION_WASM" || {
    echo -e "\n${YELLOW}⚠️  Component execution failed${NC}"
    echo "This could be due to:"
    echo "1. Missing ONNX Runtime libraries"
    echo "2. Component expecting specific input"
    echo "3. Version mismatch"
    echo ""
    echo "Try running with more debug info:"
    echo "RUST_LOG=trace $WASMTIME_BIN run -S nn=y $OBJECT_DETECTION_WASM"
}

# Step 5: Provide next steps
echo -e "\n${BLUE}📝 Next Steps${NC}"
echo "=============="
echo "1. If successful, you saw the component initialize"
echo "2. To process actual video, modify the component to accept input"
echo "3. To use in production, deploy with the built wasmtime runtime"
echo ""
echo "Wasmtime binary location: $WASMTIME_BIN"
echo "Add to PATH: export PATH=$PWD/$WASMTIME_DIR/target/release:\$PATH"