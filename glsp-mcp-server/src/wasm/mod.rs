mod execution_engine;
mod filesystem_watcher;
mod security_scanner;
mod wit_analyzer;
mod graphics_renderer;
mod sensor_bridge;
mod pipeline;
mod simulation;

pub use execution_engine::{
    ExecutionContext, ExecutionProgress, ExecutionResult, ExecutionStage, GraphicsFormat,
    GraphicsOutput, VideoFormat, WasmExecutionEngine,
};
pub use filesystem_watcher::{FileSystemWatcher, WasmChangeType, WasmComponentChange};
pub use security_scanner::{
    SecurityAnalysis, SecurityIssue, SecurityIssueType, SecurityRiskLevel, WasmSecurityScanner,
};
pub use wit_analyzer::{
    ComponentWitAnalysis, WitAnalyzer, WitCompatibilityReport, WitDependency, WitFunction, 
    WitInterface, WitInterfaceType, WitParam, WitType, WitTypeDefinition, WitValidationIssue,
    WitValidationIssueType, WitValidationSeverity,
};
pub use graphics_renderer::{
    WasmGraphicsRenderer, GraphicsConfig, CanvasCommand, ImageFormat,
};
pub use sensor_bridge::{
    SensorDataBridge, SensorBridgeConfig, SensorFrame, BridgeStatus, WasmSensorInterface,
    TimingConfig, BufferSettings, SyncMode as SensorSyncMode, SimulationTimeInfo, BufferStats,
};
pub use pipeline::{
    WasmPipelineEngine, PipelineConfig, PipelineStage, PipelineExecution, PipelineState,
    DataConnection, DataMapping, DataTransform, ConnectionType, StageExecutionSettings,
    RetryConfig, BackoffStrategy, PipelineSettings, PersistenceSettings, ExecutionMode,
    StageResult, ExecutionStats, StageStats,
};
pub use simulation::{
    WasmSimulationEngine, SimulationConfig, SimulationScenario, SimulationExecution, SimulationState,
    PipelineDependency, DependencyType, DataSharingConfig, ScenarioSettings, ResourceLimits,
    ScenarioExecutionMode, SimulationSettings, SimulationExecutionMode, RealTimeSyncSettings,
    SyncMode, OutputConfig, OutputFormat, OutputDestination, SimulationStats, ScenarioStats,
    ResourceUsage, ScenarioTrigger, TriggerType, TriggerCondition, ScenarioCondition,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn, debug};

/// WASM interface definition representing import or export interfaces
///
/// Describes a collection of functions that a WASM component either provides
/// (exports) or requires (imports) from the host environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmInterface {
    pub name: String,
    pub interface_type: String, // 'import' or 'export'
    pub functions: Vec<WasmFunction>,
}

/// WASM function signature with parameters and return types
///
/// Represents a single function within a WASM interface, including
/// its parameter list and return values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmFunction {
    pub name: String,
    pub params: Vec<WasmParam>,
    pub returns: Vec<WasmParam>,
}

/// Parameter or return value definition for WASM functions
///
/// Describes a single parameter or return value with its name and type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmParam {
    pub name: String,
    pub param_type: String,
}

/// WASM component metadata and interface information
///
/// Contains comprehensive information about a WebAssembly component including
/// its file location, interfaces, security analysis, and runtime metadata.
/// This is the primary data structure for managing WASM components in the system.
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
    pub security_analysis: Option<SecurityAnalysis>,
    pub last_security_scan: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct WasmFileWatcher {
    watch_path: PathBuf,
    components: HashMap<String, WasmComponent>,
    last_scan: DateTime<Utc>,
    security_scanner: WasmSecurityScanner,
    execution_engine: Option<Arc<WasmExecutionEngine>>,
    recent_changes: Arc<tokio::sync::Mutex<Vec<WasmComponentChange>>>,
    filesystem_watcher: Option<Arc<tokio::sync::RwLock<FileSystemWatcher>>>,
}

