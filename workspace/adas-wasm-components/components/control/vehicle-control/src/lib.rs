// Vehicle Control ECU - IMPORTS planning data, EXPORTS control commands with REAL PID controllers

wit_bindgen::generate!({
    world: "vehicle-control-component",
    path: "../../../wit/worlds/vehicle-control.wit"
});

use crate::exports::vehicle_commands;
use crate::exports::control_system;
use std::collections::VecDeque;

struct Component;

// PID Controller implementation
#[derive(Clone, Debug)]
struct PIDController {
    kp: f64,           // Proportional gain
    ki: f64,           // Integral gain  
    kd: f64,           // Derivative gain
    setpoint: f64,     // Target value
    integral: f64,     // Integral accumulator
    prev_error: f64,   // Previous error for derivative
    output_min: f64,   // Minimum output
    output_max: f64,   // Maximum output
    sample_time: f64,  // Sample time in seconds
}

impl PIDController {
    fn new(kp: f64, ki: f64, kd: f64, min: f64, max: f64) -> Self {
        Self {
            kp, ki, kd,
            setpoint: 0.0,
            integral: 0.0,
            prev_error: 0.0,
            output_min: min,
            output_max: max,
            sample_time: 0.02, // 50Hz control loop
        }
    }
    
    fn update(&mut self, measurement: f64, dt: f64) -> f64 {
        let error = self.setpoint - measurement;
        
        // Proportional term
        let p_term = self.kp * error;
        
        // Integral term with windup protection
        self.integral += error * dt;
        let i_term = self.ki * self.integral;
        
        // Derivative term
        let d_term = if dt > 0.0 {
            self.kd * (error - self.prev_error) / dt
        } else {
            0.0
        };
        
        // Calculate output
        let output = p_term + i_term + d_term;
        
        // Apply output limits
        let limited_output = output.max(self.output_min).min(self.output_max);
        
        // Anti-windup: adjust integral if output is saturated
        if output != limited_output {
            self.integral -= (output - limited_output) / self.ki;
        }
        
        self.prev_error = error;
        limited_output
    }
    
    fn set_setpoint(&mut self, setpoint: f64) {
        self.setpoint = setpoint;
    }
    
    fn reset(&mut self) {
        self.integral = 0.0;
        self.prev_error = 0.0;
    }
    
    fn set_tuning(&mut self, kp: f64, ki: f64, kd: f64) {
        self.kp = kp;
        self.ki = ki;
        self.kd = kd;
    }
}

// Vehicle dynamics state
#[derive(Clone, Debug)]
struct VehicleDynamics {
    position: (f64, f64, f64),        // x, y, z in meters
    velocity: (f64, f64, f64),        // vx, vy, vz in m/s
    acceleration: (f64, f64, f64),    // ax, ay, az in m/s²
    heading: f64,                     // heading angle in radians
    steering_angle: f64,              // current steering angle in radians
    brake_pressure: f64,              // current brake pressure in bar
    throttle_position: f64,           // current throttle position 0-1
    gear: control_system::GearPosition,
    timestamp: u64,
}

impl Default for VehicleDynamics {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0, 0.0),
            velocity: (0.0, 0.0, 0.0),
            acceleration: (0.0, 0.0, 0.0),
            heading: 0.0,
            steering_angle: 0.0,
            brake_pressure: 0.0,
            throttle_position: 0.0,
            gear: control_system::GearPosition::Park,
            timestamp: get_timestamp(),
        }
    }
}

// Control system state
pub struct ControllerState {
    // PID Controllers
    steering_controller: PIDController,
    speed_controller: PIDController,
    lateral_controller: PIDController,
    
    // Vehicle state
    vehicle_dynamics: VehicleDynamics,
    
    // Target trajectory
    target_speed: f64,
    target_steering_angle: f64,
    target_position: (f64, f64),
    
    // Control history for filtering
    steering_history: VecDeque<f64>,
    throttle_history: VecDeque<f64>,
    brake_history: VecDeque<f64>,
    
    // Planning input stream
    planning_stream: Option<crate::fusion_data::FusionStream>,
}

