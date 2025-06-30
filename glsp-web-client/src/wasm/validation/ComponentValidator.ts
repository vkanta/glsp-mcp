import { ComponentMetadata } from '../transpiler/WasmTranspiler.js';

export interface ValidationResult {
    valid: boolean;
    errors: ValidationError[];
    warnings: ValidationWarning[];
    metadata?: ComponentMetadata;
}

export interface ValidationError {
    code: string;
    message: string;
    severity: 'error';
    location?: string;
}

export interface ValidationWarning {
    code: string;
    message: string;
    severity: 'warning';
    location?: string;
}

export interface ValidationRules {
    maxFileSize?: number; // in bytes
    allowedImports?: string[];
    requiredExports?: string[];
    allowUnsafeOperations?: boolean;
    checkMemoryLimits?: boolean;
    maxMemoryPages?: number;
    maxTableSize?: number;
    allowedWasiVersions?: string[];
}

const DEFAULT_RULES: ValidationRules = {
    maxFileSize: 50 * 1024 * 1024, // 50MB
    allowedImports: ['wasi_snapshot_preview1', 'wasi_snapshot_preview2'],
    requiredExports: [],
    allowUnsafeOperations: false,
    checkMemoryLimits: true,
    maxMemoryPages: 16384, // 1GB with 64KB pages
    maxTableSize: 10000,
    allowedWasiVersions: ['preview1', 'preview2']
};

export class ComponentValidator {
    private rules: ValidationRules;

    constructor(rules: Partial<ValidationRules> = {}) {
        this.rules = { ...DEFAULT_RULES, ...rules };
    }

    async validateComponent(wasmBytes: ArrayBuffer, metadata?: ComponentMetadata): Promise<ValidationResult> {
        const errors: ValidationError[] = [];
        const warnings: ValidationWarning[] = [];

        try {
            // 1. Check file size
            this.validateFileSize(wasmBytes, errors);

            // 2. Validate WASM magic number and version
            this.validateWasmHeader(wasmBytes, errors);

            // 3. Parse and validate WASM structure
            const moduleInfo = await this.parseWasmModule(wasmBytes, errors, warnings);

            // 4. Validate imports
            if (moduleInfo) {
                this.validateImports(moduleInfo.imports, errors, warnings);
                
                // 5. Validate exports
                this.validateExports(moduleInfo.exports, errors, warnings);
                
                // 6. Check memory and table limits
                this.validateMemoryLimits(moduleInfo.memory, errors, warnings);
                this.validateTableLimits(moduleInfo.tables, errors, warnings);
                
                // 7. Security checks
                this.performSecurityChecks(moduleInfo, errors, warnings);
            }

            // 8. Validate metadata if provided
            if (metadata) {
                this.validateMetadata(metadata, errors, warnings);
            }

            return {
                valid: errors.length === 0,
                errors,
                warnings,
                metadata
            };

        } catch (error) {
            errors.push({
                code: 'VALIDATION_FATAL',
                message: `Fatal validation error: ${error instanceof Error ? error.message : 'Unknown error'}`,
                severity: 'error'
            });

            return {
                valid: false,
                errors,
                warnings
            };
        }
    }

    private validateFileSize(wasmBytes: ArrayBuffer, errors: ValidationError[]): void {
        if (this.rules.maxFileSize && wasmBytes.byteLength > this.rules.maxFileSize) {
            errors.push({
                code: 'FILE_TOO_LARGE',
                message: `File size ${(wasmBytes.byteLength / 1024 / 1024).toFixed(2)}MB exceeds maximum allowed size of ${(this.rules.maxFileSize / 1024 / 1024).toFixed(2)}MB`,
                severity: 'error'
            });
        }
    }

    private validateWasmHeader(wasmBytes: ArrayBuffer, errors: ValidationError[]): void {
        const view = new DataView(wasmBytes);
        
        // Check WASM magic number (0x00 0x61 0x73 0x6D) = '\0asm'
        if (view.byteLength < 8) {
            errors.push({
                code: 'INVALID_WASM_FILE',
                message: 'File is too small to be a valid WASM module',
                severity: 'error'
            });
            return;
        }

        const magic = view.getUint32(0, true);
        if (magic !== 0x6D736100) { // '\0asm' in little-endian
            errors.push({
                code: 'INVALID_MAGIC_NUMBER',
                message: 'Invalid WASM magic number. File does not appear to be a WebAssembly module',
                severity: 'error'
            });
            return;
        }

        // Check WASM version (should be 1 for MVP)
        const version = view.getUint32(4, true);
        if (version !== 1) {
            errors.push({
                code: 'UNSUPPORTED_WASM_VERSION',
                message: `Unsupported WASM version ${version}. Only version 1 is supported`,
                severity: 'error'
            });
        }
    }

