// Object detection AI component with WASI-NN integration
use ndarray::Array4;
use std::sync::Mutex;

// Generate bindings using generate_all pattern
wit_bindgen::generate!({
    world: "ai-component",
    path: "wit/",
    generate_all,
});

// Use the generated bindings
use crate::exports::adas::control::ai_control::{
    Guest as AiControlGuest, AiConfig, AiStatus, ModelType,
};
use crate::exports::adas::diagnostics::health_monitoring::{
    Guest as HealthGuest, HealthReport, HealthStatus, SubsystemHealth, 
    DiagnosticResult, TestExecution, TestResult,
};
use crate::exports::adas::diagnostics::performance_monitoring::{
    Guest as PerformanceGuest, ExtendedPerformance, ResourceUsage, Metric,
};

// Import common types
use crate::adas::common_types::types::{
    Timestamp, PerformanceMetrics,
};

// Import WASI-NN types from generated bindings
use crate::wasi::nn::graph::{Graph, GraphEncoding, ExecutionTarget, GraphBuilder};
use crate::wasi::nn::tensor::{Tensor, TensorType};
use crate::wasi::nn::inference::NamedTensor;

// Static data - embedded test video and model
static VIDEO_DATA: &[u8] = include_bytes!("../data/test_video_320x200.h264");
static ONNX_MODEL: &[u8] = include_bytes!("../models/yolov5n.onnx");

// Global state for model and processing
static mut MODEL_LOADED: bool = false;
static mut WASI_NN_READY: bool = false;
static mut MODEL_GRAPH: Option<Graph> = None;
static mut CURRENT_CONFIG: Option<AiConfig> = None;
static mut INFERENCE_ACTIVE: bool = false;

// Storage for last detections
static mut LAST_DETECTION_COUNT: u32 = 0;
static mut LAST_INFERENCE_TIME_MS: f32 = 0.0;

// Thread-safe metrics storage
lazy_static::lazy_static! {
    static ref PERFORMANCE_METRICS: Mutex<PerformanceMetrics> = Mutex::new(PerformanceMetrics {
        latency_avg_ms: 0.0,
        latency_max_ms: 0.0,
        cpu_utilization: 0.0,
        memory_usage_mb: 0,
        throughput_hz: 0.0,
        error_rate: 0.0,
    });
    
    static ref EXTENDED_PERFORMANCE: Mutex<ExtendedPerformance> = Mutex::new(ExtendedPerformance {
        base_metrics: PerformanceMetrics {
            latency_avg_ms: 0.0,
            latency_max_ms: 0.0,
            cpu_utilization: 0.0,
            memory_usage_mb: 0,
            throughput_hz: 0.0,
            error_rate: 0.0,
        },
        component_specific: Vec::new(),
        resource_usage: ResourceUsage {
            cpu_cores_used: 1.0,
            memory_allocated_mb: 128,
            memory_peak_mb: 256,
            disk_io_mb: 0.0,
            network_io_mb: 0.0,
            gpu_utilization: 0.75,
            gpu_memory_mb: 512,
        },
        timestamp: 0,
    });
    
    static ref HEALTH_REPORT: Mutex<HealthReport> = Mutex::new(HealthReport {
        component_id: "adas-object-detection".to_string(),
        overall_health: HealthStatus::Ok,
        subsystem_health: Vec::new(),
        last_diagnostic: None,
        timestamp: 0,
    });
}

// COCO class names for YOLOv5
const COCO_CLASSES: &[&str] = &[
    "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck",
    "boat", "traffic light", "fire hydrant", "stop sign", "parking meter", "bench",
    "bird", "cat", "dog", "horse", "sheep", "cow", "elephant", "bear", "zebra",
    "giraffe", "backpack", "umbrella", "handbag", "tie", "suitcase", "frisbee",
    "skis", "snowboard", "sports ball", "kite", "baseball bat", "baseball glove",
    "skateboard", "surfboard", "tennis racket", "bottle", "wine glass", "cup",
    "fork", "knife", "spoon", "bowl", "banana", "apple", "sandwich", "orange",
    "broccoli", "carrot", "hot dog", "pizza", "donut", "cake", "chair", "couch",
    "potted plant", "bed", "dining table", "toilet", "tv", "laptop", "mouse",
    "remote", "keyboard", "cell phone", "microwave", "oven", "toaster", "sink",
    "refrigerator", "book", "clock", "vase", "scissors", "teddy bear", "hair drier",
    "toothbrush",
];

