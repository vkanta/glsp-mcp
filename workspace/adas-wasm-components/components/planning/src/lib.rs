use wit_bindgen::generate;

// Generate bindings for the standalone planning component
generate!({
    world: "planning-component",
    path: "../../wit/planning-standalone.wit"
});

use exports::adas::planning::planning::{Guest, Waypoint, Trajectory, PlanningStatus};

struct PlanningComponent {
    initialized: bool,
    status: PlanningStatus,
    current_trajectory: Option<Trajectory>,
}

static mut PLANNING: PlanningComponent = PlanningComponent {
    initialized: false,
    status: PlanningStatus::Idle,
    current_trajectory: None,
};

impl Guest for PlanningComponent {
    fn initialize() -> Result<(), String> {
        unsafe {
            PLANNING.initialized = true;
            PLANNING.status = PlanningStatus::Ready;
        }
        Ok(())
    }

    fn plan_path(start_x: f64, start_y: f64, goal_x: f64, goal_y: f64) -> Result<Trajectory, String> {
        unsafe {
            if !PLANNING.initialized {
                return Err("Planning not initialized".to_string());
            }
            
            PLANNING.status = PlanningStatus::Planning;
            
            // Create a simple straight-line trajectory with waypoints
            let num_waypoints = 5;
            let mut waypoints = Vec::new();
            
            for i in 0..=num_waypoints {
                let t = i as f64 / num_waypoints as f64;
                let x = start_x + t * (goal_x - start_x);
                let y = start_y + t * (goal_y - start_y);
                
                waypoints.push(Waypoint {
                    x,
                    y,
                    heading: 0.0,
                    velocity: 10.0, // 10 m/s target velocity
                });
            }
            
            let trajectory = Trajectory {
                waypoints,
                total_distance: ((goal_x - start_x).powi(2) + (goal_y - start_y).powi(2)).sqrt(),
                estimated_time: 30.0, // 30 seconds
                timestamp: 1234567890,
            };
            
            PLANNING.current_trajectory = Some(trajectory.clone());
            PLANNING.status = PlanningStatus::Ready;
            
            Ok(trajectory)
        }
    }

    fn update_trajectory() -> Result<(), String> {
        unsafe {
            if !PLANNING.initialized {
                return Err("Planning not initialized".to_string());
            }
            
            if PLANNING.current_trajectory.is_none() {
                return Err("No trajectory to update".to_string());
            }
            
            // Mock trajectory update logic
            PLANNING.status = PlanningStatus::Executing;
        }
        Ok(())
    }

    fn get_status() -> PlanningStatus {
        unsafe { PLANNING.status.clone() }
    }

    fn get_current_trajectory() -> Option<Trajectory> {
        unsafe { PLANNING.current_trajectory.clone() }
    }

    fn abort_planning() -> Result<(), String> {
        unsafe {
            PLANNING.status = PlanningStatus::Aborted;
            PLANNING.current_trajectory = None;
        }
        Ok(())
    }
}

export!(PlanningComponent);