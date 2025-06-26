use wit_bindgen::generate;

// Generate bindings for the standalone perception component
generate!({
    world: "perception-component",
    path: "../../wit/perception-standalone.wit"
});

use exports::adas::perception::perception::{Guest, ObjectType, Detection, SceneUnderstanding};

struct PerceptionComponent {
    initialized: bool,
    last_scene: Option<SceneUnderstanding>,
}

static mut PERCEPTION: PerceptionComponent = PerceptionComponent {
    initialized: false,
    last_scene: None,
};

impl Guest for PerceptionComponent {
    fn initialize() -> Result<(), String> {
        unsafe {
            PERCEPTION.initialized = true;
        }
        Ok(())
    }

    fn process_camera_data() -> Result<Vec<Detection>, String> {
        unsafe {
            if !PERCEPTION.initialized {
                return Err("Perception not initialized".to_string());
            }
            
            // Mock detection data
            let detections = vec![
                Detection {
                    object_type: ObjectType::Vehicle,
                    confidence: 0.95,
                    x: 100.0,
                    y: 150.0,
                    width: 200.0,
                    height: 120.0,
                },
                Detection {
                    object_type: ObjectType::Pedestrian,
                    confidence: 0.87,
                    x: 350.0,
                    y: 200.0,
                    width: 80.0,
                    height: 180.0,
                },
            ];
            
            Ok(detections)
        }
    }

    fn process_lidar_data() -> Result<Vec<Detection>, String> {
        unsafe {
            if !PERCEPTION.initialized {
                return Err("Perception not initialized".to_string());
            }
            
            // Mock LiDAR detection data
            let detections = vec![
                Detection {
                    object_type: ObjectType::Vehicle,
                    confidence: 0.92,
                    x: 105.0,
                    y: 155.0,
                    width: 195.0,
                    height: 118.0,
                },
            ];
            
            Ok(detections)
        }
    }

    fn get_scene_understanding() -> Result<SceneUnderstanding, String> {
        unsafe {
            if !PERCEPTION.initialized {
                return Err("Perception not initialized".to_string());
            }
            
            let scene = SceneUnderstanding {
                timestamp: 1234567890,
                num_vehicles: 2,
                num_pedestrians: 1,
                num_obstacles: 0,
                road_clear: true,
            };
            
            PERCEPTION.last_scene = Some(scene.clone());
            Ok(scene)
        }
    }

    fn is_initialized() -> bool {
        unsafe { PERCEPTION.initialized }
    }
}

export!(PerceptionComponent);