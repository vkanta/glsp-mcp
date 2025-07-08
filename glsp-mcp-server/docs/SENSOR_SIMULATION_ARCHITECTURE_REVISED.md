# Sensor-Driven WASM Component Simulation Architecture

## Executive Summary

Based on analysis of our existing codebase and Bytecode Alliance tooling, this document outlines a comprehensive sensor replay simulation system that:
- Uses TimescaleDB for time-series sensor data
- Leverages WAC (WebAssembly Compositions) for component linking
- Implements a timing orchestrator for synchronized execution
- Provides simulation controls (play/pause/speed/seek)
- Renders component outputs via our existing graphics renderer

## Existing Foundation Analysis

### ✅ What We Already Have

1. **Working 5-Component ADAS Pipeline** (`test-pipeline.rs`)
   - Video-Decoder → Object-Detection → Visualizer → Safety-Monitor
   - Orchestrator coordinates all components
   - 30 FPS real-time processing (98.7% efficiency)
   - Pub/sub data flow via WIT interfaces

2. **Data Flow Infrastructure** (`data-flow.wit`)
   - Publisher/Subscriber pattern
   - Structured data types (video-frame, detection-result)
   - Non-blocking and blocking data access
   - Status monitoring

3. **Component Orchestration** (`orchestrator/src/lib.rs`)
   - Component lifecycle management
   - Performance monitoring
   - Health diagnostics
   - Pipeline execution control

4. **Graphics Rendering** (from our recent work)
   - Server-side WASM graphics renderer
   - SVG/Canvas/Image output formats
   - Security sanitization
   - Streaming updates

### 🔧 What We Need to Add

1. **Sensor Database Layer**
2. **Timing Controller with Simulation Controls**
3. **WAC-based Component Composition**
4. **Sensor Data Bridge**
5. **State Management and Checkpointing**

## Architecture Design

### 1. Sensor Database Layer

```rust
// TimescaleDB-backed sensor storage
pub struct SensorDatabase {
    pool: sqlx::PgPool,
    interpolator: SensorInterpolator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub sensor_id: String,
    pub timestamp_us: i64,  // Microseconds since epoch
    pub data_type: SensorDataType,
    pub payload: Vec<u8>,
    pub quality: f32,       // 0.0-1.0 data quality score
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorDataType {
    Camera { width: u32, height: u32, format: String },
    Radar { point_count: u32 },
    Lidar { point_count: u32 },
    Ultrasonic { distance_m: f32 },
    IMU { accel: Vec3, gyro: Vec3 },
    GPS { lat: f64, lon: f64, alt: f64 },
    CAN { msg_id: u32, data: [u8; 8] },
}

impl SensorDatabase {
    // High-performance time-range queries
    pub async fn get_readings_in_range(
        &self,
        start_us: i64,
        end_us: i64,
        sensors: &[String],
    ) -> Result<Vec<SensorReading>> {
        // Use TimescaleDB hypertables for efficient time-range queries
        sqlx::query_as!(
            SensorReading,
            "SELECT * FROM sensor_readings 
             WHERE timestamp_us BETWEEN $1 AND $2 
             AND sensor_id = ANY($3) 
             ORDER BY timestamp_us",
            start_us, end_us, sensors
        )
        .fetch_all(&self.pool)
        .await
    }
    
    // Real-time interpolation for missing data
    pub async fn get_interpolated_reading(
        &self,
        sensor_id: &str,
        target_time_us: i64,
    ) -> Result<SensorReading> {
        self.interpolator.interpolate(sensor_id, target_time_us).await
    }
}
```

### 2. Simulation Timing Controller

