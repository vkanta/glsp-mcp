/**
 * WebAssembly Component File Watcher
 * Watches directories for WASM component files and parses their interfaces
 */

import { WasmComponentConfig } from './wasm-component-types.js';

export interface WasmFileWatcherConfig {
    watchPaths: string[];
    extensions: string[];
    pollInterval: number; // in milliseconds
    extractionMethod: 'wasm-tools' | 'custom-tool' | 'mock';
    customToolPath?: string;
}

export interface WasmFileChangeEvent {
    type: 'added' | 'changed' | 'removed';
    path: string;
    component?: WasmComponentConfig;
    missingComponent?: WasmComponentConfig; // For removed files
}

export type WasmFileChangeHandler = (event: WasmFileChangeEvent) => void;

export class WasmFileWatcher {
    private config: WasmFileWatcherConfig;
    private handlers: WasmFileChangeHandler[] = [];
    private knownFiles = new Map<string, { modified: number; component: WasmComponentConfig }>();
    private missingFiles = new Map<string, { component: WasmComponentConfig; removedAt: number }>(); // Track removed files
    private watchInterval?: number;
    private isWatching = false;

    constructor(config: Partial<WasmFileWatcherConfig> = {}) {
        // Get default watch path from CLI args or environment
        const defaultWatchPath = this.getDefaultWatchPath();
        
        this.config = {
            watchPaths: [defaultWatchPath],
            extensions: ['.wasm', '.component.wasm', '.wat'],
            pollInterval: 2000,
            extractionMethod: 'mock', // Will be updated when tools are available
            ...config
        };
        
        console.log(`WASM File Watcher initialized with paths: ${this.config.watchPaths.join(', ')}`);
    }

    addChangeHandler(handler: WasmFileChangeHandler): void {
        this.handlers.push(handler);
    }

    removeChangeHandler(handler: WasmFileChangeHandler): void {
        const index = this.handlers.indexOf(handler);
        if (index > -1) {
            this.handlers.splice(index, 1);
        }
    }

    startWatching(): void {
        if (this.isWatching) return;

        this.isWatching = true;
        this.scanInitialFiles();
        
        this.watchInterval = window.setInterval(() => {
            this.scanForChanges();
        }, this.config.pollInterval);

        console.log('WASM file watcher started');
    }

    stopWatching(): void {
        if (!this.isWatching) return;

        this.isWatching = false;
        if (this.watchInterval) {
            clearInterval(this.watchInterval);
            this.watchInterval = undefined;
        }

        console.log('WASM file watcher stopped');
    }

    async scanDirectory(path: string): Promise<WasmComponentConfig[]> {
        console.log(`Scanning directory: ${path} using ${this.config.extractionMethod} method`);
        
        // Try to scan for real WASM files
        const wasmFiles = await this.findWasmFiles(path);
        
        if (wasmFiles.length === 0) {
            throw new Error(`No WASM files found in directory: ${path}`);
        }
        
        console.log(`Found ${wasmFiles.length} WASM files`);
        return await this.extractComponentsFromWasmFiles(wasmFiles);
    }

    private async scanInitialFiles(): Promise<void> {
        try {
            for (const path of this.config.watchPaths) {
                const components = await this.scanDirectory(path);
                
                components.forEach(component => {
                    const filePath = `${path}/${component.name}.wasm`;
                    this.knownFiles.set(filePath, {
                        modified: Date.now(),
                        component
                    });

                    this.notifyHandlers({
                        type: 'added',
                        path: filePath,
                        component
                    });
                });
            }
        } catch (error) {
            console.error('Error scanning initial files:', error);
        }
    }

