import { WasmComponentPalette } from '../diagrams/wasm-component-palette.js';
import { McpService } from '../services/McpService.js';
import { DiagramService } from '../services/DiagramService.js';
import { WasmComponentRenderer } from '../diagrams/wasm-component-renderer.js';
import { ModelElement } from '../model/diagram.js';
import { CanvasRenderer } from '../renderer/canvas-renderer.js';
import { WasmChangeNotifier, WasmChangeListener } from '../services/WasmChangeNotifier.js';
import { WasmFileChangeEvent } from '../diagrams/wasm-file-watcher.js';

export class WasmComponentManager implements WasmChangeListener {
    private wasmComponentPalette: WasmComponentPalette;
    protected mcpService: McpService;
    private diagramService: DiagramService;
    private renderer?: CanvasRenderer;
    private loadedComponents: Map<string, any> = new Map(); // Track loaded WASM components
    private jcoRuntime?: any; // Will hold jco/wasi-gfx runtime
    private changeNotifier: WasmChangeNotifier;

    constructor(mcpService: McpService, diagramService: DiagramService, renderer?: CanvasRenderer) {
        this.mcpService = mcpService;
        this.diagramService = diagramService;
        this.renderer = renderer;
        this.wasmComponentPalette = new WasmComponentPalette(mcpService as any);
        
        // Initialize MCP notification listener
        this.changeNotifier = new WasmChangeNotifier(mcpService);
        this.changeNotifier.addListener(this);
        this.changeNotifier.start();
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
    
    // Handle real-time WASM file changes
    public onWasmChange(event: WasmFileChangeEvent): void {
        console.log('WASM component change detected:', event);
        
        switch (event.type) {
            case 'added':
                // Refresh the palette to show new component
                this.wasmComponentPalette.refreshComponents();
                break;
                
            case 'removed':
                // Update palette and flag any components in diagrams
                this.wasmComponentPalette.refreshComponents();
                this.flagMissingComponentsInDiagram(event.component?.name || '');
                break;
                
            case 'changed':
                // Refresh palette and update any instances in diagrams
                this.wasmComponentPalette.refreshComponents();
                this.updateComponentsInDiagram(event.component?.name || '');
                break;
        }
    }
    
    private flagMissingComponentsInDiagram(componentName: string): void {
        // Flag components in the current diagram as missing
        const currentDiagram = this.diagramService.getCurrentDiagram();
        if (!currentDiagram) return;
        
        let needsUpdate = false;
        currentDiagram.elements.forEach(element => {
            if (element.type === 'wasm-component' && 
                element.properties?.componentPath?.includes(componentName)) {
                element.properties.isMissing = true;
                needsUpdate = true;
            }
        });
        
        if (needsUpdate && this.renderer) {
            this.renderer.render();
        }
    }
    
    private updateComponentsInDiagram(componentName: string): void {
        // Update components in the current diagram
        const currentDiagram = this.diagramService.getCurrentDiagram();
        if (!currentDiagram) return;
        
        let needsUpdate = false;
        currentDiagram.elements.forEach(element => {
            if (element.type === 'wasm-component' && 
                element.properties?.componentPath?.includes(componentName)) {
                // Clear missing flag if it was set
                if (element.properties.isMissing) {
                    element.properties.isMissing = false;
                    needsUpdate = true;
                }
                // Could also update interfaces here if needed
            }
        });
        
        if (needsUpdate && this.renderer) {
            this.renderer.render();
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
                    console.log('WASM component drop - received drag data:', dragData);
                    
                    const canvasRect = canvas.getBoundingClientRect();
                    const position = {
                        x: e.clientX - canvasRect.left,
                        y: e.clientY - canvasRect.top
                    };
                    
                    // Handle both 'componentName' and 'name' fields for compatibility
                    const componentName = dragData.componentName || dragData.name;
                    if (!componentName) {
                        throw new Error('No component name found in drag data');
                    }
                    
                    console.log(`Adding WASM component "${componentName}" at position:`, position);
                    await this.addWasmComponentToDiagram(componentName, position);
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
            console.log(`Adding WASM component "${componentName}" to diagram at position:`, position);
            
            // Use the load_wasm_component tool which properly adds interface data
            const result = await this.mcpService.callTool('load_wasm_component', {
                diagramId,
                componentName,
                position
            });
            
            console.log('WASM component loaded:', result);
            console.log('Load result content:', result.content);
            
            // The backend should automatically set the appropriate properties
            // for a wasm-component type node based on the component name
            
            console.log('WASM component added successfully');
            this.wasmComponentPalette.onComponentAdded(componentName);
            
            // Reload the diagram and update the renderer
            const updatedDiagram = await this.diagramService.loadDiagram(diagramId);
            console.log('WasmComponentManager: Diagram reloaded after component add:', !!updatedDiagram);
            
            if (updatedDiagram && this.renderer) {
                console.log('WasmComponentManager: Updating renderer with new diagram');
                
                // Debug: Check the newly added component
                const newComponent = Object.values(updatedDiagram.elements).find(
                    el => el.type === 'wasm-component' && (el.label === componentName || el.properties?.componentName === componentName)
                );
                if (newComponent) {
                    console.log('WasmComponentManager: New component data:', newComponent);
                    console.log('WasmComponentManager: Component interfaces:', newComponent.properties?.interfaces);
                }
                
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
    protected async loadComponent(elementId: string, componentName: string): Promise<void> {
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
            
            // Refresh the diagram to show updated state
            await this.refreshDiagramAfterStateChange();
            
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
            
            // Refresh the diagram to show updated state
            await this.refreshDiagramAfterStateChange();
            
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
    
    // Refresh the diagram after component state changes
    private async refreshDiagramAfterStateChange(): Promise<void> {
        const diagramId = this.diagramService.getCurrentDiagramId();
        if (!diagramId) return;
        
        try {
            // Reload the diagram from backend to get updated properties
            const updatedDiagram = await this.diagramService.loadDiagram(diagramId);
            
            if (updatedDiagram && this.renderer) {
                // Update the canvas renderer with the refreshed diagram
                this.renderer.setDiagram(updatedDiagram);
                console.log('Diagram refreshed after component state change');
            }
        } catch (error) {
            console.error('Failed to refresh diagram after state change:', error);
        }
    }

    // WIT Analysis Methods using MCP Resources

    /**
     * Get WIT analysis for all components
     */
    public async getWitInterfacesOverview(): Promise<any> {
        try {
            const result = await this.mcpService.readResource('wasm://wit/interfaces');
            return JSON.parse(result.text || '{}');
        } catch (error) {
            console.error('Failed to get WIT interfaces overview:', error);
            throw error;
        }
    }

    /**
     * Get WIT types catalog
     */
    public async getWitTypesCatalog(): Promise<any> {
        try {
            const result = await this.mcpService.readResource('wasm://wit/types');
            return JSON.parse(result.text || '{}');
        } catch (error) {
            console.error('Failed to get WIT types catalog:', error);
            throw error;
        }
    }

    /**
     * Get WIT dependencies graph
     */
    public async getWitDependenciesGraph(): Promise<any> {
        try {
            const result = await this.mcpService.readResource('wasm://wit/dependencies');
            return JSON.parse(result.text || '{}');
        } catch (error) {
            console.error('Failed to get WIT dependencies graph:', error);
            throw error;
        }
    }

    /**
     * Get WIT analysis for a specific component
     */
    public async getComponentWitAnalysis(componentName: string): Promise<any> {
        try {
            const result = await this.mcpService.readResource(`wasm://component/${componentName}/wit`);
            return JSON.parse(result.text || '{}');
        } catch (error) {
            console.error(`Failed to get WIT analysis for component ${componentName}:`, error);
            throw error;
        }
    }

    /**
     * Get raw WIT content for a specific component
     */
    public async getComponentRawWit(componentName: string): Promise<string> {
        try {
            const result = await this.mcpService.readResource(`wasm://component/${componentName}/wit/raw`);
            return result.text || '// No WIT content available';
        } catch (error) {
            console.error(`Failed to get raw WIT for component ${componentName}:`, error);
            throw error;
        }
    }

    /**
     * Get all interfaces for a specific component
     */
    public async getComponentInterfaces(componentName: string): Promise<any> {
        try {
            const result = await this.mcpService.readResource(`wasm://component/${componentName}/interfaces`);
            return JSON.parse(result.text || '{}');
        } catch (error) {
            console.error(`Failed to get interfaces for component ${componentName}:`, error);
            throw error;
        }
    }

    /**
     * Analyze WIT interfaces from a WASM file using MCP tool
     */
    public async analyzeWitFromFile(filePath: string, componentName?: string): Promise<any> {
        try {
            const params: any = { filePath };
            if (componentName) {
                params.componentName = componentName;
            }
            
            const result = await this.mcpService.callTool('analyze_wit_interfaces', params);
            
            // Extract the JSON analysis from the result
            const jsonContent = result.content?.find((content: any) => 
                content.content_type === 'application/json');
            
            if (jsonContent) {
                return JSON.parse(jsonContent.text);
            }
            
            // Return the text analysis if no JSON found
            const textContent = result.content?.[0]?.text || 'No analysis available';
            return { analysis: textContent, rawResult: result };
            
        } catch (error) {
            console.error(`Failed to analyze WIT from file ${filePath}:`, error);
            throw error;
        }
    }

    /**
     * Get all WASM components with their WIT status
     */
    public async getComponentsWithWitStatus(): Promise<any> {
        try {
            const result = await this.mcpService.readResource('wasm://components/list');
            return JSON.parse(result.text || '{}');
        } catch (error) {
            console.error('Failed to get components with WIT status:', error);
            throw error;
        }
    }

    /**
     * Refresh interface data for existing WASM components in the current diagram
     */
    public async refreshComponentInterfaces(): Promise<void> {
        const diagramId = this.diagramService.getCurrentDiagramId();
        if (!diagramId) return;

        const diagram = this.diagramService.getCurrentDiagram();
        if (!diagram) return;

        console.log('Refreshing WASM component interfaces...');

        try {
            // Get the list of all available components with their data
            const componentsData = await this.getComponentsWithWitStatus();
            const componentMap = new Map<string, any>();
            
            // Build a map for quick lookup
            if (componentsData.components && Array.isArray(componentsData.components)) {
                for (const comp of componentsData.components) {
                    componentMap.set(comp.name, comp);
                }
            }

            // Update each WASM component node in the diagram
            let updated = false;
            for (const [elementId, element] of Object.entries(diagram.elements)) {
                if (element.type === 'wasm-component') {
                    const componentName = element.label || element.properties?.componentName;
                    if (componentName && componentMap.has(componentName)) {
                        const componentData = componentMap.get(componentName);
                        
                        // Update the element properties with interface data
                        const updateProps: any = {
                            interfaces: componentData.interfaces || []
                        };

                        // Also update other properties if missing
                        if (!element.properties?.componentPath) {
                            updateProps.componentPath = componentData.path;
                        }
                        if (!element.properties?.description) {
                            updateProps.description = componentData.description;
                        }

                        // Update via MCP
                        await this.mcpService.callTool('update_element', {
                            diagramId,
                            elementId,
                            properties: updateProps
                        });
                        
                        updated = true;
                        console.log(`Updated interfaces for component: ${componentName}`, componentData.interfaces);
                    }
                }
            }

            if (updated) {
                // Reload the diagram to reflect changes
                await this.diagramService.loadDiagram(diagramId);
                console.log('Component interfaces refreshed successfully');
            }
        } catch (error) {
            console.error('Failed to refresh component interfaces:', error);
        }
    }
}