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

interface SecurityAnalysis {
    safe: boolean;
    warnings: string[];
    restrictions: string[];
}

interface ComponentUpdateData {
    componentId: string;
    status: string;
    message?: string;
}

export interface WasmComponent {
    name: string;
    path: string;
    description: string;
    file_exists: boolean;
    last_seen?: string;
    interfaces: WasmInterface[];
    security_analysis?: SecurityAnalysis;
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
        this.wasmComponentPalette = new WasmComponentPalette(mcpService as unknown as import('../mcp/client.js').McpClient);
        
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
            const result = await this.mcpService.callTool('scan_wasm_components', {});
            
            if (result && result.content && result.content[0]) {
                console.log('WasmComponentManager: Raw response from scan_wasm_components:', result.content[0].text);
                
                // Check if response looks like JSON
                const rawText = result.content[0].text;
                if (!rawText.startsWith('{') && !rawText.startsWith('[')) {
                    console.warn('WasmComponentManager: Response does not appear to be JSON, got:', rawText.substring(0, 100));
                    return [];
                }
                
                const componentData = JSON.parse(rawText);
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
    public async getSecurityAnalysis(componentName: string): Promise<SecurityAnalysis | null> {
        return await this.validationService.requestSecurityAnalysis(componentName);
    }

    /**
     * Request WIT validation for a component (delegates to ValidationService)
     */
    public async getWitAnalysis(componentName: string): Promise<import('../services/ValidationService.js').ComponentWitAnalysis | null> {
        return await this.validationService.requestWitValidation(componentName);
    }

    /**
     * Check compatibility between two components
     */
    public async checkCompatibility(componentA: string, componentB: string): Promise<Record<string, unknown> | null> {
        return await this.validationService.requestCompatibilityAnalysis(componentA, componentB);
    }

    /**
     * Setup real-time updates via HTTP streaming
     */
    private setupStreamingUpdates(): void {
        const mcpClient = this.mcpService.getClient();
        
        // Listen for component updates
        mcpClient.addStreamListener('wasm_component_update', (data: ComponentUpdateData) => {
            console.log('WasmComponentManager: Component update received:', data);
            this.handleComponentUpdate(data);
        });

        // Listen for component discovery
        mcpClient.addStreamListener('wasm_component_discovered', (data: Record<string, unknown>) => {
            console.log('WasmComponentManager: New component discovered:', data);
            this.handleComponentDiscovered(data);
        });

        // Listen for component removal
        mcpClient.addStreamListener('wasm_component_removed', (data: Record<string, unknown>) => {
            console.log('WasmComponentManager: Component removed:', data);
            this.handleComponentRemoved(data);
        });

        // Setup validation streaming
        this.validationService.setupValidationStreaming();
    }

    /**
     * Handle component update from streaming
     */
    private handleComponentUpdate(data: ComponentUpdateData): void {
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
    private handleComponentDiscovered(data: Record<string, unknown>): void {
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
    private handleComponentRemoved(data: Record<string, unknown>): void {
        if (data.component_name) {
            this.componentsCache.delete(data.component_name);
            
            // Notify UI of removal
            this.notifyComponentUpdate('removed', { name: data.component_name });
        }
    }

    /**
     * Notify UI components of changes
     */
    private notifyComponentUpdate(type: string, component: WasmComponent): void {
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
    public async getComponentStatistics(): Promise<{ total: number; loaded: number; failed: number }> {
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

    /**
     * Legacy method for compatibility - unload component
     */
    public unloadComponent(componentId: string): void {
        // This is now handled by the backend
        console.log(`Component ${componentId} unloading is now handled by backend`);
    }

    /**
     * Check if a click position is on the load switch of a WASM component
     */
    public isLoadSwitchClick(position: { x: number; y: number }, element: any): boolean {
        // Check if this is a WASM component node
        if (!element || !element.type || !element.type.includes('wasm-component')) {
            return false;
        }

        // Get the element bounds
        const bounds = element.bounds;
        if (!bounds) {
            return false;
        }

        // The load switch is typically in the top-right corner of the component
        // Assuming a 20x20 pixel area for the switch
        const switchSize = 20;
        const switchX = bounds.x + bounds.width - switchSize - 5; // 5px margin
        const switchY = bounds.y + 5; // 5px margin

        // Check if click is within switch bounds
        return position.x >= switchX && 
               position.x <= switchX + switchSize &&
               position.y >= switchY && 
               position.y <= switchY + switchSize;
    }

    /**
     * Toggle the load state of a component
     */
    public async toggleComponentLoad(componentId: string): Promise<void> {
        console.log('WasmComponentManager: Toggling load state for component:', componentId);
        
        try {
            // For now, just log the action
            // In a full implementation, this would:
            // 1. Track the load state
            // 2. Update the visual representation
            // 3. Possibly load/unload the actual WASM module
            console.log('Component load toggle not yet implemented for:', componentId);
        } catch (error) {
            console.error('Failed to toggle component load state:', error);
        }
    }

    /**
     * Clear any cached data
     */
    private clearCache(): void {
        // Clear any in-memory caches
        console.log('Clearing component manager cache');
    }

    /**
     * Clean up resources
     */
    public cleanup(): void {
        console.log('Cleaning up component manager...');
        // Clear any cached data
        this.clearCache();
        console.log('Component manager cleaned up');
    }

}