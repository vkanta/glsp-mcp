#!/bin/bash

# Run ADAS WASM components with WASI-NN support
# This script sets up and runs the components with proper WASI-NN runtime

set -e

echo "🚗 ADAS WASM Components with WASI-NN"
echo "====================================="

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Option 1: Try WasmEdge (has built-in WASI-NN support)
echo -e "${BLUE}🔍 Checking for WasmEdge...${NC}"
if ! command -v wasmedge &> /dev/null; then
    echo -e "${YELLOW}⚠️  WasmEdge not found. Installing...${NC}"
    
    # Install WasmEdge with ONNX plugin
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        brew install wasmedge
    else
        # Linux
        curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash -s -- -e all
        source $HOME/.wasmedge/env
    fi
fi

# Check if we have WasmEdge
if command -v wasmedge &> /dev/null; then
    echo -e "${GREEN}✅ WasmEdge found!${NC}"
    wasmedge --version
    echo ""
    
    # Try to run object detection component
    OBJECT_DETECTION_WASM="target/wasm32-wasip2/debug/adas_object_detection_ai.wasm"
    
    if [ -f "$OBJECT_DETECTION_WASM" ]; then
        echo -e "${BLUE}🧠 Running Object Detection Component...${NC}"
        echo "Component: $OBJECT_DETECTION_WASM"
        echo "Size: $(ls -lh "$OBJECT_DETECTION_WASM" | awk '{print $5}')"
        echo ""
        
        # Run with WasmEdge
        echo -e "${BLUE}🚀 Executing with WasmEdge WASI-NN...${NC}"
        wasmedge --dir .:. "$OBJECT_DETECTION_WASM" || {
            echo -e "${YELLOW}⚠️  Component requires ONNX plugin. Installing...${NC}"
            
            # Install ONNX plugin for WasmEdge
            if [[ "$OSTYPE" == "darwin"* ]]; then
                echo "For macOS, ONNX plugin installation:"
                echo "  brew install onnxruntime"
                echo "  wasmedge_plugin_install onnx"
            else
                wasmedge_plugin_install onnx
            fi
        }
    else
        echo -e "${RED}❌ Object detection WASM not found!${NC}"
        echo "Please run ./build.sh first"
    fi
else
    echo -e "${RED}❌ No WASI-NN compatible runtime found!${NC}"
    echo ""
    echo "Options:"
    echo "1. Install WasmEdge (recommended):"
    echo "   brew install wasmedge  # macOS"
    echo "   curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash  # Linux"
    echo ""
    echo "2. Build wasmtime with WASI-NN:"
    echo "   ./build-wasmtime-with-wasi-nn.sh"
fi

echo ""
echo -e "${BLUE}📊 Component Analysis${NC}"
echo "===================="

# Analyze the WASM component structure
if command -v wasm-tools &> /dev/null; then
    echo "Analyzing object detection component..."
    wasm-tools component wit target/wasm32-wasip2/debug/adas_object_detection_ai.wasm 2>/dev/null | head -20 || {
        echo "Component uses WASI-NN interfaces"
    }
fi

echo ""
echo -e "${BLUE}🎯 Expected Behavior${NC}"
echo "===================="
echo "When running with proper WASI-NN support:"
echo "1. Component loads embedded YOLOv5n ONNX model (3.8MB)"
echo "2. Initializes WASI-NN graph for inference"
echo "3. Processes video frames through neural network"
echo "4. Returns object detections with bounding boxes"
echo "5. Achieves <20ms inference time per frame"

echo ""
echo -e "${BLUE}🔧 Troubleshooting${NC}"
echo "=================="
echo "If you see 'wasi:nn not found' errors:"
echo "1. Ensure you have a WASI-NN compatible runtime"
echo "2. Install required ML backend (ONNX, OpenVINO, etc.)"
echo "3. Check that the runtime version matches component requirements"
echo ""
echo "Current component requires: wasi:nn@0.2.0-rc-2024-10-28"