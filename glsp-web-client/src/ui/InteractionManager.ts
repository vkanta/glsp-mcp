import { CanvasRenderer, InteractionEvent } from '../renderer/canvas-renderer.js';
import { DiagramService } from '../services/DiagramService.js';
import { WasmComponentManager } from '../wasm/WasmComponentManager.js';

export class InteractionManager {
    private renderer: CanvasRenderer;
    private diagramService: DiagramService;
    private wasmComponentManager?: WasmComponentManager;
    private currentMode: string = 'select';
    private currentNodeType: string = '';
    private currentEdgeType: string = '';
    private spacePressed: boolean = false;
    private originalMode: string = 'select';

    constructor(renderer: CanvasRenderer, diagramService: DiagramService) {
        this.renderer = renderer;
        this.diagramService = diagramService;
    }

    public setupEventHandlers(): void {
        this.renderer.addInteractionHandler((event: InteractionEvent) => {
            this.handleRendererInteraction(event);
        });

        document.addEventListener('keydown', (event) => {
            this.handleKeyboardShortcut(event);
            this.handleSpaceKeyDown(event);
        });

        document.addEventListener('keyup', (event) => {
            this.handleSpaceKeyUp(event);
        });
        
        // Listen for toolbar events
        window.addEventListener('toolbar-mode-change', (event: any) => {
            this.currentMode = event.detail.mode;
            console.log('Mode changed to:', this.currentMode);
            
            // Get the current node/edge type from UIManager when mode changes
            if (this.currentMode === 'create-node') {
                const uiManager = (window as any).app?.uiManager;
                if (uiManager) {
                    this.currentNodeType = uiManager.getCurrentNodeType();
                }
            } else if (this.currentMode === 'create-edge') {
                const uiManager = (window as any).app?.uiManager;
                if (uiManager) {
                    this.currentEdgeType = uiManager.getCurrentEdgeType();
                }
            }
            
            // Update renderer mode
            if (this.currentMode === 'pan') {
                this.renderer.setInteractionMode('pan');
            } else if (this.currentMode === 'create-node') {
                this.renderer.setInteractionMode('create-node');
            } else if (this.currentMode === 'create-edge') {
                this.renderer.setInteractionMode('create-edge');
            } else {
                this.renderer.setInteractionMode('select');
            }
        });
        
        window.addEventListener('toolbar-zoom', (event: any) => {
            const direction = event.detail.direction;
            if (direction === 'in') {
                this.renderer.zoom(1.2);
            } else {
                this.renderer.zoom(0.8);
            }
        });
        
        window.addEventListener('toolbar-fit-content', () => {
            this.renderer.fitToContent();
        });
        
        window.addEventListener('toolbar-reset-view', () => {
            this.renderer.resetView();
        });
        
        window.addEventListener('toolbar-apply-layout', async () => {
            const diagramId = this.diagramService.getCurrentDiagramId();
            if (diagramId) {
                await this.diagramService.applyLayout(diagramId, 'hierarchical');
            }
        });
    }

    // Set the WASM component manager reference
    public setWasmComponentManager(manager: WasmComponentManager): void {
        this.wasmComponentManager = manager;
    }

    private async handleRendererInteraction(event: InteractionEvent): Promise<void> {
        const diagramId = this.diagramService.getCurrentDiagramId();
        if (!diagramId) return;

        switch (event.type) {
            case 'click':
                await this.handleElementClick(event);
                break;
            case 'drag-end':
                // This will be handled by a dedicated method
                break;
            case 'canvas-click':
                await this.handleCanvasClick(event);
                break;
            case 'edge-end':
                if (event.sourceElement && event.element) {
                    const edgeType = this.currentEdgeType || 'flow';
                    await this.diagramService.createEdge(diagramId, edgeType, event.sourceElement.id, event.element.id);
                }
                break;
        }
    }

    private async handleCanvasClick(event: InteractionEvent): Promise<void> {
        const { position } = event;
        if (!position) return;
        
        let diagramId = this.diagramService.getCurrentDiagramId();
        if (!diagramId) {
            console.warn('No diagram loaded! Creating a new diagram...');
            const newDiagram = await this.diagramService.createNewDiagram('workflow', 'Untitled Diagram');
            if (newDiagram) {
                diagramId = newDiagram;
                // Load the newly created diagram
                const loadedDiagram = await this.diagramService.loadDiagram(newDiagram);
                if (loadedDiagram && this.renderer) {
                    this.renderer.setDiagram(loadedDiagram);
                }
            } else {
                console.error('Failed to create a new diagram! Cannot create node.');
                return;
            }
        }
        
        console.log('Canvas clicked at:', position, 'Mode:', this.currentMode);
        
        if (this.currentMode === 'create-node' && this.currentNodeType) {
            // Create a new node at the clicked position
            console.log('Creating node:', this.currentNodeType, 'at position:', position);
            await this.diagramService.createNode(diagramId, this.currentNodeType, position, `New ${this.currentNodeType}`);
            console.log('Created node:', this.currentNodeType);
        } else {
            console.log('Canvas click ignored - mode:', this.currentMode, 'nodeType:', this.currentNodeType);
        }
    }

