// Vehicle Control ECU Component Implementation

// The bindings are generated as a separate crate based on the BUILD target name
use vehicle_control_ecu_bindings::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        format!("Vehicle Control ECU - Frame processed")
    }
}

// Export the component using the generated macro with proper path
vehicle_control_ecu_bindings::export!(Component with_types_in vehicle_control_ecu_bindings);
