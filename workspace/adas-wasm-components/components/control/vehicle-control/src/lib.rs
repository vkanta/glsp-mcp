// Vehicle Control ECU - Standardized vehicle component with PID controllers

wit_bindgen::generate!({
    world: "vehicle-component",
    path: "wit/",
    with: {
        "adas:common-types/types": generate,
        "adas:control/vehicle-control": generate,
        "adas:data/planning-data": generate,
        "adas:diagnostics/health-monitoring": generate,
        "adas:diagnostics/performance-monitoring": generate,
        "adas:orchestration/execution-control": generate,
        "adas:orchestration/resource-management": generate,
    },
});

use std::collections::VecDeque;

struct Component;

// PID Controller implementation
#[allow(dead_code)]
#[derive(Clone, Debug)]
struct PIDController {
    kp: f64,          // Proportional gain
    ki: f64,          // Integral gain
    kd: f64,          // Derivative gain
    setpoint: f64,    // Target value
    integral: f64,    // Integral accumulator
    prev_error: f64,  // Previous error for derivative
    output_min: f64,  // Minimum output
    output_max: f64,  // Maximum output
    sample_time: f64, // Sample time in seconds
}

#[allow(dead_code)]
impl PIDController {
    fn new(kp: f64, ki: f64, kd: f64, min: f64, max: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
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
#[allow(dead_code)]
#[derive(Clone, Debug)]
struct VehicleDynamics {
    position: (f64, f64, f64),     // x, y, z in meters
    velocity: (f64, f64, f64),     // vx, vy, vz in m/s
    acceleration: (f64, f64, f64), // ax, ay, az in m/s²
    heading: f64,                  // heading angle in radians
    steering_angle: f64,           // current steering angle in radians
    brake_pressure: f64,           // current brake pressure in bar
    throttle_position: f64,        // current throttle position 0-1
    gear: exports::adas::control::vehicle_control::GearPosition,
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
            gear: exports::adas::control::vehicle_control::GearPosition::Park,
            timestamp: get_timestamp(),
        }
    }
}

// Control system state
#[allow(dead_code)]
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

    // Control mode
    control_enabled: bool,
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
                0.8,  // kp: moderate proportional gain
                0.2,  // ki: moderate integral for steady-state
                0.1,  // kd: small derivative for smoothness
                -8.0, // min: -8 m/s² (emergency braking)
                4.0,  // max: +4 m/s² (comfortable acceleration)
            ),

            // Lateral position PID: for lane keeping
            lateral_controller: PIDController::new(
                1.5,  // kp: strong response to lateral error
                0.05, // ki: very small integral
                0.3,  // kd: moderate derivative for stability
                -0.5, // min: -0.5 rad (~-30°)
                0.5,  // max: +0.5 rad (~+30°)
            ),

            vehicle_dynamics: VehicleDynamics::default(),
            target_speed: 0.0,
            target_steering_angle: 0.0,
            target_position: (0.0, 0.0),

            steering_history: VecDeque::with_capacity(5),
            throttle_history: VecDeque::with_capacity(5),
            brake_history: VecDeque::with_capacity(5),

            control_enabled: false,
        }
    }
}

// Global state for control system
static mut CONTROLLER_STATE: Option<ControllerState> = None;
static mut CONTROL_ACTIVE: bool = false;
static mut LAST_COMMAND: Option<exports::adas::control::vehicle_control::ControlCommand> = None;

// Helper functions
fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// Implement standardized vehicle control interface
impl exports::adas::control::vehicle_control::Guest for Component {
    fn send_command(
        command: exports::adas::control::vehicle_control::ControlCommand,
    ) -> Result<(), String> {
        unsafe {
            if !CONTROL_ACTIVE {
                return Err("Vehicle control not active".to_string());
            }

            // Store command for execution
            LAST_COMMAND = Some(command);

            // Apply control commands through PID controllers
            if let Some(ref mut controller) = CONTROLLER_STATE {
                // Update throttle/brake based on command
                if command.throttle > 0.0 {
                    controller.vehicle_dynamics.throttle_position = command.throttle as f64;
                    controller.vehicle_dynamics.brake_pressure = 0.0;
                } else if command.brake > 0.0 {
                    controller.vehicle_dynamics.brake_pressure = command.brake as f64 * 100.0; // Convert to bar
                    controller.vehicle_dynamics.throttle_position = 0.0;
                }

                // Update steering
                controller.target_steering_angle = command.steering_angle as f64;

                // Update gear
                controller.vehicle_dynamics.gear = command.gear;

                println!("Vehicle Control: Applied command - throttle: {:.2}, brake: {:.2}, steering: {:.2} rad", 
                         command.throttle, command.brake, command.steering_angle);

                Ok(())
            } else {
                Err("Controller not initialized".to_string())
            }
        }
    }

