// Camera Front ECU - Exports camera data stream

wit_bindgen::generate!({
    world: "camera-front-component",
    path: "../../../wit/worlds/camera-front.wit",
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
        // Simulate camera frame capture with 320x320 for efficient AI processing
        Ok(camera_data::CameraFrame {
            width: 320,
            height: 320,
            data: generate_synthetic_scene(320, 320), // RGB8 format
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
            principal_point_x: 160.0,  // Updated for 320x320
            principal_point_y: 160.0,  // Updated for 320x320
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

// Generate synthetic automotive scene data for AI training/demo
fn generate_synthetic_scene(width: u32, height: u32) -> Vec<u8> {
    let mut data = vec![0u8; (width * height * 3) as usize];
    let w = width as usize;
    let h = height as usize;
    
    // Fill with sky gradient (top) and road (bottom)
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 3;
            
            if y < h / 3 {
                // Sky - blue gradient
                data[idx] = 135;     // R
                data[idx + 1] = 206; // G  
                data[idx + 2] = 235; // B
            } else if y < h * 2 / 3 {
                // Horizon/buildings - gray
                data[idx] = 128;     // R
                data[idx + 1] = 128; // G
                data[idx + 2] = 128; // B
            } else {
                // Road - dark gray
                data[idx] = 64;      // R
                data[idx + 1] = 64;  // G
                data[idx + 2] = 64;  // B
            }
        }
    }
    
    // Add synthetic vehicles as colored rectangles
    add_synthetic_vehicle(&mut data, w, h, 80, 180, 40, 25, [255, 0, 0]); // Red car
    add_synthetic_vehicle(&mut data, w, h, 200, 190, 35, 20, [0, 255, 0]); // Green car
    add_synthetic_vehicle(&mut data, w, h, 120, 200, 50, 30, [0, 0, 255]); // Blue truck
    
    // Add synthetic pedestrians as small rectangles
    add_synthetic_pedestrian(&mut data, w, h, 50, 210, [255, 255, 0]); // Yellow person
    add_synthetic_pedestrian(&mut data, w, h, 250, 205, [255, 0, 255]); // Magenta person
    
    data
}

// Add a synthetic vehicle to the scene
fn add_synthetic_vehicle(
    data: &mut [u8],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color: [u8; 3],
) {
    for dy in 0..h {
        for dx in 0..w {
            let px = x + dx;
            let py = y + dy;
            
            if px < width && py < height {
                let idx = (py * width + px) * 3;
                data[idx] = color[0];     // R
                data[idx + 1] = color[1]; // G
                data[idx + 2] = color[2]; // B
            }
        }
    }
}

// Add a synthetic pedestrian to the scene  
fn add_synthetic_pedestrian(
    data: &mut [u8],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    color: [u8; 3],
) {
    // Small 8x15 rectangle for person
    for dy in 0..15 {
        for dx in 0..8 {
            let px = x + dx;
            let py = y + dy;
            
            if px < width && py < height {
                let idx = (py * width + px) * 3;
                data[idx] = color[0];     // R
                data[idx + 1] = color[1]; // G
                data[idx + 2] = color[2]; // B
            }
        }
    }
}

export!(Component);