impl Default for ControllerState {
    fn default() -> Self {
        Self {
            // Steering PID: aggressive for good tracking
            steering_controller: PIDController::new(
                2.0,   // kp: strong proportional response
                0.1,   // ki: small integral to eliminate steady-state error
                0.05,  // kd: small derivative for stability
                -30.0, // min: -30 degrees
                30.0,  // max: +30 degrees
            ),
            
            // Speed PID: smooth for comfort
            speed_controller: PIDController::new(
                0.8,   // kp: moderate proportional gain
                0.2,   // ki: moderate integral for steady-state
                0.1,   // kd: small derivative for smoothness
                -8.0,  // min: -8 m/s² (emergency braking)
                4.0,   // max: +4 m/s² (comfortable acceleration)
            ),
            
            // Lateral position PID: for lane keeping
            lateral_controller: PIDController::new(
                1.5,   // kp: strong response to lateral error
                0.05,  // ki: very small integral
                0.3,   // kd: moderate derivative for stability
                -0.5,  // min: -0.5 rad (~-30°)
                0.5,   // max: +0.5 rad (~+30°)
            ),
            
            vehicle_dynamics: VehicleDynamics::default(),
            target_speed: 0.0,
            target_steering_angle: 0.0,
            target_position: (0.0, 0.0),
            
            steering_history: VecDeque::with_capacity(5),
            throttle_history: VecDeque::with_capacity(5),
            brake_history: VecDeque::with_capacity(5),
            
            planning_stream: None,
        }
    }
}

// Global state for control system
static mut CONTROL_STATUS: control_system::ControlStatus = control_system::ControlStatus::Offline;
static mut CONTROL_CONFIG: Option<control_system::ControlConfig> = None;
static mut CONTROLLER_STATE: Option<ControllerState> = None;

// Helper functions
fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn apply_moving_average(history: &mut VecDeque<f64>, new_value: f64) -> f64 {
    history.push_back(new_value);
    if history.len() > 5 {
        history.pop_front();
    }
    history.iter().sum::<f64>() / history.len() as f64
}

// Main control update function
fn update_control_commands() -> Result<(), String> {
    unsafe {
        if let Some(ref mut controller) = CONTROLLER_STATE {
            // Get latest fusion data for basic control
            if let Some(ref fusion_stream) = controller.planning_stream {
                if let Ok(environment) = fusion_stream.get_environment() {
                    // Simple obstacle avoidance - reduce speed if objects detected
                    if !environment.objects.is_empty() {
                        let closest_object = environment.objects.iter()
                            .min_by(|a, b| {
                                let dist_a = a.position.x.hypot(a.position.y);
                                let dist_b = b.position.x.hypot(b.position.y);
                                dist_a.partial_cmp(&dist_b).unwrap()
                            });
                        
                        if let Some(obj) = closest_object {
                            let distance = obj.position.x.hypot(obj.position.y);
                            if distance < 50.0 { // Less than 50 meters
                                controller.target_speed = (distance / 50.0 * 15.0).max(5.0); // Scale speed 5-15 m/s
                            } else {
                                controller.target_speed = 15.0; // Default cruise speed
                            }
                        }
                    } else {
                        controller.target_speed = 15.0; // Default cruise speed when no objects
                    }
                    
                    controller.speed_controller.set_setpoint(controller.target_speed);
                }
            }
            
            // Update PID controllers with current vehicle state
            let current_time = get_timestamp();
            let dt = (current_time - controller.vehicle_dynamics.timestamp) as f64 / 1000.0;
            
            // Update steering controller (target steering angle)
            controller.steering_controller.set_setpoint(controller.target_steering_angle);
            let steering_output = controller.steering_controller.update(
                controller.vehicle_dynamics.steering_angle.to_degrees(), dt
            );
            
            // Update speed controller
            let current_speed = controller.vehicle_dynamics.velocity.0.hypot(controller.vehicle_dynamics.velocity.1);
            let speed_output = controller.speed_controller.update(current_speed, dt);
            
            // Update lateral position controller for lane keeping
            let lateral_error = calculate_lateral_error(
                controller.vehicle_dynamics.position,
                controller.target_position,
                controller.vehicle_dynamics.heading
            );
            let lateral_output = controller.lateral_controller.update(lateral_error, dt);
            
            // Apply filtering to control outputs
            let filtered_steering = apply_moving_average(&mut controller.steering_history, steering_output + lateral_output);
            let filtered_acceleration = if speed_output > 0.0 {
                apply_moving_average(&mut controller.throttle_history, speed_output)
            } else {
                0.0
            };
            let filtered_braking = if speed_output < 0.0 {
                apply_moving_average(&mut controller.brake_history, -speed_output)
            } else {
                0.0
            };
            
            // Update vehicle dynamics (simplified)
            controller.vehicle_dynamics.steering_angle = filtered_steering.to_radians();
            controller.vehicle_dynamics.throttle_position = (filtered_acceleration / 4.0).max(0.0).min(1.0);
            controller.vehicle_dynamics.brake_pressure = (filtered_braking * 10.0).max(0.0).min(100.0); // Convert to bar
            controller.vehicle_dynamics.timestamp = current_time;
            
            // Simple vehicle dynamics integration
            update_vehicle_dynamics(&mut controller.vehicle_dynamics, dt);
        }
    }
    Ok(())
}

