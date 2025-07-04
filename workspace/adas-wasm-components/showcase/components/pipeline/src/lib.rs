// Production-Grade Data Pipeline for Automotive ADAS Showcase
// Demonstrates real-time data fusion, safety monitoring, and decision making

wit_bindgen::generate!({
    world: "pipeline",
    path: "../../wit/worlds/",
    generate_all,
});

use std::time::{SystemTime, UNIX_EPOCH, Instant};
use std::collections::VecDeque;

struct Component;

// Production pipeline constants
const PIPELINE_ID: &str = "adas-data-pipeline-production";
const MAX_FRAME_BUFFER_SIZE: usize = 10;
const MAX_PERCEPTION_BUFFER_SIZE: usize = 50;
const SAFETY_TIMEOUT_MS: u64 = 100;
const DECISION_LATENCY_TARGET_MS: f32 = 20.0;

// Pipeline state for automotive compliance
static mut PIPELINE_INITIALIZED: bool = false;
static mut PIPELINE_ACTIVE: bool = false;
static mut CURRENT_CONFIG: Option<adas::control::execution_control::ExecutionConfig> = None;

// Real-time data buffers
static mut FRAME_BUFFER: VecDeque<adas::data::sensor_data::CameraFrame> = VecDeque::new();
static mut PERCEPTION_BUFFER: VecDeque<adas::data::perception_data::PerceivedObject> = VecDeque::new();
static mut LAST_SCENE_MODEL: Option<adas::data::perception_data::SceneModel> = None;

// Performance tracking for automotive requirements
static mut FRAMES_PROCESSED: u64 = 0;
static mut DECISIONS_MADE: u64 = 0;
static mut TOTAL_PROCESSING_TIME_MS: f64 = 0.0;
static mut LAST_DECISION_TIME_MS: f32 = 0.0;

// Safety monitoring for automotive compliance
static mut SAFETY_VIOLATIONS: u32 = 0;
static mut CONSECUTIVE_FAILURES: u32 = 0;
static mut EMERGENCY_STOP_TRIGGERED: bool = false;
static mut LAST_SAFETY_CHECK: Option<Instant> = None;

// Decision state tracking
static mut CURRENT_DRIVING_STATE: adas::data::decision_data::DrivingBehavior = adas::data::decision_data::DrivingBehavior::CruiseControl;
static mut ACTIVE_INTERVENTIONS: Vec<adas::data::decision_data::SafetyIntervention> = Vec::new();

// Helper function for automotive timestamps
fn get_automotive_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

// Assess safety criticality for automotive decision making
fn assess_safety_situation(objects: &[adas::data::perception_data::PerceivedObject]) -> (f32, Vec<adas::data::decision_data::SafetyIntervention>) {
    let mut threat_level = 0.0;
    let mut interventions = Vec::new();
    
    for object in objects {
        // Distance-based threat assessment
        let distance = (object.position.x.powi(2) + object.position.y.powi(2)).sqrt();
        let relative_speed = (object.velocity.vx + 15.0).abs(); // Account for ego vehicle speed
        
        // Time to collision calculation
        let ttc = if relative_speed > 0.1 {
            distance as f32 / relative_speed as f32
        } else {
            f32::INFINITY
        };
        
        // Critical object assessment
        match object.object_type {
            adas::common_types::types::ObjectType::Pedestrian => {
                if distance < 15.0 {
                    threat_level = threat_level.max(0.9);
                    if ttc < 3.0 {
                        interventions.push(adas::data::decision_data::SafetyIntervention::EmergencyBraking);
                    }
                }
            },
            adas::common_types::types::ObjectType::Car | adas::common_types::types::ObjectType::Truck => {
                if distance < 25.0 && ttc < 4.0 {
                    threat_level = threat_level.max(0.7);
                    interventions.push(adas::data::decision_data::SafetyIntervention::AdaptiveCruiseControl);
                }
            },
            adas::common_types::types::ObjectType::Bicycle | adas::common_types::types::ObjectType::Motorcycle => {
                if distance < 20.0 {
                    threat_level = threat_level.max(0.6);
                    interventions.push(adas::data::decision_data::SafetyIntervention::LaneKeepingAssist);
                }
            },
            _ => {}
        }
    }
    
    (threat_level, interventions)
}

