// Video AI Pipeline ECU Component Implementation

// The bindings are generated as a separate crate based on the BUILD target name
use video_ai_pipeline_ecu_bindings::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        format!("Video AI Pipeline ECU - Frame processed")
    }
}

// Export the component using the generated macro with proper path
video_ai_pipeline_ecu_bindings::export!(Component with_types_in video_ai_pipeline_ecu_bindings);
