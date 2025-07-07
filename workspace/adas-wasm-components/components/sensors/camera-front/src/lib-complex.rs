// Camera Front ECU Component Implementation
// Using our enhanced wit-bindgen with improved error messages and validation

wit_bindgen::generate!({
    world: "camera-front-simple-component",
    path: "../../../wit/adas-components",
    show_module_paths: true,  // Our enhancement to show generated module paths
});

struct Component;

// Camera stream resource implementation
struct CameraStreamImpl;

impl exports::adas::sensor_data::camera_data::CameraStream for CameraStreamImpl {
    fn get_frame(&self) -> Result<exports::adas::sensor_data::camera_data::CameraFrame, String> {
        // TODO: Implement actual camera frame capture
        Ok(exports::adas::sensor_data::camera_data::CameraFrame {
            width: 1920,
            height: 1080,
            data: vec![0; 1920 * 1080 * 3 / 2], // YUV420 placeholder
            format: exports::adas::sensor_data::camera_data::PixelFormat::Yuv420,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            exposure_time: 16.0,
            gain: 1.0,
            sensor_pose: exports::adas::sensor_data::camera_data::CameraPose {
                position: exports::adas::sensor_data::spatial_types::Position3d {
                    x: 0.0,
                    y: 0.0, 
                    z: 1.5, // Camera height
                },
                orientation: exports::adas::sensor_data::spatial_types::Quaternion {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
            },
        })
    }

    fn get_intrinsics(&self) -> exports::adas::sensor_data::camera_data::CameraIntrinsics {
        // TODO: Load actual camera intrinsics from calibration file
        exports::adas::sensor_data::camera_data::CameraIntrinsics {
            focal_length_x: 1000.0,
            focal_length_y: 1000.0,
            principal_point_x: 960.0,
            principal_point_y: 540.0,
            distortion: vec![0.0; 5], // No distortion for now
        }
    }

    fn is_available(&self) -> bool {
        true // Camera is always available for now
    }
}

// Export camera data interface implementation
impl exports::adas::sensor_data::camera_data::Guest for Component {
    fn create_stream() -> exports::adas::sensor_data::camera_data::CameraStream {
        exports::adas::sensor_data::camera_data::CameraStream::new(CameraStreamImpl)
    }
}

// Export camera control interface implementation  
impl exports::camera_control::Guest for Component {
    fn initialize(config: exports::camera_control::CameraConfig) -> Result<(), String> {
        println!("Initializing camera with resolution: {:?}", config.resolution);
        println!("Frame rate: {} fps", config.frame_rate);
        println!("Auto-focus enabled: {}", config.auto_focus);
        Ok(())
    }

    fn start_streaming() -> Result<(), String> {
        println!("Camera front: Starting video streaming...");
        Ok(())
    }

    fn stop_streaming() -> Result<(), String> {
        println!("Camera front: Stopping video streaming...");
        Ok(())
    }

    fn update_config(config: exports::camera_control::CameraConfig) -> Result<(), String> {
        println!("Camera front: Updating configuration...");
        Ok(())
    }

    fn get_status() -> exports::camera_control::CameraStatus {
        exports::camera_control::CameraStatus::Streaming
    }

    fn run_diagnostic() -> Result<exports::camera_control::CameraDiagnosticResult, String> {
        // Simulate running camera diagnostics
        Ok(exports::camera_control::CameraDiagnosticResult {
            lens_health: exports::camera_control::TestResult::Passed,
            sensor_health: exports::camera_control::TestResult::Passed,
            auto_focus_test: exports::camera_control::TestResult::Passed,
            exposure_test: exports::camera_control::TestResult::Passed,
            color_calibration: exports::camera_control::TestResult::Passed,
            overall_health: exports::camera_control::HealthStatus::Ok,
            performance: exports::camera_control::PerformanceMetrics {
                latency_avg_ms: 15.0,
                latency_max_ms: 25.0,
                cpu_utilization: 0.45,
                memory_usage_mb: 128,
                throughput_hz: 30.0,
                error_rate: 0.001,
            },
        })
    }
}

export!(Component);