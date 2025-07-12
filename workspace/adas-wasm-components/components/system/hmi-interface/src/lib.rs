// HMI Interface ECU Component Implementation

// The bindings are generated as a separate crate based on the BUILD target name
use hmi_interface_ecu_bindings::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        format!("HMI Interface ECU - Frame processed")
    }
}

// Export the component using the generated macro with proper path
hmi_interface_ecu_bindings::export!(Component with_types_in hmi_interface_ecu_bindings);
