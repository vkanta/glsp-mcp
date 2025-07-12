// Behavior Prediction AI Component Implementation

// The bindings are generated as a separate crate based on the BUILD target name
use behavior_prediction_ai_bindings::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        format!("Behavior Prediction AI - Frame processed")
    }
}

// Export the component using the generated macro with proper path
behavior_prediction_ai_bindings::export!(Component with_types_in behavior_prediction_ai_bindings);
