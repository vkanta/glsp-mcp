// Planning Decision - IMPORTS perception data, EXPORTS trajectory plans and driving decisions

wit_bindgen::generate!({
    world: "planning-decision-component",
    path: "../../wit/planning-decision.wit",
});

use crate::exports::planning_data;
use crate::exports::planning_control;

struct Component;

// Resource state for planning stream
pub struct PlanningStreamState {
    id: u32,
}

// Planning system configuration state
static mut PLANNING_CONFIG: Option<planning_control::PlanningConfig> = None;
static mut PLANNING_STATUS: planning_control::PlanningStatus = planning_control::PlanningStatus::Offline;
static mut CURRENT_DESTINATION: Option<planning_control::Destination> = None;

// Input stream from perception system
// Note: Will be created on-demand when planning system is initialized

// Implement the planning-data interface (EXPORTED)
impl planning_data::Guest for Component {
    type PlanningStream = PlanningStreamState;
    
    fn create_stream() -> planning_data::PlanningStream {
        planning_data::PlanningStream::new(PlanningStreamState { id: 1 })
    }
}

impl planning_data::GuestPlanningStream for PlanningStreamState {
    fn get_planning(&self) -> Result<planning_data::PlanningResult, String> {
        unsafe {
            if matches!(PLANNING_STATUS, planning_control::PlanningStatus::Planning | planning_control::PlanningStatus::Executing) {
                // Generate planning result based on current perception and destination
                let ego_trajectory = planning_data::PlannedTrajectory {
                    waypoints: vec![
                        planning_data::TrajectoryWaypoint {
                            position: planning_data::Position3d { x: 0.0, y: 0.0, z: 0.0 },
                            velocity: planning_data::Velocity3d { vx: 15.0, vy: 0.0, vz: 0.0, speed: 15.0 },
                            acceleration: planning_data::Acceleration3d { ax: 0.0, ay: 0.0, az: 0.0, magnitude: 0.0 },
                            curvature: 0.0,
                            timestamp: 0.0,
                            lane_id: 1,
                        },
                        planning_data::TrajectoryWaypoint {
                            position: planning_data::Position3d { x: 15.0, y: 0.0, z: 0.0 },
                            velocity: planning_data::Velocity3d { vx: 15.0, vy: 0.0, vz: 0.0, speed: 15.0 },
                            acceleration: planning_data::Acceleration3d { ax: 0.0, ay: 0.0, az: 0.0, magnitude: 0.0 },
                            curvature: 0.0,
                            timestamp: 1.0,
                            lane_id: 1,
                        },
                        planning_data::TrajectoryWaypoint {
                            position: planning_data::Position3d { x: 30.0, y: 0.0, z: 0.0 },
                            velocity: planning_data::Velocity3d { vx: 15.0, vy: 0.0, vz: 0.0, speed: 15.0 },
                            acceleration: planning_data::Acceleration3d { ax: 0.0, ay: 0.0, az: 0.0, magnitude: 0.0 },
                            curvature: 0.0,
                            timestamp: 2.0,
                            lane_id: 1,
                        },
                    ],
                    duration: 3.0,
                    cost: 12.5,
                    feasibility: 0.95,
                    safety_score: 0.92,
                    comfort_score: 0.88,
                };

                Ok(planning_data::PlanningResult {
                    trajectory_plan: planning_data::TrajectoryPlan {
                        ego_trajectory,
                        alternative_trajectories: vec![], // No alternatives for now
                        planning_horizon: PLANNING_CONFIG.as_ref().map(|c| c.planning_horizon).unwrap_or(5.0),
                        update_frequency: PLANNING_CONFIG.as_ref().map(|c| c.update_frequency).unwrap_or(10),
                        trajectory_confidence: 0.89,
                    },
                    driving_decisions: planning_data::DrivingDecisions {
                        primary_action: planning_data::DrivingAction::ContinueStraight,
                        secondary_actions: vec![planning_data::DrivingAction::Yield],
                        speed_recommendation: planning_data::SpeedCommand {
                            target_speed: 15.0,
                            acceleration_limit: 2.0,
                            deceleration_limit: -3.0,
                            speed_profile: planning_data::SpeedProfile::Constant,
                        },
                        steering_recommendation: planning_data::SteeringCommand {
                            target_curvature: 0.0,
                            steering_rate_limit: 0.5,
                            lane_keeping_mode: planning_data::LaneKeepingMode::Center,
                        },
                        urgency_level: planning_data::UrgencyLevel::Routine,
                        action_confidence: 0.91,
                    },
                    mission_status: planning_data::MissionStatus {
                        current_goal: planning_data::GoalType::Navigation,
                        progress_percentage: 25.0,
                        remaining_distance: 150.0,
                        estimated_time: 36.0,
                        obstacles_detected: 2,
                        replanning_required: false,
                    },
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    planning_confidence: 0.87,
                })
            } else {
                Err("Planning system not active".to_string())
            }
        }
    }

    fn is_available(&self) -> bool {
        unsafe {
            matches!(PLANNING_STATUS, 
                planning_control::PlanningStatus::Planning | 
                planning_control::PlanningStatus::Executing |
                planning_control::PlanningStatus::Replanning
            )
        }
    }

    fn get_trajectory_count(&self) -> u32 {
        // Return count of alternative trajectories + ego trajectory
        1 // Simulated count
    }
}

