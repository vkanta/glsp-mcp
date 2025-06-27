import { McpService } from './services/McpService.js';
import { DiagramService } from './services/DiagramService.js';
import { UIManager } from './ui/UIManager.js';
import { InteractionManager } from './ui/InteractionManager.js';
import { CanvasRenderer } from './renderer/canvas-renderer.js';
import { AIService } from './services/AIService.js';
import { WasmRuntimeManager } from './wasm/WasmRuntimeManager.js';
import { statusManager } from './services/StatusManager.js';

export class AppController {
    private mcpService: McpService;
    private diagramService: DiagramService;
    public uiManager: UIManager;
    private renderer: CanvasRenderer;
    private interactionManager: InteractionManager;
    private aiService: AIService;
    private wasmRuntimeManager: WasmRuntimeManager;

    constructor(private canvas: HTMLCanvasElement) {
        this.mcpService = new McpService();
        this.diagramService = new DiagramService(this.mcpService);
        this.uiManager = new UIManager();
        this.renderer = new CanvasRenderer(canvas);
        this.interactionManager = new InteractionManager(this.renderer, this.diagramService);
        this.aiService = new AIService(this.mcpService);
        this.wasmRuntimeManager = new WasmRuntimeManager(this.mcpService, this.diagramService, {
            enableClientSideTranspilation: true,
            maxConcurrentExecutions: 5,
            maxCachedComponents: 50
        }, this.renderer);

        // Expose for debugging
        (window as any).testOllama = () => this.aiService.testOllamaConnection();
        (window as any).appController = this;
        (window as any).wasmRuntime = this.wasmRuntimeManager;
        (window as any).uploadWasm = () => this.wasmRuntimeManager.showUploadPanel();

        this.mountUI();

        this.initialize();
    }

    private mountUI(): void {
        console.log('AppController: Mounting UI elements');
        
        const toolbarContainer = document.getElementById('toolbar-container');
        if (toolbarContainer) {
            toolbarContainer.appendChild(this.uiManager.getToolbarElement());
            console.log('AppController: Toolbar mounted');
        } else {
            console.warn('AppController: toolbar-container not found');
        }

        const statusContainer = document.getElementById('status-container');
        if (statusContainer) {
            statusContainer.appendChild(this.uiManager.getStatusElement());
            console.log('AppController: Status bar mounted');
        } else {
            console.warn('AppController: status-container not found');
        }

        const diagramListContainer = document.getElementById('diagram-list-container');
        console.log('AppController: Looking for diagram-list-container, found:', !!diagramListContainer);
        if (diagramListContainer) {
            const diagramListElement = this.uiManager.getDiagramListElement();
            console.log('AppController: Got diagram list element:', !!diagramListElement);
            diagramListContainer.appendChild(diagramListElement);
            console.log('AppController: Diagram list mounted');
        } else {
            console.error('AppController: diagram-list-container not found in DOM');
        }

        // Mount AI panel as floating element
        document.body.appendChild(this.uiManager.getAIPanelElement());
        console.log('AppController: AI panel mounted as floating element');

        // Mount WASM palette as floating element
        document.body.appendChild(this.wasmRuntimeManager.getPaletteElement());
        console.log('AppController: WASM palette mounted as floating element');
    }

