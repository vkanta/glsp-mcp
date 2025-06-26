use wit_bindgen::generate;

// Generate bindings for ultrasonic-ecu
generate!({
    world: "ultrasonic-component",
    path: "../../wit/ultrasonic-ecu-standalone.wit"
});

use exports::adas::ultrasonic::ultrasonic::*;

// Component implementation
struct Component;

impl Guest for Component {
    fn initialize(config: UltrasonicConfig) -> Result<(), String> {
        println!("Initializing ultrasonic system with {} enabled sensors", config.enabled_sensors.len());
        Ok(())
    }

    fn start_sensing() -> Result<(), String> {
        println!("Starting ultrasonic sensing");
        Ok(())
    }

    fn stop_sensing() -> Result<(), String> {
        println!("Stopping ultrasonic sensing");
        Ok(())
    }

    fn get_measurements() -> Result<Vec<UltrasonicMeasurement>, String> {
        // Return mock ultrasonic measurements for all sensor positions
        Ok(vec![
            UltrasonicMeasurement {
                sensor_position: SensorPosition::FrontLeft,
                distance: 1.2,
                confidence: 0.95,
                temperature_compensated: true,
                measurement_time: 1000000000, // Mock timestamp
            },
            UltrasonicMeasurement {
                sensor_position: SensorPosition::FrontCenterLeft,
                distance: 0.8,
                confidence: 0.92,
                temperature_compensated: true,
                measurement_time: 1000000000,
            },
            UltrasonicMeasurement {
                sensor_position: SensorPosition::FrontCenterRight,
                distance: 0.6,
                confidence: 0.88,
                temperature_compensated: true,
                measurement_time: 1000000000,
            },
            UltrasonicMeasurement {
                sensor_position: SensorPosition::FrontRight,
                distance: 1.5,
                confidence: 0.93,
                temperature_compensated: true,
                measurement_time: 1000000000,
            },
            UltrasonicMeasurement {
                sensor_position: SensorPosition::RearLeft,
                distance: 2.1,
                confidence: 0.90,
                temperature_compensated: true,
                measurement_time: 1000000000,
            },
            UltrasonicMeasurement {
                sensor_position: SensorPosition::RearCenterLeft,
                distance: 0.9,
                confidence: 0.96,
                temperature_compensated: true,
                measurement_time: 1000000000,
            },
            UltrasonicMeasurement {
                sensor_position: SensorPosition::RearCenterRight,
                distance: 0.7,
                confidence: 0.94,
                temperature_compensated: true,
                measurement_time: 1000000000,
            },
            UltrasonicMeasurement {
                sensor_position: SensorPosition::RearRight,
                distance: 2.3,
                confidence: 0.89,
                temperature_compensated: true,
                measurement_time: 1000000000,
            },
        ])
    }

    fn get_parking_assistance() -> Result<ParkingAssistance, String> {
        let measurements = Self::get_measurements()?;
        
        // Find closest obstacle
        let closest = measurements.iter()
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
            .cloned();

        let closest_obstacle = closest.map(|m| ObstacleInfo {
            position: m.sensor_position,
            distance: m.distance,
            obstacle_type: if m.distance < 1.0 { ObstacleType::Wall } else { ObstacleType::Vehicle },
            confidence: m.confidence,
        });

        let warning_level = match closest_obstacle.as_ref().map(|o| o.distance) {
            Some(d) if d < 0.5 => WarningLevel::Critical,
            Some(d) if d < 1.0 => WarningLevel::Warning,
            Some(d) if d < 1.5 => WarningLevel::Caution,
            Some(d) if d < 2.0 => WarningLevel::Info,
            _ => WarningLevel::None,
        };

        let guidance = ParkingGuidance {
            direction: match warning_level {
                WarningLevel::Critical => GuidanceDirection::Stop,
                WarningLevel::Warning => GuidanceDirection::Reverse,
                _ => GuidanceDirection::Forward,
            },
            recommended_action: match warning_level {
                WarningLevel::Critical => ParkingAction::Stop,
                WarningLevel::Warning => ParkingAction::SlowDown,
                WarningLevel::Caution => ParkingAction::Continue,
                _ => ParkingAction::Continue,
            },
            distance_to_target: Some(2.5),
            steering_guidance: Some(0.0),
        };

        Ok(ParkingAssistance {
            all_measurements: measurements,
            closest_obstacle,
            parking_guidance: guidance,
            warning_level,
        })
    }

    fn get_status() -> SystemStatus {
        SystemStatus::Active
    }

    fn update_config(_config: UltrasonicConfig) -> Result<(), String> {
        println!("Updating ultrasonic configuration");
        Ok(())
    }

    fn calibrate_sensors() -> Result<(), String> {
        println!("Calibrating ultrasonic sensors");
        Ok(())
    }

    fn run_diagnostic() -> Result<DiagnosticResult, String> {
        let sensor_positions = vec![
            SensorPosition::FrontLeft,
            SensorPosition::FrontCenterLeft,
            SensorPosition::FrontCenterRight,
            SensorPosition::FrontRight,
            SensorPosition::RearLeft,
            SensorPosition::RearCenterLeft,
            SensorPosition::RearCenterRight,
            SensorPosition::RearRight,
        ];

        let sensor_health: Vec<SensorHealth> = sensor_positions.into_iter().map(|pos| {
            SensorHealth {
                position: pos,
                operational: true,
                signal_quality: 0.95,
                response_time: 0.05, // 50ms response time
            }
        }).collect();

        Ok(DiagnosticResult {
            sensor_health,
            overall_health: 0.96,
            temperature: 22.5, // Celsius
            voltage: 12.0, // Volts
        })
    }
}

export!(Component);