    private async handleElementClick(event: InteractionEvent): Promise<void> {
        const { element, position } = event;
        if (!element || !position) return;

        console.log('Element clicked:', element.id, 'Mode:', this.currentMode);
        
        // Handle edge creation mode
        if (this.currentMode === 'create-edge' && this.currentEdgeType) {
            // Tell renderer to start edge creation from this element
            this.renderer.startEdgeCreation(element, this.currentEdgeType);
            return;
        }

        // Check if this is a WASM component and if the click is on the load switch
        if (this.wasmComponentManager && element.bounds) {
            const isLoadSwitchClick = this.wasmComponentManager.isLoadSwitchClick(position, element);
            
            if (isLoadSwitchClick) {
                console.log('Load switch clicked for component:', element.id);
                await this.wasmComponentManager.toggleComponentLoad(element.id);
                return; // Don't process other click actions for load switch
            }
            
            // Check if this is a click on a loaded component (for execution view)
            const nodeType = element.type || element.element_type || '';
            if (nodeType === 'wasm-component' && this.wasmComponentManager.isComponentLoaded(element.id)) {
                console.log('Loaded WASM component clicked - opening execution view:', element.id);
                this.openComponentExecutionView(element.id);
                return;
            }
        }

        // Handle other types of clicks here...
    }

    private openComponentExecutionView(elementId: string): void {
        if (!this.wasmComponentManager) return;

        const loadedComponent = this.wasmComponentManager.getLoadedComponent(elementId);
        if (!loadedComponent) return;

        // Create and open execution view modal
        this.createExecutionView(elementId, loadedComponent);
    }

    private createExecutionView(elementId: string, loadedComponent: any): void {
        // Create modal overlay
        const modal = document.createElement('div');
        modal.className = 'execution-view-modal';
        modal.innerHTML = `
            <div class="execution-view-content">
                <div class="execution-view-header">
                    <h2>🚀 ${loadedComponent.name} - Execution View</h2>
                    <button class="close-btn" onclick="this.closest('.execution-view-modal').remove()">✖</button>
                </div>
                <div class="execution-view-body">
                    <div class="component-info">
                        <p><strong>Component:</strong> ${loadedComponent.name}</p>
                        <p><strong>Loaded:</strong> ${loadedComponent.loadedAt}</p>
                        <p><strong>Path:</strong> ${loadedComponent.path}</p>
                    </div>
                    <div class="javascript-examples">
                        <h3>📝 JavaScript Examples</h3>
                        <div class="code-examples" id="code-examples-${elementId}">
                            <div class="loading-container" style="background: #1e1e1e; padding: 20px; display: flex; align-items: center; justify-content: center;">
                                <div class="loading-pulse">
                                    <span style="background: #654FF0;"></span>
                                    <span style="background: #654FF0;"></span>
                                    <span style="background: #654FF0;"></span>
                                </div>
                                <span style="color: #7D8590; margin-left: 12px;">Generating JavaScript examples...</span>
                            </div>
                        </div>
                    </div>
                    <div class="execution-console">
                        <h3>💻 Console</h3>
                        <div class="console-output" id="console-${elementId}"></div>
                        <div class="console-input">
                            <input type="text" placeholder="Enter JavaScript to execute..." id="console-input-${elementId}">
                            <button onclick="window.executeWasmCode('${elementId}')">Run</button>
                        </div>
                    </div>
                </div>
            </div>
        `;

        // Style the modal
        Object.assign(modal.style, {
            position: 'fixed',
            top: '0',
            left: '0',
            width: '100%',
            height: '100%',
            backgroundColor: 'rgba(0, 0, 0, 0.8)',
            zIndex: '10000',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center'
        });

        // Add CSS for the modal content
        const style = document.createElement('style');
        style.textContent = `
            .execution-view-content {
                background: white;
                border-radius: 12px;
                width: 90%;
                max-width: 800px;
                max-height: 80%;
                display: flex;
                flex-direction: column;
                box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
            }
            .execution-view-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 20px;
                border-bottom: 1px solid #eee;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                border-radius: 12px 12px 0 0;
            }
            .execution-view-header h2 {
                margin: 0;
                font-size: 1.2rem;
            }
            .close-btn {
                background: rgba(255, 255, 255, 0.2);
                border: none;
                color: white;
                width: 30px;
                height: 30px;
                border-radius: 50%;
                cursor: pointer;
                font-size: 14px;
            }
            .execution-view-body {
                padding: 20px;
                overflow-y: auto;
                flex: 1;
            }
            .component-info {
                background: #f8f9fa;
                padding: 15px;
                border-radius: 8px;
                margin-bottom: 20px;
            }
            .javascript-examples {
                margin-bottom: 20px;
            }
            .code-examples {
                background: #1e1e1e;
                color: #d4d4d4;
                padding: 15px;
                border-radius: 8px;
                font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
                font-size: 12px;
                max-height: 300px;
                overflow-y: auto;
            }
            .execution-console {
                border: 1px solid #ddd;
                border-radius: 8px;
                overflow: hidden;
            }
            .console-output {
                background: #000;
                color: #0f0;
                padding: 15px;
                min-height: 150px;
                max-height: 200px;
                overflow-y: auto;
                font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
                font-size: 12px;
            }
            .console-input {
                display: flex;
                padding: 10px;
                background: #f8f9fa;
            }
            .console-input input {
                flex: 1;
                padding: 8px;
                border: 1px solid #ddd;
                border-radius: 4px;
                margin-right: 10px;
            }
            .console-input button {
                background: #007bff;
                color: white;
                border: none;
                padding: 8px 16px;
                border-radius: 4px;
                cursor: pointer;
            }
        `;
        
        document.head.appendChild(style);
        document.body.appendChild(modal);

        // Generate JavaScript examples
        this.generateJavaScriptExamples(elementId, loadedComponent);

        // Set up console execution
        this.setupConsoleExecution(elementId, loadedComponent);
    }

