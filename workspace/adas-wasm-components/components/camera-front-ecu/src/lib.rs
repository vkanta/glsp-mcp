// Camera Front ECU - Exports camera data stream

wit_bindgen::generate!({
    world: "camera-front-component",
    path: "../../wit/camera-front.wit",
});

use crate::exports::camera_data;
use crate::exports::camera_control;

struct Component;

// Resource state for camera stream
pub struct CameraStreamState {
    id: u32,
}

// Camera configuration state
static mut CAMERA_CONFIG: Option<camera_control::CameraConfig> = None;
static mut CAMERA_STATUS: camera_control::CameraStatus = camera_control::CameraStatus::Offline;

// Implement the camera-data interface
impl camera_data::Guest for Component {
    type CameraStream = CameraStreamState;
    
    fn create_stream() -> camera_data::CameraStream {
        camera_data::CameraStream::new(CameraStreamState { id: 1 })
    }
}

impl camera_data::GuestCameraStream for CameraStreamState {
    fn get_frame(&self) -> Result<camera_data::CameraFrame, String> {
        // Simulate camera frame capture
        Ok(camera_data::CameraFrame {
            width: 1920,
            height: 1080,
            data: vec![0; 1920 * 1080 * 3], // RGB8 format
            format: camera_data::PixelFormat::Rgb8,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            exposure_time: 16.67,
            gain: 1.0,
            sensor_pose: camera_data::CameraPose {
                position: camera_data::Position3d { x: 0.0, y: 0.0, z: 1.2 },
                orientation: camera_data::Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            },
        })
    }

    fn get_intrinsics(&self) -> camera_data::CameraIntrinsics {
        camera_data::CameraIntrinsics {
            focal_length_x: 1000.0,
            focal_length_y: 1000.0,
            principal_point_x: 960.0,
            principal_point_y: 540.0,
            distortion: vec![-0.1, 0.05, 0.0, 0.0, 0.0],
        }
    }

    fn is_available(&self) -> bool {
        unsafe {
            matches!(CAMERA_STATUS, camera_control::CameraStatus::Streaming)
        }
    }
}

// Implement the camera control interface
impl camera_control::Guest for Component {
    fn initialize(config: camera_control::CameraConfig) -> Result<(), String> {
        unsafe {
            CAMERA_CONFIG = Some(config);
            CAMERA_STATUS = camera_control::CameraStatus::Initializing;
        }
        Ok(())
    }

    fn start_streaming() -> Result<(), String> {
        unsafe {
            if CAMERA_CONFIG.is_some() {
                CAMERA_STATUS = camera_control::CameraStatus::Streaming;
                Ok(())
            } else {
                Err("Camera not initialized".to_string())
            }
        }
    }

    fn stop_streaming() -> Result<(), String> {
        unsafe {
            CAMERA_STATUS = camera_control::CameraStatus::Offline;
        }
        Ok(())
    }

    fn update_config(config: camera_control::CameraConfig) -> Result<(), String> {
        unsafe {
            CAMERA_CONFIG = Some(config);
        }
        Ok(())
    }

    fn get_status() -> camera_control::CameraStatus {
        unsafe { CAMERA_STATUS.clone() }
    }

    fn run_diagnostic() -> Result<camera_control::DiagnosticResult, String> {
        Ok(camera_control::DiagnosticResult {
            lens_health: camera_control::TestResult::Passed,
            sensor_health: camera_control::TestResult::Passed,
            auto_focus_test: camera_control::TestResult::Passed,
            exposure_test: camera_control::TestResult::Passed,
            color_calibration: camera_control::TestResult::Passed,
            overall_score: 98.5,
        })
    }
}

export!(Component);