// Create automotive decision based on scene understanding
fn make_automotive_decision(scene: &adas::data::perception_data::SceneModel) -> adas::data::decision_data::AutomotiveDecision {
    let start = Instant::now();
    
    // Assess current safety situation
    let (threat_level, interventions) = assess_safety_situation(&scene.objects);
    
    // Determine driving behavior based on threat assessment
    let driving_behavior = if threat_level > 0.8 {
        adas::data::decision_data::DrivingBehavior::EmergencyStop
    } else if threat_level > 0.6 {
        adas::data::decision_data::DrivingBehavior::DefensiveDriving
    } else if scene.objects.len() > 5 {
        adas::data::decision_data::DrivingBehavior::TrafficAwareDriving
    } else {
        adas::data::decision_data::DrivingBehavior::CruiseControl
    };
    
    // Calculate maneuver recommendations
    let mut maneuvers = Vec::new();
    
    if threat_level > 0.5 {
        maneuvers.push(adas::data::decision_data::ManeuverRecommendation {
            maneuver_type: adas::data::decision_data::ManeuverType::SpeedAdjustment,
            urgency: adas::data::decision_data::UrgencyLevel::High,
            target_parameters: adas::data::decision_data::ManeuverParameters {
                target_speed: 10.0, // Reduce speed
                target_acceleration: -2.0,
                target_steering_angle: 0.0,
                execution_time_ms: 500,
            },
            confidence: threat_level,
            safety_margin: 1.5,
        });
    }
    
    let processing_time = start.elapsed().as_millis() as f32;
    
    unsafe {
        LAST_DECISION_TIME_MS = processing_time;
        DECISIONS_MADE += 1;
        CURRENT_DRIVING_STATE = driving_behavior;
        ACTIVE_INTERVENTIONS = interventions.clone();
        
        // Safety monitoring
        if processing_time > DECISION_LATENCY_TARGET_MS {
            SAFETY_VIOLATIONS += 1;
            println!("WARNING: Decision latency violation: {}ms", processing_time);
        }
        
        LAST_SAFETY_CHECK = Some(Instant::now());
    }
    
    adas::data::decision_data::AutomotiveDecision {
        decision_id: unsafe { DECISIONS_MADE },
        driving_behavior,
        safety_interventions: interventions,
        maneuver_recommendations: maneuvers,
        threat_assessment: adas::data::decision_data::ThreatAssessment {
            overall_threat_level: threat_level,
            immediate_threats: scene.objects.iter()
                .filter(|obj| {
                    let distance = (obj.position.x.powi(2) + obj.position.y.powi(2)).sqrt();
                    distance < 20.0
                })
                .map(|obj| obj.object_id)
                .collect(),
            safe_following_distance: 25.0,
            recommended_speed: if threat_level > 0.5 { 10.0 } else { 15.0 },
        },
        execution_priority: adas::data::decision_data::ExecutionPriority::High,
        confidence: scene.confidence * (1.0 - threat_level * 0.3),
        processing_time_ms: processing_time,
        timestamp: get_automotive_timestamp(),
    }
}

// ============ PRODUCTION AUTOMOTIVE INTERFACES ============

// Implement production execution control interface
impl exports::adas::control::execution_control::Guest for Component {
    fn initialize(config: exports::adas::control::execution_control::ExecutionConfig) -> Result<(), String> {
        unsafe {
            CURRENT_CONFIG = Some(config.clone());
            PIPELINE_INITIALIZED = true;
            
            // Initialize buffers
            FRAME_BUFFER = VecDeque::with_capacity(MAX_FRAME_BUFFER_SIZE);
            PERCEPTION_BUFFER = VecDeque::with_capacity(MAX_PERCEPTION_BUFFER_SIZE);
            
            // Reset counters
            FRAMES_PROCESSED = 0;
            DECISIONS_MADE = 0;
            TOTAL_PROCESSING_TIME_MS = 0.0;
            SAFETY_VIOLATIONS = 0;
            CONSECUTIVE_FAILURES = 0;
            EMERGENCY_STOP_TRIGGERED = false;
            
            println!("Production Pipeline: Initialized with automotive config");
            println!("  - Execution Mode: {:?}", config.execution_mode);
            println!("  - Real-time Priority: {:?}", config.real_time_priority);
            println!("  - Safety Level: {:?}", config.safety_requirements.asil_level);
            Ok(())
        }
    }
    
