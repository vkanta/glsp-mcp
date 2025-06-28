# ASIL-B Functional Safety Interface Design Proposal

## Executive Summary

This document proposes a comprehensive redesign of all WebAssembly Interface Types (WIT) to comply with ASIL-B functional safety requirements according to ISO 26262. The proposal introduces standardized error handling patterns, diagnostic capabilities, and streaming interfaces with built-in safety mechanisms.

## Background

### ASIL-B Requirements
- **Risk Level**: Systems where failure could cause non-serious injuries
- **Diagnostic Coverage**: Minimum 90% single point fault metric (SPFM)
- **Independence**: Level I0 (minimal independence for audits)
- **Examples**: Brake lights, headlights, reversing cameras, instrument clusters

### Key Safety Principles
1. **Fail-Safe Design**: All interfaces must gracefully handle failures
2. **Diagnostic Coverage**: Every component must report its health status
3. **Deterministic Behavior**: Predictable timing and resource usage
4. **Error Propagation**: Clear error reporting through the system
5. **Redundancy Support**: Interfaces should support redundant data paths

## Proposed Interface Patterns

### 1. Safety Result Type
Every function that can fail MUST return a `safety-result` type:

```wit
variant safety-result<T> {
    ok(T),
    degraded(degraded-result<T>),
    error(safety-error),
}

record degraded-result<T> {
    value: T,
    warnings: list<safety-warning>,
    confidence: f32,  // 0.0 to 1.0
}

record safety-error {
    code: error-code,
    severity: error-severity,
    message: string,
    timestamp: u64,
    component: string,
    diagnostic-data: option<list<u8>>,
}

enum error-severity {
    critical,    // System must transition to safe state
    major,       // Function degraded but operational
    minor,       // Performance impact only
    info,        // Diagnostic information
}

record safety-warning {
    code: warning-code,
    message: string,
    timestamp: u64,
}
```

### 2. Safety Stream Pattern
For continuous data flows (sensors, AI outputs), use safety-aware streams:

```wit
resource safety-stream<T> {
    /// Get next value with safety status
    next: func() -> stream-result<T>;
    
    /// Check stream health without consuming data
    health-check: func() -> stream-health;
    
    /// Get diagnostic information
    get-diagnostics: func() -> stream-diagnostics;
    
    /// Subscribe to health status changes
    subscribe-health: func(callback: health-callback);
    
    /// Configure timeout and retry behavior
    configure: func(config: stream-config) -> result<_, safety-error>;
}

variant stream-result<T> {
    data(safety-data<T>),
    timeout,
    end-of-stream,
    error(safety-error),
}

record safety-data<T> {
    value: T,
    timestamp: u64,
    sequence-number: u64,
    validity: data-validity,
    source-health: component-health,
}

record data-validity {
    confidence: f32,        // 0.0 to 1.0
    quality-flags: u32,     // Bit flags for quality indicators
    age-ms: u32,           // Data age in milliseconds
}

enum component-health {
    healthy,
    degraded,
    faulty,
    unknown,
}
```

### 3. Diagnostic Interface
Every component MUST implement a diagnostic interface:

```wit
interface safety-diagnostics {
    /// Perform self-test
    self-test: func() -> safety-result<diagnostic-report>;
    
    /// Get current health status
    get-health: func() -> component-health-status;
    
    /// Get performance metrics
    get-metrics: func() -> performance-metrics;
    
    /// Configure diagnostic parameters
    configure-diagnostics: func(config: diagnostic-config) -> result<_, safety-error>;
    
    /// Register for health notifications
    subscribe-health-changes: func(callback: health-change-callback);
}

record component-health-status {
    overall-health: component-health,
    sub-components: list<sub-component-health>,
    last-self-test: u64,
    uptime-ms: u64,
    error-count: u32,
    warning-count: u32,
}

record diagnostic-report {
    timestamp: u64,
    test-results: list<test-result>,
    recommendations: list<string>,
    overall-status: component-health,
}
```

### 4. Watchdog Pattern
Components processing critical data must implement watchdog interfaces:

```wit
interface watchdog {
    /// Start watchdog with timeout
    start: func(timeout-ms: u32, action: timeout-action) -> result<watchdog-handle, safety-error>;
    
    /// Reset watchdog timer
    kick: func(handle: watchdog-handle) -> result<_, safety-error>;
    
    /// Stop watchdog
    stop: func(handle: watchdog-handle) -> result<_, safety-error>;
    
    /// Get watchdog status
    status: func(handle: watchdog-handle) -> watchdog-status;
}

enum timeout-action {
    notify,           // Just send notification
    degrade,          // Switch to degraded mode
    safe-state,       // Transition to safe state
    reset-component,  // Reset the component
}
```