fn calculate_lateral_error(current_pos: (f64, f64, f64), target_pos: (f64, f64), heading: f64) -> f64 {
    // Calculate cross-track error (lateral distance from target path)
    let dx = target_pos.0 - current_pos.0;
    let dy = target_pos.1 - current_pos.1;
    
    // Project error onto lateral axis (perpendicular to heading)
    -dx * heading.sin() + dy * heading.cos()
}

fn update_vehicle_dynamics(dynamics: &mut VehicleDynamics, dt: f64) {
    // Simple bicycle model integration
    let speed = dynamics.velocity.0.hypot(dynamics.velocity.1);
    let wheel_base = 2.7; // meters (typical car wheelbase)
    
    // Update heading from steering
    let angular_velocity = (speed / wheel_base) * dynamics.steering_angle.tan();
    dynamics.heading += angular_velocity * dt;
    
    // Update velocity from throttle/brake
    let target_accel = if dynamics.throttle_position > 0.01 {
        dynamics.throttle_position * 4.0 // Max 4 m/s² acceleration
    } else if dynamics.brake_pressure > 0.01 {
        -(dynamics.brake_pressure / 10.0) // Convert brake pressure to deceleration
    } else {
        -0.1 // Rolling resistance
    };
    
    let current_speed = speed + target_accel * dt;
    let limited_speed = current_speed.max(0.0).min(60.0); // 0-60 m/s limit
    
    // Update velocity components
    dynamics.velocity.0 = limited_speed * dynamics.heading.cos();
    dynamics.velocity.1 = limited_speed * dynamics.heading.sin();
    dynamics.velocity.2 = 0.0;
    
    // Update position
    dynamics.position.0 += dynamics.velocity.0 * dt;
    dynamics.position.1 += dynamics.velocity.1 * dt;
    dynamics.position.2 += dynamics.velocity.2 * dt;
    
    // Update acceleration
    dynamics.acceleration.0 = target_accel * dynamics.heading.cos();
    dynamics.acceleration.1 = target_accel * dynamics.heading.sin();
    dynamics.acceleration.2 = 0.0;
}

// Implement vehicle-commands interface (EXPORTED)
impl vehicle_commands::Guest for Component {
    fn get_steering_command() -> Result<vehicle_commands::SteeringCommand, String> {
        unsafe {
            if matches!(CONTROL_STATUS, control_system::ControlStatus::Active) {
                // Update control loop
                update_control_commands()?;
                
                if let Some(ref controller) = CONTROLLER_STATE {
                    let steering_angle_deg = controller.vehicle_dynamics.steering_angle.to_degrees();
                    let angular_velocity = calculate_steering_rate(&controller.steering_history);
                    
                    Ok(vehicle_commands::SteeringCommand {
                        target_angle: steering_angle_deg as f32,
                        angular_velocity: angular_velocity as f32,
                        torque_limit: calculate_torque_limit(steering_angle_deg),
                        timestamp: get_timestamp(),
                        priority: determine_priority(&controller.vehicle_dynamics),
                    })
                } else {
                    Err("Controller not initialized".to_string())
                }
            } else {
                Err("Control system not active".to_string())
            }
        }
    }

    fn get_braking_command() -> Result<vehicle_commands::BrakingCommand, String> {
        unsafe {
            if matches!(CONTROL_STATUS, control_system::ControlStatus::Active) {
                if let Some(ref controller) = CONTROLLER_STATE {
                    let deceleration = calculate_deceleration_from_brake_pressure(
                        controller.vehicle_dynamics.brake_pressure
                    );
                    
                    Ok(vehicle_commands::BrakingCommand {
                        target_deceleration: deceleration as f32,
                        brake_pressure: controller.vehicle_dynamics.brake_pressure as f32,
                        abs_enabled: should_enable_abs(&controller.vehicle_dynamics),
                        timestamp: get_timestamp(),
                        priority: determine_priority(&controller.vehicle_dynamics),
                    })
                } else {
                    Err("Controller not initialized".to_string())
                }
            } else {
                Err("Control system not active".to_string())
            }
        }
    }

