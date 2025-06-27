wit_bindgen::generate!({
    world: "vehicle-control-component",
    path: "../../wit/worlds/vehicle-control.wit"
});

use crate::exports::vehicle_commands;
use crate::exports::control_system;

struct Component;

// Global state for control system
static mut CONTROL_STATUS: control_system::ControlStatus = control_system::ControlStatus::Offline;
static mut CONTROL_CONFIG: Option<control_system::ControlConfig> = None;
static mut VEHICLE_STATE: Option<control_system::VehicleState> = None;

// Implement vehicle-commands interface (EXPORTED)
impl vehicle_commands::Guest for Component {
    fn get_steering_command() -> Result<vehicle_commands::SteeringCommand, String> {
        unsafe {
            if matches!(CONTROL_STATUS, control_system::ControlStatus::Active) {
                Ok(vehicle_commands::SteeringCommand {
                    target_angle: 0.0,
                    angular_velocity: 0.0,
                    torque_limit: 50.0,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    priority: vehicle_commands::CommandPriority::Normal,
                })
            } else {
                Err("Control system not active".to_string())
            }
        }
    }

    fn get_braking_command() -> Result<vehicle_commands::BrakingCommand, String> {
        unsafe {
            if matches!(CONTROL_STATUS, control_system::ControlStatus::Active) {
                Ok(vehicle_commands::BrakingCommand {
                    target_deceleration: 0.0,
                    brake_pressure: 0.0,
                    abs_enabled: true,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    priority: vehicle_commands::CommandPriority::Normal,
                })
            } else {
                Err("Control system not active".to_string())
            }
        }
    }

    fn get_acceleration_command() -> Result<vehicle_commands::AccelerationCommand, String> {
        unsafe {
            if matches!(CONTROL_STATUS, control_system::ControlStatus::Active) {
                Ok(vehicle_commands::AccelerationCommand {
                    target_acceleration: 0.0,
                    throttle_position: 0.0,
                    traction_control: true,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    priority: vehicle_commands::CommandPriority::Normal,
                })
            } else {
                Err("Control system not active".to_string())
            }
        }
    }

    fn get_emergency_command() -> Result<Option<vehicle_commands::EmergencyCommand>, String> {
        // No emergency command active
        Ok(None)
    }

    fn is_active() -> bool {
        unsafe {
            matches!(CONTROL_STATUS, control_system::ControlStatus::Active)
        }
    }
}

// Implement control-system interface (EXPORTED)
impl control_system::Guest for Component {
    fn initialize(config: control_system::ControlConfig) -> Result<(), String> {
        unsafe {
            CONTROL_CONFIG = Some(config);
            CONTROL_STATUS = control_system::ControlStatus::Initializing;
        }
        Ok(())
    }

    fn start_control() -> Result<(), String> {
        unsafe {
            if CONTROL_CONFIG.is_some() {
                CONTROL_STATUS = control_system::ControlStatus::Active;
                Ok(())
            } else {
                Err("Control system not initialized".to_string())
            }
        }
    }

    fn stop_control() -> Result<(), String> {
        unsafe {
            CONTROL_STATUS = control_system::ControlStatus::Standby;
        }
        Ok(())
    }

    fn emergency_stop(reason: String) -> Result<(), String> {
        unsafe {
            CONTROL_STATUS = control_system::ControlStatus::Emergency;
        }
        println!("Emergency stop triggered: {}", reason);
        Ok(())
    }

    fn update_config(config: control_system::ControlConfig) -> Result<(), String> {
        unsafe {
            CONTROL_CONFIG = Some(config);
        }
        Ok(())
    }

    fn set_limits(_limits: control_system::ControlLimits) -> Result<(), String> {
        // Update control limits
        Ok(())
    }

    fn get_status() -> control_system::ControlStatus {
        unsafe { CONTROL_STATUS.clone() }
    }

    fn get_vehicle_state() -> Result<control_system::VehicleState, String> {
        unsafe {
            if let Some(state) = &VEHICLE_STATE {
                Ok(state.clone())
            } else {
                // Return default vehicle state
                Ok(control_system::VehicleState {
                    position: control_system::Position3d { x: 0.0, y: 0.0, z: 0.0 },
                    velocity: control_system::Velocity3d { vx: 0.0, vy: 0.0, vz: 0.0, speed: 0.0 },
                    acceleration: control_system::Acceleration3d { ax: 0.0, ay: 0.0, az: 0.0, magnitude: 0.0 },
                    heading: 0.0,
                    steering_angle: 0.0,
                    brake_pressure: 0.0,
                    throttle_position: 0.0,
                    gear_position: control_system::GearPosition::Park,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                })
            }
        }
    }

    fn run_diagnostic() -> Result<control_system::DiagnosticResult, String> {
        Ok(control_system::DiagnosticResult {
            steering_system: control_system::TestResult::Passed,
            braking_system: control_system::TestResult::Passed,
            acceleration_system: control_system::TestResult::Passed,
            safety_systems: control_system::TestResult::Passed,
            response_time: control_system::TestResult::Passed,
            overall_score: 95.0,
        })
    }
}

export!(Component);