// CAN Gateway ECU Component Implementation

// The bindings are generated as a separate crate based on the BUILD target name
use can_gateway_ecu_bindings::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        format!("CAN Gateway ECU - Frame processed")
    }
}

// Export the component using the generated macro with proper path
can_gateway_ecu_bindings::export!(Component with_types_in can_gateway_ecu_bindings);