impl WasmFileWatcher {
    pub fn new(watch_path: PathBuf) -> Self {
        Self {
            watch_path,
            components: HashMap::new(),
            last_scan: Utc::now(),
            security_scanner: WasmSecurityScanner::new(),
            execution_engine: None,
            recent_changes: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            filesystem_watcher: None,
        }
    }
    
    /// List all executions (active and recent)
    pub fn list_executions(&self) -> Vec<ExecutionResult> {
        if let Some(engine) = &self.execution_engine {
            engine.list_executions()
        } else {
            vec![]
        }
    }
    
    /// Get execution progress by ID
    pub fn get_execution_progress(&self, execution_id: &str) -> Option<ExecutionProgress> {
        self.execution_engine.as_ref()?.get_execution_progress(execution_id)
    }
    
    /// Get execution result by ID
    pub fn get_execution_result(&self, execution_id: &str) -> Option<ExecutionResult> {
        self.execution_engine.as_ref()?.get_execution_result(execution_id)
    }
    
    /// Start filesystem watcher for real-time component monitoring
    pub async fn start_file_watching(&mut self) -> Result<(), anyhow::Error> {
        // Create and start the filesystem watcher
        let mut fs_watcher = FileSystemWatcher::new(self.watch_path.clone());
        fs_watcher.start_watching().await?;
        
        // Get the changes receiver before storing watcher
        let changes_rx = fs_watcher.get_changes_receiver();
        let recent_changes = self.recent_changes.clone();
        
        // Spawn a task to collect changes
        tokio::spawn(async move {
            let mut rx = changes_rx.write().await;
            while let Some(change) = rx.recv().await {
                info!("Component change detected: {:?}", change);
                
                // Add to recent changes
                let mut changes = recent_changes.lock().await;
                changes.push(change);
                
                // Keep only last 100 changes
                if changes.len() > 100 {
                    let drain_count = changes.len() - 100;
                    changes.drain(0..drain_count);
                }
            }
        });
        
        // Store the watcher
        self.filesystem_watcher = Some(Arc::new(tokio::sync::RwLock::new(fs_watcher)));
        
        info!("Filesystem watcher started for path: {:?}", self.watch_path);
        Ok(())
    }
    
    /// Get recent component changes
    pub async fn get_recent_changes(&self) -> Vec<WasmComponentChange> {
        self.recent_changes.lock().await.clone()
    }