    private async parseWasmModule(wasmBytes: ArrayBuffer, errors: ValidationError[], _warnings: ValidationWarning[]): Promise<any> {
        try {
            // Use WebAssembly.validate for basic validation
            const isValid = await WebAssembly.validate(wasmBytes);
            if (!isValid) {
                errors.push({
                    code: 'INVALID_WASM_MODULE',
                    message: 'WebAssembly validation failed. The module contains invalid bytecode',
                    severity: 'error'
                });
                return null;
            }

            // Try to compile to get more information
            const module = await WebAssembly.compile(wasmBytes);
            
            // Extract module information using WebAssembly.Module methods
            const imports = WebAssembly.Module.imports(module);
            const exports = WebAssembly.Module.exports(module);
            
            // Parse custom sections for additional metadata
            const customSections = this.parseCustomSections(wasmBytes);

            return {
                imports,
                exports,
                customSections,
                memory: this.extractMemoryInfo(imports, exports),
                tables: this.extractTableInfo(imports, exports)
            };

        } catch (error) {
            errors.push({
                code: 'WASM_COMPILE_ERROR',
                message: `Failed to compile WASM module: ${error instanceof Error ? error.message : 'Unknown error'}`,
                severity: 'error'
            });
            return null;
        }
    }

    private validateImports(imports: WebAssembly.ModuleImportDescriptor[], errors: ValidationError[], warnings: ValidationWarning[]): void {
        const importsByModule = new Map<string, WebAssembly.ModuleImportDescriptor[]>();
        
        // Group imports by module
        imports.forEach(imp => {
            const moduleImports = importsByModule.get(imp.module) || [];
            moduleImports.push(imp);
            importsByModule.set(imp.module, moduleImports);
        });

        // Check allowed imports
        if (this.rules.allowedImports) {
            importsByModule.forEach((moduleImports, moduleName) => {
                if (!this.rules.allowedImports!.includes(moduleName)) {
                    warnings.push({
                        code: 'UNRECOGNIZED_IMPORT',
                        message: `Module imports from '${moduleName}' which is not in the allowed imports list`,
                        severity: 'warning',
                        location: moduleName
                    });
                }
            });
        }

        // Check for dangerous imports
        const dangerousImports = ['fs', 'process', 'child_process', 'net'];
        importsByModule.forEach((moduleImports, moduleName) => {
            if (dangerousImports.includes(moduleName)) {
                errors.push({
                    code: 'DANGEROUS_IMPORT',
                    message: `Module imports from potentially dangerous module '${moduleName}'`,
                    severity: 'error',
                    location: moduleName
                });
            }
        });
    }

    private validateExports(exports: WebAssembly.ModuleExportDescriptor[], errors: ValidationError[], warnings: ValidationWarning[]): void {
        const exportNames = exports.map(exp => exp.name);

        // Check required exports
        if (this.rules.requiredExports) {
            this.rules.requiredExports.forEach(requiredExport => {
                if (!exportNames.includes(requiredExport)) {
                    errors.push({
                        code: 'MISSING_REQUIRED_EXPORT',
                        message: `Required export '${requiredExport}' not found in module`,
                        severity: 'error',
                        location: requiredExport
                    });
                }
            });
        }

        // Check for standard component model exports
        const componentExports = ['_start', '_initialize', '__wasm_call_ctors'];
        const hasComponentExports = componentExports.some(exp => exportNames.includes(exp));
        
        if (!hasComponentExports && exportNames.length > 0) {
            warnings.push({
                code: 'NON_STANDARD_EXPORTS',
                message: 'Module does not export standard component model functions',
                severity: 'warning'
            });
        }
    }

    private validateMemoryLimits(memoryInfo: any, errors: ValidationError[], warnings: ValidationWarning[]): void {
        if (!this.rules.checkMemoryLimits || !memoryInfo) return;

        if (memoryInfo.initial > this.rules.maxMemoryPages!) {
            errors.push({
                code: 'MEMORY_LIMIT_EXCEEDED',
                message: `Initial memory size ${memoryInfo.initial} pages exceeds maximum allowed ${this.rules.maxMemoryPages} pages`,
                severity: 'error'
            });
        }

        if (memoryInfo.maximum && memoryInfo.maximum > this.rules.maxMemoryPages!) {
            warnings.push({
                code: 'HIGH_MEMORY_MAXIMUM',
                message: `Maximum memory size ${memoryInfo.maximum} pages is very high`,
                severity: 'warning'
            });
        }
    }

