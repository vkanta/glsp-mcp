use crate::exports::hello;

wit_bindgen::generate!({
    world: "simple",
    exports: {
        "hello": HelloComponent,
    },
});

pub struct HelloComponent;

impl hello::Guest for HelloComponent {
    fn hello(name: String) -> String {
        format!("Hello, {}!", name)
    }
}