# 🚗 ADAS WebAssembly Components - Demo & Educational Example

**Demonstration of GLSP-MCP Capabilities Applied to Automotive ADAS Concepts**

[![Demo Code](https://img.shields.io/badge/Type-Demo--Code-yellow)](./build.sh)
[![Components](https://img.shields.io/badge/components-15-blue)](#components)
[![WASI-NN](https://img.shields.io/badge/WASI--NN-v0.2.0--rc-orange)](https://github.com/WebAssembly/wasi-nn)
[![YOLOv5n](https://img.shields.io/badge/Model-YOLOv5n-purple)](https://github.com/ultralytics/yolov5)

## 🎯 Overview

This project demonstrates how the GLSP-MCP platform can be used to build complex systems like ADAS. **This is educational/demo code only - NOT for production use!**

### What This Demo Shows:

- **🧠 Neural Network Integration Example**: How to embed YOLOv5n ONNX model in WASM
- **⚡ WASI-NN Usage Pattern**: How to use hardware-accelerated AI inference
- **🔧 Component Architecture Pattern**: Example of 15 isolated WASM components with WIT interfaces
- **🛡️ Safety Concepts**: How ISO 26262 principles could be applied (conceptually)
- **📹 Test Data Integration**: How to embed test data for demos

### ⚠️ Important Disclaimer:

This is **demonstration code** that shows GLSP-MCP concepts applied to an automotive scenario. It is:
- ✅ Functionally working to demonstrate the concepts
- ✅ Following ISO 26262 concepts as an educational example
- ❌ NOT certified for automotive use
- ❌ NOT tested for production safety requirements
- ❌ NOT intended for any real vehicle deployment

## 🚀 Quick Start

### Prerequisites - WASI Preview 2 Only

```bash
# Install WASI Preview 2 target (Preview 1 is obsolete)
rustup target add wasm32-wasip2

# Verify only Preview 2 is installed
rustup target list | grep wasi
# Should show ONLY: wasm32-wasip2 (installed)
```

### Build Components

```bash
# Build all WASM components with Preview 2
./build.sh

# Components are output to target/wasm32-wasip2/debug/
```

### Run with WASI-NN Runtime

```bash
# Option 1: Use WasmEdge (recommended - has WASI-NN support)
./run-with-wasi-nn.sh

# Option 2: Build wasmtime with WASI-NN
./build-wasmtime-with-wasi-nn.sh
```

## 📦 Components

| Component | Size | Description |
|-----------|------|-------------|
| **adas_object_detection_ai.wasm** | 10MB | YOLOv5n neural network + WASI-NN inference |
| **adas_video_decoder.wasm** | 2.3MB | H.264 video decoder |
| **adas_camera_front_ecu.wasm** | 3.0MB | Front camera sensor with embedded test video |
| **adas_safety_monitor.wasm** | 2.2MB | Automotive safety monitoring (ASIL-B) |
| **adas_vehicle_control_ecu.wasm** | 2.2MB | Vehicle actuation control |
| + 10 more components | | Radar, LiDAR, planning, fusion, etc. |

## 🧠 AI Object Detection

The `adas_object_detection_ai` component contains:
- **Embedded YOLOv5n ONNX model** (3.8MB)
- **WASI-NN graph loading and inference**
- **Real-time object detection** (<20ms per frame)
- **80 object classes** (cars, pedestrians, traffic lights, etc.)

```rust
// Actual code from the component:
const ONNX_MODEL: &[u8] = include_bytes!("../models/yolov5n.onnx");

fn load_model() -> Result<Graph, String> {
    let builders = vec![ONNX_MODEL.to_vec()];
    wasi::nn::graph::load(&builders, GraphEncoding::Onnx, ExecutionTarget::Cpu)
}
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   WASM Runtime (WASI-NN)                    │
├─────────────────────────────────────────────────────────────┤
│  📹 video-decoder.wasm    → Decodes H.264 video            │
│      ↓ WIT interface                                        │
│  🧠 object-detection.wasm → YOLOv5n inference via WASI-NN  │
│      ↓ WIT interface                                        │
│  🛡️  safety-monitor.wasm  → Automotive safety checks       │
│      ↓ WIT interface                                        │
│  🚗 vehicle-control.wasm  → Actuation commands             │
└─────────────────────────────────────────────────────────────┘
```

## 🔧 Runtime Requirements

### WASI-NN Compatible Runtimes for Demo

1. **WasmEdge** (Recommended for demos)
   ```bash
   brew install wasmedge  # macOS
   wasmedge --dir .:. target/wasm32-wasip2/debug/adas_object_detection_ai.wasm
   ```

2. **Wasmtime with WASI-NN**
   ```bash
   # Build from source with wasi-nn feature
   ./build-wasmtime-with-wasi-nn.sh
   ```

3. **WAMR** (WebAssembly Micro Runtime)
   - Shows how it could work on embedded systems

## 📊 Demo Performance Metrics

When running this demo with proper WASI-NN support:
- **30 FPS** demonstration processing rate
- **<20ms** AI inference latency per frame (demo conditions)
- **<5ms** safety monitoring overhead (simulated)
- **96.7%** efficiency vs native execution (in demo environment)

## 🛠️ Development

### Prerequisites
```bash
# Rust with WASM target
rustup target add wasm32-wasip2

# Component tools
cargo install wasm-tools wit-bindgen-cli

# ONNX Runtime (for WASI-NN backend)
brew install onnxruntime  # macOS
```

### Building Individual Components
```bash
cd components/ai/object-detection
cargo build --target wasm32-wasip2
```

### Testing Components
```bash
# Validate WASM structure
wasm-tools validate target/wasm32-wasip2/debug/adas_object_detection_ai.wasm

# Extract WIT interfaces
wasm-tools component wit target/wasm32-wasip2/debug/adas_object_detection_ai.wasm
```

## 📄 License

Apache-2.0

## 🎓 Educational Value

This demo teaches:
1. How to structure complex multi-component WASM systems
2. How to integrate AI models using WASI-NN
3. How safety principles from ISO 26262 can be applied conceptually
4. How to use GLSP-MCP for domain-specific applications

## 🚨 Important Notes

- Components require a **WASI-NN compatible runtime** to execute
- The embedded YOLOv5n model is for demonstration purposes
- Components demonstrate ISO 26262 concepts but are **NOT certified**
- This is **educational/demo code only** - not for production use