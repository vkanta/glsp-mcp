# WASM Component Simulation Architecture

## Executive Summary

This document outlines a comprehensive architecture for a time-based simulation system that:
- Replays sensor data from a database
- Orchestrates linked WASM components
- Controls timing and execution order
- Renders component outputs
- Provides simulation controls (play/pause/speed)

## Core Concepts

### 1. Simulation Timeline

```
Timeline: [T0]--[T1]--[T2]--[T3]--[T4]--[T5]-->
           |     |     |     |     |     |
Sensors:   S1    S1    S1    S1    S1    S1
           S2    S2    S2    S2    S2    S2
           S3    S3    S3    S3    S3    S3
           
Components: C1 -> C2 -> C3 -> Render
             ↓     ↓     ↓
           State State State
```

### 2. Component Graph

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Camera    │────▶│  Detection  │────▶│   Fusion    │
│  Component  │     │  Component  │     │  Component  │
└─────────────┘     └─────────────┘     └─────────────┘
       │                    │                    │
       ▼                    ▼                    ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Radar     │────▶│  Tracking   │────▶│  Planning   │
│  Component  │     │  Component  │     │  Component  │
└─────────────┘     └─────────────┘     └─────────────┘
```

## Architecture Components

### 1. Sensor Database Layer

```rust
// Sensor data model
pub struct SensorReading {
    pub sensor_id: String,
    pub timestamp: i64,  // Microseconds since epoch
    pub data_type: SensorDataType,
    pub payload: Vec<u8>,
    pub metadata: HashMap<String, Value>,
}

pub enum SensorDataType {
    Camera { format: ImageFormat, resolution: (u32, u32) },
    Radar { points: Vec<RadarPoint> },
    Lidar { cloud: PointCloud },
    Ultrasonic { distance: f32 },
    IMU { acceleration: Vec3, rotation: Vec3 },
    GPS { lat: f64, lon: f64, altitude: f64 },
    CAN { message_id: u32, data: [u8; 8] },
}

// Time-series database interface
pub trait SensorDatabase {
    async fn get_readings(
        &self,
        start_time: i64,
        end_time: i64,
        sensors: &[String],
    ) -> Result<Vec<SensorReading>>;
    
    async fn get_reading_at_time(
        &self,
        timestamp: i64,
        sensor_id: &str,
    ) -> Result<Option<SensorReading>>;
    
    async fn get_time_range(&self) -> Result<(i64, i64)>;
}
```

### 2. Component Composition Layer (WAC)

```wit
// sensor-interface.wit
interface sensor-data {
    record sensor-reading {
        sensor-id: string,
        timestamp: u64,
        data: list<u8>,
    }
    
    import get-sensor-data: func(sensor-id: string) -> option<sensor-reading>
}

// component interfaces
interface camera-processing {
    use sensor-data.{sensor-reading}
    
    record detected-object {
        id: string,
        class: string,
        confidence: float32,
        bbox: bbox2d,
    }
    
    export process-frame: func(reading: sensor-reading) -> list<detected-object>
}

interface sensor-fusion {
    use camera-processing.{detected-object}
    use radar-processing.{radar-track}
    
    record fused-object {
        id: string,
        position: vec3,
        velocity: vec3,
        classification: string,
        sources: list<string>,
    }
    
    export fuse-detections: func(
        camera-objects: list<detected-object>,
        radar-tracks: list<radar-track>,
    ) -> list<fused-object>
}
```

### 3. Simulation Engine

```rust
pub struct SimulationEngine {
    // Component graph
    component_graph: ComponentGraph,
    
    // Timing control
    simulation_clock: SimulationClock,
    
    // Data flow
    data_bus: DataBus,
    
    // State management
    state_store: StateStore,
    
    // Execution engine
    wasm_runtime: WasmRuntime,
}

pub struct SimulationClock {
    current_time: AtomicI64,
    playback_speed: AtomicF32,
    is_playing: AtomicBool,
    tick_interval: Duration,
}

pub struct ComponentGraph {
    nodes: HashMap<ComponentId, ComponentNode>,
    edges: Vec<ComponentEdge>,
    execution_order: Vec<ComponentId>,
}