    private generateJavaScriptExamples(elementId: string, loadedComponent: any): void {
        const examplesContainer = document.getElementById(`code-examples-${elementId}`);
        if (!examplesContainer) return;

        // Generate examples based on component interfaces
        const examples = this.createJavaScriptExamples(loadedComponent);
        
        examplesContainer.innerHTML = examples;
    }

    private createJavaScriptExamples(loadedComponent: any): string {
        // TODO: Extract actual interfaces from loaded component
        // For now, create generic examples based on ADAS component patterns
        
        const componentName = loadedComponent.name;
        const examples = `
// 🚀 Example 1: Initialize ${componentName}
const ${componentName.replace(/-/g, '_')} = await wasmComponent.instantiate();
console.log('Component initialized:', ${componentName.replace(/-/g, '_')});

// 📡 Example 2: Call main processing function
const sensorData = {
    timestamp: Date.now(),
    data: [1.0, 2.0, 3.0, 4.0]
};

const result = await ${componentName.replace(/-/g, '_')}.process(sensorData);
console.log('Processing result:', result);

// 🔧 Example 3: Check component status
const status = await ${componentName.replace(/-/g, '_')}.getStatus();
console.log('Component status:', status);

// 🎯 Example 4: Configure component
await ${componentName.replace(/-/g, '_')}.configure({
    mode: 'production',
    sensitivity: 0.8,
    debug: false
});

// 🔄 Example 5: Process continuous data stream
const processStream = async () => {
    for (let i = 0; i < 10; i++) {
        const data = generateRandomSensorData();
        const output = await ${componentName.replace(/-/g, '_')}.process(data);
        console.log(\`Step \${i + 1}:\`, output);
        await sleep(100); // Wait 100ms between calls
    }
};

// Helper functions
function generateRandomSensorData() {
    return {
        timestamp: Date.now(),
        data: Array.from({length: 4}, () => Math.random() * 10)
    };
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}
        `;

        return examples;
    }

