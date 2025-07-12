// Tracking Prediction ECU Component Implementation

// The bindings are generated as a separate crate based on the BUILD target name
use tracking_prediction_ecu_bindings::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        format!("Tracking Prediction ECU - Frame processed")
    }
}

// Export the component using the generated macro with proper path
tracking_prediction_ecu_bindings::export!(Component with_types_in tracking_prediction_ecu_bindings);
