// Camera Front ECU Component Implementation
// Using the new rust_wasm_component_bindgen API

// The bindings are generated as a separate crate based on the BUILD target name
use camera_front_ecu_bindings::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        // For now, return a simple string until we add metadata support
        format!("Camera Front ECU - Frame processed")
    }
}

// Export the component using the generated macro
camera_front_ecu_bindings::export!(Component);