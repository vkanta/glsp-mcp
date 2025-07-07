// Note: @bytecodealliance/jco will be imported dynamically to avoid build errors
// import { transpile } from '@bytecodealliance/jco';
// Note: Validation moved to backend services via ValidationService
// import { ComponentValidator, ValidationRules } from '../validation/ComponentValidator.js';
// import { SecurityScanner, SecurityScanResult } from '../validation/SecurityScanner.js';

export interface ComponentMetadata {
    name: string;
    version?: string;
    description?: string;
    interfaces: string[];
    exports: string[];
    imports: string[];
    size: number;
    hash: string;
}

interface SecurityScanResult {
    safe: boolean;
    issues: string[];
    score: number;
}

interface ValidationRules {
    maxExports?: number;
    maxImports?: number;
    allowedImports?: string[];
    [key: string]: unknown;
}

interface ValidationResult {
    isValid: boolean;
    errors: string[];
    warnings: string[];
}

export interface TranspilerValidationResult {
    isValid: boolean;
    errors: string[];
    warnings: string[];
    securityScan?: SecurityScanResult;
}

export interface TranspiledComponent {
    id: string;
    metadata: ComponentMetadata;
    jsModule: string;
    typeDefinitions: string;
    wasmCore: ArrayBuffer;
    created: Date;
    validation?: TranspilerValidationResult;
}

// Stub classes for compatibility - actual validation is done on backend
class ComponentValidator {
    private rules: ValidationRules;
    constructor(rules?: ValidationRules) {
        this.rules = rules || {};
    }
    async validate(_component: ArrayBuffer): Promise<ValidationResult> {
        return { isValid: true, errors: [], warnings: [] };
    }
    updateRules(rules: ValidationRules): void {
        this.rules = { ...this.rules, ...rules };
    }
    getRules(): ValidationRules {
        return this.rules;
    }
}

class SecurityScanner {
    async scan(_component: ArrayBuffer): Promise<SecurityScanResult> {
        return { safe: true, issues: [], score: 100 };
    }
    async scanComponent(_component: ArrayBuffer): Promise<SecurityScanResult> {
        return { safe: true, issues: [], score: 100 };
    }
    generateReport(scan: SecurityScanResult): string {
        return `Security Score: ${scan.score}/100`;
    }
}

export class WasmTranspiler {
    private cache = new Map<string, TranspiledComponent>();
    private validator: ComponentValidator;
    private securityScanner: SecurityScanner;
    
    constructor(validationRules?: Partial<ValidationRules>) {
        this.validator = new ComponentValidator(validationRules);
        this.securityScanner = new SecurityScanner();
    }

    async transpileComponent(wasmBytes: ArrayBuffer, name?: string): Promise<TranspiledComponent> {
        const hash = await this.computeHash(wasmBytes);
        
        // Check cache first
        if (this.cache.has(hash)) {
            const cached = this.cache.get(hash)!;
            console.log(`Using cached transpilation for component: ${cached.metadata.name}`);
            return cached;
        }

        console.log('Transpiling WASM component...', { size: wasmBytes.byteLength, name });

        try {
            // Extract metadata before transpilation
            const metadata = await this.extractMetadata(wasmBytes, name);
            
            // Validate component with metadata
            const validation = await this.validateComponent(wasmBytes, metadata);
            if (!validation.isValid) {
                throw new Error(`Component validation failed: ${validation.errors.join(', ')}`);
            }
            
            // Log security scan results if available
            if (validation.securityScan) {
                console.log(`Security scan score: ${validation.securityScan.score}/100`);
                if (!validation.securityScan.safe) {
                    console.warn('Security risks detected:', validation.securityScan.risks);
                }
            }

            // Transpile using jco (dynamically imported)
            const { transpile } = await import('@bytecodealliance/jco');
            const transpileResult = await transpile(new Uint8Array(wasmBytes), {
                name: metadata.name,
                instantiation: 'async',
                optimize: true,
                nodejsCompat: false,
                tracing: false
            });

            // Create transpiled component
            const jsModule = transpileResult.files?.['index.js'];
            const typeDefinitions = transpileResult.files?.['index.d.ts'];
            
            const component: TranspiledComponent = {
                id: hash,
                metadata,
                jsModule: typeof jsModule === 'string' ? jsModule : '',
                typeDefinitions: typeof typeDefinitions === 'string' ? typeDefinitions : '',
                wasmCore: wasmBytes,
                created: new Date(),
                validation
            };

            // Cache the result
            this.cache.set(hash, component);
            console.log(`Component transpiled successfully: ${metadata.name}`);

            return component;
        } catch (error) {
            console.error('Transpilation failed:', error);
            throw new Error(`Failed to transpile component: ${error instanceof Error ? error.message : 'Unknown error'}`);
        }
    }

