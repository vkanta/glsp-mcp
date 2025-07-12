// Radar Corner ECU Component Implementation

// The bindings are generated as a separate crate based on the BUILD target name
use radar_corner_ecu_bindings::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        format!("Radar Corner ECU - Frame processed")
    }
}

// Export the component using the generated macro with proper path
radar_corner_ecu_bindings::export!(Component with_types_in radar_corner_ecu_bindings);