```rust
pub struct SimulationController {
    // Simulation state
    current_time_us: AtomicI64,
    playback_speed: AtomicOrderedFloat<f32>,
    is_playing: AtomicBool,
    
    // Data range
    simulation_start_us: i64,
    simulation_end_us: i64,
    
    // Component timing
    component_frequencies: HashMap<ComponentId, f32>, // Hz
    last_execution_times: Arc<RwLock<HashMap<ComponentId, i64>>>,
    
    // Synchronization
    sync_barrier: Arc<Barrier>,
    step_notifier: tokio::sync::Notify,
}

impl SimulationController {
    pub async fn play(&self) -> Result<()> {
        self.is_playing.store(true, Ordering::Relaxed);
        self.start_timing_loop().await
    }
    
    pub async fn pause(&self) -> Result<()> {
        self.is_playing.store(false, Ordering::Relaxed);
        Ok(())
    }
    
    pub async fn seek(&self, target_time_us: i64) -> Result<()> {
        // Validate time is within range
        if target_time_us < self.simulation_start_us || target_time_us > self.simulation_end_us {
            return Err(anyhow!("Time out of range"));
        }
        
        // Pause simulation
        self.pause().await?;
        
        // Set new time
        self.current_time_us.store(target_time_us, Ordering::Relaxed);
        
        // Restore component states (from checkpoint if available)
        self.restore_state_at_time(target_time_us).await?;
        
        Ok(())
    }
    
    pub async fn set_playback_speed(&self, speed: f32) -> Result<()> {
        if speed < 0.0 || speed > 10.0 {
            return Err(anyhow!("Invalid playback speed"));
        }
        self.playback_speed.store(OrderedFloat(speed), Ordering::Relaxed);
        Ok(())
    }
    
    // Core timing loop
    async fn start_timing_loop(&self) -> Result<()> {
        let mut next_tick = Instant::now();
        let base_tick_duration = Duration::from_micros(33333); // 30 Hz base rate
        
        while self.is_playing.load(Ordering::Relaxed) {
            let current_speed = self.playback_speed.load(Ordering::Relaxed).0;
            
            if current_speed > 0.0 {
                // Calculate time step
                let time_step_us = (base_tick_duration.as_micros() as f32 * current_speed) as i64;
                let new_time = self.current_time_us.load(Ordering::Relaxed) + time_step_us;
                
                // Check bounds
                if new_time > self.simulation_end_us {
                    self.pause().await?;
                    continue;
                }
                
                // Update simulation time
                self.current_time_us.store(new_time, Ordering::Relaxed);
                
                // Notify components that need to execute
                self.step_notifier.notify_waiters();
            }
            
            // Sleep until next tick
            next_tick += base_tick_duration.div_f32(current_speed);
            tokio::time::sleep_until(next_tick.into()).await;
        }
        
        Ok(())
    }
    
    // Check if component should execute at current time
    pub fn should_execute(&self, component_id: &ComponentId) -> bool {
        let current_time = self.current_time_us.load(Ordering::Relaxed);
        let frequency = self.component_frequencies.get(component_id).unwrap_or(&30.0);
        let period_us = (1_000_000.0 / frequency) as i64;
        
        let last_times = self.last_execution_times.read().unwrap();
        let last_execution = last_times.get(component_id).unwrap_or(&0);
        
        current_time - last_execution >= period_us
    }
}
```

### 3. WAC-based Component Composition

```wac
// composition.wac - Define how components are linked
let video_decoder = new video-decoder:video-decoder;
let object_detection = new object-detection:object-detection;
let sensor_fusion = new sensor-fusion:sensor-fusion;
let visualizer = new visualizer:visualizer;

// Connect video pipeline
let video_pipeline = compose {
    export video_decoder.decode_frame as decode_frame;
    export object_detection.detect_objects as detect_objects;
    
    // Link video output to AI input
    video_decoder.video_output <-> object_detection.video_input;
    
    // Link AI output to fusion input
    object_detection.detection_output <-> sensor_fusion.vision_input;
};

// Connect visualization
let complete_system = compose {
    include video_pipeline;
    export visualizer.render as render;
    
    // Link fusion output to visualizer
    sensor_fusion.fused_output <-> visualizer.data_input;
};
```

```rust
// Runtime component composition using wac
pub struct ComponentComposer {
    wac_engine: WacEngine,
    compositions: HashMap<String, ComposedComponent>,
}

impl ComponentComposer {
    pub async fn compose_from_wac_file(&mut self, wac_file: &Path) -> Result<ComposedComponent> {
        // Use wac tool to compile composition
        let output = Command::new("wac")
            .args(&["compose", wac_file.to_str().unwrap()])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(anyhow!("WAC composition failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // Load the composed component
        let composed_wasm = output.stdout;
        let component = self.load_composed_component(composed_wasm).await?;
        
        Ok(component)
    }
}
```

### 4. Sensor Data Bridge