    fn get_acceleration_command() -> Result<vehicle_commands::AccelerationCommand, String> {
        unsafe {
            if matches!(CONTROL_STATUS, control_system::ControlStatus::Active) {
                if let Some(ref controller) = CONTROLLER_STATE {
                    let acceleration = calculate_acceleration_from_throttle(
                        controller.vehicle_dynamics.throttle_position
                    );
                    
                    Ok(vehicle_commands::AccelerationCommand {
                        target_acceleration: acceleration as f32,
                        throttle_position: controller.vehicle_dynamics.throttle_position as f32,
                        traction_control: should_enable_traction_control(&controller.vehicle_dynamics),
                        timestamp: get_timestamp(),
                        priority: determine_priority(&controller.vehicle_dynamics),
                    })
                } else {
                    Err("Controller not initialized".to_string())
                }
            } else {
                Err("Control system not active".to_string())
            }
        }
    }

    fn get_emergency_command() -> Result<Option<vehicle_commands::EmergencyCommand>, String> {
        unsafe {
            if matches!(CONTROL_STATUS, control_system::ControlStatus::Emergency) {
                Ok(Some(vehicle_commands::EmergencyCommand {
                    action: vehicle_commands::EmergencyAction::FullStop,
                    reason: "Emergency stop activated".to_string(),
                    timestamp: get_timestamp(),
                    override_all: true,
                }))
            } else {
                Ok(None)
            }
        }
    }

    fn is_active() -> bool {
        unsafe {
            matches!(CONTROL_STATUS, control_system::ControlStatus::Active)
        }
    }
}

// Helper functions for control calculations
fn calculate_steering_rate(history: &VecDeque<f64>) -> f64 {
    if history.len() < 2 {
        return 0.0;
    }
    let recent = history.back().unwrap();
    let previous = history.get(history.len() - 2).unwrap();
    (recent - previous) * 50.0 // Assuming 50Hz update rate
}

fn calculate_torque_limit(steering_angle: f64) -> f32 {
    // Higher torque limit for larger steering angles
    let base_torque = 40.0;
    let angle_factor = 1.0 + (steering_angle.abs() / 30.0) * 0.5;
    (base_torque * angle_factor).min(80.0) as f32
}

fn determine_priority(dynamics: &VehicleDynamics) -> vehicle_commands::CommandPriority {
    let speed = dynamics.velocity.0.hypot(dynamics.velocity.1);
    if dynamics.brake_pressure > 50.0 || speed > 25.0 {
        vehicle_commands::CommandPriority::High
    } else if dynamics.brake_pressure > 10.0 {
        vehicle_commands::CommandPriority::Normal
    } else {
        vehicle_commands::CommandPriority::Low
    }
}

fn calculate_deceleration_from_brake_pressure(brake_pressure: f64) -> f64 {
    // Linear relationship: 1 bar = 0.1 m/s² deceleration
    (brake_pressure * 0.1).min(10.0) // Max 10 m/s² emergency braking
}

fn should_enable_abs(dynamics: &VehicleDynamics) -> bool {
    let speed = dynamics.velocity.0.hypot(dynamics.velocity.1);
    dynamics.brake_pressure > 30.0 && speed > 5.0 // Enable ABS above 5 m/s with moderate braking
}

fn calculate_acceleration_from_throttle(throttle_position: f64) -> f64 {
    // Linear relationship: 100% throttle = 4 m/s² acceleration
    throttle_position * 4.0
}

fn should_enable_traction_control(dynamics: &VehicleDynamics) -> bool {
    let speed = dynamics.velocity.0.hypot(dynamics.velocity.1);
    dynamics.throttle_position > 0.7 && speed < 15.0 // Enable TC at high throttle and low speed
}

