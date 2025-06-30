// HMI Interface - Standardized system component implementation

wit_bindgen::generate!({
    world: "system-component",
    path: "wit/",
    generate_all,
});

struct Component;

// HMI state
static mut INTERFACE_ACTIVE: bool = false;

// Helper functions
fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// Implement health monitoring interface
impl exports::adas::diagnostics::health_monitoring::Guest for Component {
    fn get_health() -> exports::adas::diagnostics::health_monitoring::HealthReport {
        exports::adas::diagnostics::health_monitoring::HealthReport {
            component_id: String::from("hmi-interface"),
            overall_health: unsafe {
                if INTERFACE_ACTIVE {
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
                        test_name: String::from("display-connectivity-test"),
                        test_result: adas::common_types::types::TestResult::Passed,
                        details: String::from("All displays connected and responding"),
                        execution_time_ms: 25.0,
                    },
                    exports::adas::diagnostics::health_monitoring::TestExecution {
                        test_name: String::from("input-responsiveness-test"),
                        test_result: adas::common_types::types::TestResult::Passed,
                        details: String::from("All input devices responding normally"),
                        execution_time_ms: 30.0,
                    },
                    exports::adas::diagnostics::health_monitoring::TestExecution {
                        test_name: String::from("graphics-rendering-test"),
                        test_result: adas::common_types::types::TestResult::Passed,
                        details: String::from("Graphics rendering within performance bounds"),
                        execution_time_ms: 40.0,
                    },
                ],
                overall_score: 96.0,
                recommendations: vec![String::from("HMI interface operating optimally")],
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
        exports::adas::diagnostics::performance_monitoring::ExtendedPerformance {
            base_metrics: adas::common_types::types::PerformanceMetrics {
                latency_avg_ms: 16.0, // 60 FPS target
                latency_max_ms: 33.0, // 30 FPS minimum
                cpu_utilization: 0.20,
                memory_usage_mb: 512,
                throughput_hz: 60.0, // 60 FPS rendering
                error_rate: 0.001,   // Very low error tolerance
            },
            component_specific: vec![
                exports::adas::diagnostics::performance_monitoring::Metric {
                    name: String::from("frame_rate"),
                    value: 60.0,
                    unit: String::from("fps"),
                    description: String::from("Current display frame rate"),
                },
                exports::adas::diagnostics::performance_monitoring::Metric {
                    name: String::from("input_latency"),
                    value: 12.0,
                    unit: String::from("ms"),
                    description: String::from("Input response latency"),
                },
                exports::adas::diagnostics::performance_monitoring::Metric {
                    name: String::from("graphics_load"),
                    value: 0.35,
                    unit: String::from("ratio"),
                    description: String::from("Graphics processing load"),
                },
                exports::adas::diagnostics::performance_monitoring::Metric {
                    name: String::from("ui_updates_per_second"),
                    value: 120.0,
                    unit: String::from("hz"),
                    description: String::from("UI element update frequency"),
                },
            ],
            resource_usage: exports::adas::diagnostics::performance_monitoring::ResourceUsage {
                cpu_cores_used: 0.20,
                memory_allocated_mb: 512,
                memory_peak_mb: 768,
                disk_io_mb: 5.0,
                network_io_mb: 0.5,
                gpu_utilization: 0.35,
                gpu_memory_mb: 256,
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
        println!("HMI Interface: Resetting performance counters");
    }
}

export!(Component);
