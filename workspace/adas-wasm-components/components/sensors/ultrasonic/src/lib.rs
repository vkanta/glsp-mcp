// Ultrasonic ECU - Exports close-range ultrasonic data for parking assistance

wit_bindgen::generate!({
    world: "ultrasonic-component",
    path: "../../../wit/worlds/ultrasonic.wit",
});

use crate::exports::ultrasonic_data;
use crate::exports::ultrasonic_control;

struct Component;

// Resource state for ultrasonic stream
pub struct UltrasonicStreamState {
    id: u32,
}

// Ultrasonic configuration state
static mut ULTRASONIC_CONFIG: Option<ultrasonic_control::UltrasonicConfig> = None;
static mut ULTRASONIC_STATUS: ultrasonic_control::UltrasonicStatus = ultrasonic_control::UltrasonicStatus::Offline;

// Implement the ultrasonic-data interface
impl ultrasonic_data::Guest for Component {
    type UltrasonicStream = UltrasonicStreamState;
    
    fn create_stream() -> ultrasonic_data::UltrasonicStream {
        ultrasonic_data::UltrasonicStream::new(UltrasonicStreamState { id: 1 })
    }
}

impl ultrasonic_data::GuestUltrasonicStream for UltrasonicStreamState {
    fn get_scan(&self) -> Result<ultrasonic_data::UltrasonicScan, String> {
        // Simulate ultrasonic sensor readings from 8 sensors
        let sensors = vec![
            ultrasonic_data::SensorReading {
                sensor_id: 1,
                distance: 1.2, // 1.2m to obstacle
                confidence: 0.95,
                sensor_position: ultrasonic_data::SensorPosition::FrontLeft,
                sensor_pose: ultrasonic_data::UltrasonicPose {
                    position: ultrasonic_data::Position3d { x: 2.0, y: 0.8, z: 0.5 },
                    orientation: ultrasonic_data::Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
                },
            },
            ultrasonic_data::SensorReading {
                sensor_id: 2,
                distance: 0.8, // Close obstacle
                confidence: 0.98,
                sensor_position: ultrasonic_data::SensorPosition::FrontCenterLeft,
                sensor_pose: ultrasonic_data::UltrasonicPose {
                    position: ultrasonic_data::Position3d { x: 2.1, y: 0.3, z: 0.5 },
                    orientation: ultrasonic_data::Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
                },
            },
            ultrasonic_data::SensorReading {
                sensor_id: 5,
                distance: 0.4, // Very close rear obstacle
                confidence: 0.99,
                sensor_position: ultrasonic_data::SensorPosition::RearCenterLeft,
                sensor_pose: ultrasonic_data::UltrasonicPose {
                    position: ultrasonic_data::Position3d { x: -2.1, y: 0.3, z: 0.5 },
                    orientation: ultrasonic_data::Quaternion { x: 0.0, y: 0.0, z: 1.0, w: 0.0 }, // 180° rotated
                },
            },
        ];

        Ok(ultrasonic_data::UltrasonicScan {
            sensors,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            scan_id: 11111,
        })
    }

    fn is_available(&self) -> bool {
        unsafe {
            matches!(ULTRASONIC_STATUS, ultrasonic_control::UltrasonicStatus::Monitoring | ultrasonic_control::UltrasonicStatus::Detecting)
        }
    }

    fn get_min_distance(&self) -> f64 {
        // Return minimum distance from all sensors
        0.4 // From the scan above
    }
}

// Implement the ultrasonic control interface
impl ultrasonic_control::Guest for Component {
    fn initialize(config: ultrasonic_control::UltrasonicConfig) -> Result<(), String> {
        unsafe {
            ULTRASONIC_CONFIG = Some(config);
            ULTRASONIC_STATUS = ultrasonic_control::UltrasonicStatus::Initializing;
        }
        Ok(())
    }

    fn start_monitoring() -> Result<(), String> {
        unsafe {
            if ULTRASONIC_CONFIG.is_some() {
                ULTRASONIC_STATUS = ultrasonic_control::UltrasonicStatus::Monitoring;
                Ok(())
            } else {
                Err("Ultrasonic system not initialized".to_string())
            }
        }
    }

    fn stop_monitoring() -> Result<(), String> {
        unsafe {
            ULTRASONIC_STATUS = ultrasonic_control::UltrasonicStatus::Offline;
        }
        Ok(())
    }

    fn update_config(config: ultrasonic_control::UltrasonicConfig) -> Result<(), String> {
        unsafe {
            ULTRASONIC_CONFIG = Some(config);
        }
        Ok(())
    }

    fn get_status() -> ultrasonic_control::UltrasonicStatus {
        unsafe { ULTRASONIC_STATUS.clone() }
    }

    fn run_diagnostic() -> Result<ultrasonic_control::DiagnosticResult, String> {
        Ok(ultrasonic_control::DiagnosticResult {
            sensor_integrity: ultrasonic_control::TestResult::Passed,
            range_accuracy: ultrasonic_control::TestResult::Passed,
            response_time: ultrasonic_control::TestResult::Passed,
            environmental_immunity: ultrasonic_control::TestResult::Passed,
            cross_talk_suppression: ultrasonic_control::TestResult::Passed,
            overall_score: 94.3,
        })
    }
}

export!(Component);