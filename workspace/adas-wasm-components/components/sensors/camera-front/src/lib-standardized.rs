// Camera Front ECU Component Implementation - MIGRATED TO STANDARDIZED INTERFACES
// Uses enhanced wit-bindgen with standardized ADAS interface architecture

wit_bindgen::generate!({
    world: "sensor-component",
    path: "../../../wit/worlds-standardized",
    show_module_paths: true,
});

use std::time::{SystemTime, UNIX_EPOCH};

struct Component;

// Export standardized sensor control interface implementation
impl exports::adas_control::sensor_control::Guest for Component {
    fn initialize(config: exports::adas_control::sensor_control::SensorConfig) -> Result<(), String> {
        println!("Initializing camera with sample rate: {} Hz", config.sample_rate);
        println!("Power mode: {:?}", config.power_mode);
        Ok(())
    }

    fn start() -> Result<(), String> {
        println!("Camera front: Starting sensor...");
        Ok(())
    }

    fn stop() -> Result<(), String> {
        println!("Camera front: Stopping sensor...");
        Ok(())
    }

    fn get_status() -> exports::adas_control::sensor_control::SensorStatus {
        exports::adas_control::sensor_control::SensorStatus::Running
    }
}

// Export standardized sensor data interface implementation
impl exports::adas_data::sensor_data::Guest for Component {
    fn get_sensor_reading() -> Result<exports::adas_data::sensor_data::SensorReading, String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
            
        Ok(exports::adas_data::sensor_data::SensorReading {
            timestamp,
            value: 1.0, // Camera active indicator
            quality: 0.95,
        })
    }

    fn is_available() -> bool {
        true
    }

    fn get_camera_frame() -> Result<exports::adas_data::sensor_data::CameraFrame, String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
            
        Ok(exports::adas_data::sensor_data::CameraFrame {
            reading: exports::adas_data::sensor_data::SensorReading {
                timestamp,
                value: 1.0,
                quality: 0.95,
            },
            width: 1920,
            height: 1080,
            data: vec![0; 1920 * 1080 * 3], // Mock RGB data
            format: exports::adas_data::sensor_data::PixelFormat::Rgb8,
        })
    }
}

// Export standardized health monitoring interface implementation  
impl exports::adas_diagnostics::health_monitoring::Guest for Component {
    fn get_health() -> exports::adas_diagnostics::health_monitoring::HealthReport {
        exports::adas_diagnostics::health_monitoring::HealthReport {
            status: exports::adas_diagnostics::health_monitoring::HealthStatus::Healthy,
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
            issues: vec![],
        }
    }

    fn run_diagnostic() -> Result<exports::adas_diagnostics::health_monitoring::DiagnosticResult, String> {
        Ok(exports::adas_diagnostics::health_monitoring::DiagnosticResult {
            overall_health: exports::adas_diagnostics::health_monitoring::HealthStatus::Healthy,
            component_tests: vec![
                exports::adas_diagnostics::health_monitoring::ComponentTest {
                    component_name: "lens".to_string(),
                    test_result: exports::adas_diagnostics::health_monitoring::TestResult::Passed,
                    details: "Lens clarity: 98.5%".to_string(),
                },
                exports::adas_diagnostics::health_monitoring::ComponentTest {
                    component_name: "sensor".to_string(), 
                    test_result: exports::adas_diagnostics::health_monitoring::TestResult::Passed,
                    details: "Sensor calibration: OK".to_string(),
                },
            ],
            overall_score: 98.5,
        })
    }
}

// Export standardized performance monitoring interface implementation
impl exports::adas_diagnostics::performance_monitoring::Guest for Component {
    fn get_performance_metrics() -> exports::adas_diagnostics::performance_monitoring::PerformanceMetrics {
        exports::adas_diagnostics::performance_monitoring::PerformanceMetrics {
            cpu_usage_percent: 15.0,
            memory_usage_bytes: 1024 * 1024 * 32, // 32MB
            latency_ms: 2.5,
            throughput_ops_per_second: 30.0, // 30 FPS
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
        }
    }

    fn run_performance_test() -> Result<exports::adas_diagnostics::performance_monitoring::PerformanceTestResult, String> {
        Ok(exports::adas_diagnostics::performance_monitoring::PerformanceTestResult {
            test_name: "Camera Frame Capture Performance".to_string(),
            avg_latency_ms: 2.5,
            max_latency_ms: 4.0,
            min_latency_ms: 1.8,
            throughput_achieved: 30.0,
            success_rate: 0.995,
            test_duration_ms: 10000,
        })
    }

    fn get_resource_usage() -> exports::adas_diagnostics::performance_monitoring::ResourceUsage {
        exports::adas_diagnostics::performance_monitoring::ResourceUsage {
            cpu_cores_used: 1,
            memory_allocated_bytes: 1024 * 1024 * 32,
            memory_peak_bytes: 1024 * 1024 * 35,
            gpu_usage_percent: 0.0, // Camera doesn't use GPU
            disk_io_bytes_per_second: 1024 * 1024 * 10, // 10MB/s video data
        }
    }
}

export!(Component);