struct Component;

impl AiControlGuest for Component {
    fn load_model(config: AiConfig) -> Result<(), String> {
        unsafe {
            CURRENT_CONFIG = Some(config.clone());
            
            // Initialize WASI-NN
            let model_bytes = match config.model_type {
                ModelType::Detection => ONNX_MODEL,
                _ => return Err("Unsupported model type".to_string()),
            };
            
            // Load model through WASI-NN
            let builders: Vec<GraphBuilder> = vec![model_bytes.to_vec()];
            let encoding = GraphEncoding::Onnx;
            let target = ExecutionTarget::Cpu;
            
            match wasi::nn::graph::load(&builders, encoding, target) {
                Ok(graph) => {
                    MODEL_GRAPH = Some(graph);
                    MODEL_LOADED = true;
                    WASI_NN_READY = true;
                    
                    // Update health status
                    let mut health = HEALTH_REPORT.lock().unwrap();
                    health.overall_health = HealthStatus::Ok;
                    health.subsystem_health = vec![
                        SubsystemHealth {
                            subsystem_name: "wasi-nn".to_string(),
                            status: HealthStatus::Ok,
                            details: "Model loaded successfully".to_string(),
                        },
                        SubsystemHealth {
                            subsystem_name: "model".to_string(),
                            status: HealthStatus::Ok,
                            details: format!("YOLOv5n loaded, confidence threshold: {}", config.confidence_threshold),
                        }
                    ];
                    
                    Ok(())
                }
                Err(e) => {
                    let mut health = HEALTH_REPORT.lock().unwrap();
                    health.overall_health = HealthStatus::Error;
                    health.subsystem_health.push(SubsystemHealth {
                        subsystem_name: "wasi-nn".to_string(),
                        status: HealthStatus::Error,
                        details: format!("Failed to load model: {:?}", e),
                    });
                    Err(format!("WASI-NN model load failed: {:?}", e))
                }
            }
        }
    }
    
    fn start_inference() -> Result<(), String> {
        unsafe {
            if !MODEL_LOADED {
                return Err("Model not loaded".to_string());
            }
            
            INFERENCE_ACTIVE = true;
            
            // Start processing video frames
            process_video_stream()?;
            
            Ok(())
        }
    }
    
    fn stop_inference() -> Result<(), String> {
        unsafe {
            INFERENCE_ACTIVE = false;
            Ok(())
        }
    }
    
    fn update_config(config: AiConfig) -> Result<(), String> {
        unsafe {
            CURRENT_CONFIG = Some(config);
            Ok(())
        }
    }
    
    fn get_status() -> AiStatus {
        unsafe {
            if MODEL_LOADED && INFERENCE_ACTIVE {
                HealthStatus::Ok
            } else if MODEL_LOADED {
                HealthStatus::Degraded
            } else {
                HealthStatus::Offline
            }
        }
    }
    
    fn get_performance() -> PerformanceMetrics {
        PERFORMANCE_METRICS.lock().unwrap().clone()
    }
}

impl HealthGuest for Component {
    fn get_health() -> HealthReport {
        HEALTH_REPORT.lock().unwrap().clone()
    }
    