    private setupConsoleExecution(elementId: string, loadedComponent: any): void {
        // Make the execution function globally available
        (window as any).executeWasmCode = (id: string) => {
            if (id !== elementId) return;
            
            const input = document.getElementById(`console-input-${elementId}`) as HTMLInputElement;
            const output = document.getElementById(`console-${elementId}`);
            
            if (!input || !output) return;
            
            const code = input.value.trim();
            if (!code) return;
            
            // Add the input to console output
            output.innerHTML += `<div style="color: #fff;">> ${code}</div>`;
            
            try {
                // Create a safe execution context
                const result = this.executeWasmCode(code, loadedComponent);
                output.innerHTML += `<div style="color: #0f0;">${result}</div>`;
            } catch (error) {
                const errorMessage = error instanceof Error ? error.message : String(error);
                output.innerHTML += `<div style="color: #f00;">Error: ${errorMessage}</div>`;
            }
            
            // Clear input and scroll to bottom
            input.value = '';
            output.scrollTop = output.scrollHeight;
        };

        // Handle Enter key in input
        const input = document.getElementById(`console-input-${elementId}`) as HTMLInputElement;
        if (input) {
            input.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    (window as any).executeWasmCode(elementId);
                }
            });
        }
    }

    private executeWasmCode(code: string, loadedComponent: any): string {
        // Safely execute JavaScript code with access to the loaded WASM component
        try {
            // Create a safe execution context with the loaded component
            const context = {
                wasmComponent: loadedComponent,
                componentName: loadedComponent.name,
                console: {
                    log: (...args: any[]) => JSON.stringify(args)
                }
            };
            
            // Simple evaluation (in a real implementation, this should be more secure)
            const func = new Function('context', `
                const { wasmComponent, componentName, console } = context;
                return ${code};
            `);
            
            const result = func(context);
            return JSON.stringify(result, null, 2);
        } catch (error) {
            throw error;
        }
    }

    private handleKeyboardShortcut(event: KeyboardEvent): void {
        const diagramId = this.diagramService.getCurrentDiagramId();
        if (!diagramId) return;

        if (event.ctrlKey || event.metaKey) {
            switch (event.key) {
                case 'n':
                    event.preventDefault();
                    this.createNewDiagram();
                    break;
                case 's':
                    event.preventDefault();
                    this.diagramService.saveDiagram(diagramId);
                    break;
                case '=':
                case '+':
                    event.preventDefault();
                    this.renderer.zoom(1.2);
                    break;
                case '-':
                    event.preventDefault();
                    this.renderer.zoom(0.8);
                    break;
                case '0':
                    event.preventDefault();
                    this.renderer.fitToContent();
                    break;
                case 'r':
                    event.preventDefault();
                    this.renderer.resetView();
                    break;
                case 'a':
                    event.preventDefault();
                    this.selectAll();
                    break;
            }
        }

        switch (event.key) {
            case 'Delete':
            case 'Backspace':
                event.preventDefault();
                this.deleteSelected();
                break;
            case 'Escape':
                event.preventDefault();
                this.clearSelection();
                break;
            case ' ':
                // Space key for pan mode - this is handled separately in mouse events
                break;
        }
    }

    private async createNewDiagram(): Promise<void> {
        try {
            console.log('Creating new diagram via keyboard shortcut...');
            const newDiagram = await this.diagramService.createNewDiagram('workflow', 'New Diagram');
            if (newDiagram) {
                const loadedDiagram = await this.diagramService.loadDiagram(newDiagram);
                if (loadedDiagram) {
                    this.renderer.setDiagram(loadedDiagram);
                    console.log('New diagram created:', newDiagram);
                }
            }
        } catch (error) {
            console.error('Failed to create new diagram:', error);
        }
    }

    private selectAll(): void {
        try {
            console.log('Select all triggered via keyboard shortcut');
            // TODO: Implement select all when selection manager is available
            // For now, just log that the shortcut was triggered
        } catch (error) {
            console.error('Failed to select all:', error);
        }
    }

    private deleteSelected(): void {
        try {
            const diagramId = this.diagramService.getCurrentDiagramId();
            if (!diagramId) return;

            console.log('Delete selected triggered via keyboard shortcut');
            // TODO: Implement delete selected when selection manager is available
            // For now, just log that the shortcut was triggered
        } catch (error) {
            console.error('Failed to delete selected elements:', error);
        }
    }

    private clearSelection(): void {
        try {
            console.log('Clear selection triggered via keyboard shortcut');
            // TODO: Implement clear selection when selection manager is available
            // For now, just log that the shortcut was triggered
        } catch (error) {
            console.error('Failed to clear selection:', error);
        }
    }

    private handleSpaceKeyDown(event: KeyboardEvent): void {
        // Only trigger if not in an input field and space not already pressed
        if (event.target instanceof HTMLInputElement || 
            event.target instanceof HTMLTextAreaElement || 
            event.target instanceof HTMLSelectElement ||
            this.spacePressed) {
            return;
        }

        if (event.code === 'Space') {
            event.preventDefault();
            this.spacePressed = true;
            this.originalMode = this.currentMode;
            this.currentMode = 'pan';
            this.renderer.setInteractionMode('pan');
            console.log('Space pressed - entering temporary pan mode');
        }
    }

    private handleSpaceKeyUp(event: KeyboardEvent): void {
        if (event.code === 'Space' && this.spacePressed) {
            event.preventDefault();
            this.spacePressed = false;
            this.currentMode = this.originalMode;
            this.renderer.setInteractionMode(this.originalMode);
            console.log('Space released - returning to', this.originalMode, 'mode');
        }
    }
}