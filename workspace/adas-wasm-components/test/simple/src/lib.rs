// The bindings are generated as a separate crate based on the BUILD target name
use simple_component_bindings::Guest;

struct Component;

impl Guest for Component {
    fn hello(name: String) -> String {
        format!("Hello, {}!", name)
    }
}

// Export the component using the generated macro with proper path
simple_component_bindings::export!(Component with_types_in simple_component_bindings);