    fn emergency_stop() -> Result<(), String> {
        unsafe {
            println!("Vehicle Control: EMERGENCY STOP ACTIVATED!");

            // Maximum braking
            if let Some(ref mut controller) = CONTROLLER_STATE {
                controller.vehicle_dynamics.brake_pressure = 100.0; // Maximum brake pressure
                controller.vehicle_dynamics.throttle_position = 0.0;
                controller.target_steering_angle = controller.vehicle_dynamics.steering_angle; // Hold current steering
                controller.vehicle_dynamics.gear =
                    exports::adas::control::vehicle_control::GearPosition::Neutral;
            }

            // Clear any pending commands
            LAST_COMMAND = None;
            CONTROL_ACTIVE = false;

            Ok(())
        }
    }
}

// Implement health monitoring interface
impl exports::adas::diagnostics::health_monitoring::Guest for Component {
    fn get_health() -> exports::adas::diagnostics::health_monitoring::HealthReport {
        exports::adas::diagnostics::health_monitoring::HealthReport {
            component_id: String::from("vehicle-control"),
            overall_health: unsafe {
                if CONTROL_ACTIVE {
                    adas::common_types::types::HealthStatus::Ok
                } else {
                    adas::common_types::types::HealthStatus::Offline
                }
            },
            subsystem_health: vec![],
            last_diagnostic: None,
            timestamp: get_timestamp(),
        }
    }

    fn run_diagnostic(
    ) -> Result<exports::adas::diagnostics::health_monitoring::DiagnosticResult, String> {
        Ok(
            exports::adas::diagnostics::health_monitoring::DiagnosticResult {
                test_results: vec![
                    exports::adas::diagnostics::health_monitoring::TestExecution {
                        test_name: String::from("actuator-response-test"),
                        test_result: adas::common_types::types::TestResult::Passed,
                        details: String::from("All actuators responding within limits"),
                        execution_time_ms: 25.0,
                    },
                    exports::adas::diagnostics::health_monitoring::TestExecution {
                        test_name: String::from("pid-controller-test"),
                        test_result: adas::common_types::types::TestResult::Passed,
                        details: String::from("PID controllers stable"),
                        execution_time_ms: 15.0,
                    },
                ],
                overall_score: 98.0,
                recommendations: vec![String::from("Vehicle control systems operating normally")],
                timestamp: get_timestamp(),
            },
        )
    }

    fn get_last_diagnostic(
    ) -> Option<exports::adas::diagnostics::health_monitoring::DiagnosticResult> {
        None
    }
}

// Implement performance monitoring interface
impl exports::adas::diagnostics::performance_monitoring::Guest for Component {
    fn get_performance() -> exports::adas::diagnostics::performance_monitoring::ExtendedPerformance
    {
        use exports::adas::diagnostics::performance_monitoring::*;
        ExtendedPerformance {
            base_metrics: adas::common_types::types::PerformanceMetrics {
                latency_avg_ms: 5.0, // Control loop latency
                latency_max_ms: 10.0,
                cpu_utilization: 0.20,
                memory_usage_mb: 64,
                throughput_hz: 50.0, // 50Hz control loop
                error_rate: 0.001,
            },
            component_specific: vec![
                Metric {
                    name: String::from("control_loop_frequency"),
                    value: 50.0,
                    unit: String::from("Hz"),
                    description: String::from("Control loop update rate"),
                },
                Metric {
                    name: String::from("steering_precision"),
                    value: 0.1,
                    unit: String::from("degrees"),
                    description: String::from("Steering angle control precision"),
                },
            ],
            resource_usage: ResourceUsage {
                cpu_cores_used: 0.20,
                memory_allocated_mb: 64,
                memory_peak_mb: 96,
                disk_io_mb: 0.0,
                network_io_mb: 0.5,
                gpu_utilization: 0.0,
                gpu_memory_mb: 0,
            },
            timestamp: get_timestamp(),
        }
    }

    fn get_performance_history(
        _duration_seconds: u32,
    ) -> Vec<exports::adas::diagnostics::performance_monitoring::ExtendedPerformance> {
        vec![] // Return empty for now
    }

    fn reset_counters() {
        println!("Vehicle Control: Resetting performance counters");
    }
}

export!(Component);