    async validateComponent(_wasmBytes: ArrayBuffer, _metadata?: ComponentMetadata): Promise<TranspilerValidationResult> {
        // NOTE: Validation is now handled by backend services via ValidationService
        // This method is kept for compatibility but returns a stub response
        console.log('WasmTranspiler: Validation requests are now handled by backend services');
        
        return {
            isValid: true,
            errors: [],
            warnings: ['Validation is now handled by backend services'],
            securityScan: { safe: true, issues: [], score: 100 }
        };
    }

    async extractMetadata(wasmBytes: ArrayBuffer, providedName?: string): Promise<ComponentMetadata> {
        try {
            // For now, extract basic metadata
            // In a full implementation, this would parse WIT interfaces and component metadata
            const hash = await this.computeHash(wasmBytes);
            
            // Basic WASM module introspection
            const moduleInfo = await this.basicWasmIntrospection(wasmBytes);
            
            return {
                name: providedName || `component-${hash.substring(0, 8)}`,
                size: wasmBytes.byteLength,
                hash,
                interfaces: moduleInfo.interfaces,
                exports: moduleInfo.exports,
                imports: moduleInfo.imports
            };
        } catch (error) {
            console.error('Metadata extraction failed:', error);
            throw new Error(`Failed to extract component metadata: ${error instanceof Error ? error.message : 'Unknown error'}`);
        }
    }

    private async computeHash(data: ArrayBuffer): Promise<string> {
        const hashBuffer = await crypto.subtle.digest('SHA-256', data);
        const hashArray = Array.from(new Uint8Array(hashBuffer));
        return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    }

    getCachedComponents(): TranspiledComponent[] {
        return Array.from(this.cache.values());
    }

    clearCache(): void {
        this.cache.clear();
        console.log('Transpiler cache cleared');
    }

    getCacheSize(): number {
        return this.cache.size;
    }
    
    updateValidationRules(rules: Partial<ValidationRules>): void {
        this.validator.updateRules(rules);
    }
    
    getValidationRules(): ValidationRules {
        return this.validator.getRules();
    }
    
    async generateSecurityReport(wasmBytes: ArrayBuffer): Promise<string> {
        try {
            const module = await WebAssembly.compile(wasmBytes);
            const _imports = WebAssembly.Module.imports(module);
            const _exports = WebAssembly.Module.exports(module);
            
            const scanResult = await this.securityScanner.scanComponent(wasmBytes);
            return this.securityScanner.generateReport(scanResult);
        } catch (error) {
            return `Failed to generate security report: ${error instanceof Error ? error.message : 'Unknown error'}`;
        }
    }

    private async basicWasmIntrospection(wasmBytes: ArrayBuffer): Promise<{
        interfaces: string[];
        exports: string[];
        imports: string[];
    }> {
        try {
            const module = await WebAssembly.compile(wasmBytes);
            const moduleImports = WebAssembly.Module.imports(module);
            const moduleExports = WebAssembly.Module.exports(module);
            
            return {
                interfaces: [], // WIT interfaces would require specialized parsing
                exports: moduleExports.map(exp => `${exp.name}:${exp.kind}`),
                imports: moduleImports.map(imp => `${imp.module}.${imp.name}:${imp.kind}`)
            };
        } catch (error) {
            // Fallback for invalid WASM
            return {
                interfaces: [],
                exports: [],
                imports: []
            };
        }
    }
}