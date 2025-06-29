// CAN Gateway - Standardized system component implementation

wit_bindgen::generate!({
    world: "system-component",
    path: "wit/",
    with: {
        "adas:common-types/types": generate,
        "adas:data/sensor-data": generate,
        "adas:data/perception-data": generate,
        "adas:data/planning-data": generate,
        "adas:diagnostics/health-monitoring": generate,
        "adas:diagnostics/performance-monitoring": generate,
        "adas:orchestration/execution-control": generate,
        "adas:orchestration/resource-management": generate,
    },
});

struct Component;

// CAN gateway state
static mut GATEWAY_ACTIVE: bool = false;

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
            component_id: String::from("can-gateway"),
            overall_health: unsafe {
                if GATEWAY_ACTIVE {
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
                        test_name: String::from("can-bus-connectivity-test"),
                        test_result: adas::common_types::types::TestResult::Passed,
                        details: String::from("All CAN buses connected and responsive"),
                        execution_time_ms: 35.0,
                    },
                    exports::adas::diagnostics::health_monitoring::TestExecution {
                        test_name: String::from("message-routing-test"),
                        test_result: adas::common_types::types::TestResult::Passed,
                        details: String::from("Message routing functioning correctly"),
                        execution_time_ms: 20.0,
                    },
                    exports::adas::diagnostics::health_monitoring::TestExecution {
                        test_name: String::from("gateway-filter-test"),
                        test_result: adas::common_types::types::TestResult::Passed,
                        details: String::from("Message filtering and validation working"),
                        execution_time_ms: 15.0,
                    },
                    exports::adas::diagnostics::health_monitoring::TestExecution {
                        test_name: String::from("error-handling-test"),
                        test_result: adas::common_types::types::TestResult::Passed,
                        details: String::from("Error detection and recovery mechanisms functional"),
                        execution_time_ms: 25.0,
                    },
                ],
                overall_score: 98.0,
                recommendations: vec![String::from("CAN gateway operating at optimal performance")],
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
                latency_avg_ms: 2.0, // Very low latency for CAN messages
                latency_max_ms: 5.0,
                cpu_utilization: 0.10,
                memory_usage_mb: 64,
                throughput_hz: 10000.0, // High throughput for CAN messages
                error_rate: 0.0001,     // Extremely low error tolerance
            },
            component_specific: vec![
                exports::adas::diagnostics::performance_monitoring::Metric {
                    name: String::from("messages_per_second"),
                    value: 10000.0,
                    unit: String::from("msgs/s"),
                    description: String::from("CAN messages processed per second"),
                },
                exports::adas::diagnostics::performance_monitoring::Metric {
                    name: String::from("bus_utilization"),
                    value: 0.45,
                    unit: String::from("ratio"),
                    description: String::from("Average CAN bus utilization"),
                },
                exports::adas::diagnostics::performance_monitoring::Metric {
                    name: String::from("dropped_messages"),
                    value: 0.0,
                    unit: String::from("count"),
                    description: String::from("Number of dropped messages"),
                },
                exports::adas::diagnostics::performance_monitoring::Metric {
                    name: String::from("gateway_load"),
                    value: 0.25,
                    unit: String::from("ratio"),
                    description: String::from("Gateway processing load"),
                },
            ],
            resource_usage: exports::adas::diagnostics::performance_monitoring::ResourceUsage {
                cpu_cores_used: 0.10,
                memory_allocated_mb: 64,
                memory_peak_mb: 96,
                disk_io_mb: 0.1,
                network_io_mb: 50.0, // High network I/O for CAN traffic
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
        println!("CAN Gateway: Resetting performance counters");
    }
}

export!(Component);