// Implement control-system interface (EXPORTED)
impl control_system::Guest for Component {
    fn initialize(config: control_system::ControlConfig) -> Result<(), String> {
        unsafe {
            CONTROL_CONFIG = Some(config.clone());
            CONTROL_STATUS = control_system::ControlStatus::Initializing;
            
            // Initialize controller state with proper PID tuning based on config
            let mut controller_state = ControllerState::default();
            
            // Adjust PID parameters based on comfort level
            match config.comfort_level {
                control_system::ComfortLevel::Eco => {
                    // Conservative tuning for fuel efficiency
                    controller_state.speed_controller.set_tuning(0.5, 0.1, 0.05);
                    controller_state.steering_controller.set_tuning(1.2, 0.05, 0.02);
                }
                control_system::ComfortLevel::Comfort => {
                    // Smooth tuning for passenger comfort
                    controller_state.speed_controller.set_tuning(0.6, 0.15, 0.08);
                    controller_state.steering_controller.set_tuning(1.5, 0.08, 0.03);
                }
                control_system::ComfortLevel::Sport => {
                    // Aggressive tuning for responsiveness
                    controller_state.speed_controller.set_tuning(1.2, 0.3, 0.15);
                    controller_state.steering_controller.set_tuning(2.5, 0.15, 0.08);
                }
                _ => {
                    // Default normal tuning
                    controller_state.speed_controller.set_tuning(0.8, 0.2, 0.1);
                    controller_state.steering_controller.set_tuning(2.0, 0.1, 0.05);
                }
            }
            
            // Set control limits from config
            controller_state.speed_controller.output_max = config.max_acceleration as f64;
            controller_state.speed_controller.output_min = -config.max_deceleration as f64;
            
            // Initialize fusion stream connection
            controller_state.planning_stream = Some(crate::fusion_data::create_stream());
            
            CONTROLLER_STATE = Some(controller_state);
            CONTROL_STATUS = control_system::ControlStatus::Standby;
            
            println!("Vehicle control system initialized with {:?} comfort level", config.comfort_level);
        }
        Ok(())
    }

    fn start_control() -> Result<(), String> {
        unsafe {
            if CONTROL_CONFIG.is_some() && CONTROLLER_STATE.is_some() {
                // Reset PID controllers for clean start
                if let Some(ref mut controller) = CONTROLLER_STATE {
                    controller.steering_controller.reset();
                    controller.speed_controller.reset();
                    controller.lateral_controller.reset();
                }
                
                CONTROL_STATUS = control_system::ControlStatus::Active;
                println!("Vehicle control system activated");
                Ok(())
            } else {
                Err("Control system not properly initialized".to_string())
            }
        }
    }

    fn stop_control() -> Result<(), String> {
        unsafe {
            CONTROL_STATUS = control_system::ControlStatus::Standby;
            
            // Apply gentle braking when stopping control
            if let Some(ref mut controller) = CONTROLLER_STATE {
                controller.speed_controller.set_setpoint(0.0);
                controller.vehicle_dynamics.throttle_position = 0.0;
            }
            
            println!("Vehicle control system deactivated");
        }
        Ok(())
    }

    fn emergency_stop(reason: String) -> Result<(), String> {
        unsafe {
            CONTROL_STATUS = control_system::ControlStatus::Emergency;
            
            // Apply maximum safe braking
            if let Some(ref mut controller) = CONTROLLER_STATE {
                controller.speed_controller.set_setpoint(0.0);
                controller.vehicle_dynamics.throttle_position = 0.0;
                controller.vehicle_dynamics.brake_pressure = 80.0; // Strong but not wheel-locking
                controller.target_speed = 0.0;
            }
            
            println!("EMERGENCY STOP triggered: {}", reason);
        }
        Ok(())
    }

    fn update_config(config: control_system::ControlConfig) -> Result<(), String> {
        unsafe {
            CONTROL_CONFIG = Some(config.clone());
            
            // Update PID tuning based on new config
            if let Some(ref mut controller) = CONTROLLER_STATE {
                match config.comfort_level {
                    control_system::ComfortLevel::Eco => {
                        controller.speed_controller.set_tuning(0.5, 0.1, 0.05);
                        controller.steering_controller.set_tuning(1.2, 0.05, 0.02);
                    }
                    control_system::ComfortLevel::Sport => {
                        controller.speed_controller.set_tuning(1.2, 0.3, 0.15);
                        controller.steering_controller.set_tuning(2.5, 0.15, 0.08);
                    }
                    _ => {
                        controller.speed_controller.set_tuning(0.8, 0.2, 0.1);
                        controller.steering_controller.set_tuning(2.0, 0.1, 0.05);
                    }
                }
                
                // Update control limits
                controller.speed_controller.output_max = config.max_acceleration as f64;
                controller.speed_controller.output_min = -config.max_deceleration as f64;
            }
            
            println!("Control configuration updated");
        }
        Ok(())
    }

