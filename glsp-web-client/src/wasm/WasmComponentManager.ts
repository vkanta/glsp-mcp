import { WasmComponentPalette } from '../diagrams/wasm-component-palette.js';
import { McpService } from '../services/McpService.js';
import { DiagramService } from '../services/DiagramService.js';
import { WasmComponentRenderer } from '../diagrams/wasm-component-renderer.js';
import { ModelElement } from '../model/diagram.js';
import { CanvasRenderer } from '../renderer/canvas-renderer.js';

export class WasmComponentManager {
    private wasmComponentPalette: WasmComponentPalette;
    protected mcpService: McpService;
    private diagramService: DiagramService;
    private renderer?: CanvasRenderer;
    private loadedComponents: Map<string, any> = new Map(); // Track loaded WASM components
    private jcoRuntime?: any; // Will hold jco/wasi-gfx runtime

    constructor(mcpService: McpService, diagramService: DiagramService, renderer?: CanvasRenderer) {
        this.mcpService = mcpService;
        this.diagramService = diagramService;
        this.renderer = renderer;
        this.wasmComponentPalette = new WasmComponentPalette(mcpService as any);
    }

    public getPaletteElement(): HTMLElement {
        return this.wasmComponentPalette.getElement();
    }

    public async showPalette(): Promise<void> {
        await this.wasmComponentPalette.show();
    }

    public hidePalette(): void {
        this.wasmComponentPalette.hide();
    }

    public async initializeWasmComponents(): Promise<void> {
        try {
            await this.mcpService.callTool('scan_wasm_components', {});
            console.log('WASM components scanned via MCP');
        } catch (error) {
            console.warn('WASM component initialization failed:', error);
        }
    }

    public setupCanvasDragAndDrop(canvas: HTMLCanvasElement): void {
        canvas.addEventListener('dragover', (e) => {
            e.preventDefault();
            e.dataTransfer!.dropEffect = 'copy';
        });
        
        canvas.addEventListener('drop', async (e) => {
            e.preventDefault();
            
            const wasmComponentData = e.dataTransfer!.getData('application/wasm-component');
            if (wasmComponentData) {
                try {
                    const dragData = JSON.parse(wasmComponentData);
                    const canvasRect = canvas.getBoundingClientRect();
                    const position = {
                        x: e.clientX - canvasRect.left,
                        y: e.clientY - canvasRect.top
                    };
                    
                    await this.addWasmComponentToDiagram(dragData.componentName, position);
                } catch (error) {
                    console.error('Failed to add WASM component to diagram:', error);
                    this.wasmComponentPalette.onComponentAddFailed('unknown', String(error));
                }
            }
        });
    }

    private async addWasmComponentToDiagram(componentName: string, position: { x: number; y: number }): Promise<void> {
        const diagramId = this.diagramService.getCurrentDiagramId();
        if (!diagramId) {
            throw new Error('No active diagram');
        }
        
        try {
            const result = await this.mcpService.callTool('load_wasm_component', {
                diagramId: diagramId,
                componentName: componentName,
                position: position
            });
            
            console.log('WASM component added:', result);
            this.wasmComponentPalette.onComponentAdded(componentName);
            
            // Reload the diagram and update the renderer
            const updatedDiagram = await this.diagramService.loadDiagram(diagramId);
            console.log('WasmComponentManager: Diagram reloaded after component add:', !!updatedDiagram);
            
            if (updatedDiagram && this.renderer) {
                console.log('WasmComponentManager: Updating renderer with new diagram');
                this.renderer.setDiagram(updatedDiagram);
            } else {
                console.warn('WasmComponentManager: Could not update renderer - missing diagram or renderer', {
                    hasDiagram: !!updatedDiagram,
                    hasRenderer: !!this.renderer
                });
            }
            
        } catch (error) {
            console.error('Failed to add WASM component:', error);
            this.wasmComponentPalette.onComponentAddFailed(componentName, String(error));
            throw error;
        }
    }

    // Check if a click position is on a component's load switch
    // NOTE: V2 design uses status indicator only (no clickable switch)
    public isLoadSwitchClick(position: { x: number; y: number }, element: ModelElement): boolean {
        // V2 renderer doesn't have a clickable load switch - status is informational only
        return false;
    }