    private async scanForChanges(): Promise<void> {
        try {
            // Get current WASM files
            const currentFiles = new Set<string>();
            
            for (const path of this.config.watchPaths) {
                const wasmFiles = await this.findWasmFiles(path);
                wasmFiles.forEach(file => currentFiles.add(file));
            }
            
            // Detect removed files
            for (const [filePath, fileInfo] of this.knownFiles.entries()) {
                if (!currentFiles.has(filePath)) {
                    // File was removed
                    console.log(`WASM file removed: ${filePath}`);
                    
                    // Move to missing files tracking
                    this.missingFiles.set(filePath, {
                        component: fileInfo.component,
                        removedAt: Date.now()
                    });
                    
                    // Remove from known files
                    this.knownFiles.delete(filePath);
                    
                    // Notify handlers
                    this.notifyHandlers({
                        type: 'removed',
                        path: filePath,
                        missingComponent: fileInfo.component
                    });
                }
            }
            
            // Detect new or changed files
            for (const filePath of currentFiles) {
                const existing = this.knownFiles.get(filePath);
                
                if (!existing) {
                    // Check if this was a previously missing file being restored
                    const missing = this.missingFiles.get(filePath);
                    if (missing) {
                        console.log(`WASM file restored: ${filePath}`);
                        this.missingFiles.delete(filePath);
                    } else {
                        console.log(`New WASM file detected: ${filePath}`);
                    }
                    
                    // Extract component and add to known files
                    try {
                        const component = await this.extractComponentFromWasm(filePath);
                        if (component) {
                            this.knownFiles.set(filePath, {
                                modified: Date.now(),
                                component
                            });
                            
                            this.notifyHandlers({
                                type: 'added',
                                path: filePath,
                                component
                            });
                        }
                    } catch (error) {
                        console.warn(`Failed to process new file ${filePath}:`, error);
                    }
                }
                // TODO: Detect file changes by checking modification time or hash
            }
            
        } catch (error) {
            console.warn('Error during file change scan:', error);
        }
    }

    private notifyHandlers(event: WasmFileChangeEvent): void {
        this.handlers.forEach(handler => {
            try {
                handler(event);
            } catch (error) {
                console.error('Error in WASM file change handler:', error);
            }
        });
    }

    private getMockComponents(): WasmComponentConfig[] {
        return [
            {
                name: 'http-server',
                path: './components/http-server.wasm',
                description: 'HTTP server component',
                interfaces: [
                    {
                        name: 'wasi:http/incoming-handler',
                        type: 'export',
                        interfaceType: 'wasi:http/incoming-handler',
                        functions: [
                            {
                                name: 'handle',
                                params: [{ name: 'request', type: 'incoming-request' }],
                                returns: [{ name: 'response', type: 'outgoing-response' }]
                            }
                        ]
                    },
                    {
                        name: 'wasi:sockets/tcp',
                        type: 'import',
                        interfaceType: 'wasi:sockets/tcp'
                    }
                ]
            },
            {
                name: 'json-parser',
                path: './components/json-parser.wasm',
                description: 'JSON parsing and manipulation component',
                interfaces: [
                    {
                        name: 'custom:json/parser',
                        type: 'export',
                        interfaceType: 'custom:json/parser',
                        functions: [
                            {
                                name: 'parse',
                                params: [{ name: 'json-string', type: 'string' }],
                                returns: [{ name: 'value', type: 'json-value' }]
                            },
                            {
                                name: 'stringify',
                                params: [{ name: 'value', type: 'json-value' }],
                                returns: [{ name: 'json-string', type: 'string' }]
                            }
                        ]
                    }
                ]
            }
        ];
    }

    private generateRandomComponent(): WasmComponentConfig {
        const names = ['processor', 'validator', 'transformer', 'analyzer', 'filter'];
        const name = names[Math.floor(Math.random() * names.length)] + '-' + Date.now();
        
        return {
            name,
            path: `./components/${name}.wasm`,
            description: `Generated ${name} component`,
            interfaces: [
                {
                    name: `custom:${name}/service`,
                    type: 'export',
                    interfaceType: `custom:${name}/service`,
                    functions: [
                        {
                            name: 'process',
                            params: [{ name: 'input', type: 'string' }],
                            returns: [{ name: 'output', type: 'string' }]
                        }
                    ]
                }
            ]
        };
    }

    getKnownComponents(): WasmComponentConfig[] {
        return Array.from(this.knownFiles.values()).map(entry => entry.component);
    }

    isFileWatched(path: string): boolean {
        return this.knownFiles.has(path);
    }

