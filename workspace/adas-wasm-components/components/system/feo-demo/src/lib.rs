// FEO Demo - Demonstrates Fixed Execution Order component orchestration

wit_bindgen::generate!({
    world: "feo-demo-component",
    path: "../../../wit/feo-demo.wit",
});

use crate::exports::feo_demo;
use std::time::Instant;

struct Component;

// Demo state management
pub struct DemoState {
    config: Option<feo_demo::DemoConfig>,
    total_cycles: u32,
    video_executions: u32,
    detection_executions: u32,
    total_execution_time: u64,
    last_execution_time: Instant,
}

// Global demo state
static mut DEMO_STATE: Option<DemoState> = None;

// Implement FEO demo interface (EXPORTED)
impl feo_demo::Guest for Component {
    fn configure_demo(config: feo_demo::DemoConfig) -> Result<(), String> {
        unsafe {
            DEMO_STATE = Some(DemoState {
                config: Some(config),
                total_cycles: 0,
                video_executions: 0,
                detection_executions: 0,
                total_execution_time: 0,
                last_execution_time: Instant::now(),
            });
            
            println!("FEO Demo configured: mode={:?}, max_cycles={}, delay={}ms",
                     config.mode, config.max_cycles, config.delay_between_cycles_ms);
            Ok(())
        }
    }

    fn execute_single_cycle() -> Result<String, String> {
        unsafe {
            if let Some(ref mut state) = DEMO_STATE {
                let cycle_start = Instant::now();
                let mut results = Vec::new();
                
                // Step 1: Execute Video Decoder
                results.push("=== Video Decoder Execution ===".to_string());
                match crate::video_decoder_feo::execute_cycle() {
                    Ok(metrics) => {
                        state.video_executions += 1;
                        results.push(format!(
                            "Video Decoder: {} μs, {} inputs consumed, {} outputs produced",
                            metrics.execution_time_us,
                            metrics.input_items_consumed, 
                            metrics.output_items_produced
                        ));
                        
                        // Check if video decoder has output for next stage
                        if crate::video_decoder_feo::has_output() {
                            results.push("✓ Video frame ready for object detection".to_string());
                        } else {
                            results.push("⚠ No video frame output available".to_string());
                        }
                    }
                    Err(e) => {
                        results.push(format!("✗ Video Decoder Error: {}", e));
                    }
                }
                
                // Step 2: Execute Object Detection (if video decoder has output)
                results.push("\\n=== Object Detection Execution ===".to_string());
                if crate::video_decoder_feo::has_output() {
                    // In real FEO implementation, data would be transferred by external orchestrator
                    // Here we simulate the data transfer
                    match crate::object_detection_feo::execute_cycle() {
                        Ok(metrics) => {
                            state.detection_executions += 1;
                            results.push(format!(
                                "Object Detection: {} μs, {} inputs consumed, {} outputs produced",
                                metrics.execution_time_us,
                                metrics.input_items_consumed,
                                metrics.output_items_produced
                            ));
                            
                            if crate::object_detection_feo::has_output() {
                                results.push("✓ Object detections ready for next stage".to_string());
                            }
                        }
                        Err(e) => {
                            results.push(format!("✗ Object Detection Error: {}", e));
                        }
                    }
                } else {
                    results.push("⚠ Skipping object detection - no input frame available".to_string());
                }
                
                // Update metrics
                let cycle_time = cycle_start.elapsed();
                state.total_cycles += 1;
                state.total_execution_time += cycle_time.as_micros() as u64;
                state.last_execution_time = Instant::now();
                
                results.push(format!("\\n=== Cycle {} Complete ===", state.total_cycles));
                results.push(format!("Total cycle time: {} μs", cycle_time.as_micros()));
                results.push(format!("Average cycle time: {:.1} μs", 
                                   state.total_execution_time as f64 / state.total_cycles as f64));
                
                Ok(results.join("\\n"))
            } else {
                Err("Demo not configured".to_string())
            }
        }
    }

    fn execute_n_cycles(count: u32) -> Result<String, String> {
        let mut all_results = Vec::new();
        
        for i in 1..=count {
            match Self::execute_single_cycle() {
                Ok(result) => {
                    all_results.push(format!("\\n======== CYCLE {} / {} ========", i, count));
                    all_results.push(result);
                }
                Err(e) => {
                    return Err(format!("Cycle {} failed: {}", i, e));
                }
            }
            
            // Add delay between cycles if configured
            unsafe {
                if let Some(ref state) = DEMO_STATE {
                    if let Some(ref config) = state.config {
                        if config.delay_between_cycles_ms > 0 {
                            std::thread::sleep(std::time::Duration::from_millis(
                                config.delay_between_cycles_ms as u64
                            ));
                        }
                    }
                }
            }
        }
        
        unsafe {
            if let Some(ref state) = DEMO_STATE {
                all_results.push(format!("\\n======== SUMMARY ========"));
                all_results.push(format!("Total cycles executed: {}", count));
                all_results.push(format!("Video decoder executions: {}", state.video_executions));
                all_results.push(format!("Object detection executions: {}", state.detection_executions));
                all_results.push(format!("Average cycle time: {:.1} μs", 
                                       state.total_execution_time as f64 / state.total_cycles as f64));
            }
        }
        
        Ok(all_results.join("\\n"))
    }

