// Camera Front ECU Component Implementation
use component_metadata::embed_metadata;

// Embed metadata into the WASM binary
embed_metadata!();

wit_bindgen::generate!();

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        // Get component metadata to demonstrate it's available
        let metadata = get_component_metadata().unwrap_or_else(|_| {
            panic!("Failed to load component metadata");
        });
        
        format!("Camera Front ECU v{} - Frame processed", metadata.version)
    }
}

export!(Component);