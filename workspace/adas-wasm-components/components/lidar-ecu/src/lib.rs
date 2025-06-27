// LiDAR ECU - Exports high-resolution 3D point cloud data

wit_bindgen::generate!({
    world: "lidar-component",
    path: "../../wit/lidar.wit",
});

use crate::exports::lidar_data;
use crate::exports::lidar_control;

struct Component;

// Resource state for LiDAR stream
pub struct LidarStreamState {
    id: u32,
}

// LiDAR configuration state
static mut LIDAR_CONFIG: Option<lidar_control::LidarConfig> = None;
static mut LIDAR_STATUS: lidar_control::LidarStatus = lidar_control::LidarStatus::Offline;

// Implement the lidar-data interface
impl lidar_data::Guest for Component {
    type LidarStream = LidarStreamState;
    
    fn create_stream() -> lidar_data::LidarStream {
        lidar_data::LidarStream::new(LidarStreamState { id: 1 })
    }
}

impl lidar_data::GuestLidarStream for LidarStreamState {
    fn get_cloud(&self) -> Result<lidar_data::PointCloud, String> {
        // Simulate LiDAR point cloud with high-resolution 3D data
        let mut points = Vec::new();
        
        // Generate sample points in a 360° scan
        for ring in 0..32 {
            for angle_deg in (0..360).step_by(5) {
                let angle_rad = (angle_deg as f64).to_radians();
                let elevation = (ring as f64 - 16.0) * 2.0; // -16° to +16°
                let range = 50.0 + (ring as f64 * 3.0); // Variable range by ring
                
                let x = range * angle_rad.cos() * elevation.to_radians().cos();
                let y = range * angle_rad.sin() * elevation.to_radians().cos();
                let z = range * elevation.to_radians().sin();
                
                points.push(lidar_data::LidarPoint {
                    position: lidar_data::Position3d { x, y, z },
                    intensity: 0.8 + (ring as f32 * 0.01), // Varying intensity
                    ring,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                });
            }
        }

        Ok(lidar_data::PointCloud {
            points,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            frame_id: "lidar_frame".to_string(),
            sensor_pose: lidar_data::LidarPose {
                position: lidar_data::Position3d { x: 0.0, y: 0.0, z: 2.1 }, // Roof-mounted
                orientation: lidar_data::Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            },
        })
    }

    fn is_available(&self) -> bool {
        unsafe {
            matches!(LIDAR_STATUS, lidar_control::LidarStatus::Scanning)
        }
    }

    fn get_point_count(&self) -> u32 {
        32 * 72 // 32 rings × 72 points per ring (5° resolution)
    }
}

// Implement the lidar control interface
impl lidar_control::Guest for Component {
    fn initialize(config: lidar_control::LidarConfig) -> Result<(), String> {
        unsafe {
            LIDAR_CONFIG = Some(config);
            LIDAR_STATUS = lidar_control::LidarStatus::Initializing;
        }
        Ok(())
    }

    fn start_scanning() -> Result<(), String> {
        unsafe {
            if LIDAR_CONFIG.is_some() {
                LIDAR_STATUS = lidar_control::LidarStatus::WarmingUp;
                // Simulate warmup time
                std::thread::sleep(std::time::Duration::from_millis(100));
                LIDAR_STATUS = lidar_control::LidarStatus::Scanning;
                Ok(())
            } else {
                Err("LiDAR not initialized".to_string())
            }
        }
    }

    fn stop_scanning() -> Result<(), String> {
        unsafe {
            LIDAR_STATUS = lidar_control::LidarStatus::Offline;
        }
        Ok(())
    }

    fn update_config(config: lidar_control::LidarConfig) -> Result<(), String> {
        unsafe {
            LIDAR_CONFIG = Some(config);
        }
        Ok(())
    }

    fn get_status() -> lidar_control::LidarStatus {
        unsafe { LIDAR_STATUS.clone() }
    }

    fn run_diagnostic() -> Result<lidar_control::DiagnosticResult, String> {
        Ok(lidar_control::DiagnosticResult {
            laser_health: lidar_control::TestResult::Passed,
            mirror_alignment: lidar_control::TestResult::Passed,
            range_accuracy: lidar_control::TestResult::Passed,
            point_density: lidar_control::TestResult::Passed,
            intensity_calibration: lidar_control::TestResult::Passed,
            overall_score: 96.8,
        })
    }
}

export!(Component);