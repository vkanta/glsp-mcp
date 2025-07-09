# Bazel Build System Analysis for ADAS WebAssembly Components

## Executive Summary

This document analyzes how Bazel would integrate with the ADAS WebAssembly components build system, identifying benefits, challenges, and implementation strategies. The project's complex multi-component architecture, WIT interface dependencies, and safety-critical requirements make it an excellent candidate for Bazel's deterministic and scalable build approach.

## Current Build System Overview

### Structure
- **Workspace**: Cargo workspace with 23+ components
- **Build Tool**: Cargo + wasm-tools + wac (WebAssembly Composition)
- **Target**: wasm32-wasip2
- **Composition**: wac tool for component composition
- **Metadata**: Custom build.rs scripts embed metadata

### Key Characteristics
1. **Component Categories**: sensors, AI, control, fusion, system, integration
2. **WIT Dependencies**: Complex interface dependency graph
3. **Build Artifacts**: Individual .wasm files + composed system
4. **Safety Requirements**: ASIL-B/D compliance metadata
5. **Real-time Constraints**: 50ms cycle time target

## Bazel Benefits for ADAS Components

### 1. Deterministic Builds
```python
# Example: Reproducible WASM builds
wasm_rust_binary(
    name = "camera_front_ecu",
    srcs = ["src/lib.rs"],
    edition = "2021",
    target = "wasm32-wasip2",
    build_id = "deterministic",  # Ensures reproducible builds
)
```

### 2. Dependency Management
```python
# WIT interface dependencies as first-class citizens
wit_library(
    name = "adas_common_types",
    srcs = ["wit/interfaces/adas-common-types/types.wit"],
    deps = [],
)

wit_library(
    name = "adas_control",
    srcs = ["wit/interfaces/adas-control/control.wit"],
    deps = [":adas_common_types"],
)
```

### 3. Incremental Compilation
```python
# Component groups for parallel builds
component_group(
    name = "sensor_layer",
    components = [
        "//components/sensors/camera-front",
        "//components/sensors/camera-surround",
        "//components/sensors/radar-front",
        "//components/sensors/radar-corner",
        "//components/sensors/lidar",
        "//components/sensors/ultrasonic",
    ],
    parallel = True,
)
```

### 4. Build Configuration Management
```python
# Build configurations for debug/release/safety
config_setting(
    name = "release_build",
    values = {"compilation_mode": "opt"},
)

config_setting(
    name = "safety_critical",
    values = {"define": "safety_level=asil_d"},
)

wasm_component(
    name = "safety_monitor",
    srcs = ["src/lib.rs"],
    features = select({
        ":safety_critical": ["redundancy", "fail_safe", "diagnostics"],
        "//conditions:default": ["diagnostics"],
    }),
)
```

## Proposed Bazel Structure

### Repository Layout
```
├── WORKSPACE.bazel
├── .bazelrc
├── .bazelversion
├── BUILD.bazel              # Root build file
├── bazel/
│   ├── wasm_rules.bzl       # WASM build rules
│   ├── wit_rules.bzl        # WIT processing rules
│   ├── metadata_rules.bzl   # Component metadata
│   └── composition_rules.bzl # WAC composition rules
├── components/
│   ├── sensors/
│   │   ├── BUILD.bazel      # Sensor layer builds
│   │   └── camera-front/
│   │       ├── BUILD.bazel
│   │       ├── src/
│   │       └── wit/
│   └── ...
├── wit/
│   ├── BUILD.bazel          # WIT interface definitions
│   └── interfaces/
└── third_party/
    ├── BUILD.bazel
    └── cargo/               # Cargo dependencies
```

### WORKSPACE Configuration
```python
workspace(name = "adas_wasm_components")

# Rust toolchain
load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")
rules_rust_dependencies()
rust_register_toolchains(
    edition = "2021",
    extra_target_triples = ["wasm32-wasip2"],
)

# WASM tools
load("@rules_wasm//wasm:repositories.bzl", "wasm_rules_dependencies")
wasm_rules_dependencies()

# WIT bindgen
load("//bazel:wit_deps.bzl", "wit_bindgen_dependencies")
wit_bindgen_dependencies()

# Component metadata
load("//bazel:metadata_deps.bzl", "component_metadata_dependencies")
component_metadata_dependencies()
```

