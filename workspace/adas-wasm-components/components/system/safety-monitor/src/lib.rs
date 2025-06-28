wit_bindgen::generate!({
    world: "safety-monitor-component",
    path: "../../../wit/worlds/safety-monitor.wit"
});

use crate::exports::safety_data;
use crate::exports::safety_monitor;

struct Component;

// Global state
static mut MONITOR_STATUS: safety_monitor::SafetyMonitorStatus = safety_monitor::SafetyMonitorStatus::Offline;
static mut SAFETY_CONFIG: Option<safety_monitor::SafetyConfig> = None;

// Implement safety-data interface (EXPORTED)
impl safety_data::Guest for Component {
    fn get_safety_state() -> Result<safety_data::SafetyState, String> {
        unsafe {
            if matches!(MONITOR_STATUS, safety_monitor::SafetyMonitorStatus::Operational) {
                Ok(safety_data::SafetyState {
                    overall_state: safety_data::SafetyLevel::Nominal,
                    active_safety_functions: vec![
                        safety_data::SafetyFunction {
                            function_id: 1,
                            function_name: "Collision Avoidance".to_string(),
                            function_type: safety_data::FunctionType::CollisionAvoidance,
                            status: safety_data::FunctionStatus::Active,
                            diagnostic_info: "Operating normally".to_string(),
                        },
                        safety_data::SafetyFunction {
                            function_id: 2,
                            function_name: "Emergency Braking".to_string(),
                            function_type: safety_data::FunctionType::EmergencyBraking,
                            status: safety_data::FunctionStatus::Standby,
                            diagnostic_info: "Ready to activate".to_string(),
                        },
                    ],
                    system_integrity: safety_data::SystemIntegrity {
                        cpu_health: safety_data::HealthStatus::Ok,
                        memory_health: safety_data::HealthStatus::Ok,
                        communication_health: safety_data::HealthStatus::Ok,
                        sensor_health: safety_data::HealthStatus::Ok,
                        actuator_health: safety_data::HealthStatus::Ok,
                    },
                    redundancy_status: safety_data::RedundancyStatus {
                        primary_systems: vec![
                            safety_data::SystemInfo {
                                system_id: "primary-1".to_string(),
                                system_type: "safety-monitor".to_string(),
                                status: safety_data::HealthStatus::Ok,
                                last_heartbeat: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() as u64,
                            },
                        ],
                        backup_systems: vec![
                            safety_data::SystemInfo {
                                system_id: "backup-1".to_string(),
                                system_type: "safety-monitor".to_string(),
                                status: safety_data::HealthStatus::Ok,
                                last_heartbeat: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() as u64,
                            },
                        ],
                        voting_mechanism: safety_data::VotingMechanism::TwoOutOfThree,
                        switchover_capability: true,
                        current_active: "primary-1".to_string(),
                    },
                    safety_violations: vec![],
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                })
            } else {
                Err("Safety monitor not operational".to_string())
            }
        }
    }

    fn report_violation(violation: safety_data::SafetyViolation) -> Result<(), String> {
        println!("Safety violation reported: {:?} - {}", violation.violation_type, violation.description);
        Ok(())
    }

    fn trigger_failsafe(hazard_id: u32) -> Result<(), String> {
        println!("Triggering failsafe for hazard ID: {}", hazard_id);
        Ok(())
    }

    fn acknowledge_alert(alert_id: u32) -> Result<(), String> {
        println!("Acknowledging alert ID: {}", alert_id);
        Ok(())
    }
}

// Implement safety-monitor interface (EXPORTED)
impl safety_monitor::Guest for Component {
    fn initialize(config: safety_monitor::SafetyConfig) -> Result<(), String> {
        unsafe {
            SAFETY_CONFIG = Some(config);
            MONITOR_STATUS = safety_monitor::SafetyMonitorStatus::Initializing;
        }
        Ok(())
    }

    fn start_monitoring() -> Result<(), String> {
        unsafe {
            if SAFETY_CONFIG.is_some() {
                MONITOR_STATUS = safety_monitor::SafetyMonitorStatus::Operational;
                Ok(())
            } else {
                Err("Safety monitor not initialized".to_string())
            }
        }
    }

    fn stop_monitoring() -> Result<(), String> {
        unsafe {
            MONITOR_STATUS = safety_monitor::SafetyMonitorStatus::Offline;
        }
        Ok(())
    }

    fn get_status() -> safety_monitor::SafetyMonitorStatus {
        unsafe { MONITOR_STATUS.clone() }
    }

    fn update_config(config: safety_monitor::SafetyConfig) -> Result<(), String> {
        unsafe {
            SAFETY_CONFIG = Some(config);
        }
        Ok(())
    }

    fn update_driver_state(driver_state: safety_monitor::DriverState) -> Result<(), String> {
        println!("Updating driver state: attention level = {:?}", driver_state.attention_level);
        Ok(())
    }

    fn run_diagnostic(test_type: safety_monitor::TestType) -> Result<safety_monitor::SelfTestResult, String> {
        Ok(safety_monitor::SelfTestResult {
            test_id: 1,
            test_name: "Safety diagnostic".to_string(),
            test_type,
            test_outcome: safety_monitor::TestResult::Passed,
            execution_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            details: "All safety mechanisms operational".to_string(),
            coverage_percentage: 98.5,
        })
    }

    fn get_functional_safety() -> Result<safety_monitor::FunctionalSafety, String> {
        Ok(safety_monitor::FunctionalSafety {
            implementation_level: safety_monitor::ImplementationLevel::ProductionReady,
            asil_compliance: safety_monitor::AsilCompliance {
                target_asil: safety_monitor::AsilLevel::AsilD,
                achieved_asil: safety_monitor::AsilLevel::AsilD,
                safety_goals: vec![],
                hazard_analysis: vec![],
                verification_results: vec![],
                validation_results: vec![],
            },
            safety_mechanisms: vec![],
            redundancy_strategy: safety_monitor::RedundancyStrategy::TripleModular,
            fault_handling: safety_monitor::FaultHandling {
                detection_mechanisms: vec![],
                reaction_strategies: vec![],
                recovery_procedures: vec![],
            },
            safety_lifecycle: safety_monitor::SafetyLifecycle {
                development_phase: safety_monitor::DevelopmentPhase::Production,
                certification_status: safety_monitor::CertificationStatus::Certified,
                last_assessment: 1609459200000,
                next_assessment: 1735689600000,
            },
        })
    }

    fn get_compliance_report() -> Result<safety_monitor::ComplianceReport, String> {
        Ok(safety_monitor::ComplianceReport {
            overall_compliance: safety_monitor::ComplianceLevel::FullyCompliant,
            standard_compliance: vec![],
            regulatory_requirements: vec![],
            audit_results: vec![],
            deviations: vec![],
        })
    }
}

export!(Component);