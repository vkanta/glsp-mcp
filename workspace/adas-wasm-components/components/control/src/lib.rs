use wit_bindgen::generate;

// Generate bindings for the standalone control component
generate!({
    world: "control-component",
    path: "../../wit/control-standalone.wit"
});

use exports::adas::control::control::{Guest, ControlCommand, ControlStatus};

struct ControlComponent {
    initialized: bool,
    status: ControlStatus,
    last_command: Option<ControlCommand>,
}

static mut CONTROL: ControlComponent = ControlComponent {
    initialized: false,
    status: ControlStatus::Offline,
    last_command: None,
};

impl Guest for ControlComponent {
    fn initialize() -> Result<(), String> {
        unsafe {
            CONTROL.initialized = true;
            CONTROL.status = ControlStatus::Standby;
        }
        Ok(())
    }

    fn execute_control(steering: f32, throttle: f32, brake: f32) -> Result<(), String> {
        unsafe {
            if !CONTROL.initialized {
                return Err("Control not initialized".to_string());
            }
            
            // Validate input ranges
            if steering < -1.0 || steering > 1.0 {
                return Err("Steering must be between -1.0 and 1.0".to_string());
            }
            if throttle < 0.0 || throttle > 1.0 {
                return Err("Throttle must be between 0.0 and 1.0".to_string());
            }
            if brake < 0.0 || brake > 1.0 {
                return Err("Brake must be between 0.0 and 1.0".to_string());
            }
            
            let command = ControlCommand {
                steering_angle: steering,
                throttle,
                brake,
                timestamp: 1234567890,
            };
            
            CONTROL.last_command = Some(command);
            CONTROL.status = ControlStatus::Active;
        }
        Ok(())
    }

    fn emergency_stop() -> Result<(), String> {
        unsafe {
            if !CONTROL.initialized {
                return Err("Control not initialized".to_string());
            }
            
            let command = ControlCommand {
                steering_angle: 0.0,
                throttle: 0.0,
                brake: 1.0,
                timestamp: 1234567890,
            };
            
            CONTROL.last_command = Some(command);
            CONTROL.status = ControlStatus::Emergency;
        }
        Ok(())
    }

    fn get_status() -> ControlStatus {
        unsafe { CONTROL.status.clone() }
    }

    fn get_last_command() -> Option<ControlCommand> {
        unsafe { CONTROL.last_command.clone() }
    }
}

export!(ControlComponent);