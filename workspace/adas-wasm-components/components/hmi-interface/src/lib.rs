wit_bindgen::generate!({
    world: "hmi-interface-component",
    path: "../../wit/worlds/hmi-interface-simple.wit"
});

use crate::exports::hmi_display;
use crate::exports::user_input;

struct Component;

// Global state
static mut INTERFACE_STATUS: hmi_display::InterfaceStatus = hmi_display::InterfaceStatus::Offline;
static mut DISPLAY_CONFIGS: Vec<hmi_display::DisplayConfig> = Vec::new();

// Implement hmi-display interface (EXPORTED)
impl hmi_display::Guest for Component {
    fn initialize(displays: Vec<hmi_display::DisplayConfig>) -> Result<(), String> {
        unsafe {
            DISPLAY_CONFIGS = displays;
            INTERFACE_STATUS = hmi_display::InterfaceStatus::Initializing;
            INTERFACE_STATUS = hmi_display::InterfaceStatus::Active;
        }
        Ok(())
    }

    fn update_display(_display_id: u32, _content: hmi_display::AdasDisplay) -> Result<(), String> {
        println!("Updating display content");
        Ok(())
    }

    fn show_warning(warning: hmi_display::WarningDisplay) -> Result<u32, String> {
        println!("Showing warning: {:?} - {}", warning.warning_type, warning.message);
        Ok(warning.warning_id)
    }

    fn hide_warning(warning_id: u32) -> Result<(), String> {
        println!("Hiding warning ID: {}", warning_id);
        Ok(())
    }

    fn get_status() -> hmi_display::InterfaceStatus {
        unsafe { INTERFACE_STATUS.clone() }
    }
}

// Implement user-input interface (EXPORTED)
impl user_input::Guest for Component {
    fn process_input(input: user_input::UserAction) -> Result<(), String> {
        match input.action_type {
            user_input::ActionType::ButtonPress => println!("Button press received"),
            user_input::ActionType::Touch => println!("Touch input received"),
            user_input::ActionType::Gesture => println!("Gesture received"),
            user_input::ActionType::VoiceCommand => println!("Voice command received"),
        }
        Ok(())
    }

    fn enable_input(input_type: user_input::ActionType) -> Result<(), String> {
        println!("Enabling input type: {:?}", input_type);
        Ok(())
    }

    fn disable_input(input_type: user_input::ActionType) -> Result<(), String> {
        println!("Disabling input type: {:?}", input_type);
        Ok(())
    }
}

export!(Component);