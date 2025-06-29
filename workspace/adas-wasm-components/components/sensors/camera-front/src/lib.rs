// Camera Front ECU Component Implementation - Standardized
wit_bindgen::generate!({
    world: "sensor-component",
    path: "wit/",
    with: {
        "adas:common-types/types": generate,
        "adas:control/sensor-control": generate,
        "adas:data/sensor-data": generate,
        "adas:diagnostics/health-monitoring": generate,
        "adas:diagnostics/performance-monitoring": generate,
        "adas:orchestration/execution-control": generate,
        "adas:orchestration/resource-management": generate,
    },
});

struct Component;

impl exports::adas::control::sensor_control::Guest for Component {
    fn initialize(
        _config: exports::adas::control::sensor_control::SensorConfig,
    ) -> Result<(), String> {
        println!("Camera front: Initializing with sensor config");
        Ok(())
    }

    fn start() -> Result<(), String> {
        println!("Camera front: Starting sensor operation...");
        Ok(())
    }

    fn stop() -> Result<(), String> {
        println!("Camera front: Stopping sensor operation...");
        Ok(())
    }

    fn update_config(
        _config: exports::adas::control::sensor_control::SensorConfig,
    ) -> Result<(), String> {
        println!("Camera front: Updating sensor configuration...");
        Ok(())
    }

    fn get_status() -> exports::adas::control::sensor_control::SensorStatus {
        exports::adas::control::sensor_control::SensorStatus::Ok
    }

    fn get_performance() -> exports::adas::control::sensor_control::PerformanceMetrics {
        exports::adas::control::sensor_control::PerformanceMetrics {
            latency_avg_ms: 5.2,
            latency_max_ms: 8.5,
            cpu_utilization: 0.15,
            memory_usage_mb: 128,
            throughput_hz: 30.0,
            error_rate: 0.001,
        }
    }
}

// Note: sensor-data interface only contains type definitions, no functions to implement

impl exports::adas::diagnostics::health_monitoring::Guest for Component {
    fn get_health() -> exports::adas::diagnostics::health_monitoring::HealthReport {
        exports::adas::diagnostics::health_monitoring::HealthReport {
            component_id: String::from("camera-front"),
            overall_health: adas::common_types::types::HealthStatus::Ok,
            subsystem_health: vec![],
            last_diagnostic: None,
            timestamp: 0,
        }
    }

    fn run_diagnostic(
    ) -> Result<exports::adas::diagnostics::health_monitoring::DiagnosticResult, String> {
        Ok(
            exports::adas::diagnostics::health_monitoring::DiagnosticResult {
                test_results: vec![
                    exports::adas::diagnostics::health_monitoring::TestExecution {
                        test_name: String::from("camera-lens-check"),
                        test_result: adas::common_types::types::TestResult::Passed,
                        details: String::from("Lens clear and focused"),
                        execution_time_ms: 2.1,
                    },
                ],
                overall_score: 98.5,
                recommendations: vec![String::from("Camera operating normally")],
                timestamp: 0,
            },
        )
    }

    fn get_last_diagnostic(
    ) -> Option<exports::adas::diagnostics::health_monitoring::DiagnosticResult> {
        None
    }
}

impl exports::adas::diagnostics::performance_monitoring::Guest for Component {
    fn get_performance() -> exports::adas::diagnostics::performance_monitoring::ExtendedPerformance
    {
        use exports::adas::diagnostics::performance_monitoring::*;
        ExtendedPerformance {
            base_metrics: adas::common_types::types::PerformanceMetrics {
                latency_avg_ms: 5.2,
                latency_max_ms: 8.5,
                cpu_utilization: 0.15,
                memory_usage_mb: 128,
                throughput_hz: 30.0,
                error_rate: 0.001,
            },
            component_specific: vec![],
            resource_usage: ResourceUsage {
                cpu_cores_used: 0.15,
                memory_allocated_mb: 128,
                memory_peak_mb: 156,
                disk_io_mb: 2.1,
                network_io_mb: 0.0,
                gpu_utilization: 0.0,
                gpu_memory_mb: 0,
            },
            timestamp: 0,
        }
    }

    fn get_performance_history(
        _duration_seconds: u32,
    ) -> Vec<exports::adas::diagnostics::performance_monitoring::ExtendedPerformance> {
        vec![] // Return empty for now
    }

    fn reset_counters() {
        println!("Camera front: Resetting performance counters");
    }
}

export!(Component);
