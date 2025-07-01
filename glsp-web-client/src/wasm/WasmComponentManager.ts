/**
 * Thin Client WASM Component Manager
 * 
 * Simplified version that acts as a proxy to backend services.
 * Part of the architecture transformation to move business logic to the backend.
 * 
 * Reduced from 600+ lines to ~150 lines by removing:
 * - Client-side validation
 * - Client-side transpilation
 * - Complex component introspection
 * - Heavy change notification system
 */

import { WasmComponentPalette } from '../diagrams/wasm-component-palette.js';
import { McpService } from '../services/McpService.js';
import { DiagramService } from '../services/DiagramService.js';
import { ValidationService } from '../services/ValidationService.js';
import { CanvasRenderer } from '../renderer/canvas-renderer.js';

export interface WasmComponent {
    name: string;
    path: string;
    description: string;
    file_exists: boolean;
    last_seen?: string;
    interfaces: WasmInterface[];
    security_analysis?: any;
    last_security_scan?: string;
}

export interface WasmInterface {
    name: string;
    interface_type: string;
    functions: WasmFunction[];
}

export interface WasmFunction {
    name: string;
    params: WasmParam[];
    returns: WasmParam[];
}

export interface WasmParam {
    name: string;
    param_type: string;
}

export class WasmComponentManager {
    private wasmComponentPalette: WasmComponentPalette;
    protected mcpService: McpService;
    protected diagramService: DiagramService;
    private validationService: ValidationService;
    private renderer?: CanvasRenderer;
    private componentsCache: Map<string, WasmComponent> = new Map();
    private cacheTimeout = 5 * 60 * 1000; // 5 minutes
    private lastCacheUpdate = 0;

    constructor(mcpService: McpService, diagramService: DiagramService, renderer?: CanvasRenderer) {
        this.mcpService = mcpService;
        this.diagramService = diagramService;
        this.renderer = renderer;
        this.validationService = new ValidationService(mcpService);
        this.wasmComponentPalette = new WasmComponentPalette(mcpService as any);
        
        // Setup real-time updates via streaming
        this.setupStreamingUpdates();
    }

    /**
     * Get the palette UI element
     */
    public getPaletteElement(): HTMLElement {
        return this.wasmComponentPalette.getElement();
    }

    /**
     * Show the component palette
     */
    public async showPalette(): Promise<void> {
        await this.wasmComponentPalette.show();
    }

    /**
     * Hide the component palette
     */
    public hidePalette(): void {
        this.wasmComponentPalette.hide();
    }

    /**
     * Initialize WASM components (triggers backend scan)
     */
    public async initializeWasmComponents(): Promise<void> {
        console.log('WasmComponentManager: Initializing components...');
        
        try {
            // Trigger backend component scan
            await this.mcpService.callTool('scan_wasm_components', {});
            
            // Load component list
            await this.refreshComponentList();
            
            console.log('WasmComponentManager: Components initialized');
        } catch (error) {
            console.error('WasmComponentManager: Initialization failed:', error);
        }
    }

    /**
     * Get list of available WASM components from backend
     */
    public async getComponents(): Promise<WasmComponent[]> {
        // Check cache first
        if (this.isCacheValid()) {
            console.log('WasmComponentManager: Using cached component list');
            return Array.from(this.componentsCache.values());
        }

        return await this.refreshComponentList();
    }

    /**
     * Get a specific component by name
     */
    public async getComponent(name: string): Promise<WasmComponent | null> {
        const components = await this.getComponents();
        return components.find(comp => comp.name === name) || null;
    }

    /**
     * Refresh component list from backend
     */
    private async refreshComponentList(): Promise<WasmComponent[]> {
        console.log('WasmComponentManager: Refreshing component list from backend...');
        
        try {
            const result = await this.mcpService.callTool('list_wasm_components', {});
            
            if (result && result.content && result.content[0]) {
                const componentData = JSON.parse(result.content[0].text);
                const components: WasmComponent[] = componentData.components || [];
                
                // Update cache
                this.updateCache(components);
                
                console.log(`WasmComponentManager: Loaded ${components.length} components`);
                return components;
            }
            
            return [];
        } catch (error) {
            console.error('WasmComponentManager: Failed to refresh component list:', error);
            return [];
        }
    }