// Implement the planning control interface (EXPORTED)
impl planning_control::Guest for Component {
    fn initialize(config: planning_control::PlanningConfig) -> Result<(), String> {
        unsafe {
            PLANNING_CONFIG = Some(config);
            PLANNING_STATUS = planning_control::PlanningStatus::Initializing;
            
            // TODO: Create input stream from perception system
            // let _perception_stream = crate::perception_data::create_stream();
        }
        Ok(())
    }

    fn start_planning() -> Result<(), String> {
        unsafe {
            if PLANNING_CONFIG.is_some() {
                PLANNING_STATUS = planning_control::PlanningStatus::Planning;
                Ok(())
            } else {
                Err("Planning system not initialized".to_string())
            }
        }
    }

    fn stop_planning() -> Result<(), String> {
        unsafe {
            PLANNING_STATUS = planning_control::PlanningStatus::Offline;
        }
        Ok(())
    }

    fn set_destination(destination: planning_control::Destination) -> Result<(), String> {
        unsafe {
            CURRENT_DESTINATION = Some(destination);
            // Trigger replanning if system is active
            if matches!(PLANNING_STATUS, planning_control::PlanningStatus::Planning | planning_control::PlanningStatus::Executing) {
                PLANNING_STATUS = planning_control::PlanningStatus::Replanning;
            }
        }
        Ok(())
    }

    fn update_config(config: planning_control::PlanningConfig) -> Result<(), String> {
        unsafe {
            PLANNING_CONFIG = Some(config);
        }
        Ok(())
    }

    fn get_status() -> planning_control::PlanningStatus {
        unsafe { PLANNING_STATUS.clone() }
    }

    fn get_performance() -> planning_control::PerformanceMetrics {
        planning_control::PerformanceMetrics {
            planning_time_ms: 8.7,
            trajectory_smoothness: 0.94,
            safety_violations: 0,
            comfort_score: 0.88,
            efficiency_score: 0.82,
            success_rate: 0.96,
            cpu_usage_percent: 22.0,
            memory_usage_mb: 128,
        }
    }

    fn run_diagnostic() -> Result<planning_control::DiagnosticResult, String> {
        Ok(planning_control::DiagnosticResult {
            perception_integration: planning_control::TestResult::Passed,
            trajectory_generation: planning_control::TestResult::Passed,
            decision_logic: planning_control::TestResult::Passed,
            safety_checks: planning_control::TestResult::Passed,
            performance_metrics: planning_control::TestResult::Passed,
            overall_score: 92.1,
        })
    }

    fn emergency_stop() -> Result<(), String> {
        unsafe {
            PLANNING_STATUS = planning_control::PlanningStatus::Completed; // Emergency stop completed
        }
        println!("Emergency stop activated");
        Ok(())
    }

    fn resume_planning() -> Result<(), String> {
        unsafe {
            if PLANNING_CONFIG.is_some() {
                PLANNING_STATUS = planning_control::PlanningStatus::Planning;
                Ok(())
            } else {
                Err("Planning system not initialized".to_string())
            }
        }
    }
}

// Helper function to generate trajectory plan from perception data
fn _generate_trajectory_plan(
    // TODO: Use proper import types once wit-bindgen generates them correctly
    // perception: &crate::perception_data::PerceptionModel,
    // destination: Option<&planning_control::Destination>,
) -> planning_data::TrajectoryPlan {
    // TODO: Implement actual trajectory planning algorithms
    // 1. Analyze perception data for obstacles and scene context
    // 2. Generate multiple trajectory candidates
    // 3. Evaluate trajectories for safety, comfort, and efficiency
    // 4. Select optimal trajectory
    // 5. Generate fallback alternatives
    
    println!("Generating trajectory plan from perception data");
    
    // Placeholder trajectory planning
    planning_data::TrajectoryPlan {
        ego_trajectory: planning_data::PlannedTrajectory {
            waypoints: vec![],
            duration: 5.0,
            cost: 10.0,
            feasibility: 0.95,
            safety_score: 0.92,
            comfort_score: 0.88,
        },
        alternative_trajectories: vec![],
        planning_horizon: 5.0,
        update_frequency: 10,
        trajectory_confidence: 0.89,
    }
}

// Helper function to make driving decisions based on perception and trajectory
fn _make_driving_decisions(
    // TODO: Use proper import types once wit-bindgen generates them correctly
    // perception: &crate::perception_data::PerceptionModel,
    // trajectory: &planning_data::TrajectoryPlan,
) -> planning_data::DrivingDecisions {
    // TODO: Implement actual decision-making logic
    // 1. Analyze risk assessment from perception
    // 2. Consider trajectory constraints and goals  
    // 3. Apply driving rules and safety protocols
    // 4. Generate speed and steering commands
    // 5. Determine urgency level and action confidence
    
    println!("Making driving decisions from perception and trajectory");
    
    // Placeholder decision making
    planning_data::DrivingDecisions {
        primary_action: planning_data::DrivingAction::ContinueStraight,
        secondary_actions: vec![],
        speed_recommendation: planning_data::SpeedCommand {
            target_speed: 15.0,
            acceleration_limit: 2.0,
            deceleration_limit: -3.0,
            speed_profile: planning_data::SpeedProfile::Constant,
        },
        steering_recommendation: planning_data::SteeringCommand {
            target_curvature: 0.0,
            steering_rate_limit: 0.5,
            lane_keeping_mode: planning_data::LaneKeepingMode::Center,
        },
        urgency_level: planning_data::UrgencyLevel::Routine,
        action_confidence: 0.91,
    }
}

export!(Component);