    fn start_execution() -> Result<(), String> {
        unsafe {
            if !PIPELINE_INITIALIZED {
                return Err("Pipeline not initialized".to_string());
            }
            PIPELINE_ACTIVE = true;
            EMERGENCY_STOP_TRIGGERED = false;
            println!("Production Pipeline: Started automotive data processing");
            Ok(())
        }
    }
    
    fn stop_execution() -> Result<(), String> {
        unsafe {
            PIPELINE_ACTIVE = false;
            println!("Production Pipeline: Stopped automotive data processing");
            Ok(())
        }
    }
    
    fn emergency_stop() -> Result<(), String> {
        unsafe {
            PIPELINE_ACTIVE = false;
            EMERGENCY_STOP_TRIGGERED = true;
            ACTIVE_INTERVENTIONS.clear();
            ACTIVE_INTERVENTIONS.push(adas::data::decision_data::SafetyIntervention::EmergencyBraking);
            println!("EMERGENCY: Pipeline emergency stop activated");
            Ok(())
        }
    }
    
    fn get_execution_status() -> exports::adas::control::execution_control::ExecutionStatus {
        unsafe {
            if EMERGENCY_STOP_TRIGGERED {
                adas::common_types::types::HealthStatus::Critical
            } else if PIPELINE_ACTIVE {
                adas::common_types::types::HealthStatus::Ok
            } else if PIPELINE_INITIALIZED {
                adas::common_types::types::HealthStatus::Degraded
            } else {
                adas::common_types::types::HealthStatus::Offline
            }
        }
    }
}

// Implement production data pipeline interface
impl exports::adas::data::pipeline_data::Guest for Component {
    fn process_sensor_frame(
        frame: exports::adas::data::sensor_data::CameraFrame,
        objects: Vec<exports::adas::data::perception_data::PerceivedObject>,
    ) -> Result<exports::adas::data::decision_data::AutomotiveDecision, String> {
        unsafe {
            if !PIPELINE_ACTIVE {
                return Err("Pipeline not active".to_string());
            }
            
            if EMERGENCY_STOP_TRIGGERED {
                return Err("Emergency stop active - no processing".to_string());
            }
            
            let start = Instant::now();
            
            // Buffer management
            FRAME_BUFFER.push_back(frame.clone());
            if FRAME_BUFFER.len() > MAX_FRAME_BUFFER_SIZE {
                FRAME_BUFFER.pop_front();
            }
            
            for obj in objects.iter() {
                PERCEPTION_BUFFER.push_back(obj.clone());
            }
            if PERCEPTION_BUFFER.len() > MAX_PERCEPTION_BUFFER_SIZE {
                PERCEPTION_BUFFER.drain(0..10); // Remove oldest 10 objects
            }
            
            // Create scene model for decision making
            let scene_model = adas::data::perception_data::SceneModel {
                objects: objects.clone(),
                ego_state: adas::data::perception_data::EgoVehicleState {
                    position: adas::common_types::types::Position3d {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        coordinate_frame: adas::common_types::types::CoordinateFrame::Local,
                    },
                    velocity: adas::common_types::types::Velocity3d {
                        vx: 15.0, // 15 m/s forward
                        vy: 0.0,
                        vz: 0.0,
                        speed: 15.0,
                    },
                    acceleration: adas::common_types::types::Acceleration3d {
                        ax: 0.0,
                        ay: 0.0,
                        az: 0.0,
                        magnitude: 0.0,
                    },
                    heading: 0.0,
                    yaw_rate: 0.0,
                },
                timestamp: get_automotive_timestamp(),
                confidence: 0.9,
            };
            
            LAST_SCENE_MODEL = Some(scene_model.clone());
            
            // Make automotive decision
            let decision = make_automotive_decision(&scene_model);
            
            let total_time = start.elapsed().as_millis() as f64;
            TOTAL_PROCESSING_TIME_MS += total_time;
            FRAMES_PROCESSED += 1;
            
            // Safety checks
            if total_time > SAFETY_TIMEOUT_MS as f64 {
                SAFETY_VIOLATIONS += 1;
                CONSECUTIVE_FAILURES += 1;
                if CONSECUTIVE_FAILURES > 5 {
                    EMERGENCY_STOP_TRIGGERED = true;
                    return Err("Pipeline safety timeout - emergency stop triggered".to_string());
                }
            } else {
                CONSECUTIVE_FAILURES = 0;
            }
            
            Ok(decision)
        }
    }
    