### Component BUILD File Example
```python
load("//bazel:wasm_rules.bzl", "wasm_component")
load("//bazel:metadata_rules.bzl", "component_metadata")

# Generate metadata at build time
component_metadata(
    name = "camera_front_metadata",
    component_name = "adas-camera-front-ecu",
    component_type = "sensor",
    safety_level = "ASIL-B",
    version = "0.1.0",
    toml_config = "//metadata:camera-front-ecu.toml",
)

# Build WASM component
wasm_component(
    name = "camera_front_ecu",
    srcs = glob(["src/**/*.rs"]),
    wit_bindings = [
        "//wit/components:sensor-component",
        "//wit/interfaces:adas-data",
        "//wit/interfaces:adas-control",
    ],
    metadata = ":camera_front_metadata",
    cargo_deps = [
        "@crates//:wit-bindgen",
        "@crates//:serde_json",
    ],
    rustc_flags = [
        "--cfg=feature=\"automotive\"",
        "-C", "opt-level=s",
        "-C", "lto=true",
    ],
    visibility = ["//visibility:public"],
)
```

### WIT Interface Management
```python
load("//bazel:wit_rules.bzl", "wit_library", "wit_world")

wit_library(
    name = "adas_common_types",
    srcs = ["adas-common-types/types.wit"],
    visibility = ["//visibility:public"],
)

wit_library(
    name = "adas_data",
    srcs = ["adas-data/data.wit"],
    deps = [":adas_common_types"],
    visibility = ["//visibility:public"],
)

wit_world(
    name = "sensor_component_world",
    srcs = ["sensor-component.wit"],
    deps = [
        ":adas_common_types",
        ":adas_data",
        ":adas_control",
        ":adas_orchestration",
    ],
)
```

### WAC Composition with Bazel
```python
load("//bazel:composition_rules.bzl", "wac_compose")

# Component groups for composition
filegroup(
    name = "sensor_components",
    srcs = [
        "//components/sensors/camera-front:camera_front_ecu",
        "//components/sensors/camera-surround:camera_surround_ecu",
        "//components/sensors/radar-front:radar_front_ecu",
        "//components/sensors/radar-corner:radar_corner_ecu",
        "//components/sensors/lidar:lidar_ecu",
        "//components/sensors/ultrasonic:ultrasonic_ecu",
    ],
)

# Compose complete system
wac_compose(
    name = "adas_complete_system",
    components = [
        ":sensor_components",
        "//components/ai:ai_components",
        "//components/fusion:fusion_components",
        "//components/control:control_components",
        "//components/system:system_components",
    ],
    wac_config = "wac.toml",
    validate_timing = True,
    validate_safety = True,
    output = "adas-complete-system.wasm",
)
```

### Debug/Release Build Configurations

#### .bazelrc Configuration
```bash
# Build configurations
build:debug --compilation_mode=dbg
build:debug --copt=-g
build:debug --strip=never
build:debug --define=safety_checks=enabled

build:release --compilation_mode=opt
build:release --copt=-Os  # Optimize for size
build:release --linkopt=-s  # Strip symbols
build:release --define=safety_checks=runtime

build:safety_critical --config=release
build:safety_critical --define=asil_level=D
build:safety_critical --define=redundancy=enabled
build:safety_critical --features=fail_safe

# Platform configurations
build:wasm --platforms=//platforms:wasm32_wasip2
build:wasm --features=wasm_simd
build:wasm --features=bulk_memory

# Test configurations
test:unit --test_env=RUST_BACKTRACE=1
test:integration --test_size_filters=large
test:safety --test_tag_filters=safety_critical
```