```rust
// Bridge between sensor database and WASM components
pub struct SensorDataBridge {
    database: Arc<SensorDatabase>,
    current_readings: Arc<RwLock<HashMap<String, SensorReading>>>,
    simulation_controller: Arc<SimulationController>,
}

impl SensorDataBridge {
    // Called by WASM components via host function
    pub async fn get_sensor_data(&self, sensor_id: &str) -> Result<Vec<u8>> {
        let current_time = self.simulation_controller.get_current_time();
        
        // Try current readings cache first
        {
            let readings = self.current_readings.read().await;
            if let Some(reading) = readings.get(sensor_id) {
                // Check if reading is close enough to current time
                if (reading.timestamp_us - current_time).abs() < 16_667 { // ~1 frame at 60fps
                    return Ok(reading.payload.clone());
                }
            }
        }
        
        // Fetch from database with interpolation
        let reading = self.database.get_interpolated_reading(sensor_id, current_time).await?;
        
        // Update cache
        {
            let mut readings = self.current_readings.write().await;
            readings.insert(sensor_id.to_string(), reading.clone());
        }
        
        Ok(reading.payload)
    }
    
    // Update readings for current time window
    pub async fn update_readings_for_time(&self, time_us: i64) -> Result<()> {
        let window_size_us = 100_000; // 100ms window
        let start_time = time_us - window_size_us / 2;
        let end_time = time_us + window_size_us / 2;
        
        // Get all sensors that might need data
        let active_sensors = self.get_active_sensors();
        
        // Fetch readings in time window
        let readings = self.database.get_readings_in_range(
            start_time,
            end_time,
            &active_sensors
        ).await?;
        
        // Update current readings
        let mut current = self.current_readings.write().await;
        for reading in readings {
            current.insert(reading.sensor_id.clone(), reading);
        }
        
        Ok(())
    }
}

// Host function exposed to WASM components
#[async_trait]
impl wasmtime::component::ResourceTable for SensorDataBridge {
    async fn get_sensor_reading(&mut self, sensor_id: String) -> wasmtime::Result<Vec<u8>> {
        self.get_sensor_data(&sensor_id)
            .await
            .map_err(|e| wasmtime::Error::msg(e.to_string()))
    }
}
```

### 5. Enhanced Orchestrator with Simulation Support

```rust
// Enhanced orchestrator that integrates with our existing pipeline
pub struct SimulationOrchestrator {
    // Existing components
    component_graph: ComponentGraph,
    data_flow_manager: Arc<Mutex<DataFlowManager>>,
    
    // New simulation components
    simulation_controller: Arc<SimulationController>,
    sensor_bridge: Arc<SensorDataBridge>,
    state_manager: Arc<StateManager>,
    graphics_renderer: Arc<WasmGraphicsRenderer>,
    
    // WAC composition
    composer: ComponentComposer,
    composed_system: Option<ComposedComponent>,
}

impl SimulationOrchestrator {
    pub async fn load_simulation(&mut self, config: SimulationConfig) -> Result<()> {
        // 1. Load sensor data time range
        let (start_time, end_time) = self.sensor_bridge.database
            .get_time_range().await?;
        
        // 2. Configure timing controller
        self.simulation_controller.configure(
            start_time,
            end_time,
            config.component_frequencies
        ).await?;
        
        // 3. Compose components using WAC
        if let Some(wac_file) = config.composition_file {
            self.composed_system = Some(
                self.composer.compose_from_wac_file(&wac_file).await?
            );
        }
        
        // 4. Initialize component states
        self.state_manager.initialize_simulation_state().await?;
        
        Ok(())
    }
    
    pub async fn step_simulation(&mut self) -> Result<SimulationFrame> {
        let current_time = self.simulation_controller.get_current_time();
        
        // 1. Update sensor data for current time
        self.sensor_bridge.update_readings_for_time(current_time).await?;
        
        // 2. Execute components in order
        let mut execution_results = Vec::new();
        
        for component_id in &self.component_graph.execution_order {
            if self.simulation_controller.should_execute(component_id) {
                // Execute component
                let result = self.execute_component_with_sensors(
                    component_id,
                    current_time
                ).await?;
                
                execution_results.push(result);
                
                // Update last execution time
                self.simulation_controller.mark_component_executed(
                    component_id,
                    current_time
                );
            }
        }
        
        // 3. Render visualization
        let viz_frame = self.render_simulation_frame(
            current_time,
            &execution_results
        ).await?;
        
        // 4. Save state checkpoint if needed
        if self.should_checkpoint(current_time) {
            self.state_manager.create_checkpoint(current_time).await?;
        }
        
        Ok(SimulationFrame {
            timestamp_us: current_time,
            component_results: execution_results,
            visualization: viz_frame,
            metrics: self.collect_performance_metrics(),
        })
    }
    
    async fn execute_component_with_sensors(
        &self,
        component_id: &ComponentId,
        time_us: i64
    ) -> Result<ComponentExecutionResult> {
        // Get component instance
        let component = self.component_graph.get_component(component_id)?;
        
        // Set up sensor data access via host functions
        let mut store = wasmtime::Store::new(
            &self.wasmtime_engine,
            SensorDataBridge::clone(&self.sensor_bridge)
        );
        
        // Execute component method
        let start_time = Instant::now();
        let result = component.call_execute(&mut store, &[]).await?;
        let execution_time = start_time.elapsed();
        
        Ok(ComponentExecutionResult {
            component_id: component_id.clone(),
            timestamp_us: time_us,
            execution_time_us: execution_time.as_micros() as i64,
            output_data: result,
            error: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SimulationFrame {
    pub timestamp_us: i64,
    pub component_results: Vec<ComponentExecutionResult>,
    pub visualization: GraphicsOutput,
    pub metrics: SimulationMetrics,
}

#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub composition_file: Option<PathBuf>,
    pub component_frequencies: HashMap<ComponentId, f32>,
    pub enable_checkpointing: bool,
    pub checkpoint_interval_us: i64,
    pub target_fps: f32,
}
```

