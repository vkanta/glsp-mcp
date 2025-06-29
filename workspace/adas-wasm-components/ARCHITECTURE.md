# ADAS Standardized Component Architecture

## Overview

This project implements a standardized ADAS (Advanced Driver Assistance Systems) component architecture using WebAssembly Component Model and WIT (WebAssembly Interface Types) interfaces. The architecture provides modular, interoperable components with standardized interfaces for sensor data, AI processing, vehicle control, and system management.

## Architecture Principles

- **Standardized Interfaces**: Common interface contracts across all component types
- **Modular Design**: Independent, composable components with clear boundaries
- **Type Safety**: WIT-based interface definitions ensure compile-time type safety
- **Performance**: Optimized WebAssembly modules for real-time automotive applications
- **Interoperability**: Components can communicate through standardized data exchange

## Component Categories

### 🔧 Sensor Components (6 components)
**World**: `sensor-component`
**Purpose**: Hardware sensor abstraction and data acquisition

- **adas_camera_front_ecu.wasm** - Front-facing camera sensor
- **adas_camera_surround_ecu.wasm** - 360° surround view cameras
- **adas_radar_front_ecu.wasm** - Front radar sensor
- **adas_radar_corner_ecu.wasm** - Corner radar sensors
- **adas_lidar_ecu.wasm** - LiDAR point cloud sensor
- **adas_ultrasonic_ecu.wasm** - Ultrasonic proximity sensors

**Interfaces**:
- Export: `adas:control/sensor-control`, `adas:data/sensor-data`
- Import: `adas:orchestration/*`

### 🤖 AI Components (2 components)
**World**: `ai-component`
**Purpose**: Machine learning and perception processing

- **adas_object_detection_ai.wasm** - Object detection and classification
- **adas_behavior_prediction_ai.wasm** - Vehicle behavior prediction

**Interfaces**:
- Export: `adas:control/ai-control`, `adas:data/perception-data`
- Import: `adas:data/sensor-data`, `adas:orchestration/*`

### 🎯 Control Components (2 components)
**World**: `control-component`
**Purpose**: Planning, decision making, and vehicle control

- **adas_vehicle_control_ecu.wasm** - Vehicle actuator control (steering, throttle, brake)
- **adas_planning_decision.wasm** - Path planning and decision logic

**Interfaces**:
- Export: `adas:data/planning-data`
- Import: `adas:data/perception-data`, `adas:orchestration/*`

### 🛡️ System Components (3 components)
**World**: `system-component`
**Purpose**: System monitoring, safety, and communication

- **adas_safety_monitor.wasm** - Safety monitoring and emergency response
- **adas_hmi_interface.wasm** - Human-machine interface management
- **adas_can_gateway.wasm** - CAN bus communication gateway

**Interfaces**:
- Export: `adas:diagnostics/*`
- Import: `adas:data/*`, `adas:orchestration/*`

## Standardized Interface Packages

### adas:control
Type-specific control interfaces for different component categories:
- `sensor-control` - Sensor initialization, configuration, and control
- `ai-control` - AI model loading, inference control
- `vehicle-control` - Vehicle actuator control

### adas:data
Standardized data exchange formats:
- `sensor-data` - Raw sensor measurements and metadata
- `perception-data` - Processed perception results (objects, lanes, etc.)
- `planning-data` - Planning decisions and trajectories

### adas:diagnostics
Health and performance monitoring:
- `health-monitoring` - Component health reports and diagnostics
- `performance-monitoring` - Performance metrics and resource usage

### adas:orchestration
System-level coordination (FEO compliant):
- `execution-control` - Component lifecycle and execution management
- `resource-management` - Resource allocation and scheduling

## Build System

### Quick Build
```bash
# Build all standardized components
./build-standardized.sh
```

### Manual Build
```bash
# Build specific component
cargo build --target wasm32-wasip2 --package adas-camera-front-ecu

# Build all components
cargo build --target wasm32-wasip2 --workspace
```

### Requirements
- Rust (latest stable)
- wasm32-wasip2 target: `rustup target add wasm32-wasip2`
- Enhanced wit-bindgen with dependency management

## Generated Artifacts

All components compile to optimized WebAssembly modules (~2.2-3.0 MB each):

```
📦 target/wasm32-wasip2/debug/
├── 🔧 adas_camera_front_ecu.wasm (3.0M)
├── 🔧 adas_camera_surround_ecu.wasm (2.3M)
├── 🔧 adas_radar_front_ecu.wasm (2.3M)
├── 🔧 adas_radar_corner_ecu.wasm (2.3M)
├── 🔧 adas_lidar_ecu.wasm (2.2M)
├── 🔧 adas_ultrasonic_ecu.wasm (2.3M)
├── 🤖 adas_object_detection_ai.wasm (2.3M)
├── 🤖 adas_behavior_prediction_ai.wasm (2.3M)
├── 🎯 adas_vehicle_control_ecu.wasm (2.2M)
├── 🎯 adas_planning_decision.wasm (2.2M)
├── 🛡️ adas_safety_monitor.wasm (2.2M)
├── 🛡️ adas_hmi_interface.wasm (2.2M)
└── 🛡️ adas_can_gateway.wasm (2.2M)
```

## Interface Compatibility

### Data Flow
```
Sensors → AI Components → Control Components → Vehicle Actuators
   ↓           ↓              ↓
System Components (Monitoring, Safety, HMI)
```

### Cross-Component Communication
- **Sensor Data**: Sensors export data consumed by AI components
- **Perception Data**: AI components export processed data for control components
- **Planning Data**: Control components export decisions for vehicle actuation
- **Diagnostics**: All components export health/performance data to system components

## Development Guidelines

### Adding New Components
1. Choose appropriate world type (`sensor-component`, `ai-component`, `control-component`, `system-component`)
2. Implement required interface exports for the chosen world
3. Use standardized dependency structure with proper WIT definitions
4. Follow naming convention: `adas-{category}_{name}_{type}`

### Interface Extensions
- Extend existing interface packages rather than creating new ones
- Maintain backward compatibility
- Follow semantic versioning for interface changes

### Testing
- Each component should implement diagnostic interfaces
- Use standardized performance metrics
- Test inter-component data exchange through standard interfaces

## Technical Features

- ✅ **13 Production Components** - Fully implemented and tested
- ✅ **4 Standardized Worlds** - Sensor, AI, Control, System component types
- ✅ **Type-Safe Interfaces** - WIT-based interface definitions
- ✅ **Resource Monitoring** - Comprehensive performance and health diagnostics
- ✅ **FEO Compliance** - Functional Execution Orchestration support
- ✅ **WebAssembly Components** - Modern WASM Component Model
- ✅ **Real-Time Performance** - Optimized for automotive real-time requirements

## Future Enhancements

- Component composition and orchestration runtime
- Dynamic component loading and unloading
- Advanced inter-component communication patterns
- Integration with automotive middleware (AUTOSAR, etc.)
- Component marketplace and versioning system