    private validateTableLimits(tables: any[], errors: ValidationError[], _warnings: ValidationWarning[]): void {
        if (!this.rules.maxTableSize || !tables) return;

        tables.forEach((table, index) => {
            if (table.initial > this.rules.maxTableSize!) {
                errors.push({
                    code: 'TABLE_LIMIT_EXCEEDED',
                    message: `Table ${index} initial size ${table.initial} exceeds maximum allowed ${this.rules.maxTableSize}`,
                    severity: 'error',
                    location: `table[${index}]`
                });
            }
        });
    }

    private performSecurityChecks(moduleInfo: any, errors: ValidationError[], warnings: ValidationWarning[]): void {
        // Check for potentially unsafe operations
        if (!this.rules.allowUnsafeOperations) {
            // Check for unrestricted memory access patterns
            if (moduleInfo.exports.some((exp: any) => exp.name.includes('__indirect_function_table'))) {
                warnings.push({
                    code: 'INDIRECT_CALLS_DETECTED',
                    message: 'Module uses indirect function calls which could be a security risk',
                    severity: 'warning'
                });
            }

            // Check for suspicious export names
            const suspiciousPatterns = ['eval', 'exec', 'shell', 'system'];
            moduleInfo.exports.forEach((exp: any) => {
                if (suspiciousPatterns.some(pattern => exp.name.toLowerCase().includes(pattern))) {
                    warnings.push({
                        code: 'SUSPICIOUS_EXPORT_NAME',
                        message: `Export '${exp.name}' has a suspicious name that might indicate unsafe operations`,
                        severity: 'warning',
                        location: exp.name
                    });
                }
            });
        }
    }

    private validateMetadata(metadata: ComponentMetadata, errors: ValidationError[], warnings: ValidationWarning[]): void {
        // Validate metadata structure
        if (!metadata.name || metadata.name.trim().length === 0) {
            errors.push({
                code: 'INVALID_METADATA',
                message: 'Component name is required',
                severity: 'error',
                location: 'metadata.name'
            });
        }

        // Validate name format
        if (metadata.name && !/^[a-zA-Z0-9-_]+$/.test(metadata.name)) {
            errors.push({
                code: 'INVALID_COMPONENT_NAME',
                message: 'Component name contains invalid characters. Only alphanumeric, dash, and underscore are allowed',
                severity: 'error',
                location: 'metadata.name'
            });
        }

        // Check for required interfaces
        if (metadata.interfaces && metadata.interfaces.length === 0) {
            warnings.push({
                code: 'NO_INTERFACES',
                message: 'Component does not declare any interfaces',
                severity: 'warning'
            });
        }
    }

    private parseCustomSections(_wasmBytes: ArrayBuffer): Map<string, Uint8Array> {
        // Simple custom section parser
        // In a real implementation, this would properly parse the WASM binary format
        const sections = new Map<string, Uint8Array>();
        
        // TODO: Implement proper WASM binary parsing for custom sections
        // For now, return empty map
        return sections;
    }

    private extractMemoryInfo(imports: WebAssembly.ModuleImportDescriptor[], exports: WebAssembly.ModuleExportDescriptor[]): any {
        // Extract memory information from imports/exports
        const memoryImport = imports.find(imp => imp.kind === 'memory');
        const memoryExport = exports.find(exp => exp.kind === 'memory');
        
        if (memoryImport || memoryExport) {
            // In a real implementation, we'd extract the actual limits
            return {
                initial: 256, // placeholder
                maximum: 16384 // placeholder
            };
        }
        
        return null;
    }

    private extractTableInfo(imports: WebAssembly.ModuleImportDescriptor[], _exports: WebAssembly.ModuleExportDescriptor[]): any[] {
        // Extract table information from imports/exports
        const tables: any[] = [];
        
        imports.forEach(imp => {
            if (imp.kind === 'table') {
                tables.push({
                    initial: 1024, // placeholder
                    element: 'funcref'
                });
            }
        });
        
        return tables;
    }

    // Utility method to update validation rules
    updateRules(rules: Partial<ValidationRules>): void {
        this.rules = { ...this.rules, ...rules };
    }

    // Get current validation rules
    getRules(): ValidationRules {
        return { ...this.rules };
    }
}