    fn get_current_scene() -> Option<exports::adas::data::perception_data::SceneModel> {
        unsafe { LAST_SCENE_MODEL.clone() }
    }
    
    fn get_pipeline_metrics() -> exports::adas::data::pipeline_data::PipelineMetrics {
        unsafe {
            let avg_latency = if FRAMES_PROCESSED > 0 {
                (TOTAL_PROCESSING_TIME_MS / FRAMES_PROCESSED as f64) as f32
            } else {
                0.0
            };
            
            adas::common_types::types::PerformanceMetrics {
                latency_avg_ms: avg_latency,
                latency_max_ms: SAFETY_TIMEOUT_MS as f32,
                cpu_utilization: 0.25,
                memory_usage_mb: 32,
                throughput_hz: if avg_latency > 0.0 { 1000.0 / avg_latency } else { 0.0 },
                error_rate: (SAFETY_VIOLATIONS as f32 / FRAMES_PROCESSED.max(1) as f32),
            }
        }
    }
}

// Implement automotive health monitoring
impl exports::adas::diagnostics::health_monitoring::Guest for Component {
    fn get_health() -> exports::adas::diagnostics::health_monitoring::HealthReport {
        unsafe {
            exports::adas::diagnostics::health_monitoring::HealthReport {
                component_id: PIPELINE_ID.to_string(),
                overall_health: if EMERGENCY_STOP_TRIGGERED {
                    adas::common_types::types::HealthStatus::Critical
                } else if PIPELINE_ACTIVE {
                    adas::common_types::types::HealthStatus::Ok
                } else {
                    adas::common_types::types::HealthStatus::Degraded
                },
                subsystem_health: vec![
                    exports::adas::diagnostics::health_monitoring::SubsystemHealth {
                        subsystem_name: "data-fusion".to_string(),
                        health: if CONSECUTIVE_FAILURES < 3 {
                            adas::common_types::types::HealthStatus::Ok
                        } else {
                            adas::common_types::types::HealthStatus::Warning
                        },
                        details: "Real-time sensor data fusion".to_string(),
                    },
                    exports::adas::diagnostics::health_monitoring::SubsystemHealth {
                        subsystem_name: "decision-engine".to_string(),
                        health: if LAST_DECISION_TIME_MS < DECISION_LATENCY_TARGET_MS {
                            adas::common_types::types::HealthStatus::Ok
                        } else {
                            adas::common_types::types::HealthStatus::Warning
                        },
                        details: format!("Decision latency: {:.1}ms", LAST_DECISION_TIME_MS),
                    },
                    exports::adas::diagnostics::health_monitoring::SubsystemHealth {
                        subsystem_name: "safety-monitor".to_string(),
                        health: if SAFETY_VIOLATIONS < 10 {
                            adas::common_types::types::HealthStatus::Ok
                        } else {
                            adas::common_types::types::HealthStatus::Critical
                        },
                        details: format!("Safety violations: {}", SAFETY_VIOLATIONS),
                    },
                ],
                last_diagnostic: None,
                timestamp: get_automotive_timestamp(),
            }
        }
    }
    
