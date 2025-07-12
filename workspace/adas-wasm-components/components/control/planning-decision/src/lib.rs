// Planning Decision ECU Component Implementation

// The bindings are generated as a separate crate based on the BUILD target name
use planning_decision_ecu_bindings::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        format!("Planning Decision ECU - Frame processed")
    }
}

// Export the component using the generated macro with proper path
planning_decision_ecu_bindings::export!(Component with_types_in planning_decision_ecu_bindings);
