import { WasmComponentManager } from './WasmComponentManager.js';
import { WasmTranspiler } from './transpiler/WasmTranspiler.js';
import { ComponentRegistry } from './runtime/ComponentRegistry.js';
import { ExecutionEngine, ExecutionContext, ExecutionResult } from './runtime/ExecutionEngine.js';
import { ComponentUploadPanel } from './ui/ComponentUploadPanel.js';
import { McpService } from '../services/McpService.js';
import { DiagramService } from '../services/DiagramService.js';
import { CanvasRenderer } from '../renderer/canvas-renderer.js';

export interface WasmRuntimeConfig {
    maxConcurrentExecutions?: number;
    maxCachedComponents?: number;
    enableClientSideTranspilation?: boolean;
}

export class WasmRuntimeManager extends WasmComponentManager {
    private transpiler: WasmTranspiler;
    private registry: ComponentRegistry;
    private executionEngine: ExecutionEngine;
    private uploadPanel: ComponentUploadPanel;
    private config: WasmRuntimeConfig;

    constructor(
        mcpService: McpService, 
        diagramService: DiagramService,
        config: WasmRuntimeConfig = {},
        renderer?: CanvasRenderer
    ) {
        super(mcpService, diagramService, renderer);
        
        this.config = {
            maxConcurrentExecutions: 5,
            maxCachedComponents: 50,
            enableClientSideTranspilation: true,
            ...config
        };

        // Initialize new runtime services
        this.transpiler = new WasmTranspiler();
        this.registry = new ComponentRegistry();
        this.executionEngine = new ExecutionEngine(this.registry);
        
        // Configure execution engine
        this.executionEngine.setMaxConcurrentExecutions(this.config.maxConcurrentExecutions!);

        // Create upload panel
        this.uploadPanel = new ComponentUploadPanel(
            this.transpiler,
            this.registry,
            (componentId) => this.onComponentUploadComplete(componentId),
            (error) => this.onComponentUploadError(error)
        );

        document.body.appendChild(this.uploadPanel.getElement());
    }

    // Enhanced component loading with client-side transpilation support
    async uploadAndTranspileComponent(file: File, componentName?: string): Promise<string> {
        if (!this.config.enableClientSideTranspilation) {
            throw new Error('Client-side transpilation is disabled');
        }

        console.log('Uploading and transpiling component...', { fileName: file.name, size: file.size });

        try {
            // Read file as ArrayBuffer
            const arrayBuffer = await this.readFileAsArrayBuffer(file);
            
            // Transpile component
            const transpiledComponent = await this.transpiler.transpileComponent(
                arrayBuffer, 
                componentName || file.name.replace('.wasm', '')
            );

            // Register component
            const componentId = this.registry.registerComponent(transpiledComponent);
            
            console.log(`Component uploaded and registered: ${transpiledComponent.metadata.name} (${componentId})`);
            return componentId;
        } catch (error) {
            console.error('Failed to upload and transpile component:', error);
            throw error;
        }
    }

    async loadTranspiledComponent(componentId: string): Promise<any> {
        console.log(`Loading transpiled component: ${componentId}`);
        
        try {
            const instance = await this.registry.loadComponent(componentId);
            console.log(`Transpiled component loaded successfully: ${componentId}`);
            return instance;
        } catch (error) {
            console.error(`Failed to load transpiled component ${componentId}:`, error);
            throw error;
        }
    }

    async executeComponent(componentId: string, method: string, args: any[] = [], timeout?: number): Promise<ExecutionResult> {
        console.log(`Executing component method: ${componentId}.${method}`, { args, timeout });

        const context: ExecutionContext = {
            componentId,
            method,
            args,
            timeout,
            onProgress: (progress) => {
                console.log(`Execution progress: ${progress.stage} - ${progress.message} (${progress.progress}%)`);
            }
        };

        try {
            const result = await this.executionEngine.executeComponent(context);
            console.log(`Component execution completed:`, { 
                success: result.success, 
                executionTime: result.executionTime 
            });
            return result;
        } catch (error) {
            console.error(`Component execution failed:`, error);
            throw error;
        }
    }

    async getComponentMethods(componentId: string) {
        return await this.executionEngine.getComponentMethods(componentId);
    }

    async testComponent(componentId: string) {
        return await this.executionEngine.testComponent(componentId);
    }

    // UI Management
    showUploadPanel(): void {
        this.uploadPanel.show();
    }

    hideUploadPanel(): void {
        this.uploadPanel.hide();
    }

    // Component management
    getRegisteredComponents() {
        return this.registry.listComponents();
    }

    getLoadedComponents(): Map<string, any> {
        const loadedComponentsMetadata = this.registry.getLoadedComponents();
        const result = new Map();
        loadedComponentsMetadata.forEach((metadata) => {
            result.set(metadata.hash, { metadata, isLoaded: true });
        });
        return result;
    }

    getComponentStatus(componentId: string) {
        return this.registry.getComponentStatus(componentId);
    }

    // Override parent method signature for compatibility
    async unloadComponent(elementId: string, componentName?: string): Promise<void> {
        console.log(`Unloading component: ${elementId}`, { componentName });
        
        // If this is called for a client-side component, use elementId as componentId
        if (componentName === undefined) {
            // New client-side component
            this.registry.unloadComponent(elementId);
        } else {
            // Legacy MCP component - call parent method
            await super.unloadComponent(elementId, componentName);
        }
    }

