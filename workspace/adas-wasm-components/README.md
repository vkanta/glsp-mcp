# 🚗 ADAS WebAssembly Components

**Automotive Advanced Driver Assistance System (ADAS) built with WebAssembly Component Model and Fixed Execution Order (FEO)**

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](./build.sh)
[![Components](https://img.shields.io/badge/components-18-blue)](#component-architecture)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-Component%20Model-orange)](https://component-model.bytecodealliance.org/)
[![FEO](https://img.shields.io/badge/FEO-Fixed%20Execution%20Order-purple)](#fixed-execution-order-feo)

## 🎯 Overview

This project implements a **revolutionary AI-native automotive ADAS system** using:

- **🔧 WebAssembly Component Model**: Isolated, composable automotive ECU simulation
- **⚡ Fixed Execution Order (FEO)**: Deterministic, automotive-grade component execution  
- **🤖 Real AI Processing**: YOLOv5n object detection with embedded CarND video
- **🛡️ ASIL-B Safety**: Functional safety patterns per ISO 26262
- **🎬 Video Integration**: Real automotive footage processing at 320x200 resolution

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust with WebAssembly support
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-wasip2

# Install WebAssembly tools (optional, for validation)
cargo install wasm-tools
```

### Build All Components

```bash
# Debug build (fast, larger files)
./build.sh

# Release build (optimized, smaller files)  
./build.sh release

# Clean workspace
./clean.sh
```

### Component Output

```
dist/
├── sensors-camera-front.wasm      (2.4MB - Front camera ECU)
├── ai-object-detection.wasm       (9.9MB - YOLOv5n + 3.8MB model)
├── input-video-decoder.wasm       (10MB - MP4 decoder + 3.3MB video)
├── system-feo-demo.wasm           (2.6MB - FEO orchestration demo)
└── ... (18 total components)
```

## 🏗️ Component Architecture

```
Sensors → AI Processing → Fusion → Planning → Control → Vehicle
   ↓           ↓            ↓         ↓          ↓         ↓
Camera    Detection    Environment  Trajectory  Commands  CAN Bus
Radar     Tracking     Model        Planning    Actuation
LiDAR     Prediction
```

### Component Layers

1. **Sensor Layer** (6 components)
   - Camera ECUs (front, surround)
   - Radar ECUs (front, corner)
   - LiDAR ECU
   - Ultrasonic ECU

2. **AI/Perception Layer** (3 components)
   - Object Detection AI
   - Behavior Prediction AI  
   - Perception Fusion

3. **Fusion & Tracking Layer** (2 components)
   - Sensor Fusion ECU
   - Tracking & Prediction

4. **Planning & Control Layer** (2 components)
   - Planning & Decision
   - Vehicle Control ECU

5. **Safety & Infrastructure** (4 components)
   - Safety Monitor
   - CAN Gateway
   - HMI Interface
   - ADAS Domain Controller

## 🚀 Getting Started

### Prerequisites

- Rust (latest stable)
- `wasm-tools` CLI
- `wasm32-wasip2` target

### Building

```bash
# Add WASI target if not installed
rustup target add wasm32-wasip2

# Build all components
./scripts/build-components.sh
```

### Build Output

Successfully built components will be in the `build/` directory as `.wasm` files.

## 🔧 Component Details

Each component:
- Has a single, well-defined responsibility
- Exports specific interfaces for other components to use
- Imports only what it needs from other components
- Can be tested and deployed independently

### Example: Object Detection Flow

```
Camera → Object Detection AI → Sensor Fusion → Planning
  ↓             ↓                   ↓            ↓
Frame      Detections        Environment    Trajectory
```

## 📁 Project Structure

```
adas-wasm-components/
├── components/          # Individual ADAS components
│   ├── camera-front-ecu/
│   ├── object-detection-ai/
│   ├── sensor-fusion-ecu/
│   └── ...
├── wit/                 # WebAssembly Interface Types
│   ├── interfaces/      # Shared data interfaces
│   └── worlds/          # Component world definitions
├── scripts/            # Build and utility scripts
├── build/              # Compiled WASM components
└── docs/               # Documentation
```

## 🧪 Testing

Each component can be tested independently:

```bash
cd components/sensor-fusion-ecu
cargo test
```

## 📊 Current Status

- ✅ 17 components implemented
- ✅ 12/17 components building successfully
- ✅ Modular architecture with clear data flow
- ✅ WebAssembly Component Model ready
- 🚧 5 components need WIT updates

## 🤝 Contributing

1. Each component should have a single responsibility
2. Use WIT interfaces for all inter-component communication
3. Follow the established data flow pattern
4. Add tests for new functionality

## 📄 License

Apache-2.0

## 📚 Documentation

- [Architecture Overview](docs/ADAS_ARCHITECTURE.md)
- [Component Mapping](docs/COMPONENT_MAPPING.md)
- [Build Guide](scripts/build-components.sh)