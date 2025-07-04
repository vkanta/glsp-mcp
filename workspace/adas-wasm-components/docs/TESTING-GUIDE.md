# 🧪 ADAS WebAssembly Components Testing Guide

## Overview

This guide explains how to properly test the ADAS WebAssembly components, from unit tests to full pipeline integration testing.

## Testing Levels

### 1. Unit Tests (Component-Level)

Each component has its own unit tests that run in native Rust:

```bash
# Test a single component
cd components/ai/object-detection
cargo test

# Test with output
cargo test -- --nocapture

# Test specific function
cargo test test_object_detection
```

### 2. WASM Component Testing

To test the actual WASM binaries, you need a WASM runtime:

#### Using wasmtime

```bash
# Install wasmtime
curl https://wasmtime.dev/install.sh -sSf | bash

# Build the component
cd components/ai/object-detection
cargo build --target wasm32-wasip2

# Run with wasmtime
wasmtime run target/wasm32-wasip2/debug/adas_object_detection_ai.wasm

# With component model support
wasmtime run --wasm component-model target/wasm32-wasip2/debug/adas_object_detection_ai.wasm
```

#### Using WASI SDK

```bash
# For components that need WASI preview2
wasmtime run --wasi-modules experimental-wasi-nn \
    target/wasm32-wasip2/debug/adas_object_detection_ai.wasm
```

### 3. Integration Testing

The test files demonstrate different integration scenarios:

#### a) **test-pipeline.rs** - Basic 5-component pipeline
```bash
# Compile and run the basic pipeline test
rustc test-pipeline.rs -o test-pipeline
./test-pipeline

# This tests:
# - Video Decoder → Object Detection → Visualizer → Safety Monitor
# - Orchestrator message routing
# - 30 FPS pipeline execution
```

#### b) **test-graphics-pipeline.rs** - Graphics visualization pipeline
```bash
# Compile and run the graphics pipeline test
rustc test-graphics-pipeline.rs -o test-graphics-pipeline
./test-graphics-pipeline

# This tests:
# - Real-time graphics rendering
# - Object detection overlay visualization
# - Frame buffer operations
# - Performance metrics display
```

### 4. Component Composition Testing

Test how components work together using the WIT interfaces:

```bash
# Build all components
./build.sh

# Check component interfaces
wasm-tools component wit dist/ai-object-detection.wasm

# Validate component
wasm-tools validate dist/ai-object-detection.wasm

# Check imports/exports
wasm-tools component inspect dist/ai-object-detection.wasm
```

### 5. Performance Testing

#### Frame Rate Test
```rust
// In test-graphics-pipeline.rs
let start_time = Instant::now();
let target_frame_time = Duration::from_millis(33); // 30 FPS

while start_time.elapsed() < Duration::from_secs(5) {
    // Pipeline execution
    let frame_time = frame_start.elapsed();
    if frame_time < target_frame_time {
        thread::sleep(target_frame_time - frame_time);
    }
}

// Measure actual FPS
let avg_fps = frame_count as f32 / total_time;
println!("Average FPS: {:.1}", avg_fps);
```

#### Memory Usage Test
```bash
# Monitor memory usage during execution
/usr/bin/time -l ./test-graphics-pipeline

# Or use heaptrack for detailed analysis
heaptrack ./test-pipeline
```

### 6. Mock Component Testing

The test files use mock components to simulate the real WASM components:

```rust
// Mock Video Decoder
pub struct VideoDecoder {
    frame_count: u64,
    resolution: (u32, u32),
}

impl VideoDecoder {
    pub fn decode_frame(&mut self) -> VideoFrame {
        // Generate test pattern with moving objects
        // Simulates real video decoding
    }
}

// Mock Object Detector
pub struct ObjectDetector {
    pub fn detect(&self, frame: &VideoFrame) -> DetectionResult {
        // Simulate AI processing delay
        thread::sleep(Duration::from_millis(10));
        
        // Generate realistic detection results
        // with moving bounding boxes
    }
}
```

### 7. Data Flow Testing

Test the pub/sub messaging between components:

```rust
// In orchestrator
pub fn route_message(&mut self, from: &str, to: &str) {
    self.total_messages += 1;
    
    // Verify message routing
    match (from, to) {
        ("video-decoder", "object-detection") => {
            // Video frame routing
        }
        ("object-detection", "graphics-visualizer") => {
            // Detection results routing
        }
        _ => {}
    }
}
```

### 8. Graphics Rendering Testing

The graphics pipeline test validates:

```rust
// Frame buffer operations
pub fn composite_overlay(&mut self, overlay: &OverlayRenderer) {
    // Alpha blending test
    for each pixel:
        if overlay_pixel.a == 255:
            // Opaque - replace
        else:
            // Semi-transparent - blend
}

// Overlay rendering
pub fn draw_bounding_box(&mut self, bbox: &BoundingBox, color: Color) {
    // Test rectangle drawing
    // Test text label rendering
}
```

### 9. Safety Monitor Testing

Validate safety checks:

```rust
pub fn check_frame(&mut self, detections: &DetectionResult) {
    // Check for safety conditions
    for obj in &detections.objects {
        if obj.class_name == "person" && obj.bounding_box.x < 100.0 {
            alerts.push("⚠️ Person detected in danger zone!");
        }
    }
}
```

### 10. End-to-End Testing Checklist

- [ ] Build all components: `./build.sh`
- [ ] Validate WASM files: `wasm-tools validate dist/*.wasm`
- [ ] Run unit tests: `cargo test --workspace`
- [ ] Run pipeline test: `./test-pipeline`
- [ ] Run graphics test: `./test-graphics-pipeline`
- [ ] Check performance metrics (FPS, latency)
- [ ] Verify memory usage is within bounds
- [ ] Test error handling and edge cases

## Testing Best Practices

1. **Isolate Components**: Test each component independently first
2. **Mock External Dependencies**: Use mock implementations for testing
3. **Test Data Flow**: Verify messages flow correctly between components
4. **Performance Baselines**: Establish and monitor performance metrics
5. **Edge Cases**: Test with various video resolutions, object counts, etc.

## Debugging Tips

```bash
# Enable debug output
RUST_LOG=debug cargo test

# Use print debugging in WASM
println!("🎯 Debug: frame {} processed", frame_number);

# Check WASM module structure
wasm-tools print dist/ai-object-detection.wasm | less

# Disassemble for low-level debugging
wasm-tools disassemble dist/ai-object-detection.wasm
```

## Continuous Integration

For CI/CD pipelines:

```yaml
# .github/workflows/test.yml
- name: Build Components
  run: ./build.sh release

- name: Validate WASM
  run: |
    cargo install wasm-tools
    wasm-tools validate dist/*.wasm

- name: Run Tests
  run: |
    cargo test --workspace
    ./test-pipeline
    ./test-graphics-pipeline

- name: Check Performance
  run: |
    # Ensure 90%+ efficiency
    ./test-pipeline | grep "Efficiency" | awk '{if ($2 < 90) exit 1}'
```

## Summary

The testing approach for ADAS WebAssembly components includes:

1. **Unit tests** for individual component logic
2. **WASM validation** for component structure
3. **Integration tests** for data flow between components
4. **Performance tests** for real-time requirements
5. **Mock components** for isolated testing
6. **End-to-end pipeline** validation

The `test-graphics-pipeline.rs` demonstrates a complete testing scenario with:
- 5 integrated components
- Real-time graphics visualization
- Performance monitoring (96.4% efficiency achieved)
- Safety monitoring integration
- Realistic data simulation