# ADAS Build Utilities

This directory contains shared build utilities and macros for the ADAS WebAssembly components project.

## Macros

### `adas_component()`
Standard ADAS component build with consistent configuration.

### `adas_ai_component()`
AI component with WASI-NN integration and model embedding.

### `adas_sensor_component()`
Sensor component with standard sensor interfaces and data flow.

### `adas_system_component()`
System component with diagnostics and optional safety features.

### `adas_component_group()`
Group multiple ADAS components into a composed system.

## Usage

Load the macros in your BUILD.bazel file:

```python
load("//adas-build:adas_component.bzl", "adas_ai_component", "adas_sensor_component")

adas_ai_component(
    name = "object_detection_ai",
    srcs = ["src/lib.rs"],
    wit_world = "wit/world.wit",
    model_files = ["models/yolov5n.onnx"],
)
```

## Benefits

- **Reduced Duplication**: Eliminates repetitive BUILD file configurations
- **Consistency**: Ensures all components follow the same patterns
- **Type Safety**: Component-specific macros enforce correct dependencies
- **Maintainability**: Changes to build patterns only need to be made in one place