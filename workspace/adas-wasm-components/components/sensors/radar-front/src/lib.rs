// Radar Front ECU - Exports long-range radar data for ACC/AEB

wit_bindgen::generate!({
    world: "radar-front-component",
    path: "../../../wit/worlds/radar-front.wit",
});

use crate::exports::radar_data;
use crate::exports::radar_control;

struct Component;

// Resource state for radar stream
pub struct RadarStreamState {
    id: u32,
}

// Radar configuration state
static mut RADAR_CONFIG: Option<radar_control::RadarConfig> = None;
static mut RADAR_STATUS: radar_control::RadarStatus = radar_control::RadarStatus::Offline;

// Implement the radar-data interface
impl radar_data::Guest for Component {
    type RadarStream = RadarStreamState;
    
    fn create_stream() -> radar_data::RadarStream {
        radar_data::RadarStream::new(RadarStreamState { id: 1 })
    }
}

impl radar_data::GuestRadarStream for RadarStreamState {
    fn get_scan(&self) -> Result<radar_data::RadarScan, String> {
        // Simulate radar scan with detected targets
        let targets = vec![
            radar_data::RadarTarget {
                position: radar_data::Position3d { x: 50.0, y: 0.0, z: 0.0 },
                velocity: radar_data::Velocity3d { vx: -5.0, vy: 0.0, vz: 0.0, speed: 5.0 },
                range: 50.0,
                azimuth: 0.0,
                elevation: 0.0,
                rcs: 10.0, // Car-sized target
                signal_strength: -30.0,
                confidence: 0.95,
            },
            radar_data::RadarTarget {
                position: radar_data::Position3d { x: 120.0, y: 3.5, z: 0.0 },
                velocity: radar_data::Velocity3d { vx: -10.0, vy: 0.0, vz: 0.0, speed: 10.0 },
                range: 120.2,
                azimuth: 1.7, // degrees
                elevation: 0.0,
                rcs: 12.0,
                signal_strength: -35.0,
                confidence: 0.88,
            },
        ];

        Ok(radar_data::RadarScan {
            targets,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            scan_id: 12345,
            sensor_pose: radar_data::RadarPose {
                position: radar_data::Position3d { x: 0.0, y: 0.0, z: 0.8 },
                orientation: radar_data::Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            },
        })
    }

    fn is_available(&self) -> bool {
        unsafe {
            matches!(RADAR_STATUS, radar_control::RadarStatus::Scanning)
        }
    }

    fn get_range(&self) -> f64 {
        unsafe {
            RADAR_CONFIG.as_ref().map(|c| c.detection_range).unwrap_or(200.0)
        }
    }
}

// Implement the radar control interface
impl radar_control::Guest for Component {
    fn initialize(config: radar_control::RadarConfig) -> Result<(), String> {
        unsafe {
            RADAR_CONFIG = Some(config);
            RADAR_STATUS = radar_control::RadarStatus::Initializing;
        }
        Ok(())
    }

    fn start_scanning() -> Result<(), String> {
        unsafe {
            if RADAR_CONFIG.is_some() {
                RADAR_STATUS = radar_control::RadarStatus::Scanning;
                Ok(())
            } else {
                Err("Radar not initialized".to_string())
            }
        }
    }

    fn stop_scanning() -> Result<(), String> {
        unsafe {
            RADAR_STATUS = radar_control::RadarStatus::Offline;
        }
        Ok(())
    }

    fn update_config(config: radar_control::RadarConfig) -> Result<(), String> {
        unsafe {
            RADAR_CONFIG = Some(config);
        }
        Ok(())
    }

    fn get_status() -> radar_control::RadarStatus {
        unsafe { RADAR_STATUS.clone() }
    }

    fn run_diagnostic() -> Result<radar_control::DiagnosticResult, String> {
        Ok(radar_control::DiagnosticResult {
            rf_performance: radar_control::TestResult::Passed,
            antenna_integrity: radar_control::TestResult::Passed,
            signal_processing: radar_control::TestResult::Passed,
            target_tracking: radar_control::TestResult::Passed,
            interference_handling: radar_control::TestResult::Passed,
            overall_score: 92.5,
        })
    }
}

export!(Component);