### 6. Integration with MCP

```rust
// MCP tools for simulation control
impl crate::mcp::tools::ToolHandler {
    async fn handle_start_simulation(&self, params: serde_json::Value) -> Result<ToolResult> {
        let config: SimulationConfig = serde_json::from_value(params)?;
        
        let mut orchestrator = self.simulation_orchestrator.lock().await;
        orchestrator.load_simulation(config).await?;
        orchestrator.start().await?;
        
        Ok(ToolResult::success("Simulation started"))
    }
    
    async fn handle_simulation_control(&self, params: serde_json::Value) -> Result<ToolResult> {
        #[derive(Deserialize)]
        struct ControlParams {
            action: String,
            speed: Option<f32>,
            seek_time: Option<i64>,
        }
        
        let control: ControlParams = serde_json::from_value(params)?;
        let controller = &self.simulation_orchestrator.lock().await.simulation_controller;
        
        match control.action.as_str() {
            "play" => controller.play().await?,
            "pause" => controller.pause().await?,
            "seek" => {
                if let Some(time) = control.seek_time {
                    controller.seek(time).await?;
                }
            }
            "speed" => {
                if let Some(speed) = control.speed {
                    controller.set_playback_speed(speed).await?;
                }
            }
            _ => return Err(anyhow!("Unknown control action")),
        }
        
        Ok(ToolResult::success("Control applied"))
    }
}

// MCP resource for simulation streaming
impl crate::mcp::resources::ResourceHandler {
    async fn handle_simulation_stream(&self, uri: &str) -> Result<Resource> {
        // Stream simulation frames via HTTP/2 Server-Sent Events
        let frame_stream = self.simulation_orchestrator
            .lock().await
            .create_frame_stream(30.0).await?; // 30 FPS
        
        Ok(Resource::stream(uri, frame_stream))
    }
}
```

## Implementation Roadmap

### Phase 1: Foundation (2-3 weeks)
1. ✅ Existing pipeline working
2. 🔧 Set up TimescaleDB sensor storage
3. 🔧 Implement basic timing controller
4. 🔧 Create sensor data bridge with host functions

### Phase 2: Composition (2-3 weeks)
1. 🔧 Integrate WAC tool for component linking
2. 🔧 Enhance orchestrator with simulation support
3. 🔧 Add simulation controls (play/pause/seek/speed)
4. 🔧 Implement state checkpointing

### Phase 3: Advanced Features (3-4 weeks)
1. 🔧 Add sensor data interpolation
2. 🔧 Implement graphics visualization pipeline
3. 🔧 Add performance monitoring and metrics
4. 🔧 Create web-based simulation controls

### Phase 4: Production (2-3 weeks)
1. 🔧 Add real-time guarantees
2. 🔧 Implement distributed execution
3. 🔧 Add fault tolerance and recovery
4. 🔧 Performance optimization

## Key Benefits

1. **Reuses Existing Architecture**: Builds on our working 5-component pipeline
2. **Leverages Standard Tools**: Uses WAC for composition, TimescaleDB for sensors
3. **Maintains Performance**: 30 FPS real-time simulation capability
4. **Flexible Composition**: Easy to reconfigure component connections
5. **Professional Controls**: Industry-standard simulation playback controls
6. **Scalable**: Can handle complex sensor datasets and component graphs

## Questions for Clarification

1. **Sensor Data Sources**: What specific sensor formats do we need to support?
2. **Component Timing**: Do we need deterministic timing or is best-effort sufficient?
3. **State Persistence**: How much state should we preserve for seeking/replay?
4. **Graphics Requirements**: What visualization outputs are most important?
5. **Deployment**: Single machine or distributed execution?