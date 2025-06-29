// Video-AI Pipeline Integration - Connects CarND video to YOLOv5n object detection

wit_bindgen::generate!({
    world: "video-ai-pipeline-component",
    path: "../../../wit/worlds/video-ai-pipeline.wit",
});

use crate::exports::pipeline_control;
use std::time::Instant;

struct Component;

// Pipeline state management
pub struct PipelineState {
    frames_processed: u32,
    detections_count: u32,
    total_processing_time: f64,
    last_frame_time: Instant,
    video_stream: Option<crate::video_decoder::CameraStream>,
    detection_stream: Option<crate::object_detection::DetectionStream>,
}

// Global pipeline state
static mut PIPELINE_STATE: Option<PipelineState> = None;
static mut PIPELINE_STATUS: pipeline_control::PipelineStatus = pipeline_control::PipelineStatus::Stopped;
static mut PIPELINE_CONFIG: Option<pipeline_control::PipelineConfig> = None;

// Implement the pipeline control interface (EXPORTED)
impl pipeline_control::Guest for Component {
    fn start_pipeline(config: pipeline_control::PipelineConfig) -> Result<(), String> {
        unsafe {
            println!("Starting video-AI pipeline with config: video={}, detection={}, conf_threshold={}, max_fps={}",
                     config.enable_video_playback, config.enable_object_detection,
                     config.detection_confidence_threshold, config.max_fps);
            
            PIPELINE_STATUS = pipeline_control::PipelineStatus::Initializing;
            PIPELINE_CONFIG = Some(config.clone());
            
            // Initialize video decoder
            let video_info = crate::video_decoder::load_embedded_video()?;
            println!("Video loaded: {}x{}, {} frames", video_info.width, video_info.height, video_info.frame_count);
            
            let video_stream = if config.enable_video_playback {
                let stream = crate::video_decoder::create_stream();
                crate::video_decoder::play()?;
                Some(stream)
            } else {
                None
            };
            
            // Initialize object detection
            let detection_stream = if config.enable_object_detection {
                Some(crate::object_detection::create_stream())
            } else {
                None
            };
            
            // Initialize pipeline state
            PIPELINE_STATE = Some(PipelineState {
                frames_processed: 0,
                detections_count: 0,
                total_processing_time: 0.0,
                last_frame_time: Instant::now(),
                video_stream,
                detection_stream,
            });
            
            PIPELINE_STATUS = pipeline_control::PipelineStatus::Running;
            println!("Video-AI pipeline started successfully");
            
            Ok(())
        }
    }

    fn stop_pipeline() -> Result<(), String> {
        unsafe {
            if let Some(_) = PIPELINE_STATE.take() {
                let _ = crate::video_decoder::pause();
                PIPELINE_STATUS = pipeline_control::PipelineStatus::Stopped;
                println!("Video-AI pipeline stopped");
                Ok(())
            } else {
                Err("Pipeline not running".to_string())
            }
        }
    }

    fn pause_pipeline() -> Result<(), String> {
        unsafe {
            if PIPELINE_STATE.is_some() {
                let _ = crate::video_decoder::pause();
                PIPELINE_STATUS = pipeline_control::PipelineStatus::Paused;
                println!("Video-AI pipeline paused");
                Ok(())
            } else {
                Err("Pipeline not running".to_string())
            }
        }
    }

    fn resume_pipeline() -> Result<(), String> {
        unsafe {
            if PIPELINE_STATE.is_some() {
                let _ = crate::video_decoder::play();
                PIPELINE_STATUS = pipeline_control::PipelineStatus::Running;
                println!("Video-AI pipeline resumed");
                Ok(())
            } else {
                Err("Pipeline not running".to_string())
            }
        }
    }

    fn get_status() -> pipeline_control::PipelineStatus {
        unsafe { PIPELINE_STATUS.clone() }
    }