    fn run_diagnostic() -> Result<DiagnosticResult, String> {
        let mut test_results = Vec::new();
        let start_time = std::time::Instant::now();
        
        // Test WASI-NN availability
        unsafe {
            test_results.push(TestExecution {
                test_name: "wasi-nn-availability".to_string(),
                test_result: if WASI_NN_READY { TestResult::Passed } else { TestResult::Failed },
                details: if WASI_NN_READY { "WASI-NN interface available".to_string() } else { "WASI-NN not initialized".to_string() },
                execution_time_ms: start_time.elapsed().as_millis() as f32,
            });
            
            // Test model loading
            test_results.push(TestExecution {
                test_name: "model-loaded".to_string(),
                test_result: if MODEL_LOADED { TestResult::Passed } else { TestResult::Failed },
                details: if MODEL_LOADED { "YOLOv5n model loaded".to_string() } else { "Model not loaded".to_string() },
                execution_time_ms: start_time.elapsed().as_millis() as f32,
            });
            
            // Test inference capability
            if MODEL_LOADED {
                let test_result = test_inference();
                test_results.push(TestExecution {
                    test_name: "inference-test".to_string(),
                    test_result: if test_result.is_ok() { TestResult::Passed } else { TestResult::Failed },
                    details: match test_result {
                        Ok(_) => "Inference test passed".to_string(),
                        Err(e) => format!("Inference test failed: {}", e),
                    },
                    execution_time_ms: start_time.elapsed().as_millis() as f32,
                });
            }
        }
        
        let overall_score = test_results.iter()
            .filter(|t| t.test_result == TestResult::Passed)
            .count() as f32 / test_results.len() as f32;
        
        let diagnostic = DiagnosticResult {
            test_results,
            overall_score,
            recommendations: if overall_score < 1.0 {
                vec!["Check WASI-NN runtime support".to_string(),
                     "Verify model file integrity".to_string()]
            } else {
                vec![]
            },
            timestamp: get_timestamp(),
        };
        
        // Update last diagnostic
        let mut health = HEALTH_REPORT.lock().unwrap();
        health.last_diagnostic = Some(diagnostic.clone());
        
        Ok(diagnostic)
    }
    
    fn get_last_diagnostic() -> Option<DiagnosticResult> {
        HEALTH_REPORT.lock().unwrap().last_diagnostic.clone()
    }
}

impl PerformanceGuest for Component {
    fn get_performance() -> ExtendedPerformance {
        let mut perf = EXTENDED_PERFORMANCE.lock().unwrap();
        
        // Update component-specific metrics
        unsafe {
            perf.component_specific = vec![
                Metric {
                    name: "detection_count".to_string(),
                    value: LAST_DETECTION_COUNT as f64,
                    unit: "objects".to_string(),
                    description: "Number of objects detected in last frame".to_string(),
                },
                Metric {
                    name: "inference_time".to_string(),
                    value: LAST_INFERENCE_TIME_MS as f64,
                    unit: "ms".to_string(),
                    description: "Time taken for model inference".to_string(),
                },
                Metric {
                    name: "model_size".to_string(),
                    value: (ONNX_MODEL.len() / 1024 / 1024) as f64,
                    unit: "MB".to_string(),
                    description: "Size of the loaded ONNX model".to_string(),
                },
            ];
        }
        
        perf.timestamp = get_timestamp();
        perf.clone()
    }
    
    fn get_performance_history(duration_seconds: u32) -> Vec<ExtendedPerformance> {
        // Would implement rolling buffer in production
        vec![<Component as PerformanceGuest>::get_performance()]
    }
    
    fn reset_counters() {
        let mut metrics = PERFORMANCE_METRICS.lock().unwrap();
        *metrics = PerformanceMetrics {
            latency_avg_ms: 0.0,
            latency_max_ms: 0.0,
            cpu_utilization: 0.0,
            memory_usage_mb: 0,
            throughput_hz: 0.0,
            error_rate: 0.0,
        };
        
        let mut extended = EXTENDED_PERFORMANCE.lock().unwrap();
        extended.base_metrics = metrics.clone();
    }
}

// Helper functions
fn process_video_stream() -> Result<(), String> {
    unsafe {
        // Simulate processing frames from embedded video
        let frame_data = VIDEO_DATA.chunks(320 * 200 * 3).next()
            .ok_or("No video data")?;
        
        let timestamp = get_timestamp();
        let (detection_count, inference_time) = process_frame(frame_data.to_vec(), timestamp)?;
        
        // Store metrics
        LAST_DETECTION_COUNT = detection_count;
        LAST_INFERENCE_TIME_MS = inference_time;
        
        Ok(())
    }
}

fn process_frame(frame_data: Vec<u8>, _timestamp: Timestamp) -> Result<(u32, f32), String> {
    let start_time = std::time::Instant::now();
    
    // Decode frame (assuming raw RGB for now)
    let width = 320;
    let height = 200;
    
    // Preprocess for YOLO (640x640)
    let preprocessed = preprocess_frame(&frame_data, width, height)?;
    
    // Run inference
    let inference_start = std::time::Instant::now();
    let raw_detections = run_inference(preprocessed)?;
    let inference_time = inference_start.elapsed().as_millis() as f32;
    
    // Post-process detections
    let detection_count = count_detections(&raw_detections);
    
    // Update metrics
    let total_time = start_time.elapsed().as_millis() as f32;
    let mut metrics = PERFORMANCE_METRICS.lock().unwrap();
    metrics.latency_avg_ms = total_time;
    metrics.latency_max_ms = metrics.latency_max_ms.max(total_time);
    metrics.throughput_hz = 1000.0 / total_time;
    
    let mut extended = EXTENDED_PERFORMANCE.lock().unwrap();
    extended.base_metrics = metrics.clone();
    
    Ok((detection_count, inference_time))
}

