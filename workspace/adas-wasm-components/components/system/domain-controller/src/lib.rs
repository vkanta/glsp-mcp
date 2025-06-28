wit_bindgen::generate!({
    world: "adas-domain-controller-component",
    path: "../../../wit/worlds/adas-domain-controller.wit"
});

use crate::exports::system_management;
use crate::exports::controller_interface;

struct Component;

// Global state
static mut CONTROLLER_STATUS: controller_interface::ControllerStatus = controller_interface::ControllerStatus::Offline;
static mut CONTROLLER_CONFIG: Option<controller_interface::ControllerConfig> = None;

// Implement system-management interface (EXPORTED)
impl system_management::Guest for Component {
    fn get_system_health() -> system_management::SystemHealth {
        system_management::SystemHealth {
            overall_status: system_management::HealthStatus::Healthy,
            component_health: vec![
                system_management::ComponentHealth {
                    component: system_management::AdasComponent::SensorFusion,
                    status: system_management::HealthStatus::Healthy,
                    last_heartbeat: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    error_count: 0,
                    warning_count: 0,
                },
                system_management::ComponentHealth {
                    component: system_management::AdasComponent::VehicleControl,
                    status: system_management::HealthStatus::Healthy,
                    last_heartbeat: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    error_count: 0,
                    warning_count: 0,
                },
            ],
            system_integrity: 95.5,
            uptime: 86400000, // 24 hours in milliseconds
            last_diagnostic: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    fn get_component_statuses() -> Vec<system_management::ComponentStatus> {
        vec![
            system_management::ComponentStatus {
                component: system_management::AdasComponent::SensorFusion,
                status: system_management::HealthStatus::Healthy,
                cpu_usage: 12.5,
                memory_usage: 45.2,
                last_update: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                error_count: 0,
            },
            system_management::ComponentStatus {
                component: system_management::AdasComponent::ObjectDetection,
                status: system_management::HealthStatus::Healthy,
                cpu_usage: 28.3,
                memory_usage: 67.8,
                last_update: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                error_count: 0,
            },
        ]
    }

    fn get_system_metrics() -> system_management::SystemMetrics {
        system_management::SystemMetrics {
            total_cpu_usage: 35.7,
            total_memory_usage: 52.1,
            active_components: 8,
            data_throughput: 125.5,
            processing_latency: 12.3,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    fn get_active_features() -> Vec<system_management::AdasFeature> {
        vec![
            system_management::AdasFeature {
                feature_id: 1,
                name: "Automatic Emergency Braking".to_string(),
                feature_type: system_management::FeatureType::Safety,
                enabled: true,
                availability: system_management::FeatureAvailability::Available,
                dependencies: vec![
                    system_management::AdasComponent::ObjectDetection,
                    system_management::AdasComponent::VehicleControl,
                ],
            },
            system_management::AdasFeature {
                feature_id: 2,
                name: "Adaptive Cruise Control".to_string(),
                feature_type: system_management::FeatureType::Comfort,
                enabled: true,
                availability: system_management::FeatureAvailability::Available,
                dependencies: vec![
                    system_management::AdasComponent::TrackingPrediction,
                    system_management::AdasComponent::PlanningDecision,
                ],
            },
        ]
    }

    fn set_feature_state(feature: system_management::AdasFeature, enabled: bool) -> Result<(), String> {
        println!("Setting feature '{}' (ID: {}) to enabled: {}", feature.name, feature.feature_id, enabled);
        Ok(())
    }
}

// Implement controller-interface (EXPORTED)
impl controller_interface::Guest for Component {
    fn initialize(config: controller_interface::ControllerConfig) -> Result<(), String> {
        unsafe {
            CONTROLLER_CONFIG = Some(config);
            CONTROLLER_STATUS = controller_interface::ControllerStatus::Initializing;
        }
        Ok(())
    }

    fn start_system() -> Result<(), String> {
        unsafe {
            if CONTROLLER_CONFIG.is_some() {
                CONTROLLER_STATUS = controller_interface::ControllerStatus::Running;
                Ok(())
            } else {
                Err("Controller not initialized".to_string())
            }
        }
    }

    fn stop_system() -> Result<(), String> {
        unsafe {
            CONTROLLER_STATUS = controller_interface::ControllerStatus::Offline;
        }
        Ok(())
    }

    fn update_system(update_package: Vec<u8>) -> Result<controller_interface::UpdateResult, String> {
        println!("Processing system update with {} bytes", update_package.len());
        Ok(controller_interface::UpdateResult {
            success: true,
            updated_components: vec!["sensor-fusion".to_string(), "object-detection".to_string()],
            failed_components: vec![],
            rollback_available: true,
        })
    }

    fn run_diagnostics() -> Result<controller_interface::DiagnosticReport, String> {
        Ok(controller_interface::DiagnosticReport {
            system_health: controller_interface::HealthStatus::Healthy,
            component_diagnostics: vec![
                controller_interface::ComponentDiagnostic {
                    component: "sensor-fusion".to_string(),
                    status: controller_interface::HealthStatus::Healthy,
                    tests_passed: 25,
                    tests_failed: 0,
                    warnings: vec![],
                    errors: vec![],
                },
            ],
            performance_metrics: controller_interface::PerformanceMetrics {
                avg_response_time: 8.5,
                peak_cpu_usage: 45.2,
                peak_memory_usage: 67.8,
                throughput: 125.5,
                error_rate: 0.001,
            },
            recommendations: vec!["Consider reducing sensor polling rate to improve efficiency".to_string()],
        })
    }

    fn get_status() -> controller_interface::ControllerStatus {
        unsafe { CONTROLLER_STATUS.clone() }
    }

    fn get_config() -> controller_interface::ControllerConfig {
        unsafe {
            if let Some(config) = &CONTROLLER_CONFIG {
                config.clone()
            } else {
                // Return default config
                controller_interface::ControllerConfig {
                    system_mode: controller_interface::SystemMode::Normal,
                    diagnostic_interval: 60000, // 1 minute
                    monitoring_level: controller_interface::MonitoringLevel::Standard,
                    feature_config: controller_interface::FeatureConfig {
                        default_features: vec!["emergency-braking".to_string()],
                        safety_features: vec!["collision-avoidance".to_string()],
                        comfort_features: vec!["adaptive-cruise".to_string()],
                    },
                }
            }
        }
    }

    fn update_config(config: controller_interface::ControllerConfig) -> Result<(), String> {
        unsafe {
            CONTROLLER_CONFIG = Some(config);
        }
        Ok(())
    }
}

export!(Component);