    fn get_metrics() -> pipeline_control::PipelineMetrics {
        unsafe {
            if let Some(ref state) = PIPELINE_STATE {
                let avg_time = if state.frames_processed > 0 {
                    state.total_processing_time / state.frames_processed as f64
                } else {
                    0.0
                };
                
                let current_fps = 1000.0 / avg_time.max(1.0); // Convert ms to fps
                
                pipeline_control::PipelineMetrics {
                    frames_processed: state.frames_processed,
                    detections_count: state.detections_count,
                    avg_processing_time_ms: avg_time,
                    current_fps: current_fps as f32,
                }
            } else {
                pipeline_control::PipelineMetrics {
                    frames_processed: 0,
                    detections_count: 0,
                    avg_processing_time_ms: 0.0,
                    current_fps: 0.0,
                }
            }
        }
    }

    fn process_single_frame() -> Result<String, String> {
        unsafe {
            if let Some(ref mut state) = PIPELINE_STATE {
                let start_time = Instant::now();
                
                // Get frame from video decoder
                if let Some(ref video_stream) = state.video_stream {
                    match video_stream.get_frame() {
                        Ok(frame) => {
                            println!("Got frame: {}x{}, {} bytes, timestamp: {}", 
                                     frame.width, frame.height, frame.data.len(), frame.timestamp);
                            
                            // Process frame with object detection using FEO interface
                            if let Some(ref _detection_stream) = state.detection_stream {
                                // Transfer frame data to object detection input slot (simulated)
                                // In real FEO implementation, this would be handled by external orchestrator
                                match crate::object_detection::process_frame(&frame.data, frame.width, frame.height) {
                                    Ok(detections) => {
                                        let processing_time = start_time.elapsed().as_millis() as f64;
                                        
                                        // Update metrics
                                        state.frames_processed += 1;
                                        state.detections_count += detections.len() as u32;
                                        state.total_processing_time += processing_time;
                                        state.last_frame_time = Instant::now();
                                        
                                        let result = format!(
                                            "Frame {} processed: {} detections found\\n\
                                            Processing time: {:.1}ms\\n\
                                            Objects detected: {}",
                                            state.frames_processed,
                                            detections.len(),
                                            processing_time,
                                            detections.iter()
                                                .map(|d| format!("{:?}({:.2})", d.class, d.confidence))
                                                .collect::<Vec<_>>()
                                                .join(", ")
                                        );
                                        
                                        Ok(result)
                                    }
                                    Err(e) => Err(format!("Object detection failed: {}", e))
                                }
                            } else {
                                Ok(format!("Frame processed (detection disabled): {}x{}", frame.width, frame.height))
                            }
                        }
                        Err(e) => Err(format!("Failed to get frame: {}", e))
                    }
                } else {
                    Err("Video stream not available".to_string())
                }
            } else {
                Err("Pipeline not initialized".to_string())
            }
        }
    }

    fn run_diagnostics() -> Result<String, String> {
        unsafe {
            let video_status = crate::video_decoder::get_status();
            let pipeline_metrics = Self::get_metrics();
            
            let diagnostic_info = format!(
                "Video-AI Pipeline Diagnostics:\\n\
                =================================\\n\
                Pipeline Status: {:?}\\n\
                Video Status: {:?}\\n\
                Frames Processed: {}\\n\
                Total Detections: {}\\n\
                Avg Processing Time: {:.1}ms\\n\
                Current FPS: {:.1}\\n\
                \\n\
                Configuration:\\n\
                - Video Playback: {}\\n\
                - Object Detection: {}\\n\
                - Confidence Threshold: {:.2}\\n\
                - Max FPS: {:.1}",
                PIPELINE_STATUS,
                video_status,
                pipeline_metrics.frames_processed,
                pipeline_metrics.detections_count,
                pipeline_metrics.avg_processing_time_ms,
                pipeline_metrics.current_fps,
                PIPELINE_CONFIG.as_ref().map(|c| c.enable_video_playback).unwrap_or(false),
                PIPELINE_CONFIG.as_ref().map(|c| c.enable_object_detection).unwrap_or(false),
                PIPELINE_CONFIG.as_ref().map(|c| c.detection_confidence_threshold).unwrap_or(0.5),
                PIPELINE_CONFIG.as_ref().map(|c| c.max_fps).unwrap_or(30.0)
            );
            
            Ok(diagnostic_info)
        }
    }
}

export!(Component);