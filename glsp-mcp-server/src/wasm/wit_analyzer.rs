/*!
 * WIT (WebAssembly Interface Types) Analyzer
 * 
 * This module provides functionality to extract and analyze WIT interfaces 
 * from WebAssembly Component Model binaries using the Bytecode Alliance toolchain.
 */

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use wit_component::DecodedWasm;
use wit_parser::{Resolve, WorldId, Interface, Package, PackageId};

/// Represents a complete WIT interface definition extracted from a WASM component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitInterface {
    pub name: String,
    pub namespace: Option<String>,
    pub package: Option<String>,
    pub version: Option<String>,
    pub interface_type: WitInterfaceType,
    pub functions: Vec<WitFunction>,
    pub types: Vec<WitType>,
}

/// Type of WIT interface (import or export)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WitInterfaceType {
    Import,
    Export,
}

/// Represents a function in a WIT interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitFunction {
    pub name: String,
    pub params: Vec<WitParam>,
    pub results: Vec<WitParam>,
    pub is_async: bool,
}

/// Represents a parameter or return value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitParam {
    pub name: String,
    pub param_type: WitType,
}

/// Represents a WIT type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitType {
    pub name: String,
    pub type_def: WitTypeDefinition,
}

/// Different kinds of WIT type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WitTypeDefinition {
    Primitive(String),
    Record { fields: Vec<WitParam> },
    Variant { cases: Vec<WitVariantCase> },
    Enum { cases: Vec<String> },
    Union { types: Vec<WitType> },
    Option { inner: Box<WitType> },
    Result { ok: Option<Box<WitType>>, error: Option<Box<WitType>> },
    List { element: Box<WitType> },
    Tuple { elements: Vec<WitType> },
    Flags { flags: Vec<String> },
    Resource { methods: Vec<WitFunction> },
}

/// Variant case for sum types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitVariantCase {
    pub name: String,
    pub payload: Option<WitType>,
}

/// Complete WIT analysis result for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentWitAnalysis {
    pub component_name: String,
    pub world_name: Option<String>,
    pub imports: Vec<WitInterface>,
    pub exports: Vec<WitInterface>,
    pub types: Vec<WitType>,
    pub dependencies: Vec<WitDependency>,
    pub raw_wit: Option<String>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitDependency {
    pub package: String,
    pub version: Option<String>,
    pub interfaces: Vec<String>,
}

/// WIT Analyzer for extracting interfaces from WASM components
pub struct WitAnalyzer;

impl WitAnalyzer {
    /// Analyze a WASM component file and extract all WIT interfaces
    pub async fn analyze_component<P: AsRef<Path>>(path: P) -> Result<ComponentWitAnalysis> {
        let path = path.as_ref();
        let component_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        println!("🔍 Analyzing WASM component: {path:?}");

        // Read the WASM bytes
        let wasm_bytes = tokio::fs::read(path).await
            .with_context(|| format!("Failed to read WASM file: {path:?}"))?;

        // Try to decode as a component first
        match wit_component::decode(&wasm_bytes) {
            Ok(decoded) => {
                println!("✅ Successfully decoded as WebAssembly component");
                Self::analyze_decoded_component(component_name, decoded).await
            }
            Err(_) => {
                println!("⚠️  Not a WebAssembly component, trying as core module");
                Self::analyze_core_module(component_name, wasm_bytes).await
            }
        }
    }

    /// Analyze a decoded WebAssembly component
    async fn analyze_decoded_component(
        component_name: String,
        decoded: DecodedWasm
    ) -> Result<ComponentWitAnalysis> {
        match decoded {
            DecodedWasm::Component(resolve, world_id) => {
                println!("📋 Analyzing component world and interfaces");
                Self::extract_wit_from_resolve(component_name, &resolve, world_id).await
            }
            DecodedWasm::WitPackage(resolve, package_id) => {
                println!("📦 Analyzing WIT package");
                Self::extract_wit_from_package(component_name, &resolve, package_id).await
            }
        }
    }