    /// Initialize execution engine with given configuration
    pub fn with_execution_engine(mut self, max_concurrent: usize) -> Result<Self, anyhow::Error> {
        self.execution_engine = Some(Arc::new(WasmExecutionEngine::new(max_concurrent)?));
        Ok(self)
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

    pub fn get_component_by_path(&self, path: &str) -> Option<&WasmComponent> {
        self.components.values().find(|comp| comp.path == path)
    }

    /// Find a component by name, trying various naming conventions
    pub fn find_component_flexible(&self, component_name: &str) -> Option<&WasmComponent> {
        // Try exact match first
        if let Some(comp) = self.get_component(component_name) {
            return Some(comp);
        }

        // Try with underscores converted to hyphens
        let hyphen_name = component_name.replace('_', "-");
        if let Some(comp) = self.get_component(&hyphen_name) {
            return Some(comp);
        }

        // Try with hyphens converted to underscores
        let underscore_name = component_name.replace('-', "_");
        if let Some(comp) = self.get_component(&underscore_name) {
            return Some(comp);
        }

        // Try to find by matching any component where the name matches after normalization
        self.components.values().find(|comp| {
            let normalized_comp_name = comp.name.replace(['-', '_'], "");
            let normalized_search_name = component_name.replace(['-', '_'], "");
            normalized_comp_name.eq_ignore_ascii_case(&normalized_search_name)
        })
    }

    pub async fn scan_components(&mut self) -> anyhow::Result<()> {
        use std::ffi::OsStr;

        let watch_path = &self.watch_path;
        info!("Scanning WASM components in: {watch_path:?}");

        if !self.watch_path.exists() {
            let watch_path = &self.watch_path;
            warn!("WASM watch path does not exist: {watch_path:?}");
            return Ok(());
        }

        // Find all .wasm files in the directory tree
        let mut wasm_files = Vec::new();
        self.scan_directory_recursive(&self.watch_path, &mut wasm_files)
            .await?;

        info!("Found {} WASM files", wasm_files.len());

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
                    self.components
                        .insert(component_name.to_string(), component);
                }
            }
        }

        // Mark missing components
        for (name, component) in self.components.iter_mut() {
            if !found_components.contains(name) && component.file_exists {
                component.file_exists = false;
                component.removed_at = Some(Utc::now());
                warn!("Component {name} file is now missing");
            }
        }

        self.last_scan = Utc::now();

        // Generate and display comprehensive statistics
        self.display_scan_statistics().await;

        Ok(())
    }

    async fn scan_directory_recursive(
        &self,
        dir: &PathBuf,
        wasm_files: &mut Vec<PathBuf>,
    ) -> anyhow::Result<()> {
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
        let component_name = wasm_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        debug!("Extracting component info from: {wasm_path:?}");

        // Try to extract actual metadata using wasm-tools
        match self.extract_wasm_metadata(wasm_path).await {
            Ok((interfaces, metadata, wit_content, dependencies)) => {
                let description = metadata
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&format!("ADAS component: {component_name}"))
                    .to_string();

                // Perform security analysis
                let security_analysis = match self.security_scanner.analyze_component(wasm_path).await {
                    Ok(analysis) => {
                        info!("Security analysis completed for {}: {:?} risk", 
                                component_name, analysis.overall_risk);
                        Some(analysis)
                    }
                    Err(e) => {
                        warn!("Security analysis failed for {}: {}", component_name, e);
                        None
                    }
                };

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
                    security_analysis,
                    last_security_scan: Some(Utc::now()),
                })
            }
            Err(e) => {
                warn!(
                    "Failed to extract WASM metadata for {component_name}: {e}. Using fallback."
                );

                // Fallback to basic component info if extraction fails
                Ok(WasmComponent {
                    name: component_name.clone(),
                    path: wasm_path.to_string_lossy().to_string(),
                    description: format!(
                        "ADAS component: {component_name} (metadata extraction failed)"
                    ),
                    file_exists: true,
                    last_seen: Some(Utc::now()),
                    removed_at: None,
                    interfaces: vec![
                        // Export interface
                        WasmInterface {
                            name: {
                                let normalized_name = component_name.replace('-', "_");
                                format!("adas:{normalized_name}/component")
                            },
                            interface_type: "export".to_string(),
                            functions: vec![WasmFunction {
                                name: "process".to_string(),
                                params: vec![WasmParam {
                                    name: "input".to_string(),
                                    param_type: "sensor-data".to_string(),
                                }],
                                returns: vec![WasmParam {
                                    name: "output".to_string(),
                                    param_type: "processed-data".to_string(),
                                }],
                            }],
                        },
                        // Import interface for sensor data
                        WasmInterface {
                            name: "wasi:sensors/input".to_string(),
                            interface_type: "import".to_string(),
                            functions: vec![WasmFunction {
                                name: "read".to_string(),
                                params: vec![],
                                returns: vec![WasmParam {
                                    name: "data".to_string(),
                                    param_type: "sensor-data".to_string(),
                                }],
                            }],
                        },
                        // Import interface for configuration
                        WasmInterface {
                            name: "adas:config/reader".to_string(),
                            interface_type: "import".to_string(),
                            functions: vec![WasmFunction {
                                name: "get-config".to_string(),
                                params: vec![WasmParam {
                                    name: "key".to_string(),
                                    param_type: "string".to_string(),
                                }],
                                returns: vec![WasmParam {
                                    name: "value".to_string(),
                                    param_type: "string".to_string(),
                                }],
                            }],
                        },
                    ],
                    metadata: HashMap::new(),
                    wit_interfaces: None,
                    dependencies: Vec::new(),
                    security_analysis: None,
                    last_security_scan: None,
                })
            }
        }
    }

    async fn extract_wasm_metadata(
        &self,
        wasm_path: &PathBuf,
    ) -> anyhow::Result<(
        Vec<WasmInterface>,
        HashMap<String, serde_json::Value>,
        Option<String>,
        Vec<String>,
    )> {
        // First try to use the advanced WIT analyzer
        match WitAnalyzer::analyze_component(wasm_path).await {
            Ok(wit_analysis) => {
                debug!("Successfully analyzed component with WIT analyzer");
                self.convert_wit_analysis_to_legacy_format(wit_analysis)
                    .await
            }
            Err(wit_error) => {
                warn!("WIT analysis failed: {wit_error}, falling back to wasmparser");
                self.extract_wasm_metadata_fallback(wasm_path).await
            }
        }
    }

    /// Convert WIT analysis to legacy format for backward compatibility
    async fn convert_wit_analysis_to_legacy_format(
        &self,
        analysis: ComponentWitAnalysis,
    ) -> anyhow::Result<(
        Vec<WasmInterface>,
        HashMap<String, serde_json::Value>,
        Option<String>,
        Vec<String>,
    )> {
        let mut interfaces = Vec::new();
        let mut metadata = HashMap::new();
        let mut dependencies = Vec::new();

        // Convert imports
        for wit_interface in analysis.imports {
            let interface = WasmInterface {
                name: wit_interface.name,
                interface_type: "import".to_string(),
                functions: wit_interface
                    .functions
                    .into_iter()
                    .map(|f| WasmFunction {
                        name: f.name,
                        params: f
                            .params
                            .into_iter()
                            .map(|p| WasmParam {
                                name: p.name,
                                param_type: Self::wit_type_to_string(&p.param_type),
                            })
                            .collect(),
                        returns: f
                            .results
                            .into_iter()
                            .map(|r| WasmParam {
                                name: r.name,
                                param_type: Self::wit_type_to_string(&r.param_type),
                            })
                            .collect(),
                    })
                    .collect(),
            };
            interfaces.push(interface);
        }

        // Convert exports
        for wit_interface in analysis.exports {
            let interface = WasmInterface {
                name: wit_interface.name,
                interface_type: "export".to_string(),
                functions: wit_interface
                    .functions
                    .into_iter()
                    .map(|f| WasmFunction {
                        name: f.name,
                        params: f
                            .params
                            .into_iter()
                            .map(|p| WasmParam {
                                name: p.name,
                                param_type: Self::wit_type_to_string(&p.param_type),
                            })
                            .collect(),
                        returns: f
                            .results
                            .into_iter()
                            .map(|r| WasmParam {
                                name: r.name,
                                param_type: Self::wit_type_to_string(&r.param_type),
                            })
                            .collect(),
                    })
                    .collect(),
            };
            interfaces.push(interface);
        }

        // Extract dependencies
        for dep in analysis.dependencies {
            dependencies.push(dep.package);
        }

        // Add metadata
        metadata.insert(
            "wit_world".to_string(),
            serde_json::Value::String(analysis.world_name.unwrap_or("unknown".to_string())),
        );
        metadata.insert(
            "wit_analysis_version".to_string(),
            serde_json::Value::String("2.0".to_string()),
        );
        metadata.insert(
            "extracted_at".to_string(),
            serde_json::Value::String(Utc::now().to_rfc3339()),
        );
        metadata.insert(
            "interface_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(interfaces.len())),
        );

        Ok((interfaces, metadata, analysis.raw_wit, dependencies))
    }

    /// Convert WIT type to string representation for legacy compatibility
    fn wit_type_to_string(wit_type: &WitType) -> String {
        match &wit_type.type_def {
            WitTypeDefinition::Primitive(name) => name.clone(),
            WitTypeDefinition::Record { .. } => format!("record({})", wit_type.name),
            WitTypeDefinition::Variant { .. } => format!("variant({})", wit_type.name),
            WitTypeDefinition::Enum { .. } => format!("enum({})", wit_type.name),
            WitTypeDefinition::Union { .. } => format!("union({})", wit_type.name),
            WitTypeDefinition::Option { inner } => {
                format!("option<{}>", Self::wit_type_to_string(inner))
            }
            WitTypeDefinition::Result { .. } => format!("result({})", wit_type.name),
            WitTypeDefinition::List { element } => {
                format!("list<{}>", Self::wit_type_to_string(element))
            }
            WitTypeDefinition::Tuple { .. } => format!("tuple({})", wit_type.name),
            WitTypeDefinition::Flags { .. } => format!("flags({})", wit_type.name),
            WitTypeDefinition::Resource { .. } => format!("resource({})", wit_type.name),
        }
    }

    /// Fallback method using the original wasmparser approach
    async fn extract_wasm_metadata_fallback(
        &self,
        wasm_path: &PathBuf,
    ) -> anyhow::Result<(
        Vec<WasmInterface>,
        HashMap<String, serde_json::Value>,
        Option<String>,
        Vec<String>,
    )> {
        use wasmparser::{Parser, Payload};

        debug!("Using fallback WASM parser for basic metadata extraction");

        // Read the WASM file
        let wasm_bytes = tokio::fs::read(wasm_path).await?;

        let mut interfaces = Vec::new();
        let mut metadata = HashMap::new();
        let mut wit_content = None;
        let mut dependencies = Vec::new();

        // Parse the WASM module using wasmparser
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            match payload? {
                Payload::CustomSection(reader) => match reader.name() {
                    "component-type" => {
                        debug!("Found component-type section");
                    }
                    "wit-component" => {
                        debug!("Found wit-component section");
                        if let Ok(wit_str) = std::str::from_utf8(reader.data()) {
                            wit_content = Some(wit_str.to_string());
                        }
                    }
                    "producers" => {
                        debug!("Found producers section");
                    }
                    name if name.starts_with("adas:") => {
                        debug!("Found ADAS metadata section: {name}");
                        if let Ok(metadata_str) = std::str::from_utf8(reader.data()) {
                            if let Ok(json_val) =
                                serde_json::from_str::<serde_json::Value>(metadata_str)
                            {
                                metadata.insert(name.to_string(), json_val);
                            }
                        }
                    }
                    _ => {}
                },
                Payload::ImportSection(reader) => {
                    for import in reader {
                        let import = import?;
                        dependencies.push(format!("{}::{}", import.module, import.name));

                        if let wasmparser::TypeRef::Func(_) = import.ty {
                            let interface = WasmInterface {
                                name: format!("{}::{}", import.module, import.name),
                                interface_type: "import".to_string(),
                                functions: vec![WasmFunction {
                                    name: import.name.to_string(),
                                    params: vec![],
                                    returns: vec![],
                                }],
                            };
                            interfaces.push(interface);
                        }
                    }
                }
                Payload::ExportSection(reader) => {
                    let mut export_functions = Vec::new();

                    for export in reader {
                        let export = export?;

                        if let wasmparser::ExternalKind::Func = export.kind {
                            export_functions.push(WasmFunction {
                                name: export.name.to_string(),
                                params: vec![],
                                returns: vec![],
                            });
                        }
                    }

                    if !export_functions.is_empty() {
                        interfaces.push(WasmInterface {
                            name: "exports".to_string(),
                            interface_type: "export".to_string(),
                            functions: export_functions,
                        });
                    }
                }
                _ => {}
            }
        }

        // Add basic metadata
        metadata.insert(
            "file_size".to_string(),
            serde_json::Value::Number(serde_json::Number::from(wasm_bytes.len())),
        );
        metadata.insert(
            "extracted_at".to_string(),
            serde_json::Value::String(Utc::now().to_rfc3339()),
        );
        metadata.insert(
            "analysis_method".to_string(),
            serde_json::Value::String("wasmparser_fallback".to_string()),
        );

        // Add default interfaces if none found
        if interfaces.is_empty() {
            debug!("No interfaces found, adding default ones for visualization");
            interfaces.push(WasmInterface {
                name: "component-export".to_string(),
                interface_type: "export".to_string(),
                functions: vec![WasmFunction {
                    name: "main".to_string(),
                    params: vec![],
                    returns: vec![],
                }],
            });
            interfaces.push(WasmInterface {
                name: "wasi-imports".to_string(),
                interface_type: "import".to_string(),
                functions: vec![WasmFunction {
                    name: "imported".to_string(),
                    params: vec![],
                    returns: vec![],
                }],
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

    /// Get security analysis for a specific component
    pub fn get_security_analysis(&self, component_name: &str) -> Option<&SecurityAnalysis> {
        self.components.get(component_name)
            .and_then(|comp| comp.security_analysis.as_ref())
    }

    /// Get security risk summary for all components
    pub fn get_security_summary(&self) -> HashMap<SecurityRiskLevel, usize> {
        let mut summary = HashMap::new();
        summary.insert(SecurityRiskLevel::Low, 0);
        summary.insert(SecurityRiskLevel::Medium, 0);
        summary.insert(SecurityRiskLevel::High, 0);
        summary.insert(SecurityRiskLevel::Critical, 0);

        for component in self.components.values() {
            if let Some(analysis) = &component.security_analysis {
                *summary.entry(analysis.overall_risk.clone()).or_insert(0) += 1;
            }
        }

        summary
    }

    /// Execute a WASM component method
    pub async fn execute_component(
        &self,
        component_name: &str,
        method: &str,
        args: serde_json::Value,
        timeout_ms: u64,
        max_memory_mb: u32,
    ) -> Result<String, anyhow::Error> {
        let execution_engine = self.execution_engine.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Execution engine not initialized"))?;

        let component = self.components.get(component_name)
            .ok_or_else(|| anyhow::anyhow!("Component '{}' not found", component_name))?;

        if !component.file_exists {
            return Err(anyhow::anyhow!("Component '{}' file does not exist", component_name));
        }

        let execution_id = uuid::Uuid::new_v4().to_string();
        
        let context = ExecutionContext {
            execution_id: execution_id.clone(),
            component_name: component_name.to_string(),
            method: method.to_string(),
            args,
            timeout_ms,
            max_memory_mb,
            created_at: Utc::now(),
            sensor_config: None,
        };

        let component_path = std::path::Path::new(&component.path);
        
        execution_engine.execute_component(context, component_path).await
    }

    /// Cancel an execution
    pub fn cancel_execution(&self, execution_id: &str) -> bool {
        self.execution_engine.as_ref()
            .map(|engine| engine.cancel_execution(execution_id))
            .unwrap_or(false)
    }

    /// Display comprehensive statistics after component scan
    async fn display_scan_statistics(&self) {
        let total_components = self.components.len();
        let available_components = self.components.values().filter(|c| c.file_exists).count();
        let missing_components = total_components - available_components;

        // Interface statistics
        let mut total_interfaces = 0;
        let mut total_imports = 0;
        let mut total_exports = 0;
        let mut total_functions = 0;
        let mut components_with_wit = 0;
        let mut interface_types = std::collections::HashMap::new();
        let mut dependency_count = 0;

        // Security statistics
        let mut components_with_security_analysis = 0;
        let security_summary = self.get_security_summary();
        let mut total_security_issues = 0;

        for component in self.components.values() {
            if !component.file_exists {
                continue;
            }

            total_interfaces += component.interfaces.len();
            dependency_count += component.dependencies.len();

            if component.wit_interfaces.is_some() {
                components_with_wit += 1;
            }

            if let Some(security_analysis) = &component.security_analysis {
                components_with_security_analysis += 1;
                total_security_issues += security_analysis.issues.len();
            }

            for interface in &component.interfaces {
                total_functions += interface.functions.len();

                match interface.interface_type.as_str() {
                    "import" => total_imports += 1,
                    "export" => total_exports += 1,
                    _ => {}
                }

                // Count interface types properly
                let interface_category = match interface.interface_type.as_str() {
                    "import" => {
                        if interface.name.starts_with("wasi:") {
                            "WASI Imports"
                        } else if interface.name.starts_with("adas:") {
                            "ADAS Imports"
                        } else {
                            "Component Imports"
                        }
                    }
                    "export" => {
                        if interface.name.starts_with("wasi:") {
                            "WASI Exports"
                        } else if interface.name.starts_with("adas:") {
                            "ADAS Exports"
                        } else {
                            "Component Exports"
                        }
                    }
                    _ => "Unknown Type",
                };

                *interface_types
                    .entry(interface_category.to_string())
                    .or_insert(0) += 1;
            }
        }

        // Calculate percentages
        let wit_coverage = if available_components > 0 {
            (components_with_wit as f64 / available_components as f64) * 100.0
        } else {
            0.0
        };

        let avg_interfaces_per_component = if available_components > 0 {
            total_interfaces as f64 / available_components as f64
        } else {
            0.0
        };

        let avg_functions_per_interface = if total_interfaces > 0 {
            total_functions as f64 / total_interfaces as f64
        } else {
            0.0
        };

        // Display formatted statistics
        info!("WASM Component Scan Statistics");
        info!("=====================================");
        info!("Scan Path: {:?}", self.watch_path);
        info!(
            "Scan Time: {}",
            self.last_scan.format("%Y-%m-%d %H:%M:%S UTC")
        );
        // Empty line for formatting

        // Component Overview
        info!("Component Overview:");
        info!("   Total Components: {total_components}");
        let available_pct = if total_components > 0 {
            (available_components as f64 / total_components as f64) * 100.0
        } else {
            0.0
        };
        info!("   Available: {available_components} ({available_pct:.1}%)");
        let missing_pct = if total_components > 0 {
            (missing_components as f64 / total_components as f64) * 100.0
        } else {
            0.0
        };
        info!("   Missing: {missing_components} ({missing_pct:.1}%)");
        info!("   WIT Analysis Coverage: {components_with_wit} ({wit_coverage:.1}%)");
        // Empty line for formatting

        // Interface Statistics
        info!("Interface Analysis:");
        info!("   Total Interfaces: {total_interfaces}");
        let imports_pct = if total_interfaces > 0 {
            (total_imports as f64 / total_interfaces as f64) * 100.0
        } else {
            0.0
        };
        info!("   Imports: {total_imports} ({imports_pct:.1}%)");
        let exports_pct = if total_interfaces > 0 {
            (total_exports as f64 / total_interfaces as f64) * 100.0
        } else {
            0.0
        };
        info!("   Exports: {total_exports} ({exports_pct:.1}%)");
        info!("   Total Functions: {total_functions}");
        info!("   Dependencies: {dependency_count}");
        // Empty line for formatting

        // Averages
        info!("Averages:");
        info!("   Interfaces per Component: {avg_interfaces_per_component:.1}");
        info!("   Functions per Interface: {avg_functions_per_interface:.1}");
        let deps_per_component = if available_components > 0 {
            dependency_count as f64 / available_components as f64
        } else {
            0.0
        };
        info!("   Dependencies per Component: {deps_per_component:.1}");
        // Empty line for formatting

        // Interface Type Breakdown
        if !interface_types.is_empty() {
            info!("Interface Categories:");
            let mut sorted_types: Vec<_> = interface_types.iter().collect();
            sorted_types.sort_by(|a, b| b.1.cmp(a.1));

            for (category, count) in sorted_types {
                let percentage = if total_interfaces > 0 {
                    (*count as f64 / total_interfaces as f64) * 100.0
                } else {
                    0.0
                };
                info!("   {category}: {count} ({percentage:.1}%)");
            }
            // Empty line for formatting
        }

        // Component Details (top 5 by interface count)
        if available_components > 0 {
            info!("Top Components by Interface Count:");
            let mut components_by_interfaces: Vec<_> =
                self.components.values().filter(|c| c.file_exists).collect();
            components_by_interfaces.sort_by(|a, b| b.interfaces.len().cmp(&a.interfaces.len()));

            for (i, component) in components_by_interfaces.iter().take(5).enumerate() {
                let wit_status = if component.wit_interfaces.is_some() {
                    "✅"
                } else {
                    "❌"
                };
                let rank = i + 1;
                let name = &component.name;
                let interface_count = component.interfaces.len();
                let dep_count = component.dependencies.len();
                info!("   {rank}. {name} - {interface_count} interfaces, {dep_count} deps {wit_status}");
            }
            // Empty line for formatting
        }

        // Security Analysis Summary
        if components_with_security_analysis > 0 {
            info!("Security Analysis Summary:");
            let security_coverage = if available_components > 0 {
                (components_with_security_analysis as f64 / available_components as f64) * 100.0
            } else {
                0.0
            };
            info!("   Security Coverage: {components_with_security_analysis}/{available_components} ({security_coverage:.1}%)");
            info!("   Total Security Issues: {total_security_issues}");
            
            for (risk_level, count) in &security_summary {
                if *count > 0 {
                    let icon = match risk_level {
                        SecurityRiskLevel::Critical => "🔴",
                        SecurityRiskLevel::High => "🟠", 
                        SecurityRiskLevel::Medium => "🟡",
                        SecurityRiskLevel::Low => "🟢",
                    };
                    info!("   {icon} {:?}: {count} components", risk_level);
                }
            }
            
            // Security recommendations
            let critical_count = security_summary.get(&SecurityRiskLevel::Critical).unwrap_or(&0);
            let high_count = security_summary.get(&SecurityRiskLevel::High).unwrap_or(&0);
            
            if *critical_count > 0 {
                warn!("   URGENT: {critical_count} components have critical security issues!");
            }
            if *high_count > 0 {
                warn!("   WARNING: {high_count} components have high-risk security issues");
            }
            // Empty line for formatting
        }

        // Health Assessment
        let health_status = if missing_components == 0 && wit_coverage > 80.0 {
            "🟢 EXCELLENT"
        } else if missing_components == 0 && wit_coverage > 50.0 {
            "🟡 GOOD"
        } else if missing_components < total_components / 2 {
            "🟠 DEGRADED"
        } else {
            "🔴 CRITICAL"
        };

        info!("Component Ecosystem Health: {health_status}");

        if missing_components > 0 {
            warn!("Warning: {missing_components} components have missing files");
        }

        if wit_coverage < 50.0 && available_components > 0 {
            info!("Suggestion: Consider running WIT analysis on more components");
        }

        info!("=====================================");
    }

    /// Change the watched path and restart filesystem monitoring
    pub async fn change_watch_path(&mut self, new_path: PathBuf) -> anyhow::Result<()> {
        info!("Changing WASM file watcher path from {:?} to {:?}", self.watch_path, new_path);
        
        // Update the path
        self.watch_path = new_path;
        
        // Clear existing components since they're from the old path
        self.components.clear();
        
        // Clear recent changes
        self.recent_changes.lock().await.clear();
        
        // If we have a filesystem watcher, update its path
        if let Some(fs_watcher_arc) = &self.filesystem_watcher {
            let mut fs_watcher = fs_watcher_arc.write().await;
            fs_watcher.change_watch_path(self.watch_path.clone()).await
                .map_err(|e| anyhow::anyhow!("Failed to change filesystem watcher path: {}", e))?;
        }
        
        // Perform initial scan of the new path
        self.scan_components().await?;
        
        info!("WASM file watcher successfully changed to new path");
        Ok(())
    }
}