    async unloadClientSideComponent(componentId: string): Promise<boolean> {
        console.log(`Unloading client-side component: ${componentId}`);
        return this.registry.unloadComponent(componentId);
    }

    async removeComponent(componentId: string): Promise<boolean> {
        console.log(`Removing component: ${componentId}`);
        return this.registry.removeComponent(componentId);
    }

    // Enhanced initialization that supports both MCP and client-side components
    async initializeEnhancedWasmComponents(): Promise<void> {
        console.log('Initializing enhanced WASM components...');

        try {
            // Initialize legacy MCP-based components
            await super.initializeWasmComponents();
            console.log('Legacy MCP components initialized');

            // Load any pre-configured client-side components
            await this.loadPreconfiguredComponents();
            console.log('Pre-configured components loaded');

            console.log('Enhanced WASM components initialized successfully');
        } catch (error) {
            console.error('Failed to initialize enhanced WASM components:', error);
            throw error;
        }
    }

    private async loadPreconfiguredComponents(): Promise<void> {
        // In a real implementation, this might load components from local storage
        // or a configuration file
        console.log('Loading pre-configured components...');
        
        // For now, just log that we're ready to accept uploaded components
        const cachedComponents = this.transpiler.getCachedComponents();
        console.log(`Found ${cachedComponents.length} cached components from previous sessions`);
        
        for (const component of cachedComponents) {
            this.registry.registerComponent(component);
            console.log(`Restored cached component: ${component.metadata.name}`);
        }
    }

    // Enhanced palette integration
    async showEnhancedPalette(): Promise<void> {
        // Show both traditional palette and upload option
        await super.showPalette();
        
        // Could add upload button to the palette here
        this.addUploadButtonToPalette();
    }

    private addUploadButtonToPalette(): void {
        const paletteElement = this.getPaletteElement();
        
        // Check if upload button already exists
        if (paletteElement.querySelector('.upload-component-btn')) {
            return;
        }

        const uploadButton = document.createElement('button');
        uploadButton.className = 'upload-component-btn';
        uploadButton.textContent = '📁 Upload Component';
        uploadButton.style.cssText = `
            width: 100%;
            padding: 8px 12px;
            margin-bottom: 10px;
            background: linear-gradient(90deg, #4A9EFF, #00D4AA);
            border: none;
            border-radius: 4px;
            color: white;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
        `;

        uploadButton.addEventListener('click', () => {
            this.showUploadPanel();
        });

        // Insert at the top of the palette
        const paletteContent = paletteElement.querySelector('.wasm-components-list');
        if (paletteContent) {
            paletteContent.insertBefore(uploadButton, paletteContent.firstChild);
        }
    }

    // Event handlers
    private onComponentUploadComplete(componentId: string): void {
        console.log(`Component upload completed: ${componentId}`);
        
        // Refresh any UI that shows component lists
        this.refreshComponentDisplays();
        
        // Show success message
        this.showNotification(`Component uploaded successfully!`, 'success');
    }

    private onComponentUploadError(error: string): void {
        console.error(`Component upload failed: ${error}`);
        this.showNotification(`Upload failed: ${error}`, 'error');
    }

    private refreshComponentDisplays(): void {
        // This would refresh any UI components that display the component list
        console.log('Refreshing component displays...');
        
        // Update palette if it's showing
        // Could trigger a refresh of the component library panel here
    }

    private showNotification(message: string, type: 'success' | 'error' | 'info'): void {
        // Simple notification system
        const notification = document.createElement('div');
        notification.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            padding: 12px 20px;
            border-radius: 6px;
            color: white;
            font-weight: 500;
            z-index: 10000;
            animation: slideIn 0.3s ease;
            ${type === 'success' ? 'background: #10B981;' : 
              type === 'error' ? 'background: #EF4444;' : 
              'background: #3B82F6;'}
        `;
        notification.textContent = message;

        document.body.appendChild(notification);

        // Auto-remove after 3 seconds
        setTimeout(() => {
            notification.style.animation = 'slideOut 0.3s ease';
            setTimeout(() => notification.remove(), 300);
        }, 3000);
    }

    // Utility methods
    private readFileAsArrayBuffer(file: File): Promise<ArrayBuffer> {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = () => resolve(reader.result as ArrayBuffer);
            reader.onerror = () => reject(new Error('Failed to read file'));
            reader.readAsArrayBuffer(file);
        });
    }

    // Runtime statistics
    getRuntimeStats() {
        return {
            transpiler: {
                cachedComponents: this.transpiler.getCacheSize(),
                totalComponents: this.transpiler.getCachedComponents().length
            },
            registry: this.registry.getMemoryUsage(),
            execution: this.executionEngine.getExecutionStats(),
            config: this.config
        };
    }

    // Cleanup
    async cleanup(): Promise<void> {
        console.log('Cleaning up WASM runtime manager...');
        
        this.executionEngine.cleanup();
        this.registry.cleanup();
        this.transpiler.clearCache();
        
        if (this.uploadPanel) {
            this.uploadPanel.hide();
        }
        
        console.log('WASM runtime manager cleaned up');
    }
}