    private getDefaultWatchPath(): string {
        // Check for CLI argument or environment variable
        const args = typeof process !== 'undefined' ? process.argv : [];
        const watchPathArg = args.find(arg => arg.startsWith('--watch-path='));
        
        if (watchPathArg) {
            return watchPathArg.split('=')[1];
        }
        
        // Check environment variable
        const envPath = typeof process !== 'undefined' ? process.env.WASM_WATCH_PATH : undefined;
        if (envPath) {
            return envPath;
        }
        
        // Default to the workspace directory (relative path from web client)
        return '../workspace/adas-wasm-components';
    }

    private async findWasmFiles(path: string): Promise<string[]> {
        // In browser environment, we'll need to use File System Access API
        // For now, we'll simulate finding files based on the known structure
        
        if (typeof window !== 'undefined' && 'showDirectoryPicker' in window) {
            // Use File System Access API when available
            return await this.findWasmFilesViaFileSystemAPI(path);
        } else {
            // Fallback: assume known file structure from the ADAS components
            return this.getKnownWasmFilePaths(path);
        }
    }

    private async findWasmFilesViaFileSystemAPI(path: string): Promise<string[]> {
        try {
            // Request directory access
            const dirHandle = await (window as any).showDirectoryPicker({
                mode: 'read',
                startIn: 'downloads'
            });
            
            const wasmFiles: string[] = [];
            
            // Recursively scan for WASM files
            await this.scanDirectoryHandle(dirHandle, wasmFiles, '');
            
            return wasmFiles;
        } catch (error) {
            console.warn('File System Access API failed:', error);
            throw new Error('Unable to access file system');
        }
    }

    private async scanDirectoryHandle(
        dirHandle: any, 
        wasmFiles: string[], 
        currentPath: string
    ): Promise<void> {
        for await (const [name, handle] of dirHandle.entries()) {
            const fullPath = currentPath ? `${currentPath}/${name}` : name;
            
            if (handle.kind === 'file' && this.config.extensions.some(ext => name.endsWith(ext))) {
                wasmFiles.push(fullPath);
            } else if (handle.kind === 'directory') {
                await this.scanDirectoryHandle(handle, wasmFiles, fullPath);
            }
        }
    }

    private getKnownWasmFilePaths(basePath: string): string[] {
        // Known ADAS component files from the structure we discovered
        const knownComponents = [
            'build/localization.wasm',
            'build/perception.wasm', 
            'build/planning.wasm',
            'build/control.wasm',
            'build/sensor-fusion.wasm',
            'build/camera-front-ecu.wasm',
            'build/camera-surround-ecu.wasm',
            'build/radar-front-ecu.wasm',
            'build/radar-corner-ecu.wasm',
            'build/lidar-ecu.wasm',
            'build/ultrasonic-ecu.wasm',
            'build/object-detection-ai.wasm',
            'build/tracking-prediction-ai.wasm',
            'build/computer-vision-ai.wasm',
            'build/behavior-prediction-ai.wasm',
            'build/sensor-fusion-ecu.wasm',
            'build/perception-fusion.wasm'
        ];
        
        return knownComponents.map(file => `${basePath}/${file}`);
    }

    private async extractComponentsFromWasmFiles(wasmFiles: string[]): Promise<WasmComponentConfig[]> {
        const components: WasmComponentConfig[] = [];
        
        for (const filePath of wasmFiles) {
            try {
                const component = await this.extractComponentFromWasm(filePath);
                if (component) {
                    components.push(component);
                }
            } catch (error) {
                console.warn(`Failed to extract component from ${filePath}:`, error);
                // Continue with other files instead of failing completely
            }
        }
        
        if (components.length === 0) {
            throw new Error('No valid WASM components could be extracted');
        }
        
        return components;
    }

    private async extractComponentFromWasm(filePath: string): Promise<WasmComponentConfig | null> {
        const fileName = filePath.split('/').pop()?.replace('.wasm', '') || 'unknown';
        
        switch (this.config.extractionMethod) {
            case 'wasm-tools':
                return await this.extractWithWasmTools(filePath);
            case 'custom-tool':
                return await this.extractWithCustomTool(filePath);
            case 'mock':
            default:
                // For now, create a basic component config based on filename
                return this.createComponentFromFilename(fileName, filePath);
        }
    }