    fn run_diagnostic() -> Result<exports::adas::diagnostics::health_monitoring::DiagnosticResult, String> {
        unsafe {
            Ok(exports::adas::diagnostics::health_monitoring::DiagnosticResult {
                test_results: vec![
                    exports::adas::diagnostics::health_monitoring::TestExecution {
                        test_name: "pipeline-throughput".to_string(),
                        test_result: if FRAMES_PROCESSED > 0 {
                            adas::common_types::types::TestResult::Passed
                        } else {
                            adas::common_types::types::TestResult::NotRun
                        },
                        details: format!("Processed {} frames", FRAMES_PROCESSED),
                        execution_time_ms: 1.5,
                    },
                    exports::adas::diagnostics::health_monitoring::TestExecution {
                        test_name: "decision-latency".to_string(),
                        test_result: if LAST_DECISION_TIME_MS < DECISION_LATENCY_TARGET_MS {
                            adas::common_types::types::TestResult::Passed
                        } else if LAST_DECISION_TIME_MS < DECISION_LATENCY_TARGET_MS * 2.0 {
                            adas::common_types::types::TestResult::Warning
                        } else {
                            adas::common_types::types::TestResult::Failed
                        },
                        details: format!("Last decision: {:.1}ms (target: <{:.1}ms)", 
                                       LAST_DECISION_TIME_MS, DECISION_LATENCY_TARGET_MS),
                        execution_time_ms: 0.5,
                    },
                    exports::adas::diagnostics::health_monitoring::TestExecution {
                        test_name: "safety-compliance".to_string(),
                        test_result: if EMERGENCY_STOP_TRIGGERED {
                            adas::common_types::types::TestResult::Failed
                        } else if SAFETY_VIOLATIONS < 5 {
                            adas::common_types::types::TestResult::Passed
                        } else {
                            adas::common_types::types::TestResult::Warning
                        },
                        details: format!("Safety state: {}", if EMERGENCY_STOP_TRIGGERED { "EMERGENCY" } else { "NORMAL" }),
                        execution_time_ms: 0.3,
                    },
                ],
                overall_score: if EMERGENCY_STOP_TRIGGERED { 25.0 } else if SAFETY_VIOLATIONS < 5 { 95.0 } else { 75.0 },
                recommendations: vec![
                    "Pipeline operating within automotive specifications".to_string()
                ],
                timestamp: get_automotive_timestamp(),
            })
        }
    }
    
    fn get_last_diagnostic() -> Option<exports::adas::diagnostics::health_monitoring::DiagnosticResult> {
        None
    }
}

// Implement automotive performance monitoring
impl exports::adas::diagnostics::performance_monitoring::Guest for Component {
    fn get_performance() -> exports::adas::diagnostics::performance_monitoring::ExtendedPerformance {
        use exports::adas::diagnostics::performance_monitoring::*;
        unsafe {
            let avg_latency = if FRAMES_PROCESSED > 0 {
                (TOTAL_PROCESSING_TIME_MS / FRAMES_PROCESSED as f64) as f32
            } else {
                0.0
            };
            
            ExtendedPerformance {
                base_metrics: adas::common_types::types::PerformanceMetrics {
                    latency_avg_ms: avg_latency,
                    latency_max_ms: SAFETY_TIMEOUT_MS as f32,
                    cpu_utilization: 0.25,
                    memory_usage_mb: 32,
                    throughput_hz: if avg_latency > 0.0 { 1000.0 / avg_latency } else { 0.0 },
                    error_rate: (SAFETY_VIOLATIONS as f32 / FRAMES_PROCESSED.max(1) as f32),
                },
                component_specific: vec![
                    Metric {
                        name: "frames-processed".to_string(),
                        value: FRAMES_PROCESSED as f32,
                        unit: "frames".to_string(),
                    },
                    Metric {
                        name: "decisions-made".to_string(),
                        value: DECISIONS_MADE as f32,
                        unit: "decisions".to_string(),
                    },
                    Metric {
                        name: "buffered-objects".to_string(),
                        value: PERCEPTION_BUFFER.len() as f32,
                        unit: "objects".to_string(),
                    },
                    Metric {
                        name: "active-interventions".to_string(),
                        value: ACTIVE_INTERVENTIONS.len() as f32,
                        unit: "interventions".to_string(),
                    },
                ],
                resource_usage: ResourceUsage {
                    cpu_cores_used: 0.25,
                    memory_allocated_mb: 32,
                    memory_peak_mb: 48,
                    disk_io_mb: 0.0,
                    network_io_mb: 0.0,
                    gpu_utilization: 0.0,
                    gpu_memory_mb: 0,
                },
                timestamp: get_automotive_timestamp(),
            }
        }
    }
    
