wit_bindgen::generate!({
    world: "can-gateway-component",
    path: "../../../wit/worlds/can-gateway.wit"
});

use crate::exports::can_interface;
use crate::exports::gateway_control;

struct Component;

// Global state
static mut GATEWAY_STATUS: gateway_control::GatewayStatus = gateway_control::GatewayStatus::Offline;
static mut GATEWAY_CONFIG: Option<gateway_control::GatewayConfig> = None;

// Implement can-interface (EXPORTED)
impl can_interface::Guest for Component {
    fn send_message(message: can_interface::CanMessage) -> Result<(), String> {
        println!("Sending CAN message ID: 0x{:X} on bus {}", message.message_id, message.bus_id);
        Ok(())
    }

    fn send_batch(messages: Vec<can_interface::CanMessage>) -> Result<(), String> {
        println!("Sending batch of {} CAN messages", messages.len());
        Ok(())
    }

    fn get_messages(timeout_ms: Option<u32>) -> Result<Vec<can_interface::CanMessage>, String> {
        unsafe {
            if matches!(GATEWAY_STATUS, gateway_control::GatewayStatus::Active) {
                // Return simulated received messages
                Ok(vec![
                    can_interface::CanMessage {
                        bus_id: 1,
                        message_id: 0x123,
                        data: vec![0x01, 0x02, 0x03, 0x04],
                        is_extended: false,
                        is_remote_frame: false,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                        priority: can_interface::MessagePriority::Normal,
                    }
                ])
            } else {
                Err("Gateway not active".to_string())
            }
        }
    }

    fn get_bus_status(bus_id: u32) -> Result<can_interface::BusStatus, String> {
        Ok(can_interface::BusStatus {
            bus_id,
            state: can_interface::BusState::Active,
            error_count: 0,
            bus_load: 25.5,
            bit_rate: 500000,
        })
    }
}

// Implement gateway-control (EXPORTED)
impl gateway_control::Guest for Component {
    fn initialize(config: gateway_control::GatewayConfig) -> Result<(), String> {
        unsafe {
            GATEWAY_CONFIG = Some(config);
            GATEWAY_STATUS = gateway_control::GatewayStatus::Initializing;
        }
        Ok(())
    }

    fn start_gateway() -> Result<(), String> {
        unsafe {
            if GATEWAY_CONFIG.is_some() {
                GATEWAY_STATUS = gateway_control::GatewayStatus::Active;
                Ok(())
            } else {
                Err("Gateway not initialized".to_string())
            }
        }
    }

    fn stop_gateway() -> Result<(), String> {
        unsafe {
            GATEWAY_STATUS = gateway_control::GatewayStatus::Offline;
        }
        Ok(())
    }

    fn update_config(config: gateway_control::GatewayConfig) -> Result<(), String> {
        unsafe {
            GATEWAY_CONFIG = Some(config);
        }
        Ok(())
    }

    fn get_status() -> gateway_control::GatewayStatus {
        unsafe { GATEWAY_STATUS.clone() }
    }

    fn get_statistics() -> gateway_control::GatewayStatistics {
        gateway_control::GatewayStatistics {
            messages_sent: 12345,
            messages_received: 23456,
            messages_filtered: 1234,
            messages_routed: 11111,
            errors: 2,
            uptime_seconds: 86400,
            bus_statistics: vec![
                gateway_control::BusStatistics {
                    bus_id: 1,
                    tx_count: 5000,
                    rx_count: 7500,
                    error_count: 1,
                    bus_load_avg: 25.5,
                    bus_load_peak: 45.2,
                },
            ],
        }
    }
}

export!(Component);