fn preprocess_frame(frame_data: &[u8], width: u32, height: u32) -> Result<Array4<f32>, String> {
    // Convert raw frame to 640x640 with letterboxing
    let mut input = Array4::<f32>::zeros((1, 3, 640, 640));
    
    // Calculate letterbox dimensions
    let scale = (640.0 / width.max(height) as f32).min(1.0);
    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;
    let pad_x = (640 - new_width) / 2;
    let pad_y = (640 - new_height) / 2;
    
    // Simple RGB normalization
    for y in 0..new_height {
        for x in 0..new_width {
            let src_x = (x as f32 / scale) as u32;
            let src_y = (y as f32 / scale) as u32;
            if src_x < width && src_y < height {
                let idx = ((src_y * width + src_x) * 3) as usize;
                if idx + 2 < frame_data.len() {
                    let r = frame_data[idx] as f32 / 255.0;
                    let g = frame_data[idx + 1] as f32 / 255.0;
                    let b = frame_data[idx + 2] as f32 / 255.0;
                    
                    input[[0, 0, (pad_y + y) as usize, (pad_x + x) as usize]] = r;
                    input[[0, 1, (pad_y + y) as usize, (pad_x + x) as usize]] = g;
                    input[[0, 2, (pad_y + y) as usize, (pad_x + x) as usize]] = b;
                }
            }
        }
    }
    
    Ok(input)
}

fn run_inference(input: Array4<f32>) -> Result<Vec<f32>, String> {
    unsafe {
        if let Some(graph) = &MODEL_GRAPH {
            // Create execution context
            let context = match graph.init_execution_context() {
                Ok(ctx) => ctx,
                Err(e) => return Err(format!("Failed to create execution context: {:?}", e)),
            };
            
            // Convert input to tensor
            let input_data = input.as_slice().unwrap();
            let dimensions = vec![1, 3, 640, 640];
            let tensor_data = bytemuck::cast_slice(input_data).to_vec();
            let input_tensor = Tensor::new(
                &dimensions,
                TensorType::Fp32,
                &tensor_data,
            );
            
            // Create named tensor for input
            let inputs: Vec<NamedTensor> = vec![
                ("input".to_string(), input_tensor),
            ];
            
            // Run inference
            let outputs = match context.compute(inputs) {
                Ok(outputs) => outputs,
                Err(e) => return Err(format!("Inference failed: {:?}", e)),
            };
            
            // Extract output tensor data
            if let Some((_name, output_tensor)) = outputs.first() {
                // Get tensor data
                let output_data = output_tensor.data();
                
                // Convert bytes back to f32
                let output_floats: Vec<f32> = output_data
                    .chunks_exact(4)
                    .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                    .collect();
                
                Ok(output_floats)
            } else {
                Err("No output tensor returned".to_string())
            }
        } else {
            Err("Model graph not initialized".to_string())
        }
    }
}

fn count_detections(raw_output: &[f32]) -> u32 {
    let mut count = 0;
    
    // YOLO output format: [batch, num_detections, 85]
    // 85 = 4 bbox coords + 1 objectness + 80 class scores
    let num_detections = raw_output.len() / 85;
    
    for i in 0..num_detections {
        let offset = i * 85;
        let objectness = raw_output[offset + 4];
        
        if objectness > 0.5 {  // Confidence threshold
            // Find best class
            let mut best_score = 0.0;
            for j in 0..80 {
                let score = raw_output[offset + 5 + j];
                if score > best_score {
                    best_score = score;
                }
            }
            
            if best_score > 0.5 {  // Class confidence threshold
                count += 1;
            }
        }
    }
    
    count
}

fn test_inference() -> Result<(), String> {
    // Run a quick inference test
    let test_input = Array4::<f32>::zeros((1, 3, 640, 640));
    run_inference(test_input)?;
    Ok(())
}

fn get_timestamp() -> Timestamp {
    // In WASM, would use a proper time source
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

export!(Component);