    // Handle load/unload switch toggle
    public async toggleComponentLoad(elementId: string): Promise<void> {
        const diagramId = this.diagramService.getCurrentDiagramId();
        if (!diagramId) return;
        
        const diagram = this.diagramService.getDiagramState().getDiagram(diagramId);
        if (!diagram) return;
        
        const element = diagram.elements[elementId];
        if (!element) return;
        
        const isCurrentlyLoaded = element.properties?.isLoaded === true;
        const componentName = element.properties?.componentName?.toString();
        
        if (!componentName) {
            console.error('Component name not found for element:', elementId);
            return;
        }
        
        try {
            if (isCurrentlyLoaded) {
                await this.unloadComponent(elementId, componentName);
            } else {
                await this.loadComponent(elementId, componentName);
            }
            
            // Update the diagram to reflect the new state
            const diagramId = this.diagramService.getCurrentDiagramId();
            if (diagramId) {
                await this.diagramService.loadDiagram(diagramId);
            }
        } catch (error) {
            console.error('Failed to toggle component load state:', error);
        }
    }

    // Load a WASM component using jco wasi-gfx framework
    private async loadComponent(elementId: string, componentName: string): Promise<void> {
        console.log(`Loading WASM component: ${componentName}`);
        
        try {
            // Initialize jco runtime if not already done
            if (!this.jcoRuntime) {
                await this.initializeJcoRuntime();
            }
            
            // Get component path from backend
            const componentPath = await this.getComponentPath(componentName);
            
            // Load the WASM component using jco
            const wasmModule = await this.loadWasmModule(componentPath);
            
            // Store the loaded component
            this.loadedComponents.set(elementId, {
                name: componentName,
                module: wasmModule,
                path: componentPath,
                loadedAt: new Date().toISOString()
            });
            
            // Update component state in backend
            await this.mcpService.callTool('update_component_state', {
                elementId,
                isLoaded: true,
                loadedAt: new Date().toISOString()
            });
            
            console.log(`Successfully loaded WASM component: ${componentName}`);
            
        } catch (error) {
            console.error(`Failed to load WASM component ${componentName}:`, error);
            throw error;
        }
    }

    // Unload a WASM component
    protected async unloadComponent(elementId: string, componentName: string): Promise<void> {
        console.log(`Unloading WASM component: ${componentName}`);
        
        try {
            // Remove from loaded components
            this.loadedComponents.delete(elementId);
            
            // Update component state in backend  
            await this.mcpService.callTool('update_component_state', {
                elementId,
                isLoaded: false,
                unloadedAt: new Date().toISOString()
            });
            
            console.log(`Successfully unloaded WASM component: ${componentName}`);
            
        } catch (error) {
            console.error(`Failed to unload WASM component ${componentName}:`, error);
            throw error;
        }
    }

    // Initialize jco wasi-gfx runtime
    private async initializeJcoRuntime(): Promise<void> {
        try {
            // For now, use a placeholder - will need actual jco imports
            // TODO: Import actual jco wasi-gfx framework
            console.log('Initializing jco wasi-gfx runtime...');
            
            // Placeholder for jco runtime initialization
            this.jcoRuntime = {
                version: '1.0.0',
                initialized: true,
                loadModule: async (path: string) => {
                    console.log(`Mock loading WASM module from: ${path}`);
                    return { loaded: true, path };
                }
            };
            
            console.log('jco wasi-gfx runtime initialized');
        } catch (error) {
            console.error('Failed to initialize jco runtime:', error);
            throw error;
        }
    }

    // Get component file path from backend
    private async getComponentPath(componentName: string): Promise<string> {
        try {
            const result = await this.mcpService.callTool('get_component_path', {
                componentName
            });
            
            // Extract path from result
            const path = result.content?.[0]?.text;
            if (!path) {
                throw new Error(`Component path not found for: ${componentName}`);
            }
            
            return path;
        } catch (error) {
            console.error(`Failed to get component path for ${componentName}:`, error);
            throw error;
        }
    }

    // Load WASM module using jco
    private async loadWasmModule(path: string): Promise<any> {
        try {
            if (!this.jcoRuntime) {
                throw new Error('jco runtime not initialized');
            }
            
            // Use jco to load the WASM module
            const module = await this.jcoRuntime.loadModule(path);
            
            return module;
        } catch (error) {
            console.error(`Failed to load WASM module from ${path}:`, error);
            throw error;
        }
    }

    // Get loaded component for execution view
    public getLoadedComponent(elementId: string): any {
        return this.loadedComponents.get(elementId);
    }

    // Check if component is loaded
    public isComponentLoaded(elementId: string): boolean {
        return this.loadedComponents.has(elementId);
    }

    // Get all loaded components
    public getLoadedComponents(): Map<string, any> {
        return new Map(this.loadedComponents);
    }
}