## Example: Refactored Camera Interface

```wit
package adas:sensor-data@0.1.0;

interface camera-data {
    use safety-types.{safety-result, safety-stream, component-health};
    
    /// Camera configuration with safety parameters
    record camera-config {
        resolution: resolution,
        fps: u32,
        exposure-mode: exposure-mode,
        // Safety additions
        frame-timeout-ms: u32,
        min-quality-threshold: f32,
        enable-redundancy: bool,
        diagnostic-interval-ms: u32,
    }
    
    /// Frame with safety metadata
    record camera-frame {
        // Original data
        data: list<u8>,
        width: u32,
        height: u32,
        format: pixel-format,
        timestamp: u64,
        
        // Safety additions
        sequence-number: u64,
        validity: frame-validity,
        capture-diagnostics: capture-diagnostics,
    }
    
    record frame-validity {
        exposure-quality: f32,      // 0.0 to 1.0
        focus-quality: f32,         // 0.0 to 1.0
        motion-blur: f32,          // 0.0 (none) to 1.0 (severe)
        lighting-conditions: lighting-assessment,
        integrity-check: u32,       // CRC or checksum
    }
    
    /// Create camera stream with safety features
    create-stream: func(config: camera-config) -> safety-result<safety-stream<camera-frame>>;
    
    /// Get camera diagnostics
    get-diagnostics: func() -> camera-diagnostics;
    
    /// Perform camera self-test
    self-test: func() -> safety-result<camera-test-report>;
}
```

## Example: Refactored AI Detection Interface

```wit
interface detection-data {
    use safety-types.{safety-result, safety-stream, data-validity};
    
    /// Detection results with safety metadata
    record detection-results {
        objects: list<detected-object>,
        timestamp: u64,
        frame-id: u64,
        
        // Safety additions
        processing-time-ms: u32,
        model-confidence: f32,
        input-quality: data-validity,
        diagnostics: detection-diagnostics,
    }
    
    record detected-object {
        // Original fields
        object-id: u32,
        object-type: object-type,
        position: position-3d,
        velocity: velocity-3d,
        bounding-box: bounding-box-3d,
        confidence: f32,
        
        // Safety additions
        detection-quality: detection-quality,
        tracking-reliability: f32,
        time-since-last-update-ms: u32,
    }
    
    record detection-quality {
        spatial-accuracy: f32,      // Position accuracy estimate
        classification-certainty: f32,
        tracking-consistency: f32,
        sensor-agreement: f32,      // For fusion systems
    }
    
    /// Create detection stream with safety monitoring
    create-stream: func() -> safety-result<safety-stream<detection-results>>;
    
    /// Configure safety thresholds
    configure-safety: func(config: safety-config) -> safety-result<_>;
    
    /// Get AI model health status
    get-model-health: func() -> model-health-status;
}
```

## Implementation Guidelines

### 1. Error Handling in Rust

```rust
use crate::safety_types::{SafetyResult, SafetyError, ErrorSeverity};

impl camera_data::Guest for Component {
    fn create_stream(config: camera_data::CameraConfig) -> SafetyResult<CameraStream> {
        // Validate configuration
        if config.fps > 120 || config.fps < 10 {
            return SafetyResult::Error(SafetyError {
                code: ErrorCode::InvalidConfiguration,
                severity: ErrorSeverity::Major,
                message: format!("FPS {} out of safe range [10, 120]", config.fps),
                timestamp: get_timestamp(),
                component: "camera-front".to_string(),
                diagnostic_data: None,
            });
        }
        
        // Initialize with timeout monitoring
        let watchdog = start_watchdog(config.frame_timeout_ms)?;
        
        // Create stream with safety wrapper
        match create_camera_stream_internal(config) {
            Ok(stream) => {
                let safety_stream = SafetyStream::new(stream, watchdog);
                SafetyResult::Ok(safety_stream)
            }
            Err(e) if e.is_recoverable() => {
                // Degraded mode - lower resolution/fps
                let degraded_config = config.to_degraded();
                let stream = create_camera_stream_internal(degraded_config)?;
                SafetyResult::Degraded(DegradedResult {
                    value: SafetyStream::new(stream, watchdog),
                    warnings: vec![SafetyWarning {
                        code: WarningCode::DegradedOperation,
                        message: "Operating at reduced resolution".to_string(),
                        timestamp: get_timestamp(),
                    }],
                    confidence: 0.7,
                })
            }
            Err(e) => SafetyResult::Error(e.to_safety_error()),
        }
    }
}
```

