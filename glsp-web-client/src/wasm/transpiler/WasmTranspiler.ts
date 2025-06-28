// Note: @bytecodealliance/jco will be imported dynamically to avoid build errors
// import { transpile } from '@bytecodealliance/jco';
import { ComponentValidator, ValidationResult as ValidatorResult, ValidationRules } from '../validation/ComponentValidator.js';
import { SecurityScanner, SecurityScanResult } from '../validation/SecurityScanner.js';

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

export interface ValidationResult extends ValidatorResult {
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
    validation?: ValidationResult;
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

    async validateComponent(wasmBytes: ArrayBuffer, metadata?: ComponentMetadata): Promise<ValidationResult> {
        try {
            // Run comprehensive validation
            const validatorResult = await this.validator.validateComponent(wasmBytes, metadata);
            
            // Parse WASM module for security scanning
            let securityScan: SecurityScanResult | undefined;
            
            try {
                const module = await WebAssembly.compile(wasmBytes);
                const imports = WebAssembly.Module.imports(module);
                const exports = WebAssembly.Module.exports(module);
                
                // Run security scan
                securityScan = await this.securityScanner.scanComponent(wasmBytes, imports, exports);
                
                // Add security risks to validation warnings/errors
                securityScan.risks.forEach(risk => {
                    if (risk.type === 'high') {
                        validatorResult.errors.push({
                            code: 'SECURITY_RISK_HIGH',
                            message: `${risk.category}: ${risk.description}`,
                            severity: 'error'
                        });
                    } else if (risk.type === 'medium') {
                        validatorResult.warnings.push({
                            code: 'SECURITY_RISK_MEDIUM',
                            message: `${risk.category}: ${risk.description}`,
                            severity: 'warning'
                        });
                    }
                });
            } catch (error) {
                console.warn('Security scan failed:', error);
                // Don't fail validation if security scan fails
            }
            
            // Convert to legacy format for compatibility
            const result: ValidationResult = {
                valid: validatorResult.valid,
                isValid: validatorResult.valid,
                errors: validatorResult.errors.map(e => e.message),
                warnings: validatorResult.warnings.map(w => w.message),
                metadata: validatorResult.metadata,
                securityScan
            };
            
            return result;
        } catch (error) {
            return {
                valid: false,
                isValid: false,
                errors: [`Validation error: ${error instanceof Error ? error.message : 'Unknown error'}`],
                warnings: []
            };
        }
    }

    async extractMetadata(wasmBytes: ArrayBuffer, providedName?: string): Promise<ComponentMetadata> {
        try {
            // For now, extract basic metadata
            // In a full implementation, this would parse WIT interfaces and component metadata
            const hash = await this.computeHash(wasmBytes);
            
            return {
                name: providedName || `component-${hash.substring(0, 8)}`,
                size: wasmBytes.byteLength,
                hash,
                interfaces: [], // TODO: Parse WIT interfaces
                exports: [], // TODO: Parse component exports
                imports: [] // TODO: Parse component imports
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
            const imports = WebAssembly.Module.imports(module);
            const exports = WebAssembly.Module.exports(module);
            
            const scanResult = await this.securityScanner.scanComponent(wasmBytes, imports, exports);
            return this.securityScanner.generateReport(scanResult);
        } catch (error) {
            return `Failed to generate security report: ${error instanceof Error ? error.message : 'Unknown error'}`;
        }
    }
}