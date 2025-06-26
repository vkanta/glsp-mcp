use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmInterface {
    pub name: String,
    pub interface_type: String, // 'import' or 'export'
    pub functions: Vec<WasmFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmFunction {
    pub name: String,
    pub params: Vec<WasmParam>,
    pub returns: Vec<WasmParam>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmParam {
    pub name: String,
    pub param_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmComponent {
    pub name: String,
    pub path: String,
    pub description: String,
    pub file_exists: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub removed_at: Option<DateTime<Utc>>,
    pub interfaces: Vec<WasmInterface>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub wit_interfaces: Option<String>, // Raw WIT content
    pub dependencies: Vec<String>,
}

pub struct WasmFileWatcher {
    watch_path: PathBuf,
    components: HashMap<String, WasmComponent>,
    last_scan: DateTime<Utc>,
}

impl WasmFileWatcher {
    pub fn new(watch_path: PathBuf) -> Self {
        Self {
            watch_path,
            components: HashMap::new(),
            last_scan: Utc::now(),
        }
    }

    pub fn get_watch_path(&self) -> &PathBuf {
        &self.watch_path
    }

    pub fn get_last_scan_time(&self) -> DateTime<Utc> {
        self.last_scan
    }

    pub fn get_components(&self) -> Vec<&WasmComponent> {
        self.components.values().collect()
    }

    pub fn get_component(&self, name: &str) -> Option<&WasmComponent> {
        self.components.get(name)
    }

    pub async fn scan_components(&mut self) -> anyhow::Result<()> {
        use std::ffi::OsStr;

        println!("Scanning WASM components in: {:?}", self.watch_path);
        
        if !self.watch_path.exists() {
            println!("WASM watch path does not exist: {:?}", self.watch_path);
            return Ok(());
        }

        // Find all .wasm files in the directory tree
        let mut wasm_files = Vec::new();
        self.scan_directory_recursive(&self.watch_path, &mut wasm_files).await?;
        
        println!("Found {} WASM files", wasm_files.len());

        // Track which components we found this scan
        let mut found_components = std::collections::HashSet::new();

        // Process each WASM file
        for wasm_path in wasm_files {
            if let Some(component_name) = wasm_path.file_stem().and_then(OsStr::to_str) {
                found_components.insert(component_name.to_string());
                
                // Check if component exists or needs updating
                if let Some(existing) = self.components.get_mut(component_name) {
                    existing.file_exists = true;
                    existing.last_seen = Some(Utc::now());
                    existing.removed_at = None;
                } else {
                    // New component discovered
                    let component = self.extract_component_info(&wasm_path).await?;
                    self.components.insert(component_name.to_string(), component);
                }
            }
        }

        // Mark missing components
        for (name, component) in self.components.iter_mut() {
            if !found_components.contains(name) && component.file_exists {
                component.file_exists = false;
                component.removed_at = Some(Utc::now());
                println!("Component {} file is now missing", name);
            }
        }

        self.last_scan = Utc::now();
        Ok(())
    }

    async fn scan_directory_recursive(&self, dir: &PathBuf, wasm_files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively scan subdirectories
                Box::pin(self.scan_directory_recursive(&path, wasm_files)).await?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                wasm_files.push(path);
            }
        }
        
        Ok(())
    }

    async fn extract_component_info(&self, wasm_path: &PathBuf) -> anyhow::Result<WasmComponent> {
        let component_name = wasm_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        println!("Extracting component info from: {:?}", wasm_path);

        // Try to extract actual metadata using wasm-tools
        match self.extract_wasm_metadata(wasm_path).await {
            Ok((interfaces, metadata, wit_content, dependencies)) => {
                let description = metadata.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&format!("ADAS component: {}", component_name))
                    .to_string();

                Ok(WasmComponent {
                    name: component_name,
                    path: wasm_path.to_string_lossy().to_string(),
                    description,
                    file_exists: true,
                    last_seen: Some(Utc::now()),
                    removed_at: None,
                    interfaces,
                    metadata,
                    wit_interfaces: wit_content,
                    dependencies,
                })
            }
            Err(e) => {
                println!("Failed to extract WASM metadata for {}: {}. Using fallback.", component_name, e);
                
                // Fallback to basic component info if extraction fails
                Ok(WasmComponent {
                    name: component_name.clone(),
                    path: wasm_path.to_string_lossy().to_string(),
                    description: format!("ADAS component: {} (metadata extraction failed)", component_name),
                    file_exists: true,
                    last_seen: Some(Utc::now()),
                    removed_at: None,
                    interfaces: vec![
                        // Export interface
                        WasmInterface {
                            name: format!("adas:{}/component", component_name.replace('-', "_")),
                            interface_type: "export".to_string(),
                            functions: vec![
                                WasmFunction {
                                    name: "process".to_string(),
                                    params: vec![
                                        WasmParam {
                                            name: "input".to_string(),
                                            param_type: "sensor-data".to_string(),
                                        }
                                    ],
                                    returns: vec![
                                        WasmParam {
                                            name: "output".to_string(),
                                            param_type: "processed-data".to_string(),
                                        }
                                    ],
                                }
                            ],
                        },
                        // Import interface for sensor data
                        WasmInterface {
                            name: "wasi:sensors/input".to_string(),
                            interface_type: "import".to_string(),
                            functions: vec![
                                WasmFunction {
                                    name: "read".to_string(),
                                    params: vec![],
                                    returns: vec![
                                        WasmParam {
                                            name: "data".to_string(),
                                            param_type: "sensor-data".to_string(),
                                        }
                                    ],
                                }
                            ],
                        },
                        // Import interface for configuration
                        WasmInterface {
                            name: "adas:config/reader".to_string(),
                            interface_type: "import".to_string(),
                            functions: vec![
                                WasmFunction {
                                    name: "get-config".to_string(),
                                    params: vec![
                                        WasmParam {
                                            name: "key".to_string(),
                                            param_type: "string".to_string(),
                                        }
                                    ],
                                    returns: vec![
                                        WasmParam {
                                            name: "value".to_string(),
                                            param_type: "string".to_string(),
                                        }
                                    ],
                                }
                            ],
                        }
                    ],
                    metadata: HashMap::new(),
                    wit_interfaces: None,
                    dependencies: Vec::new(),
                })
            }
        }
    }

    async fn extract_wasm_metadata(&self, wasm_path: &PathBuf) -> anyhow::Result<(Vec<WasmInterface>, HashMap<String, serde_json::Value>, Option<String>, Vec<String>)> {
        use wasmparser::{Parser, Payload};
        
        // Read the WASM file
        let wasm_bytes = tokio::fs::read(wasm_path).await?;
        
        let mut interfaces = Vec::new();
        let mut metadata = HashMap::new();
        let mut wit_content = None;
        let mut dependencies = Vec::new();

        // Parse the WASM module
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            match payload? {
                Payload::CustomSection(reader) => {
                    match reader.name() {
                        "component-type" => {
                            // Extract component type information
                            println!("Found component-type section");
                            // TODO: Parse component type data
                        }
                        "wit-component" => {
                            // Extract WIT interface definitions
                            println!("Found wit-component section");
                            if let Ok(wit_str) = std::str::from_utf8(reader.data()) {
                                wit_content = Some(wit_str.to_string());
                            }
                        }
                        "producers" => {
                            // Extract producer information (toolchain, etc.)
                            println!("Found producers section");
                        }
                        name if name.starts_with("adas:") => {
                            // Custom ADAS-specific metadata
                            println!("Found ADAS metadata section: {}", name);
                            if let Ok(metadata_str) = std::str::from_utf8(reader.data()) {
                                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(metadata_str) {
                                    metadata.insert(name.to_string(), json_val);
                                }
                            }
                        }
                        _ => {
                            // Other custom sections
                            println!("Found custom section: {}", reader.name());
                        }
                    }
                }
                Payload::ImportSection(reader) => {
                    // Extract imports (dependencies)
                    for import in reader {
                        let import = import?;
                        dependencies.push(format!("{}::{}", import.module, import.name));
                        
                        // Create interface for imported functions
                        if let wasmparser::TypeRef::Func(_) = import.ty {
                            let interface_name = format!("{}::{}", import.module, import.name);
                            let interface = WasmInterface {
                                name: interface_name,
                                interface_type: "import".to_string(),
                                functions: vec![
                                    WasmFunction {
                                        name: import.name.to_string(),
                                        params: vec![], // TODO: Extract actual parameters
                                        returns: vec![], // TODO: Extract actual returns
                                    }
                                ],
                            };
                            interfaces.push(interface);
                        }
                    }
                }
                Payload::ExportSection(reader) => {
                    // Extract exports
                    let mut export_functions = Vec::new();
                    
                    for export in reader {
                        let export = export?;
                        
                        if let wasmparser::ExternalKind::Func = export.kind {
                            export_functions.push(WasmFunction {
                                name: export.name.to_string(),
                                params: vec![], // TODO: Extract actual parameters  
                                returns: vec![], // TODO: Extract actual returns
                            });
                        }
                    }
                    
                    if !export_functions.is_empty() {
                        let interface = WasmInterface {
                            name: "exports".to_string(),
                            interface_type: "export".to_string(),
                            functions: export_functions,
                        };
                        interfaces.push(interface);
                    }
                }
                _ => {
                    // Other sections we don't need for metadata extraction
                }
            }
        }

        // Add some basic metadata
        metadata.insert("file_size".to_string(), serde_json::Value::Number(serde_json::Number::from(wasm_bytes.len())));
        metadata.insert("extracted_at".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));

        // If no interfaces were found, add default ones for visualization
        if interfaces.is_empty() {
            println!("No interfaces found in WASM metadata, adding default interfaces for component");
            interfaces.push(WasmInterface {
                name: "component-export".to_string(),
                interface_type: "export".to_string(),
                functions: vec![
                    WasmFunction {
                        name: "main".to_string(),
                        params: vec![],
                        returns: vec![],
                    }
                ],
            });
            interfaces.push(WasmInterface {
                name: "wasi-imports".to_string(),
                interface_type: "import".to_string(),
                functions: vec![
                    WasmFunction {
                        name: "imported".to_string(),
                        params: vec![],
                        returns: vec![],
                    }
                ],
            });
        }

        Ok((interfaces, metadata, wit_content, dependencies))
    }

    pub fn remove_missing_component(&mut self, name: &str) -> bool {
        if let Some(component) = self.components.get(name) {
            if !component.file_exists {
                self.components.remove(name);
                return true;
            }
        }
        false
    }
}