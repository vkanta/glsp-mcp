// Camera Surround ECU - Exports 360° camera data stream

wit_bindgen::generate!({
    world: "camera-surround-component",
    path: "../../../wit/camera-surround.wit",
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
        // Simulate 360° surround view frame generation (320x320 for AI efficiency)
        Ok(camera_data::CameraFrame {
            width: 320,
            height: 320,
            data: generate_surround_scene(320, 320), // RGB8 stitched surround view
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
        // Surround view intrinsics (composite from multiple cameras, 320x320)
        camera_data::CameraIntrinsics {
            focal_length_x: 400.0,  // Adjusted for 320x320
            focal_length_y: 400.0,  // Adjusted for 320x320
            principal_point_x: 160.0,  // Center of 320x320
            principal_point_y: 160.0,  // Center of 320x320
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

// Generate synthetic surround view scene (top-down perspective)
fn generate_surround_scene(width: u32, height: u32) -> Vec<u8> {
    let mut data = vec![0u8; (width * height * 3) as usize];
    let w = width as usize;
    let h = height as usize;
    
    // Fill with ground/parking lot texture
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 3;
            
            // Create a parking lot texture
            if (x / 40) % 2 == (y / 40) % 2 {
                // Lighter tiles
                data[idx] = 160;     // R
                data[idx + 1] = 160; // G
                data[idx + 2] = 160; // B
            } else {
                // Darker tiles  
                data[idx] = 120;     // R
                data[idx + 1] = 120; // G
                data[idx + 2] = 120; // B
            }
        }
    }
    
    // Add vehicles around the ego vehicle (center) - surround view
    add_surround_vehicle(&mut data, w, h, 60, 50, 30, 15, [255, 100, 100]);   // Left side
    add_surround_vehicle(&mut data, w, h, 230, 60, 35, 18, [100, 255, 100]);  // Right side  
    add_surround_vehicle(&mut data, w, h, 140, 30, 40, 20, [100, 100, 255]);  // Front
    add_surround_vehicle(&mut data, w, h, 130, 270, 45, 25, [255, 255, 100]); // Rear
    
    // Add pedestrians at various positions
    add_surround_pedestrian(&mut data, w, h, 80, 120, [255, 200, 0]);  // Left
    add_surround_pedestrian(&mut data, w, h, 200, 180, [255, 0, 200]); // Right
    
    // Draw ego vehicle outline in center (our vehicle)
    let center_x = w / 2;
    let center_y = h / 2;
    add_surround_vehicle(&mut data, w, h, center_x - 20, center_y - 15, 40, 30, [50, 50, 50]); // Our car
    
    data
}

// Add a vehicle to surround view (top-down perspective)
fn add_surround_vehicle(
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

// Add a pedestrian to surround view (small circle/square)
fn add_surround_pedestrian(
    data: &mut [u8],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    color: [u8; 3],
) {
    // Small 6x6 square for person in top-down view
    for dy in 0..6 {
        for dx in 0..6 {
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