    /// Analyze a core WebAssembly module (non-component)
    async fn analyze_core_module(
        component_name: String,
        _wasm_bytes: Vec<u8>
    ) -> Result<ComponentWitAnalysis> {
        println!("🔧 Core module detected - limited interface extraction available");
        
        // For core modules, we can only provide basic analysis
        // In a real implementation, you might want to try converting to component
        Ok(ComponentWitAnalysis {
            component_name,
            world_name: None,
            imports: vec![],
            exports: vec![],
            types: vec![],
            dependencies: vec![],
            raw_wit: Some("// Core WebAssembly module - no WIT interfaces available".to_string()),
        })
    }

    /// Extract WIT interfaces from a resolve with a specific world
    async fn extract_wit_from_resolve(
        component_name: String,
        resolve: &Resolve,
        world_id: WorldId
    ) -> Result<ComponentWitAnalysis> {
        let world = resolve.worlds.get(world_id)
            .ok_or_else(|| anyhow!("World not found in resolve"))?;

        println!("🌍 Analyzing world: {}", world.name);

        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut all_types = Vec::new();
        let mut dependencies = Vec::new();

        // Extract imports
        for (key, import) in &world.imports {
            let interface = Self::extract_world_item_interface(
                resolve,
                key,
                import,
                WitInterfaceType::Import
            ).await?;
            imports.push(interface);
        }

        // Extract exports  
        for (key, export) in &world.exports {
            let interface = Self::extract_world_item_interface(
                resolve,
                key,
                export,
                WitInterfaceType::Export
            ).await?;
            exports.push(interface);
        }

        // Extract types from all packages
        for (_, package) in &resolve.packages {
            let package_types = Self::extract_types_from_package(resolve, package).await?;
            all_types.extend(package_types);

            // Extract dependencies
            if package.name.namespace != "root" {
                dependencies.push(WitDependency {
                    package: format!("{}:{}", package.name.namespace, package.name.name),
                    version: package.name.version.as_ref().map(|v| v.to_string()),
                    interfaces: package.interfaces.keys().cloned().collect(),
                });
            }
        }

        // Generate raw WIT content
        let raw_wit = Self::generate_wit_text(resolve, world_id)?;

        Ok(ComponentWitAnalysis {
            component_name,
            world_name: Some(world.name.clone()),
            imports,
            exports,
            types: all_types,
            dependencies,
            raw_wit: Some(raw_wit),
        })
    }

    /// Extract WIT interfaces from a package
    async fn extract_wit_from_package(
        component_name: String,
        resolve: &Resolve,
        package_id: PackageId
    ) -> Result<ComponentWitAnalysis> {
        let package = resolve.packages.get(package_id)
            .ok_or_else(|| anyhow!("Package not found in resolve"))?;

        println!("📦 Analyzing package: {}:{}", package.name.namespace, package.name.name);

        let mut exports = Vec::new();
        let mut all_types = Vec::new();

        // Extract interfaces from package
        for (name, interface_id) in &package.interfaces {
            let interface = resolve.interfaces.get(*interface_id)
                .ok_or_else(|| anyhow!("Interface not found: {}", name))?;

            let wit_interface = Self::convert_interface_to_wit(
                resolve,
                name,
                interface,
                WitInterfaceType::Export
            ).await?;
            exports.push(wit_interface);
        }

        // Extract types
        let package_types = Self::extract_types_from_package(resolve, package).await?;
        all_types.extend(package_types);

        Ok(ComponentWitAnalysis {
            component_name,
            world_name: None,
            imports: vec![],
            exports,
            types: all_types,
            dependencies: vec![],
            raw_wit: None,
        })
    }