impl SimulationEngine {
    pub async fn step(&mut self, target_time: i64) -> Result<SimulationState> {
        // 1. Fetch sensor data for current time window
        let sensor_data = self.fetch_sensor_data(target_time).await?;
        
        // 2. Execute components in topological order
        for component_id in &self.component_graph.execution_order {
            let inputs = self.gather_inputs(component_id, &sensor_data)?;
            let outputs = self.execute_component(component_id, inputs).await?;
            self.data_bus.publish(component_id, outputs)?;
        }
        
        // 3. Collect visualization data
        let viz_data = self.collect_visualization_data()?;
        
        // 4. Update simulation state
        self.update_state(target_time, viz_data)?;
        
        Ok(self.get_current_state())
    }
}
```

### 4. Timing and Synchronization

```rust
pub struct TimingController {
    // Simulation time (can be paused, sped up, etc.)
    simulation_time: i64,
    
    // Real-world time tracking
    real_time_start: Instant,
    
    // Playback control
    playback_speed: f32,
    is_paused: bool,
    
    // Component timing
    component_timings: HashMap<ComponentId, ComponentTiming>,
}

pub struct ComponentTiming {
    // Expected execution frequency
    frequency_hz: f32,
    
    // Last execution time
    last_execution: i64,
    
    // Execution statistics
    avg_duration_ms: f32,
    max_duration_ms: f32,
}

impl TimingController {
    pub fn should_execute(&self, component_id: &ComponentId, current_time: i64) -> bool {
        let timing = &self.component_timings[component_id];
        let period_us = (1_000_000.0 / timing.frequency_hz) as i64;
        current_time - timing.last_execution >= period_us
    }
    
    pub fn wait_for_next_tick(&self) -> Duration {
        // Calculate how long to wait based on playback speed
        let tick_duration = Duration::from_millis(10); // 100Hz base rate
        if self.playback_speed > 0.0 {
            tick_duration.div_f32(self.playback_speed)
        } else {
            Duration::from_secs(1) // Paused
        }
    }
}
```

### 5. Data Flow and Bridging

```rust
// Bridge between sensor data and WASM components
pub struct SensorDataBridge {
    // Current sensor readings
    current_readings: Arc<RwLock<HashMap<String, SensorReading>>>,
    
    // Interpolation for missing data
    interpolator: SensorInterpolator,
    
    // Data transformation
    transformers: HashMap<SensorDataType, Box<dyn DataTransformer>>,
}

impl SensorDataBridge {
    // Called by WASM components via host functions
    pub fn get_sensor_data(&self, sensor_id: &str, timestamp: i64) -> Result<Vec<u8>> {
        let readings = self.current_readings.read().unwrap();
        
        if let Some(reading) = readings.get(sensor_id) {
            // Direct match
            Ok(reading.payload.clone())
        } else {
            // Try interpolation
            self.interpolator.interpolate(sensor_id, timestamp)
        }
    }
    
    // Update bridge with new sensor data
    pub fn update_readings(&self, readings: Vec<SensorReading>) -> Result<()> {
        let mut current = self.current_readings.write().unwrap();
        
        for reading in readings {
            // Apply any necessary transformations
            let transformed = self.transform_reading(reading)?;
            current.insert(transformed.sensor_id.clone(), transformed);
        }
        
        Ok(())
    }
}
```

### 6. Component Linking with WAC

```bash
# Compose components using wac
wac plug camera-component.wasm \
    --plug radar-component.wasm \
    --plug detection-component.wasm \
    --plug fusion-component.wasm \
    --plug planning-component.wasm \
    -o composed-system.wasm
```

```rust
// Runtime component linking
pub struct ComponentLinker {
    // Component registry
    components: HashMap<ComponentId, LoadedComponent>,
    
    // Link definitions
    links: Vec<ComponentLink>,
    
    // Shared memory regions for zero-copy
    shared_memory: HashMap<LinkId, SharedMemory>,
}

pub struct ComponentLink {
    source: ComponentPort,
    target: ComponentPort,
    data_type: DataType,
    buffer_size: usize,
}

impl ComponentLinker {
    pub fn link_components(
        &mut self,
        source: &ComponentId,
        source_port: &str,
        target: &ComponentId,
        target_port: &str,
    ) -> Result<LinkId> {
        // Validate type compatibility
        let source_type = self.get_output_type(source, source_port)?;
        let target_type = self.get_input_type(target, target_port)?;
        
        if !self.types_compatible(&source_type, &target_type) {
            return Err(anyhow!("Type mismatch"));
        }
        
        // Create shared memory region
        let buffer_size = self.calculate_buffer_size(&source_type);
        let shared_mem = SharedMemory::new(buffer_size)?;
        
        // Register link
        let link_id = LinkId::new();
        self.links.push(ComponentLink {
            source: ComponentPort { component: source.clone(), port: source_port.to_string() },
            target: ComponentPort { component: target.clone(), port: target_port.to_string() },
            data_type: source_type,
            buffer_size,
        });
        
        self.shared_memory.insert(link_id.clone(), shared_mem);
        Ok(link_id)
    }
}
```

### 7. State Management

```rust
pub struct SimulationState {
    // Current simulation time
    current_time: i64,
    
