// Object Detection AI Component Implementation

// The bindings are generated as a separate crate based on the BUILD target name
use object_detection_ai_bindings::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        format!("Object Detection AI - Frame processed")
    }
}

// Export the component using the generated macro with proper path
object_detection_ai_bindings::export!(Component with_types_in object_detection_ai_bindings);
