// Domain Controller ECU Component Implementation

// The bindings are generated as a separate crate based on the BUILD target name
use domain_controller_ecu_bindings::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        format!("Domain Controller ECU - Frame processed")
    }
}

// Export the component using the generated macro with proper path
domain_controller_ecu_bindings::export!(Component with_types_in domain_controller_ecu_bindings);