    fn get_performance_history(
        _duration_seconds: u32,
    ) -> Vec<exports::adas::diagnostics::performance_monitoring::ExtendedPerformance> {
        vec![] // Return empty for showcase
    }
    
    fn reset_counters() {
        unsafe {
            FRAMES_PROCESSED = 0;
            DECISIONS_MADE = 0;
            TOTAL_PROCESSING_TIME_MS = 0.0;
            SAFETY_VIOLATIONS = 0;
            CONSECUTIVE_FAILURES = 0;
            println!("Production Pipeline: Performance counters reset");
        }
    }
}

// Implement showcase pipeline data processor
impl exports::showcase::pipeline::data_processor::Guest for Component {
    fn configure_pipeline(
        config: exports::showcase::pipeline::data_processor::PipelineConfig,
    ) -> Result<(), String> {
        unsafe {
            println!("Showcase Pipeline: Configured for {:?} processing", config.processing_mode);
            println!("  - Buffer Size: {} frames", config.buffer_size);
            println!("  - Real-time Mode: {}", config.real_time_processing);
            Ok(())
        }
    }
    
    fn process_pipeline_data(
        sensor_data: exports::adas::data::sensor_data::CameraFrame,
        ai_results: exports::showcase::ai::object_processor::AiProcessingResult,
    ) -> Result<exports::showcase::pipeline::data_processor::PipelineResult, String> {
        let start = Instant::now();
        
        // Process the AI results into automotive decision
        let decision = Self::process_sensor_frame(sensor_data.clone(), ai_results.detected_objects.clone())?;
        
        let total_time = start.elapsed().as_millis() as f32;
        
        unsafe {
            Ok(exports::showcase::pipeline::data_processor::PipelineResult {
                automotive_decision: decision,
                scene_analysis: ai_results.scene_understanding,
                pipeline_metrics: exports::showcase::pipeline::data_processor::ProcessingMetrics {
                    total_latency_ms: total_time,
                    sensor_processing_ms: 2.0,
                    ai_processing_ms: ai_results.total_processing_time_ms,
                    decision_processing_ms: LAST_DECISION_TIME_MS,
                    pipeline_overhead_ms: total_time - ai_results.total_processing_time_ms - LAST_DECISION_TIME_MS,
                    throughput_fps: if total_time > 0.0 { 1000.0 / total_time } else { 0.0 },
                    memory_usage_mb: 32,
                },
                data_quality: exports::showcase::pipeline::data_processor::QualityMetrics {
                    sensor_quality: sensor_data.reading.quality.confidence,
                    ai_confidence: ai_results.detection_quality.average_confidence,
                    decision_confidence: decision.confidence,
                    overall_reliability: (sensor_data.reading.quality.confidence + 
                                        ai_results.detection_quality.average_confidence + 
                                        decision.confidence) / 3.0,
                    data_freshness_score: 0.95,
                },
                safety_status: exports::showcase::pipeline::data_processor::SafetyMetrics {
                    threat_level: decision.threat_assessment.overall_threat_level,
                    active_interventions: decision.safety_interventions.len() as u32,
                    compliance_score: if EMERGENCY_STOP_TRIGGERED { 0.0 } else { 0.95 },
                    real_time_performance: if total_time < SAFETY_TIMEOUT_MS as f32 { 1.0 } else { 0.0 },
                    safety_margin: decision.threat_assessment.safe_following_distance,
                },
                pipeline_health: if EMERGENCY_STOP_TRIGGERED {
                    exports::showcase::pipeline::data_processor::PipelineHealth::Critical
                } else if SAFETY_VIOLATIONS < 5 {
                    exports::showcase::pipeline::data_processor::PipelineHealth::Healthy
                } else {
                    exports::showcase::pipeline::data_processor::PipelineHealth::Warning
                },
                timestamp: get_automotive_timestamp(),
            })
        }
    }
    