    private async initialize(): Promise<void> {
        try {
            console.log('AppController: Starting initialization...');
            await this.mcpService.initialize();
            console.log('MCP Service Initialized');
            this.uiManager.updateStatus('Connected to MCP server');

            this.interactionManager.setupEventHandlers();
            this.interactionManager.setWasmComponentManager(this.wasmRuntimeManager);
            this.wasmRuntimeManager.setupCanvasDragAndDrop(this.canvas);

            // Setup connection status monitoring
            this.mcpService.addConnectionListener((connected: boolean) => {
                statusManager.setMcpStatus(connected);
            });

            // Setup AI connection monitoring
            this.aiService.addConnectionListener((connected: boolean) => {
                statusManager.setAiStatus(connected);
            });

            this.uiManager.setupToolbarEventHandlers(async (newType: string) => {
                await this.handleDiagramTypeChange(newType);
            });
            
            this.uiManager.setupAIPanelEventHandlers(
                async (prompt: string) => await this.handleAICreateDiagram(prompt),
                async () => await this.handleAITestDiagram(),
                async () => await this.handleAIAnalyzeDiagram(),
                async () => await this.handleAIOptimizeLayout()
            );

            const connections = await this.aiService.checkConnections();
            // AI status will be set automatically by the connection listener

            if (connections.ollama) {
                const models = await this.aiService.getAvailableModels();
                const currentModel = this.aiService.getCurrentModel();
                this.uiManager.updateAIModelSelect(models, currentModel, (modelName) => {
                    this.aiService.setCurrentModel(modelName);
                });
            }

            await this.wasmRuntimeManager.initializeEnhancedWasmComponents();

            console.log('AppController: Creating sample diagram...');
            const diagram = await this.diagramService.createSampleDiagram();
            console.log('AppController: Sample diagram result:', diagram);
            if (diagram !== undefined) {
                this.renderer.setDiagram(diagram);
                this.uiManager.updateStatus(`Loaded diagram`);
                console.log('AppController: Sample diagram loaded successfully');
                console.log('AppController: Current diagram ID:', this.diagramService.getCurrentDiagramId());
            } else {
                console.warn('AppController: Failed to create sample diagram');
                // Still update status so user knows the app is ready
                this.uiManager.updateStatus('Ready - click canvas to create nodes');
            }

            const diagrams = await this.diagramService.getAvailableDiagrams();
            console.log('AppController: Retrieved diagrams:', diagrams);
            this.uiManager.updateDiagramList(
                diagrams, 
                async (diagramId) => {
                    console.log('AppController: Loading diagram:', diagramId);
                    const loadedDiagram = await this.diagramService.loadDiagram(diagramId);
                    if (loadedDiagram) {
                        console.log('AppController: Diagram loaded successfully:', loadedDiagram);
                        this.renderer.setDiagram(loadedDiagram);
                    } else {
                        console.warn('AppController: Failed to load diagram:', diagramId);
                    }
                },
                async (diagramId, diagramName) => {
                    console.log('AppController: Deleting diagram:', diagramId, diagramName);
                    const success = await this.diagramService.deleteDiagram(diagramId);
                    if (success) {
                        console.log('AppController: Diagram deleted successfully');
                        // Refresh the diagram list
                        await this.refreshDiagramList();
                    } else {
                        console.error('AppController: Failed to delete diagram');
                        alert('Failed to delete diagram. Please try again.');
                    }
                }
            );

            // Setup create new diagram button
            this.uiManager.setupCreateDiagramButton(async () => {
                await this.handleCreateNewDiagram();
            });

        } catch (error) {
            console.error('Failed to initialize application:', error);
            if (error instanceof Error) {
                console.error('Error details:', error.message, error.stack);
            }
            statusManager.setMcpStatus(false);
            // AI status will be updated by its own connection monitoring
        }
    }

    private async handleAICreateDiagram(prompt: string): Promise<void> {
        console.log('AI Create Diagram:', prompt);
        this.uiManager.addAIMessage('AI', '🤖 Let me create that diagram for you...');
        
        try {
            const currentDiagramId = this.diagramService.getCurrentDiagramId();
            const response = await this.aiService.createDiagramFromPrompt(prompt, currentDiagramId);
            await this._handleAIDiagramResponse(response);
        } catch (error) {
            console.error('AI diagram creation failed:', error);
            this.uiManager.addAIMessage('AI', `❌ Failed to create diagram: ${error}`);
        }
    }
    
    private async handleAITestDiagram(): Promise<void> {
        console.log('AI Test Diagram Creation');
        this.uiManager.addAIMessage('AI', '🧪 Creating a test workflow diagram for you...');
        const testPrompt = "Create a simple workflow diagram with a start node, a process task called 'Review Document', a decision gateway asking 'Approved?', and two end nodes for approved and rejected paths.";
        await this.handleAICreateDiagram(testPrompt);
    }
    
    private async handleAIAnalyzeDiagram(): Promise<void> {
        console.log('AI Analyze Current Diagram');
        this.uiManager.addAIMessage('AI', '🔍 Analyzing your current diagram...');
        
        try {
            const diagramId = this.diagramService.getCurrentDiagramId();
            if (!diagramId) {
                this.uiManager.addAIMessage('AI', '❌ No diagram loaded to analyze. Please create or load a diagram first.');
                return;
            }
            
            const analysis = await this.aiService.analyzeDiagram(diagramId);
            this.uiManager.addAIMessage('AI', `📊 **Diagram Analysis**\n\n${analysis}`);
        } catch (error) {
            console.error('AI diagram analysis failed:', error);
            this.uiManager.addAIMessage('AI', `❌ Failed to analyze diagram: ${error}`);
        }
    }
    
    private async handleAIOptimizeLayout(): Promise<void> {
        console.log('AI Optimize Layout');
        this.uiManager.addAIMessage('AI', '🔧 Optimizing your diagram layout...');
        
        try {
            const diagramId = this.diagramService.getCurrentDiagramId();
            if (!diagramId) {
                this.uiManager.addAIMessage('AI', '❌ No diagram loaded to optimize. Please create or load a diagram first.');
                return;
            }
            
            // First apply hierarchical layout
            await this.diagramService.applyLayout(diagramId, 'hierarchical');
            
            // Then get AI suggestions for further optimization
            const suggestions = await this.aiService.suggestLayoutImprovements(diagramId);
            this.uiManager.addAIMessage('AI', `✅ **Layout Optimized**\n\n${suggestions}`);
        } catch (error) {
            console.error('AI layout optimization failed:', error);
            this.uiManager.addAIMessage('AI', `❌ Failed to optimize layout: ${error}`);
        }
    }

