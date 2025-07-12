// Safety Monitor ECU Component Implementation

// The bindings are generated as a separate crate based on the BUILD target name
use safety_monitor_ecu_bindings::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        format!("Safety Monitor ECU - Frame processed")
    }
}

// Export the component using the generated macro with proper path
safety_monitor_ecu_bindings::export!(Component with_types_in safety_monitor_ecu_bindings);
