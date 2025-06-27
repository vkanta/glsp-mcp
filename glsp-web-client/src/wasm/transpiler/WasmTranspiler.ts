// Note: @bytecodealliance/jco will be imported dynamically to avoid build errors
// import { transpile } from '@bytecodealliance/jco';

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

export interface ValidationResult {
    isValid: boolean;
    errors: string[];
    warnings: string[];
}

export interface TranspiledComponent {
    id: string;
    metadata: ComponentMetadata;
    jsModule: string;
    typeDefinitions: string;
    wasmCore: ArrayBuffer;
    created: Date;
}

export class WasmTranspiler {
    private cache = new Map<string, TranspiledComponent>();

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
            // Validate component before transpilation
            const validation = await this.validateComponent(wasmBytes);
            if (!validation.isValid) {
                throw new Error(`Component validation failed: ${validation.errors.join(', ')}`);
            }

            // Extract metadata before transpilation
            const metadata = await this.extractMetadata(wasmBytes, name);

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
                created: new Date()
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

    async validateComponent(wasmBytes: ArrayBuffer): Promise<ValidationResult> {
        const result: ValidationResult = {
            isValid: true,
            errors: [],
            warnings: []
        };

        try {
            // Basic size check
            if (wasmBytes.byteLength === 0) {
                result.errors.push('Empty WASM file');
                result.isValid = false;
                return result;
            }

            if (wasmBytes.byteLength > 50 * 1024 * 1024) { // 50MB limit
                result.errors.push('Component too large (>50MB)');
                result.isValid = false;
            }

            // Check WASM magic number
            const view = new DataView(wasmBytes);
            const magic = view.getUint32(0, true);
            if (magic !== 0x6d736100) { // '\0asm'
                result.errors.push('Invalid WASM magic number');
                result.isValid = false;
            }

            // Check version
            const version = view.getUint32(4, true);
            if (version !== 1) {
                result.warnings.push(`Unexpected WASM version: ${version}`);
            }

            // Additional component-specific validation could be added here
            // For now, we rely on jco's validation during transpilation
            
        } catch (error) {
            result.errors.push(`Validation error: ${error instanceof Error ? error.message : 'Unknown error'}`);
            result.isValid = false;
        }

        return result;
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
}