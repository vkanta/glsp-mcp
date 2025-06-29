// Radar Corner ECU - Exports short-range radar data for blind spot/cross-traffic

wit_bindgen::generate!({
    world: "radar-corner-component",
    path: "../../../wit/worlds/radar-corner.wit",
});

use crate::exports::radar_data;
use crate::exports::corner_control;

struct Component;

// Resource state for radar stream
pub struct RadarStreamState {
    id: u32,
}

// Corner radar configuration state
static mut CORNER_CONFIG: Option<corner_control::CornerConfig> = None;
static mut CORNER_STATUS: corner_control::CornerStatus = corner_control::CornerStatus::Offline;

// Implement the radar-data interface
impl radar_data::Guest for Component {
    type RadarStream = RadarStreamState;
    
    fn create_stream() -> radar_data::RadarStream {
        radar_data::RadarStream::new(RadarStreamState { id: 1 })
    }
}

impl radar_data::GuestRadarStream for RadarStreamState {
    fn get_scan(&self) -> Result<radar_data::RadarScan, String> {
        // Simulate corner radar scan (shorter range, wider angle)
        let targets = vec![
            radar_data::RadarTarget {
                position: radar_data::Position3d { x: -2.0, y: 15.0, z: 0.0 }, // Vehicle in blind spot
                velocity: radar_data::Velocity3d { vx: 5.0, vy: -2.0, vz: 0.0, speed: 5.4 },
                range: 15.1,
                azimuth: 82.5, // degrees - side angle
                elevation: 0.0,
                rcs: 8.0, // Smaller cross-section
                signal_strength: -25.0,
                confidence: 0.92,
            },
        ];

        Ok(radar_data::RadarScan {
            targets,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            scan_id: 67890,
            sensor_pose: radar_data::RadarPose {
                position: radar_data::Position3d { x: -1.5, y: 0.8, z: 0.6 }, // Rear corner
                orientation: radar_data::Quaternion { x: 0.0, y: 0.0, z: 0.707, w: 0.707 }, // 90° rotated
            },
        })
    }

    fn is_available(&self) -> bool {
        unsafe {
            matches!(CORNER_STATUS, corner_control::CornerStatus::Monitoring)
        }
    }

    fn get_range(&self) -> f64 {
        unsafe {
            CORNER_CONFIG.as_ref().map(|c| c.max_range).unwrap_or(30.0)
        }
    }
}

// Implement the corner control interface
impl corner_control::Guest for Component {
    fn initialize(config: corner_control::CornerConfig) -> Result<(), String> {
        unsafe {
            CORNER_CONFIG = Some(config);
            CORNER_STATUS = corner_control::CornerStatus::Initializing;
        }
        Ok(())
    }

    fn start_monitoring() -> Result<(), String> {
        unsafe {
            if CORNER_CONFIG.is_some() {
                CORNER_STATUS = corner_control::CornerStatus::Monitoring;
                Ok(())
            } else {
                Err("Corner radar not initialized".to_string())
            }
        }
    }

    fn stop_monitoring() -> Result<(), String> {
        unsafe {
            CORNER_STATUS = corner_control::CornerStatus::Offline;
        }
        Ok(())
    }

    fn update_config(config: corner_control::CornerConfig) -> Result<(), String> {
        unsafe {
            CORNER_CONFIG = Some(config);
        }
        Ok(())
    }

    fn get_status() -> corner_control::CornerStatus {
        unsafe { CORNER_STATUS.clone() }
    }

    fn run_diagnostic() -> Result<corner_control::DiagnosticResult, String> {
        Ok(corner_control::DiagnosticResult {
            detection_accuracy: corner_control::TestResult::Passed,
            false_alarm_rate: corner_control::TestResult::Passed,
            coverage_area: corner_control::TestResult::Passed,
            signal_quality: corner_control::TestResult::Passed,
            environmental_adaptation: corner_control::TestResult::Passed,
            overall_score: 89.7,
        })
    }
}

export!(Component);