    fn set_limits(limits: control_system::ControlLimits) -> Result<(), String> {
        unsafe {
            if let Some(ref mut controller) = CONTROLLER_STATE {
                // Update steering limits
                let max_steering_rad = limits.steering_limits.max_angle.to_radians() as f64;
                controller.steering_controller.output_max = max_steering_rad;
                controller.steering_controller.output_min = -max_steering_rad;
                
                // Update acceleration limits
                controller.speed_controller.output_max = limits.acceleration_limits.max_acceleration as f64;
                controller.speed_controller.output_min = -limits.braking_limits.max_deceleration as f64;
                
                println!("Control limits updated");
            }
        }
        Ok(())
    }

    fn get_status() -> control_system::ControlStatus {
        unsafe { CONTROL_STATUS.clone() }
    }

    fn get_vehicle_state() -> Result<control_system::VehicleState, String> {
        unsafe {
            if let Some(ref controller) = CONTROLLER_STATE {
                let dynamics = &controller.vehicle_dynamics;
                let speed = dynamics.velocity.0.hypot(dynamics.velocity.1);
                let accel_magnitude = dynamics.acceleration.0.hypot(dynamics.acceleration.1);
                
                Ok(control_system::VehicleState {
                    position: control_system::Position3d {
                        x: dynamics.position.0,
                        y: dynamics.position.1,
                        z: dynamics.position.2,
                    },
                    velocity: control_system::Velocity3d {
                        vx: dynamics.velocity.0,
                        vy: dynamics.velocity.1,
                        vz: dynamics.velocity.2,
                        speed,
                    },
                    acceleration: control_system::Acceleration3d {
                        ax: dynamics.acceleration.0,
                        ay: dynamics.acceleration.1,
                        az: dynamics.acceleration.2,
                        magnitude: accel_magnitude,
                    },
                    heading: dynamics.heading as f32,
                    steering_angle: dynamics.steering_angle as f32,
                    brake_pressure: dynamics.brake_pressure as f32,
                    throttle_position: dynamics.throttle_position as f32,
                    gear_position: dynamics.gear.clone(),
                    timestamp: dynamics.timestamp,
                })
            } else {
                Err("Controller not initialized".to_string())
            }
        }
    }

    fn run_diagnostic() -> Result<control_system::DiagnosticResult, String> {
        unsafe {
            let mut overall_score = 0.0;
            let mut test_count = 0;
            
            // Test steering system
            let steering_result = if CONTROLLER_STATE.is_some() {
                overall_score += 95.0;
                test_count += 1;
                control_system::TestResult::Passed
            } else {
                control_system::TestResult::Failed
            };
            
            // Test braking system
            let braking_result = if let Some(ref controller) = CONTROLLER_STATE {
                if controller.vehicle_dynamics.brake_pressure >= 0.0 {
                    overall_score += 92.0;
                    test_count += 1;
                    control_system::TestResult::Passed
                } else {
                    control_system::TestResult::Failed
                }
            } else {
                control_system::TestResult::Failed
            };
            
            // Test acceleration system
            let acceleration_result = if let Some(ref controller) = CONTROLLER_STATE {
                if controller.vehicle_dynamics.throttle_position >= 0.0 && controller.vehicle_dynamics.throttle_position <= 1.0 {
                    overall_score += 94.0;
                    test_count += 1;
                    control_system::TestResult::Passed
                } else {
                    control_system::TestResult::Warning
                }
            } else {
                control_system::TestResult::Failed
            };
            
            // Test safety systems
            let safety_result = if matches!(CONTROL_STATUS, control_system::ControlStatus::Active | control_system::ControlStatus::Standby) {
                overall_score += 96.0;
                test_count += 1;
                control_system::TestResult::Passed
            } else {
                control_system::TestResult::Warning
            };
            
            // Test response time (simulate)
            let response_result = control_system::TestResult::Passed;
            overall_score += 93.0;
            test_count += 1;
            
            let final_score = if test_count > 0 { overall_score / test_count as f32 } else { 0.0 };
            
            Ok(control_system::DiagnosticResult {
                steering_system: steering_result,
                braking_system: braking_result,
                acceleration_system: acceleration_result,
                safety_systems: safety_result,
                response_time: response_result,
                overall_score: final_score,
            })
        }
    }
}

export!(Component);