    // Component states
    component_states: HashMap<ComponentId, ComponentState>,
    
    // Visualization data
    viz_data: VisualizationData,
    
    // Performance metrics
    metrics: SimulationMetrics,
}

pub struct ComponentState {
    // Component-specific state
    internal_state: Vec<u8>,
    
    // Last inputs/outputs for debugging
    last_input: Option<Vec<u8>>,
    last_output: Option<Vec<u8>>,
    
    // Error state
    error: Option<String>,
}

// Checkpointing for replay
pub struct SimulationCheckpoint {
    timestamp: i64,
    state: SimulationState,
    component_memories: HashMap<ComponentId, Vec<u8>>,
}
```

### 8. Visualization Pipeline

```rust
pub struct VisualizationPipeline {
    // Renderer components
    renderers: HashMap<DataType, Box<dyn Renderer>>,
    
    // Output compositor
    compositor: OutputCompositor,
    
    // Frame buffer
    frame_buffer: FrameBuffer,
}

impl VisualizationPipeline {
    pub fn render_frame(&mut self, sim_state: &SimulationState) -> Result<Frame> {
        let mut layers = Vec::new();
        
        // Render each data type
        for (component_id, state) in &sim_state.component_states {
            if let Some(viz_data) = self.extract_viz_data(state) {
                let layer = self.render_layer(component_id, viz_data)?;
                layers.push(layer);
            }
        }
        
        // Composite layers
        let frame = self.compositor.composite(layers)?;
        
        // Add overlays (time, metrics, etc.)
        self.add_overlays(&mut frame, sim_state)?;
        
        Ok(frame)
    }
}
```

### 9. Control Interface

```rust
pub struct SimulationController {
    engine: Arc<Mutex<SimulationEngine>>,
    ui_state: Arc<RwLock<UIState>>,
}

impl SimulationController {
    pub async fn play(&self) -> Result<()> {
        let mut engine = self.engine.lock().await;
        engine.play().await
    }
    
    pub async fn pause(&self) -> Result<()> {
        let mut engine = self.engine.lock().await;
        engine.pause().await
    }
    
    pub async fn seek(&self, timestamp: i64) -> Result<()> {
        let mut engine = self.engine.lock().await;
        engine.seek(timestamp).await
    }
    
    pub async fn set_playback_speed(&self, speed: f32) -> Result<()> {
        let mut engine = self.engine.lock().await;
        engine.set_playback_speed(speed).await
    }
    
    pub async fn step_forward(&self) -> Result<()> {
        let mut engine = self.engine.lock().await;
        engine.step_forward().await
    }
}
```

## Implementation Phases

### Phase 1: Core Infrastructure
1. Sensor database integration
2. Basic timing controller
3. Simple component execution

### Phase 2: Component Composition
1. WAC integration
2. Component linking runtime
3. Data flow management

### Phase 3: Advanced Features
1. State checkpointing
2. Interpolation and prediction
3. Performance optimization

### Phase 4: Production Features
1. Distributed execution
2. Real-time guarantees
3. Fault tolerance

## Key Decisions Needed

1. **Database Choice**: TimescaleDB, InfluxDB, or custom?
2. **Time Representation**: Microseconds, nanoseconds, or custom?
3. **Component Communication**: Shared memory, channels, or both?
4. **State Persistence**: In-memory, disk, or hybrid?
5. **Visualization Streaming**: Server-side rendering or client-side?

## Performance Considerations

1. **Data Locality**: Keep sensor data close to components
2. **Zero-Copy**: Use shared memory where possible
3. **Batch Processing**: Process multiple timestamps together
4. **Caching**: Cache transformed sensor data
5. **Parallelism**: Execute independent components in parallel

## Error Handling

1. **Missing Sensor Data**: Interpolation or last-known-good
2. **Component Crashes**: Restart with checkpoint
3. **Timing Violations**: Log and continue or halt?
4. **Resource Exhaustion**: Graceful degradation

## Security Considerations

1. **Component Isolation**: Each component in separate WASM instance
2. **Resource Limits**: Memory, CPU, and time bounds
3. **Data Validation**: Verify sensor data integrity
4. **Access Control**: Who can load/modify components?