    /**
     * Request security analysis for a component (delegates to ValidationService)
     */
    public async getSecurityAnalysis(componentName: string): Promise<any | null> {
        return await this.validationService.requestSecurityAnalysis(componentName);
    }

    /**
     * Request WIT validation for a component (delegates to ValidationService)
     */
    public async getWitAnalysis(componentName: string): Promise<any | null> {
        return await this.validationService.requestWitValidation(componentName);
    }

    /**
     * Check compatibility between two components
     */
    public async checkCompatibility(componentA: string, componentB: string): Promise<any | null> {
        return await this.validationService.requestCompatibilityAnalysis(componentA, componentB);
    }

    /**
     * Setup real-time updates via HTTP streaming
     */
    private setupStreamingUpdates(): void {
        const mcpClient = this.mcpService.getClient();
        
        // Listen for component updates
        mcpClient.addStreamListener('wasm_component_update', (data: any) => {
            console.log('WasmComponentManager: Component update received:', data);
            this.handleComponentUpdate(data);
        });

        // Listen for component discovery
        mcpClient.addStreamListener('wasm_component_discovered', (data: any) => {
            console.log('WasmComponentManager: New component discovered:', data);
            this.handleComponentDiscovered(data);
        });

        // Listen for component removal
        mcpClient.addStreamListener('wasm_component_removed', (data: any) => {
            console.log('WasmComponentManager: Component removed:', data);
            this.handleComponentRemoved(data);
        });

        // Setup validation streaming
        this.validationService.setupValidationStreaming();
    }

    /**
     * Handle component update from streaming
     */
    private handleComponentUpdate(data: any): void {
        if (data.component) {
            const component = data.component as WasmComponent;
            this.componentsCache.set(component.name, component);
            
            // Notify UI of update
            this.notifyComponentUpdate('updated', component);
        }
    }

    /**
     * Handle new component discovery
     */
    private handleComponentDiscovered(data: any): void {
        if (data.component) {
            const component = data.component as WasmComponent;
            this.componentsCache.set(component.name, component);
            
            // Notify UI of new component
            this.notifyComponentUpdate('discovered', component);
        }
    }

    /**
     * Handle component removal
     */
    private handleComponentRemoved(data: any): void {
        if (data.component_name) {
            this.componentsCache.delete(data.component_name);
            
            // Notify UI of removal
            this.notifyComponentUpdate('removed', { name: data.component_name });
        }
    }

    /**
     * Notify UI components of changes
     */
    private notifyComponentUpdate(type: string, component: any): void {
        const event = new CustomEvent('wasm-component-update', {
            detail: { type, component }
        });
        window.dispatchEvent(event);
    }

    /**
     * Update component cache
     */
    private updateCache(components: WasmComponent[]): void {
        this.componentsCache.clear();
        for (const component of components) {
            this.componentsCache.set(component.name, component);
        }
        this.lastCacheUpdate = Date.now();
    }

    /**
     * Check if cache is still valid
     */
    private isCacheValid(): boolean {
        return Date.now() - this.lastCacheUpdate < this.cacheTimeout && this.componentsCache.size > 0;
    }

    /**
     * Get component statistics (for UI display)
     */
    public async getComponentStatistics(): Promise<any> {
        try {
            const result = await this.mcpService.callTool('get_component_statistics', {});
            
            if (result && result.content && result.content[0]) {
                return JSON.parse(result.content[0].text);
            }
            
            return null;
        } catch (error) {
            console.error('WasmComponentManager: Failed to get statistics:', error);
            return null;
        }
    }

    /**
     * Trigger a full component rescan
     */
    public async rescanComponents(): Promise<boolean> {
        console.log('WasmComponentManager: Triggering component rescan...');
        
        try {
            await this.mcpService.callTool('rescan_wasm_components', {});
            
            // Clear cache to force refresh
            this.componentsCache.clear();
            this.lastCacheUpdate = 0;
            
            // Refresh component list
            await this.refreshComponentList();
            
            console.log('WasmComponentManager: Rescan completed');
            return true;
        } catch (error) {
            console.error('WasmComponentManager: Rescan failed:', error);
            return false;
        }
    }

    /**
     * Check if streaming is active
     */
    public isStreamingActive(): boolean {
        return this.mcpService.getClient().isStreaming();
    }

    /**
     * Get validation service for direct access
     */
    public getValidationService(): ValidationService {
        return this.validationService;
    }
}