    private async extractWithWasmTools(filePath: string): Promise<WasmComponentConfig | null> {
        // TODO: Implement wasm-tools integration
        // This will use wasm-tools to extract WIT interfaces and metadata
        console.log(`Extracting with wasm-tools: ${filePath}`);
        throw new Error('wasm-tools extraction not yet implemented');
    }

    private async extractWithCustomTool(filePath: string): Promise<WasmComponentConfig | null> {
        // TODO: Implement custom tool integration
        // This will use the custom tool provided later
        console.log(`Extracting with custom tool: ${filePath}`);
        throw new Error('Custom tool extraction not yet implemented');
    }

    private createComponentFromFilename(name: string, path: string): WasmComponentConfig {
        // Create basic component config based on filename patterns
        const componentName = name.replace(/-/g, '_');
        
        return {
            name: componentName,
            path: path,
            description: `ADAS ${name} component`,
            interfaces: [
                {
                    name: `adas:${name.replace(/-/g, '_')}/component`,
                    type: 'export',
                    interfaceType: `adas:${name.replace(/-/g, '_')}/component`,
                    functions: [
                        {
                            name: 'process',
                            params: [{ name: 'input', type: 'sensor-data' }],
                            returns: [{ name: 'output', type: 'processed-data' }]
                        }
                    ]
                }
            ]
        };
    }

    // Method to update extraction method after tools become available
    setExtractionMethod(method: 'wasm-tools' | 'custom-tool', customToolPath?: string): void {
        this.config.extractionMethod = method;
        if (customToolPath) {
            this.config.customToolPath = customToolPath;
        }
        console.log(`Updated extraction method to: ${method}`);
    }

    // Get all missing components (files that were removed but components still in diagram)
    getMissingComponents(): Array<{ path: string; component: WasmComponentConfig; removedAt: number }> {
        return Array.from(this.missingFiles.entries()).map(([path, info]) => ({
            path,
            component: info.component,
            removedAt: info.removedAt
        }));
    }

    // Check if a component is missing its file
    isComponentMissing(componentName: string): boolean {
        return Array.from(this.missingFiles.values()).some(
            info => info.component.name === componentName
        );
    }

    // Get missing component info by name
    getMissingComponentInfo(componentName: string): { path: string; component: WasmComponentConfig; removedAt: number } | null {
        for (const [path, info] of this.missingFiles.entries()) {
            if (info.component.name === componentName) {
                return { path, component: info.component, removedAt: info.removedAt };
            }
        }
        return null;
    }

    // Permanently remove a missing component from tracking
    removeMissingComponent(componentName: string): boolean {
        for (const [path, info] of this.missingFiles.entries()) {
            if (info.component.name === componentName) {
                this.missingFiles.delete(path);
                console.log(`Permanently removed missing component: ${componentName}`);
                return true;
            }
        }
        return false;
    }

    // Clear all missing components (useful for cleanup)
    clearMissingComponents(): void {
        const count = this.missingFiles.size;
        this.missingFiles.clear();
        console.log(`Cleared ${count} missing components from tracking`);
    }

    // Get combined status of all components (known + missing)
    getAllComponentsStatus(): Array<{
        component: WasmComponentConfig;
        status: 'available' | 'missing';
        path: string;
        lastSeen?: number;
        removedAt?: number;
    }> {
        const result: Array<{
            component: WasmComponentConfig;
            status: 'available' | 'missing';
            path: string;
            lastSeen?: number;
            removedAt?: number;
        }> = [];

        // Add available components
        for (const [path, info] of this.knownFiles.entries()) {
            result.push({
                component: info.component,
                status: 'available',
                path,
                lastSeen: info.modified
            });
        }

        // Add missing components
        for (const [path, info] of this.missingFiles.entries()) {
            result.push({
                component: info.component,
                status: 'missing',
                path,
                removedAt: info.removedAt
            });
        }

        return result;
    }
}