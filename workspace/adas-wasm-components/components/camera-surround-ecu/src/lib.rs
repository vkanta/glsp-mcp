// Camera Surround ECU - Exports 360° camera data stream

wit_bindgen::generate!({
    world: "camera-surround-component",
    path: "../../wit/camera-surround.wit",
});

use crate::exports::camera_data;
use crate::exports::surround_control;

struct Component;

// Resource state for camera stream
pub struct CameraStreamState {
    id: u32,
}

// Surround camera configuration state
static mut SURROUND_CONFIG: Option<surround_control::SurroundConfig> = None;
static mut SURROUND_STATUS: surround_control::SurroundStatus = surround_control::SurroundStatus::Offline;

// Implement the camera-data interface
impl camera_data::Guest for Component {
    type CameraStream = CameraStreamState;
    
    fn create_stream() -> camera_data::CameraStream {
        camera_data::CameraStream::new(CameraStreamState { id: 1 })
    }
}

impl camera_data::GuestCameraStream for CameraStreamState {
    fn get_frame(&self) -> Result<camera_data::CameraFrame, String> {
        // Simulate 360° surround view frame generation
        Ok(camera_data::CameraFrame {
            width: 1920,
            height: 1080,
            data: vec![0; 1920 * 1080 * 3], // RGB8 stitched surround view
            format: camera_data::PixelFormat::Rgb8,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            exposure_time: 16.67,
            gain: 1.0,
            sensor_pose: camera_data::CameraPose {
                position: camera_data::Position3d { x: 0.0, y: 0.0, z: 2.0 }, // Higher mounted
                orientation: camera_data::Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            },
        })
    }

    fn get_intrinsics(&self) -> camera_data::CameraIntrinsics {
        // Surround view intrinsics (composite from multiple cameras)
        camera_data::CameraIntrinsics {
            focal_length_x: 800.0,
            focal_length_y: 800.0,
            principal_point_x: 960.0,
            principal_point_y: 540.0,
            distortion: vec![-0.2, 0.1, 0.0, 0.0, 0.0], // Higher distortion for wide-angle
        }
    }

    fn is_available(&self) -> bool {
        unsafe {
            matches!(SURROUND_STATUS, surround_control::SurroundStatus::Active)
        }
    }
}

// Implement the surround control interface
impl surround_control::Guest for Component {
    fn initialize(config: surround_control::SurroundConfig) -> Result<(), String> {
        unsafe {
            SURROUND_CONFIG = Some(config);
            SURROUND_STATUS = surround_control::SurroundStatus::Initializing;
        }
        Ok(())
    }

    fn start_processing() -> Result<(), String> {
        unsafe {
            if SURROUND_CONFIG.is_some() {
                SURROUND_STATUS = surround_control::SurroundStatus::Active;
                Ok(())
            } else {
                Err("Surround camera not initialized".to_string())
            }
        }
    }

    fn stop_processing() -> Result<(), String> {
        unsafe {
            SURROUND_STATUS = surround_control::SurroundStatus::Offline;
        }
        Ok(())
    }

    fn update_config(config: surround_control::SurroundConfig) -> Result<(), String> {
        unsafe {
            SURROUND_CONFIG = Some(config);
        }
        Ok(())
    }

    fn get_status() -> surround_control::SurroundStatus {
        unsafe { SURROUND_STATUS.clone() }
    }

    fn run_calibration() -> Result<surround_control::CalibrationResult, String> {
        Ok(surround_control::CalibrationResult {
            camera_alignment: surround_control::TestResult::Passed,
            stitch_accuracy: surround_control::TestResult::Passed,
            distortion_correction: surround_control::TestResult::Passed,
            color_matching: surround_control::TestResult::Passed,
            overlap_zones: surround_control::TestResult::Passed,
            overall_score: 95.2,
        })
    }
}

export!(Component);