    fn get_component_status() -> Result<String, String> {
        let mut status = Vec::new();
        
        // Video Decoder Status
        status.push("=== Video Decoder Status ===".to_string());
        match crate::video_decoder_feo::get_execution_state() {
            state => status.push(format!("Execution State: {:?}", state)),
        }
        status.push(format!("Can Execute: {}", crate::video_decoder_feo::can_execute()));
        status.push(format!("Has Output: {}", crate::video_decoder_feo::has_output()));
        
        let video_info = crate::video_decoder_feo::get_component_info();
        status.push(format!("Component: {} v{} ({})", 
                           video_info.component_id, video_info.version, video_info.component_type));
        status.push(format!("Execution Budget: {} μs, Memory Budget: {} bytes",
                           video_info.execution_time_budget_us, video_info.memory_budget_bytes));
        
        // Object Detection Status  
        status.push("\\n=== Object Detection Status ===".to_string());
        match crate::object_detection_feo::get_execution_state() {
            state => status.push(format!("Execution State: {:?}", state)),
        }
        status.push(format!("Can Execute: {}", crate::object_detection_feo::can_execute()));
        status.push(format!("Has Output: {}", crate::object_detection_feo::has_output()));
        
        let detection_info = crate::object_detection_feo::get_component_info();
        status.push(format!("Component: {} v{} ({})", 
                           detection_info.component_id, detection_info.version, detection_info.component_type));
        status.push(format!("Execution Budget: {} μs, Memory Budget: {} bytes",
                           detection_info.execution_time_budget_us, detection_info.memory_budget_bytes));
        
        Ok(status.join("\\n"))
    }

    fn get_demo_metrics() -> feo_demo::DemoMetrics {
        unsafe {
            if let Some(ref state) = DEMO_STATE {
                feo_demo::DemoMetrics {
                    total_cycles_executed: state.total_cycles,
                    video_decoder_executions: state.video_executions,
                    object_detection_executions: state.detection_executions,
                    total_execution_time_us: state.total_execution_time,
                    average_cycle_time_us: if state.total_cycles > 0 {
                        state.total_execution_time as f64 / state.total_cycles as f64
                    } else {
                        0.0
                    },
                }
            } else {
                feo_demo::DemoMetrics {
                    total_cycles_executed: 0,
                    video_decoder_executions: 0,
                    object_detection_executions: 0,
                    total_execution_time_us: 0,
                    average_cycle_time_us: 0.0,
                }
            }
        }
    }

    fn run_full_diagnostics() -> Result<String, String> {
        let mut diagnostics = Vec::new();
        
        diagnostics.push("======== FEO COMPONENT DIAGNOSTICS ========".to_string());
        
        // Video Decoder Diagnostics
        diagnostics.push("\\n=== Video Decoder Diagnostics ===".to_string());
        match crate::video_decoder_feo::get_diagnostics() {
            Ok(diag) => diagnostics.push(diag),
            Err(e) => diagnostics.push(format!("Error getting video decoder diagnostics: {}", e)),
        }
        
        // Object Detection Diagnostics
        diagnostics.push("\\n=== Object Detection Diagnostics ===".to_string());
        match crate::object_detection_feo::get_diagnostics() {
            Ok(diag) => diagnostics.push(diag),
            Err(e) => diagnostics.push(format!("Error getting object detection diagnostics: {}", e)),
        }
        
        // Demo State Diagnostics
        diagnostics.push("\\n=== Demo State Diagnostics ===".to_string());
        unsafe {
            if let Some(ref state) = DEMO_STATE {
                diagnostics.push(format!("Demo Configuration: {:?}", state.config.as_ref().map(|c| c.mode)));
                diagnostics.push(format!("Total Cycles: {}", state.total_cycles));
                diagnostics.push(format!("Video Executions: {}", state.video_executions));
                diagnostics.push(format!("Detection Executions: {}", state.detection_executions));
                diagnostics.push(format!("Total Execution Time: {} μs", state.total_execution_time));
                diagnostics.push(format!("Average Cycle Time: {:.1} μs", 
                                       if state.total_cycles > 0 {
                                           state.total_execution_time as f64 / state.total_cycles as f64
                                       } else {
                                           0.0
                                       }));
            } else {
                diagnostics.push("Demo not initialized".to_string());
            }
        }
        
        Ok(diagnostics.join("\\n"))
    }

    fn reset_demo() -> Result<(), String> {
        unsafe {
            DEMO_STATE = None;
            println!("FEO Demo reset - all state cleared");
            Ok(())
        }
    }
}

export!(Component);