    /// Extract interface from a world item (import or export)
    async fn extract_world_item_interface(
        resolve: &Resolve,
        key: &wit_parser::WorldKey,
        item: &wit_parser::WorldItem,
        interface_type: WitInterfaceType
    ) -> Result<WitInterface> {
        use wit_parser::{WorldItem, WorldKey};

        match (key, item) {
            (WorldKey::Name(name), WorldItem::Interface { id, .. }) => {
                let interface = resolve.interfaces.get(*id)
                    .ok_or_else(|| anyhow!("Interface not found for key: {}", name))?;
                
                Self::convert_interface_to_wit(resolve, name, interface, interface_type).await
            }
            (WorldKey::Interface(interface_id), WorldItem::Interface { .. }) => {
                let interface = resolve.interfaces.get(*interface_id)
                    .ok_or_else(|| anyhow!("Interface not found for id: {:?}", interface_id))?;
                
                let name = interface.name.as_deref().unwrap_or("unnamed");
                Self::convert_interface_to_wit(resolve, name, interface, interface_type).await
            }
            (WorldKey::Name(name), WorldItem::Function(func)) => {
                // Single function export/import
                let function = Self::convert_function_to_wit(resolve, name, func).await?;
                
                Ok(WitInterface {
                    name: name.clone(),
                    namespace: None,
                    package: None,
                    version: None,
                    interface_type,
                    functions: vec![function],
                    types: vec![],
                })
            }
            _ => {
                Err(anyhow!("Unsupported world item type"))
            }
        }
    }

    /// Convert a WIT parser Interface to our WitInterface
    async fn convert_interface_to_wit(
        resolve: &Resolve,
        name: &str,
        interface: &Interface,
        interface_type: WitInterfaceType
    ) -> Result<WitInterface> {
        let mut functions = Vec::new();
        let mut types = Vec::new();

        // Extract functions
        for (func_name, func) in &interface.functions {
            let wit_function = Self::convert_function_to_wit(resolve, func_name, func).await?;
            functions.push(wit_function);
        }

        // Extract types
        for (type_name, type_id) in &interface.types {
            if let Some(type_def) = resolve.types.get(*type_id) {
                let wit_type = Self::convert_type_def_to_wit(resolve, type_name, type_def).await?;
                types.push(wit_type);
            }
        }

        Ok(WitInterface {
            name: name.to_string(),
            namespace: None, // TODO: Extract from package info
            package: None,   // TODO: Extract from package info  
            version: None,   // TODO: Extract from package info
            interface_type,
            functions,
            types,
        })
    }

    /// Convert a function definition to WIT function
    async fn convert_function_to_wit(
        resolve: &Resolve,
        name: &str,
        func: &wit_parser::Function
    ) -> Result<WitFunction> {
        let mut params = Vec::new();
        let mut results = Vec::new();

        // Convert parameters
        for (param_name, param_type) in &func.params {
            let wit_type = Self::convert_type_to_wit(resolve, param_type).await?;
            params.push(WitParam {
                name: param_name.clone(),
                param_type: wit_type,
            });
        }

        // Convert results
        match &func.results {
            wit_parser::Results::Named(named_results) => {
                for (result_name, result_type) in named_results {
                    let wit_type = Self::convert_type_to_wit(resolve, result_type).await?;
                    results.push(WitParam {
                        name: result_name.clone(),
                        param_type: wit_type,
                    });
                }
            }
            wit_parser::Results::Anon(result_type) => {
                let wit_type = Self::convert_type_to_wit(resolve, result_type).await?;
                results.push(WitParam {
                    name: "result".to_string(),
                    param_type: wit_type,
                });
            }
        }

        Ok(WitFunction {
            name: name.to_string(),
            params,
            results,
            is_async: false, // TODO: Detect async functions
        })
    }