### 2. Stream Implementation Pattern

```rust
pub struct SafetyStreamState<T> {
    inner: Box<dyn Stream<T>>,
    watchdog: WatchdogHandle,
    sequence_number: u64,
    last_timestamp: u64,
    health_status: ComponentHealth,
    diagnostics: StreamDiagnostics,
}

impl<T> safety_stream::GuestSafetyStream<T> for SafetyStreamState<T> {
    fn next(&self) -> StreamResult<T> {
        // Reset watchdog
        self.watchdog.kick()?;
        
        // Check timeout
        let start_time = get_timestamp();
        match self.inner.next_with_timeout(self.config.timeout_ms) {
            Some(value) => {
                self.sequence_number += 1;
                
                // Validate data freshness
                let age_ms = (get_timestamp() - value.timestamp) as u32;
                if age_ms > self.config.max_age_ms {
                    return StreamResult::Error(SafetyError {
                        code: ErrorCode::StaleData,
                        severity: ErrorSeverity::Major,
                        message: format!("Data age {}ms exceeds limit", age_ms),
                        // ...
                    });
                }
                
                StreamResult::Data(SafetyData {
                    value,
                    timestamp: get_timestamp(),
                    sequence_number: self.sequence_number,
                    validity: calculate_validity(&value),
                    source_health: self.health_status,
                })
            }
            None if elapsed_ms(start_time) >= self.config.timeout_ms => {
                self.health_status = ComponentHealth::Degraded;
                StreamResult::Timeout
            }
            None => StreamResult::EndOfStream,
        }
    }
    
    fn health_check(&self) -> StreamHealth {
        StreamHealth {
            status: self.health_status,
            last_data_timestamp: self.last_timestamp,
            sequence_gaps: self.diagnostics.sequence_gaps,
            timeout_count: self.diagnostics.timeout_count,
            error_count: self.diagnostics.error_count,
        }
    }
}
```

### 3. Diagnostic Implementation

```rust
impl safety_diagnostics::Guest for Component {
    fn self_test() -> SafetyResult<DiagnosticReport> {
        let mut test_results = Vec::new();
        
        // Test 1: Memory integrity
        test_results.push(test_memory_integrity());
        
        // Test 2: Configuration validation
        test_results.push(test_configuration_validity());
        
        // Test 3: Communication interfaces
        test_results.push(test_interfaces());
        
        // Test 4: Processing pipeline
        test_results.push(test_processing_pipeline());
        
        let overall_status = determine_overall_status(&test_results);
        
        SafetyResult::Ok(DiagnosticReport {
            timestamp: get_timestamp(),
            test_results,
            recommendations: generate_recommendations(&test_results),
            overall_status,
        })
    }
}
```

## Migration Strategy

### Phase 1: Core Safety Types (Week 1-2)
1. Create `safety-types.wit` with all common types
2. Create Rust implementation crate with helper functions
3. Add comprehensive tests for safety mechanisms

### Phase 2: Sensor Interfaces (Week 3-4)
1. Refactor camera interfaces (front, surround)
2. Refactor lidar interface
3. Refactor radar interfaces (front, corner)
4. Refactor ultrasonic interface

### Phase 3: AI/Processing Interfaces (Week 5-6)
1. Refactor object-detection interface
2. Refactor behavior-prediction interface
3. Refactor sensor-fusion interface
4. Refactor tracking-prediction interface

### Phase 4: Control Interfaces (Week 7-8)
1. Refactor planning-decision interface
2. Refactor vehicle-control interface
3. Refactor safety-monitor interface
4. Refactor domain-controller interface

### Phase 5: System Integration (Week 9-10)
1. Update all component implementations
2. Add end-to-end safety tests
3. Performance validation
4. Documentation and training

## Benefits

1. **Safety Compliance**: Meets ASIL-B requirements for automotive systems
2. **Error Visibility**: All failure modes are explicit and handled
3. **Diagnostic Coverage**: >90% fault detection capability
4. **Graceful Degradation**: Systems can operate at reduced capability
5. **Standardization**: Consistent patterns across all interfaces
6. **Testability**: Built-in self-test and diagnostic capabilities
7. **Traceability**: All errors include timestamp and component origin

## Conclusion

This proposal provides a comprehensive framework for upgrading all ADAS WebAssembly components to meet ASIL-B functional safety requirements. The standardized patterns ensure consistent error handling, diagnostic capabilities, and graceful degradation across the entire system.

The use of Result types, safety streams, and diagnostic interfaces provides the foundation for building reliable automotive software that can detect, report, and recover from failures while maintaining safe operation.