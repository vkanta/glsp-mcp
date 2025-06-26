use wit_bindgen::generate;

// Generate bindings for the standalone sensor-fusion component
generate!({
    world: "sensor-fusion-component",
    path: "../../wit/sensor-fusion-standalone.wit"
});

use exports::adas::sensor_fusion::fusion::{Guest, SensorData, FusedData, SensorType};

struct SensorFusionComponent {
    initialized: bool,
    last_fused_data: Option<FusedData>,
    sensor_count: u32,
}

static mut SENSOR_FUSION: SensorFusionComponent = SensorFusionComponent {
    initialized: false,
    last_fused_data: None,
    sensor_count: 0,
};

impl Guest for SensorFusionComponent {
    fn initialize() -> Result<(), String> {
        unsafe {
            SENSOR_FUSION.initialized = true;
            SENSOR_FUSION.sensor_count = 0;
        }
        Ok(())
    }

    fn add_sensor_data(sensor_data: SensorData) -> Result<(), String> {
        unsafe {
            if !SENSOR_FUSION.initialized {
                return Err("Sensor fusion not initialized".to_string());
            }
            
            SENSOR_FUSION.sensor_count += 1;
            
            // Mock sensor validation
            if sensor_data.confidence < 0.0 || sensor_data.confidence > 1.0 {
                return Err("Invalid confidence value".to_string());
            }
        }
        Ok(())
    }

    fn fuse_sensors() -> Result<FusedData, String> {
        unsafe {
            if !SENSOR_FUSION.initialized {
                return Err("Sensor fusion not initialized".to_string());
            }
            
            if SENSOR_FUSION.sensor_count == 0 {
                return Err("No sensor data available for fusion".to_string());
            }
            
            // Mock sensor fusion algorithm
            let fused_data = FusedData {
                position_x: 0.0,
                position_y: 0.0,
                velocity: 15.0,
                heading: 0.785, // 45 degrees in radians
                confidence: 0.92,
                timestamp: 1234567890,
            };
            
            SENSOR_FUSION.last_fused_data = Some(fused_data.clone());
            Ok(fused_data)
        }
    }

    fn get_last_fused_data() -> Option<FusedData> {
        unsafe { SENSOR_FUSION.last_fused_data.clone() }
    }

    fn reset() -> Result<(), String> {
        unsafe {
            SENSOR_FUSION.sensor_count = 0;
            SENSOR_FUSION.last_fused_data = None;
        }
        Ok(())
    }

    fn get_sensor_count() -> u32 {
        unsafe { SENSOR_FUSION.sensor_count }
    }
}

export!(SensorFusionComponent);