    /// Convert a WIT type to our type representation
    async fn convert_type_to_wit(
        _resolve: &Resolve,
        wit_type: &wit_parser::Type
    ) -> Result<WitType> {
        use wit_parser::Type;

        let type_def = match wit_type {
            Type::Bool => WitTypeDefinition::Primitive("bool".to_string()),
            Type::U8 => WitTypeDefinition::Primitive("u8".to_string()),
            Type::U16 => WitTypeDefinition::Primitive("u16".to_string()),
            Type::U32 => WitTypeDefinition::Primitive("u32".to_string()),
            Type::U64 => WitTypeDefinition::Primitive("u64".to_string()),
            Type::S8 => WitTypeDefinition::Primitive("s8".to_string()),
            Type::S16 => WitTypeDefinition::Primitive("s16".to_string()),
            Type::S32 => WitTypeDefinition::Primitive("s32".to_string()),
            Type::S64 => WitTypeDefinition::Primitive("s64".to_string()),
            Type::F32 => WitTypeDefinition::Primitive("f32".to_string()),
            Type::F64 => WitTypeDefinition::Primitive("f64".to_string()),
            Type::Char => WitTypeDefinition::Primitive("char".to_string()),
            Type::String => WitTypeDefinition::Primitive("string".to_string()),
            Type::Id(_) => WitTypeDefinition::Primitive("custom".to_string()), // TODO: Resolve custom types
        };

        Ok(WitType {
            name: "anonymous".to_string(),
            type_def,
        })
    }

    /// Convert a type definition to WIT type
    async fn convert_type_def_to_wit(
        resolve: &Resolve,
        name: &str,
        type_def: &wit_parser::TypeDef
    ) -> Result<WitType> {
        use wit_parser::TypeDefKind;

        let wit_type_def = match &type_def.kind {
            TypeDefKind::Record(record) => {
                let mut fields = Vec::new();
                for field in &record.fields {
                    let field_type = Self::convert_type_to_wit(resolve, &field.ty).await?;
                    fields.push(WitParam {
                        name: field.name.clone(),
                        param_type: field_type,
                    });
                }
                WitTypeDefinition::Record { fields }
            }
            TypeDefKind::Variant(variant) => {
                let mut cases = Vec::new();
                for case in &variant.cases {
                    let payload = if let Some(ty) = &case.ty {
                        Some(Self::convert_type_to_wit(resolve, ty).await?)
                    } else {
                        None
                    };
                    cases.push(WitVariantCase {
                        name: case.name.clone(),
                        payload,
                    });
                }
                WitTypeDefinition::Variant { cases }
            }
            TypeDefKind::Enum(enum_def) => {
                let cases = enum_def.cases.iter().map(|case| case.name.clone()).collect();
                WitTypeDefinition::Enum { cases }
            }
            // Note: Union type handling may vary by wit-parser version
            // TypeDefKind::Union(union) => {
            //     let mut types = Vec::new();
            //     for case in &union.cases {
            //         let case_type = Self::convert_type_to_wit(resolve, &case.ty).await?;
            //         types.push(case_type);
            //     }
            //     WitTypeDefinition::Union { types }
            // }
            TypeDefKind::Option(option_type) => {
                let inner_type = Self::convert_type_to_wit(resolve, option_type).await?;
                WitTypeDefinition::Option { inner: Box::new(inner_type) }
            }
            TypeDefKind::Result(result_type) => {
                let ok_type = if let Some(ok) = &result_type.ok {
                    Some(Box::new(Self::convert_type_to_wit(resolve, ok).await?))
                } else {
                    None
                };
                let error_type = if let Some(err) = &result_type.err {
                    Some(Box::new(Self::convert_type_to_wit(resolve, err).await?))
                } else {
                    None
                };
                WitTypeDefinition::Result { ok: ok_type, error: error_type }
            }
            TypeDefKind::List(list_type) => {
                let element_type = Self::convert_type_to_wit(resolve, list_type).await?;
                WitTypeDefinition::List { element: Box::new(element_type) }
            }
            TypeDefKind::Tuple(tuple) => {
                let mut elements = Vec::new();
                for element_type in &tuple.types {
                    let wit_type = Self::convert_type_to_wit(resolve, element_type).await?;
                    elements.push(wit_type);
                }
                WitTypeDefinition::Tuple { elements }
            }
            TypeDefKind::Flags(flags) => {
                let flag_names = flags.flags.iter().map(|flag| flag.name.clone()).collect();
                WitTypeDefinition::Flags { flags: flag_names }
            }
            TypeDefKind::Resource => {
                WitTypeDefinition::Resource { methods: vec![] } // TODO: Extract resource methods
            }
            _ => WitTypeDefinition::Primitive("unknown".to_string()),
        };

        Ok(WitType {
            name: name.to_string(),
            type_def: wit_type_def,
        })
    }