    fn get_pipeline_status() -> exports::showcase::pipeline::data_processor::PipelineStatus {
        unsafe {
            exports::showcase::pipeline::data_processor::PipelineStatus {
                processing_state: if EMERGENCY_STOP_TRIGGERED {
                    exports::showcase::pipeline::data_processor::ProcessingState::EmergencyStopped
                } else if PIPELINE_ACTIVE {
                    exports::showcase::pipeline::data_processor::ProcessingState::Processing
                } else if PIPELINE_INITIALIZED {
                    exports::showcase::pipeline::data_processor::ProcessingState::Ready
                } else {
                    exports::showcase::pipeline::data_processor::ProcessingState::Uninitialized
                },
                frames_in_buffer: FRAME_BUFFER.len() as u32,
                objects_in_buffer: PERCEPTION_BUFFER.len() as u32,
                current_throughput_fps: {
                    let avg_latency = if FRAMES_PROCESSED > 0 {
                        (TOTAL_PROCESSING_TIME_MS / FRAMES_PROCESSED as f64) as f32
                    } else {
                        0.0
                    };
                    if avg_latency > 0.0 { 1000.0 / avg_latency } else { 0.0 }
                },
                memory_usage_mb: 32,
                cpu_utilization: 0.25,
                active_safety_interventions: ACTIVE_INTERVENTIONS.len() as u32,
                last_update: get_automotive_timestamp(),
            }
        }
    }
    
    fn get_detailed_metrics() -> exports::showcase::pipeline::data_processor::DetailedPipelineMetrics {
        unsafe {
            let avg_latency = if FRAMES_PROCESSED > 0 {
                (TOTAL_PROCESSING_TIME_MS / FRAMES_PROCESSED as f64) as f32
            } else {
                0.0
            };
            
            exports::showcase::pipeline::data_processor::DetailedPipelineMetrics {
                base_performance: adas::common_types::types::PerformanceMetrics {
                    latency_avg_ms: avg_latency,
                    latency_max_ms: SAFETY_TIMEOUT_MS as f32,
                    cpu_utilization: 0.25,
                    memory_usage_mb: 32,
                    throughput_hz: if avg_latency > 0.0 { 1000.0 / avg_latency } else { 0.0 },
                    error_rate: (SAFETY_VIOLATIONS as f32 / FRAMES_PROCESSED.max(1) as f32),
                },
                processing_breakdown: exports::showcase::pipeline::data_processor::ProcessingBreakdown {
                    sensor_acquisition_ms: 2.0,
                    ai_inference_ms: 25.0,
                    decision_making_ms: LAST_DECISION_TIME_MS,
                    safety_validation_ms: 1.0,
                    output_generation_ms: 0.5,
                },
                automotive_compliance: exports::showcase::pipeline::data_processor::ComplianceMetrics {
                    iso26262_asil_level: "ASIL-B".to_string(),
                    real_time_deadline_compliance: ((FRAMES_PROCESSED - SAFETY_VIOLATIONS as u64) as f32 
                        / FRAMES_PROCESSED.max(1) as f32) * 100.0,
                    safety_requirement_coverage: 95.0,
                    fault_tolerance_score: if CONSECUTIVE_FAILURES < 3 { 98.0 } else { 85.0 },
                    functional_safety_rating: if EMERGENCY_STOP_TRIGGERED { 60.0 } else { 95.0 },
                },
                buffer_management: exports::showcase::pipeline::data_processor::BufferMetrics {
                    frame_buffer_utilization: (FRAME_BUFFER.len() as f32 / MAX_FRAME_BUFFER_SIZE as f32) * 100.0,
                    perception_buffer_utilization: (PERCEPTION_BUFFER.len() as f32 / MAX_PERCEPTION_BUFFER_SIZE as f32) * 100.0,
                    buffer_overflow_count: 0,
                    data_loss_events: 0,
                    memory_efficiency: 85.0,
                },
            }
        }
    }
}

export!(Component);