    private async _handleAIDiagramResponse(response: any): Promise<void> {
        const statusIcon = response.success ? '✅' : '❌';
        let message = `${statusIcon} ${response.message}`;
        
        if (response.steps && response.steps.length > 0) {
            message += '\n\n**Steps taken:**';
            response.steps.forEach((step: string) => {
                message += `\n• ${step}`;
            });
        }
        
        if (response.errors && response.errors.length > 0) {
            message += '\n\n**Errors:**';
            response.errors.forEach((error: string) => {
                message += `\n❌ ${error}`;
            });
        }
        
        this.uiManager.addAIMessage('AI', message);
        
        if (response.success && response.diagramId) {
            await this.diagramService.loadDiagram(response.diagramId);
            this.renderer.setDiagram(this.diagramService.getDiagramState().getDiagram(response.diagramId)!);
        }
    }

    private async handleDiagramTypeChange(diagramType: string): Promise<void> {
        console.log('=== DIAGRAM TYPE CHANGE ===');
        console.log('Diagram type changed to:', diagramType);
        
        // Update the toolbar to reflect the new diagram type
        console.log('Updating toolbar content...');
        this.uiManager.updateToolbarContent(this.uiManager.getToolbarElement(), diagramType);
        
        // Show/hide WASM palette based on diagram type
        if (diagramType === 'wasm-component') {
            await this.wasmRuntimeManager.showEnhancedPalette();
            console.log('Enhanced WASM palette shown for wasm-component diagram type');
        } else {
            this.wasmRuntimeManager.hidePalette();
            console.log('WASM palette hidden for non-wasm diagram type');
        }
        
        // Create a new diagram of the selected type
        this.createNewDiagramOfType(diagramType);
    }

    private async createNewDiagramOfType(diagramType: string): Promise<void> {
        try {
            console.log('Creating new diagram of type:', diagramType);
            const newDiagram = await this.diagramService.createNewDiagram(diagramType, `New ${diagramType} Diagram`);
            if (newDiagram) {
                // Load the new diagram
                const loadedDiagram = await this.diagramService.loadDiagram(newDiagram);
                if (loadedDiagram) {
                    this.renderer.setDiagram(loadedDiagram);
                    this.uiManager.updateStatus(`Created new ${diagramType} diagram`);
                    console.log('New diagram created and loaded:', newDiagram);
                    
                    // Update the diagram list
                    const diagrams = await this.diagramService.getAvailableDiagrams();
                    this.uiManager.updateDiagramList(diagrams, async (diagramId) => {
                        const diagram = await this.diagramService.loadDiagram(diagramId);
                        if (diagram) {
                            this.renderer.setDiagram(diagram);
                        }
                    });
                }
            }
        } catch (error) {
            console.error('Failed to create new diagram:', error);
            this.uiManager.updateStatus(`Failed to create ${diagramType} diagram`);
        }
    }

    private async refreshDiagramList(): Promise<void> {
        try {
            const diagrams = await this.diagramService.getAvailableDiagrams();
            console.log('AppController: Refreshing diagram list with', diagrams.length, 'diagrams');
            this.uiManager.updateDiagramList(
                diagrams,
                async (diagramId) => {
                    console.log('AppController: Loading diagram:', diagramId);
                    const loadedDiagram = await this.diagramService.loadDiagram(diagramId);
                    if (loadedDiagram) {
                        this.renderer.setDiagram(loadedDiagram);
                    }
                },
                async (diagramId, diagramName) => {
                    console.log('AppController: Deleting diagram:', diagramId, diagramName);
                    const success = await this.diagramService.deleteDiagram(diagramId);
                    if (success) {
                        await this.refreshDiagramList();
                    } else {
                        alert('Failed to delete diagram. Please try again.');
                    }
                }
            );
        } catch (error) {
            console.error('Failed to refresh diagram list:', error);
        }
    }

    private async handleCreateNewDiagram(): Promise<void> {
        const diagramTypes = this.diagramService.getAvailableDiagramTypes();
        
        // Create a simple selection dialog
        const diagramType = prompt(
            `Choose diagram type:\n${diagramTypes.map((type, i) => `${i + 1}. ${type.label}`).join('\n')}\n\nEnter number (1-${diagramTypes.length}):`,
            '1'
        );
        
        if (!diagramType) return; // User cancelled
        
        const typeIndex = parseInt(diagramType) - 1;
        if (typeIndex < 0 || typeIndex >= diagramTypes.length) {
            alert('Invalid diagram type selection');
            return;
        }
        
        const selectedType = diagramTypes[typeIndex];
        const diagramName = prompt(`Enter name for new ${selectedType.label} diagram:`, `New ${selectedType.label}`);
        
        if (!diagramName) return; // User cancelled
        
        try {
            console.log('AppController: Creating new diagram:', selectedType.value, diagramName);
            const result = await this.mcpService.createDiagram(selectedType.value, diagramName);
            console.log('AppController: Create diagram result:', result);
            
            if (result.content && result.content[0] && result.content[0].text) {
                this.uiManager.updateStatus(`Created: ${diagramName}`);
                // Refresh the diagram list to show the new diagram
                await this.refreshDiagramList();
            } else {
                throw new Error('Invalid response from create diagram');
            }
        } catch (error) {
            console.error('Failed to create new diagram:', error);
            alert(`Failed to create diagram: ${error instanceof Error ? error.message : 'Unknown error'}`);
        }
    }
}