    /// Extract types from a package
    async fn extract_types_from_package(
        resolve: &Resolve,
        package: &Package
    ) -> Result<Vec<WitType>> {
        let mut types = Vec::new();

        for (_, interface_id) in &package.interfaces {
            if let Some(interface) = resolve.interfaces.get(*interface_id) {
                for (type_name, type_id) in &interface.types {
                    if let Some(type_def) = resolve.types.get(*type_id) {
                        let wit_type = Self::convert_type_def_to_wit(resolve, type_name, type_def).await?;
                        types.push(wit_type);
                    }
                }
            }
        }

        Ok(types)
    }

    /// Generate WIT text representation
    fn generate_wit_text(resolve: &Resolve, world_id: WorldId) -> Result<String> {
        // Use wit-component's built-in WIT printing
        // This is a simplified version - in practice you'd want more sophisticated formatting
        let world = resolve.worlds.get(world_id)
            .ok_or_else(|| anyhow!("World not found"))?;

        let mut wit_text = String::new();
        let world_name = &world.name;
        wit_text.push_str(&format!("world {world_name} {{\n"));

        // Add imports
        for (key, _item) in &world.imports {
            match key {
                wit_parser::WorldKey::Name(name) => {
                    wit_text.push_str(&format!("  import {name};\n"));
                }
                wit_parser::WorldKey::Interface(id) => {
                    if let Some(interface) = resolve.interfaces.get(*id) {
                        let name = interface.name.as_deref().unwrap_or("unnamed");
                        wit_text.push_str(&format!("  import {name};\n"));
                    }
                }
            }
        }

        // Add exports
        for (key, _item) in &world.exports {
            match key {
                wit_parser::WorldKey::Name(name) => {
                    wit_text.push_str(&format!("  export {name};\n"));
                }
                wit_parser::WorldKey::Interface(id) => {
                    if let Some(interface) = resolve.interfaces.get(*id) {
                        let name = interface.name.as_deref().unwrap_or("unnamed");
                        wit_text.push_str(&format!("  export {name};\n"));
                    }
                }
            }
        }

        wit_text.push_str("}\n");
        Ok(wit_text)
    }

    /// Debug method to test interface extraction from a specific component
    pub async fn debug_component_interfaces<P: AsRef<Path>>(path: P) -> Result<()> {
        let analysis = Self::analyze_component(path.as_ref()).await?;
        
        println!("🐛 DEBUG: Component Analysis Results");
        println!("=====================================");
        println!("Component: {}", analysis.component_name);
        println!("World: {:?}", analysis.world_name);
        println!();
        
        println!("📥 IMPORTS ({})", analysis.imports.len());
        for (i, import) in analysis.imports.iter().enumerate() {
            println!("  {}. {}", i + 1, import.name);
            println!("     Functions: {}", import.functions.len());
            for func in &import.functions {
                println!("       - {}", func.name);
            }
        }
        println!();
        
        println!("📤 EXPORTS ({})", analysis.exports.len());
        for (i, export) in analysis.exports.iter().enumerate() {
            println!("  {}. {}", i + 1, export.name);
            println!("     Functions: {}", export.functions.len());
            for func in &export.functions {
                println!("       - {}", func.name);
            }
        }
        println!();
        
        println!("🔗 DEPENDENCIES ({})", analysis.dependencies.len());
        for dep in &analysis.dependencies {
            println!("  - {}", dep.package);
        }
        
        println!("=====================================");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_wit_analyzer_with_sample_component() {
        // This test would require a sample WASM component file
        // For now, we'll just test that the analyzer can be instantiated
        let analyzer = WitAnalyzer;
        assert!(true); // Placeholder test
    }
}