#### Build Commands
```bash
# Debug build
bazel build --config=debug --config=wasm //components/...

# Release build
bazel build --config=release --config=wasm //components/...

# Safety-critical build with validation
bazel build --config=safety_critical --config=wasm \
  //components/system/safety-monitor:safety_monitor_ecu

# Build and compose complete system
bazel build --config=release --config=wasm //:adas_complete_system

# Run all tests
bazel test --config=wasm //...

# Run safety validation tests only
bazel test --config=safety //...
```

## Advanced Bazel Features for ADAS

### 1. Remote Caching
```bash
# .bazelrc
build --remote_cache=grpc://cache.adas-team.internal:9090
build --remote_upload_local_results=true
```

### 2. Build Event Protocol for Monitoring
```bash
build --build_event_json_file=/tmp/build_events.json
build --build_metadata=branch=$GIT_BRANCH
build --build_metadata=commit=$GIT_COMMIT
```

### 3. Query for Dependency Analysis
```bash
# Find all components depending on sensor-data interface
bazel query 'rdeps(//..., //wit/interfaces:adas_data)'

# Visualize component dependencies
bazel query --output=graph 'deps(//components/ai/object-detection:object_detection_ai)' | dot -Tpng > deps.png
```

### 4. Aspect-Based Metadata Collection
```python
# Collect metadata from all components
ComponentInfo = provider(fields = ["name", "type", "safety_level", "interfaces"])

def _collect_metadata_aspect_impl(target, ctx):
    # Collect metadata from component
    pass

collect_metadata_aspect = aspect(
    implementation = _collect_metadata_aspect_impl,
    attr_aspects = ["deps"],
)
```

## Migration Strategy

### Phase 1: Parallel Build System (Weeks 1-2)
- Set up Bazel alongside Cargo
- Create basic WASM rules
- Build one component as proof-of-concept

### Phase 2: WIT Integration (Weeks 3-4)
- Implement WIT processing rules
- Create dependency graph for interfaces
- Generate bindings with Bazel

### Phase 3: Component Migration (Weeks 5-8)
- Migrate components layer by layer
- Start with sensor layer (simpler dependencies)
- Progress to AI and control layers

### Phase 4: Composition Integration (Weeks 9-10)
- Implement wac composition rules
- Create build configurations
- Set up CI/CD with Bazel

### Phase 5: Full Migration (Weeks 11-12)
- Remove Cargo build scripts
- Update documentation
- Train team on Bazel

## Benefits Summary

1. **Reproducible Builds**: Critical for automotive safety certification
2. **Incremental Compilation**: Faster development cycles
3. **Dependency Visibility**: Clear understanding of component relationships
4. **Build Configuration**: Unified debug/release/safety configurations
5. **Scalability**: Handles growing number of components efficiently
6. **Remote Execution**: Distribute builds across build farm
7. **Caching**: Share build artifacts across team
8. **Toolchain Management**: Consistent toolchain versions

## Challenges and Solutions

### Challenge 1: WIT Dependency Management
**Solution**: Custom Starlark rules for WIT processing with proper dependency tracking

### Challenge 2: Metadata Generation
**Solution**: Build-time providers and aspects to collect and embed metadata

### Challenge 3: WAC Integration
**Solution**: Custom rule wrapping wac tool with proper input/output tracking

### Challenge 4: Cargo Dependencies
**Solution**: Use rules_rust's crate_universe for Cargo dependency management

### Challenge 5: Team Training
**Solution**: Gradual migration with extensive documentation and examples

## Conclusion

Bazel provides significant advantages for the ADAS WebAssembly components project:
- Deterministic builds essential for safety-critical automotive software
- Efficient handling of complex component dependencies
- Unified build configuration management
- Scalable to hundreds of components
- Better visibility into build process and dependencies

The migration requires initial investment but will pay dividends in:
- Reduced build times through better caching
- Improved developer productivity
- Enhanced build reproducibility for certification
- Better dependency management for WIT interfaces
- Unified tooling for entire ADAS system