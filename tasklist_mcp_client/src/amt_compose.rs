use std::collections::HashMap;

pub mod codegen_core {
    pub trait CanonicalNameExt {}
}

pub mod wit_bindgen_core {
    pub mod wit_parser {
        use std::collections::HashMap;

        #[derive(Default)]
        pub struct Package {
            pub name: String,
            pub interfaces: HashMap<String, usize>,
        }

        #[derive(Default)]
        pub struct Interface {
            pub functions: Vec<(String, usize)>,
        }

        #[derive(Default)]
        pub struct Resolve {
            pub packages: HashMap<String, Package>,
            pub interfaces: Vec<Interface>,
        }

        impl Resolve {
            pub fn interface_canon_by_id(&self, id: usize) -> Option<String> {
                // Return a placeholder name for compilation/runtime use
                Some(format!("interface_{}", id))
            }
        }
    }
}

pub mod core {
    pub struct LogTracker;
    impl LogTracker {
        pub fn new_boxed() -> Box<Self> {
            Box::new(LogTracker)
        }
    }
}

pub mod project {
    use super::wit_bindgen_core::wit_parser::Resolve;
    use anyhow::Result;

    pub struct ProjectContext;

    pub struct WitHandle;
    impl WitHandle {
        pub fn resolve(&self) -> Resolve {
            Resolve::default()
        }
    }

    impl ProjectContext {
        pub fn new(_config: &str, _project: &str, _quiet: bool, _logger: Box<crate::amt_compose::core::LogTracker>) -> Result<Self> {
            Ok(ProjectContext)
        }

        pub fn wit(&self) -> WitHandle {
            WitHandle
        }
    }
}
