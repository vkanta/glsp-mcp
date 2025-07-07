import { McpService } from './services/McpService.js';
import { DiagramService } from './services/DiagramService.js';
import { UIManager } from './ui/UIManager.js';
import { InteractionManager } from './ui/InteractionManager.js';
import { CanvasRenderer } from './renderer/canvas-renderer.js';
import { AIService } from './services/AIService.js';
import { WasmRuntimeManager } from './wasm/WasmRuntimeManager.js';
import { statusManager } from './services/StatusManager.js';
import { BaseDialog } from './ui/dialogs/base/BaseDialog.js';

// Debug interface for window properties
interface WindowDebug {
    testOllama: () => Promise<boolean>;
    appController: AppController;
    wasmRuntime: WasmRuntimeManager;
    uploadWasm: () => void;
    testSidebar: () => void;
    theme: import('./ui/ThemeController.js').ThemeController;
    toggleSidebar: () => void;
    checkSidebar: () => void;
    testWasm: () => void;
    selectDialog: (config?: unknown) => void;
    getStatus: () => unknown;
    checkMCP: () => Promise<boolean>;
    headerIcons: import('./ui/HeaderIconManager.js').HeaderIconManager;
    testAIMinimize: () => void;
    debugDialogs: () => void;
}

declare global {
    interface Window {
        debug?: Partial<WindowDebug>;
        [key: string]: unknown; // For dynamic properties like deletion flags
    }
}

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
        this.interactionManager = new InteractionManager(this.renderer, this.diagramService, this.mcpService);
        this.aiService = new AIService(this.mcpService);
        this.wasmRuntimeManager = new WasmRuntimeManager(this.mcpService, this.diagramService, {
            enableClientSideTranspilation: true,
            maxConcurrentExecutions: 5,
            maxCachedComponents: 50
        }, this.renderer);

        // Development-only debug utilities
        if (process.env.NODE_ENV === 'development' || window.location.hostname === 'localhost') {
            if (!window.debug) window.debug = {};
            window.debug.testOllama = () => this.aiService.testOllamaConnection();
            window.debug.appController = this;
            window.debug.wasmRuntime = this.wasmRuntimeManager;
            window.debug.uploadWasm = () => this.wasmRuntimeManager.showUploadPanel();
            window.debug.testSidebar = () => this.testSidebarIntegration();
            window.debug.theme = this.uiManager.getThemeController();
            window.debug.toggleSidebar = () => this.uiManager.toggleSidebar();
            window.debug.checkSidebar = () => ({
                sidebarCollapsed: this.uiManager.isSidebarCollapsed(),
                bodyClasses: document.body.className,
                collapseButton: document.querySelector('.sidebar-collapse-btn'),
                buttonVisible: document.querySelector('.sidebar-collapse-btn') ? 
                    window.getComputedStyle(document.querySelector('.sidebar-collapse-btn')!).display !== 'none' : false
            });
            window.debug.headerIcons = this.uiManager.getHeaderIconManager();
            window.debug.testAIMinimize = () => {
                this.uiManager.getHeaderIconManager().addIcon({
                    id: 'test-ai',
                    title: 'Test AI',
                    icon: '🤖',
                    color: 'var(--accent-wasm)',
                    onClick: () => { /* Test AI clicked */ },
                    onClose: () => { /* Test AI closed */ }
                });
            };
            window.debug.debugDialogs = () => BaseDialog.debugDialogState();
        }

        this.mountUI();

        this.initialize();
    }

    private mountUI(): void {
        console.log('AppController: Mounting UI elements');
        
        // Initialize modern sidebar
        const sidebarContainer = document.querySelector('.sidebar');
        if (sidebarContainer) {
            console.log('AppController: Initializing modern sidebar');
            // Clear existing sidebar content
            sidebarContainer.innerHTML = '';
            // Initialize modern sidebar with diagram type change handler
            this.uiManager.initializeModernSidebar(
                sidebarContainer as HTMLElement,
                async (newType: string) => await this.handleDiagramTypeChange(newType)
            );
            console.log('AppController: Modern sidebar initialized');
        } else {
            // Fallback to old UI
            console.log('AppController: Using legacy UI (sidebar not found)');
            const toolbarContainer = document.getElementById('toolbar-container');
            if (toolbarContainer) {
                toolbarContainer.appendChild(this.uiManager.getToolbarElement());
                console.log('AppController: Toolbar mounted');
            }

            const diagramListContainer = document.getElementById('diagram-list-container');
            if (diagramListContainer) {
                const diagramListElement = this.uiManager.getDiagramListElement();
                diagramListContainer.appendChild(diagramListElement);
                console.log('AppController: Diagram list mounted');
            }
        }

        // Always mount status bar
        const statusContainer = document.getElementById('status-container');
        if (statusContainer) {
            statusContainer.appendChild(this.uiManager.getStatusElement());
            console.log('AppController: Status bar mounted');
        }

        // Mount AI panel as floating element
        document.body.appendChild(this.uiManager.getAIPanelElement());
        console.log('AppController: AI panel mounted as floating element');

        // Mount WASM palette as floating element (only if not using modern sidebar)
        if (!sidebarContainer) {
            document.body.appendChild(this.wasmRuntimeManager.getPaletteElement());
            console.log('AppController: WASM palette mounted as floating element');
        } else {
            console.log('AppController: Skipping WASM palette (using sidebar components instead)');
        }
    }
    
    private async loadWasmComponentsToSidebar(): Promise<void> {
        try {
            console.log('=== LOADING WASM COMPONENTS TO SIDEBAR ===');
            
            // Clear any existing error components first
            this.uiManager.clearWasmComponents();
            
            // Trigger component scan to ensure we have the latest data
            console.log('Triggering component scan...');
            const scanResult = await this.mcpService.callTool('scan_wasm_components', {});
            console.log('Scan result:', scanResult);
            
            // Use the scan result directly which contains detailed interface data
            if (scanResult && scanResult.content && scanResult.content[0] && scanResult.content[0].text) {
                console.log('Raw scan result text:', scanResult.content[0].text);
                
                // Parse the scan result which contains the full component data
                const scanData = JSON.parse(scanResult.content[0].text);
                console.log('Parsed scan data:', JSON.stringify(scanData, null, 2));
                
                const components = scanData.components || [];
                console.log(`Found ${components.length} WASM components from scan`);
                console.log('First few components with interfaces:', components.slice(0, 3));
                
                if (components.length === 0) {
                    console.warn('Components array is empty!');
                    this.uiManager.addWasmComponentToLibrary({
                        id: 'no-components',
                        name: 'No WASM components found',
                        description: 'Upload a component or check your backend connection',
                        interfaces: [],
                        status: 'error',
                        version: '',
                        category: 'Status'
                    });
                    return;
                }
                
                components.forEach((component: import('./wasm/WasmComponentManager.js').WasmComponent, index: number) => {
                    console.log(`Processing component ${index + 1}:`, JSON.stringify(component, null, 2));
                    
                    // Extract component info from the MCP data
                    const name = component.name || 'Unknown Component';
                    const status = component.status || 'available';
                    
                    const componentData = {
                        id: component.name || name,
                        name: name,
                        description: component.description || `WASM component: ${name}`,
                        interfaces: component.interfaces || [], // Pass the full interface array from the server
                        status: status,
                        version: '1.0.0', // Default version as server doesn't provide this yet
                        category: this.categorizeWasmComponent(component),
                        path: component.path  // Include the path for drag and drop
                    };
                    
                    console.log(`Adding component to sidebar:`, componentData);
                    this.uiManager.addWasmComponentToLibrary(componentData);
                    
                    console.log(`✅ Added WASM component to sidebar: ${name} (${status})`);
                });
                
                console.log(`🎉 Successfully loaded ${components.length} WASM components to sidebar`);
            } else {
                console.warn('Invalid scan result structure:', {
                    hasScanResult: !!scanResult,
                    hasContent: !!(scanResult?.content),
                    hasText: !!(scanResult?.content?.[0]?.text)
                });
                
                this.uiManager.addWasmComponentToLibrary({
                    id: 'no-components',
                    name: 'No WASM components found',
                    description: 'Upload a component or check your backend connection',
                    interfaces: [],
                    status: 'error',
                    version: '',
                    category: 'Status'
                });
            }
        } catch (error) {
            console.error('❌ Failed to load WASM components to sidebar:', error);
            console.error('Error details:', {
                message: error instanceof Error ? error.message : 'Unknown error',
                stack: error instanceof Error ? error.stack : undefined
            });
            
            // Add an error indicator component
            this.uiManager.addWasmComponentToLibrary({
                id: 'error-loading',
                name: 'Error Loading Components',
                description: `Failed to connect to backend: ${error instanceof Error ? error.message : 'Unknown error'}`,
                interfaces: [],
                status: 'error',
                version: '',
                category: 'Status'
            });
        }
    }
    
    private categorizeWasmComponent(component: import('./wasm/WasmComponentManager.js').WasmComponent): string {
        const name = component.name?.toLowerCase() || '';
        const path = component.path?.toLowerCase() || '';
        
        // Categorize based on component name and path patterns  
        if (name.includes('camera') || path.includes('camera')) {
            return 'Vision';
        } else if (name.includes('object-detection') || name.includes('detection') || path.includes('detection')) {
            return 'AI/ML';
        } else if (name.includes('adas') || name.includes('vehicle') || name.includes('automotive') || path.includes('adas')) {
            return 'Automotive';
        } else if (name.includes('compute') || name.includes('math') || name.includes('calc')) {
            return 'Computation';
        } else if (name.includes('image') || name.includes('media') || name.includes('vision')) {
            return 'Media';
        } else if (name.includes('validate') || name.includes('data')) {
            return 'Utilities';
        } else if (name.includes('crypto') || name.includes('security')) {
            return 'Security';
        } else if (name.includes('ai') || name.includes('ml') || name.includes('neural')) {
            return 'AI/ML';
        } else {
            return 'WASM Components';
        }
    }

    private async initialize(): Promise<void> {
        try {
            console.log('AppController: Starting initialization...');
            await this.mcpService.initialize();
            console.log('MCP Service Initialized');
            this.uiManager.updateStatus('Connected to MCP server');

            this.interactionManager.setupEventHandlers();
            this.interactionManager.setWasmComponentManager(this.wasmRuntimeManager);
            this.interactionManager.setUIManager(this.uiManager);
            // Setup drag and drop for WASM components
            this.setupCanvasDragAndDrop();

            // Setup connection status monitoring
            this.mcpService.addConnectionListener((connected: boolean) => {
                statusManager.setMcpStatus(connected);
            });

            // Setup AI connection monitoring
            this.aiService.addConnectionListener((connected: boolean) => {
                statusManager.setAiStatus(connected);
            });

            // Check current MCP connection status since the listener was added after initialization
            const mcpConnected = this.mcpService.isConnected();
            console.log('AppController: Current MCP connection status after listener setup:', mcpConnected);
            statusManager.setMcpStatus(mcpConnected);

            this.uiManager.setupToolbarEventHandlers(async (newType: string) => {
                await this.handleDiagramTypeChange(newType);
            });
            
            this.uiManager.setupAIPanelEventHandlers(
                async (prompt: string) => await this.handleAICreateDiagram(prompt),
                async () => await this.handleAITestDiagram(),
                async () => await this.handleAIAnalyzeDiagram(),
                async () => await this.handleAIOptimizeLayout()
            );
            
            // Setup diagram close event handler
            window.addEventListener('diagram-close-requested', () => {
                console.log('AppController: Diagram close requested - clearing canvas');
                this.renderer.clear();
            });

            // Show the AI panel
            this.uiManager.showAIPanel();

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
            
            // Connect header icon manager to WASM runtime manager
            this.wasmRuntimeManager.setHeaderIconManager(this.uiManager.getHeaderIconManager());
            
            // Load WASM components into the sidebar if modern sidebar is active
            await this.loadWasmComponentsToSidebar();

            // Load existing diagrams instead of creating a new one
            console.log('AppController: Loading available diagrams...');
            const diagrams = await this.diagramService.getAvailableDiagrams();
            console.log('AppController: Retrieved diagrams:', diagrams);
            
            // Update the diagram list UI
            this.uiManager.updateDiagramList(
                diagrams,
                this.loadDiagramCallback.bind(this),
                this.deleteDiagramCallback.bind(this)
            );
            
            // If there are existing diagrams, load the first one
            if (diagrams.length > 0) {
                console.log('AppController: Loading first available diagram:', diagrams[0].id);
                const firstDiagram = await this.diagramService.loadDiagram(diagrams[0].id);
                if (firstDiagram) {
                    this.renderer.setDiagram(firstDiagram);
                    this.uiManager.updateStatus(`Loaded diagram: ${firstDiagram.name}`);
                    console.log('AppController: Diagram loaded successfully');
                    
                    // Update the toolbar to show the correct node/edge types for this diagram type
                    // Handle both camelCase and snake_case naming conventions
                    const diagramType = firstDiagram.diagramType || firstDiagram.diagram_type || 'workflow';
                    console.log('AppController: Updating toolbar for initial diagram type:', diagramType);
                    this.uiManager.updateToolbarContent(this.uiManager.getToolbarElement(), diagramType);
                    
                    // Refresh WASM component interfaces if this is a WASM diagram
                    if (diagramType === 'wasm-component' && this.wasmRuntimeManager) {
                        console.log('AppController: Refreshing WASM component interfaces...');
                        try {
                            await this.wasmRuntimeManager.refreshComponentInterfaces('all');
                        } catch (error) {
                            console.error('Failed to refresh component interfaces:', error);
                        }
                    }
                }
            } else {
                console.log('AppController: No existing diagrams found');
                this.uiManager.updateStatus('No diagrams found - Create a new diagram to start');
                // Clear the canvas to show empty state
                this.renderer.clear();
            }

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

    private async _handleAIDiagramResponse(response: import('./ai/diagram-agent.js').AgentResponse): Promise<void> {
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
                    
                    // Refresh the diagram list (will use the callbacks from initialize())
                    await this.refreshDiagramList();
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
            
            // Use the same callbacks as the initial setup to avoid duplicate event handlers
            this.uiManager.updateDiagramList(
                diagrams,
                this.loadDiagramCallback.bind(this),
                this.deleteDiagramCallback.bind(this)
            );
        } catch (error) {
            console.error('Failed to refresh diagram list:', error);
        }
    }

    // Extract callbacks as class methods to reuse them
    private async loadDiagramCallback(diagramId: string): Promise<void> {
        console.log('AppController: Loading diagram:', diagramId);
        const loadedDiagram = await this.diagramService.loadDiagram(diagramId);
        if (loadedDiagram) {
            console.log('AppController: Diagram loaded successfully:', loadedDiagram);
            this.renderer.setDiagram(loadedDiagram);
            
            // Update the toolbar to show the correct node/edge types for this diagram type
            // Handle both camelCase and snake_case naming conventions
            const diagramType = loadedDiagram.diagramType || loadedDiagram.diagram_type || 'workflow';
            console.log('AppController: Updating toolbar for loaded diagram type:', diagramType);
            this.uiManager.updateToolbarContent(this.uiManager.getToolbarElement(), diagramType);
        } else {
            console.warn('AppController: Failed to load diagram:', diagramId);
        }
    }

    private async deleteDiagramCallback(diagramId: string, diagramName: string): Promise<void> {
        // Prevent double deletion
        const deleteKey = `deleting-${diagramId}`;
        if (window[deleteKey]) {
            console.log('AppController: Delete already in progress for', diagramId);
            return;
        }
        window[deleteKey] = true;

        try {
            console.log('AppController: Delete request for diagram:', diagramId, diagramName);
            
            // Check if this is the current diagram being worked on
            const isCurrentDiagram = this.diagramService.isCurrentDiagramDeletable(diagramId);
            const hasUnsavedChanges = this.diagramService.hasUnsavedChanges();
            
            let confirmMessage = 'This diagram and all its content will be permanently removed.';
            let _confirmTitle = `Delete "${diagramName}"?`;
            
            // Enhanced warning for current diagram
            if (isCurrentDiagram) {
                _confirmTitle = `⚠️ Delete Current Diagram "${diagramName}"?`;
                confirmMessage = `You are about to delete the diagram you are currently working on.\n\n`;
                
                if (hasUnsavedChanges) {
                    confirmMessage += `⚠️ WARNING: You have unsaved changes that will be lost!\n\n`;
                }
                
                confirmMessage += `This action will:\n• Permanently delete the diagram from the server\n• Clear the current canvas\n• Remove all diagram content\n\nThis cannot be undone.`;
            }
            
            // Show confirmation dialog with enhanced messaging
            const confirmed = await this.uiManager.showDeleteConfirm(
                diagramName,
                confirmMessage
            );
            
            if (!confirmed) {
                console.log('AppController: Diagram deletion cancelled by user');
                return;
            }
            
            console.log('AppController: User confirmed deletion, proceeding...');
            
            // If this is the current diagram, clear the canvas BEFORE calling the service
            // to prevent any visual artifacts during the deletion process
            if (isCurrentDiagram) {
                console.log('AppController: Pre-clearing canvas for current diagram deletion');
                this.renderer.clear(); // This sets currentDiagram = undefined and re-renders
            }
            
            const success = await this.diagramService.deleteDiagram(diagramId);
            console.log('AppController: Delete operation result:', success);
            
            if (success) {
                console.log('AppController: Diagram deleted successfully');
                
                // Canvas is already cleared if it was the current diagram
                if (isCurrentDiagram) {
                    console.log('AppController: Canvas was pre-cleared for current diagram');
                }
                
                // Refresh the diagram list
                await this.refreshDiagramList();
                await this.uiManager.showSuccess(`Successfully deleted "${diagramName}"`);
            } else {
                console.error('AppController: Failed to delete diagram - server error or unknown tool');
                await this.uiManager.showError(
                    'Failed to delete diagram', 
                    'The server does not support diagram deletion yet. This feature needs to be implemented on the backend.'
                );
            }
        } finally {
            // Clear the lock
            delete window[deleteKey];
        }
    }

    private async handleCreateNewDiagram(): Promise<void> {
        try {
            // Get existing diagram names for validation
            const existingDiagrams = await this.diagramService.getAvailableDiagrams();
            const existingNames = existingDiagrams.map(d => d.name);
            
            // Show professional diagram creation dialog
            const result = await this.uiManager.showDiagramTypeSelector(existingNames);
            
            if (!result) return; // User cancelled
            
            const { type: selectedType, name: diagramName } = result;
            
            console.log('AppController: Creating new diagram:', selectedType.type, diagramName);
            const createResult = await this.mcpService.createDiagram(selectedType.type, diagramName);
            console.log('AppController: Create diagram result:', createResult);
            
            if (createResult.content && createResult.content[0] && createResult.content[0].text) {
                // Extract the diagram ID from the response
                const match = createResult.content[0].text.match(/ID: ([a-f0-9-]+)/);
                if (match) {
                    const diagramId = match[1];
                    console.log('AppController: New diagram ID:', diagramId);
                    
                    // Load the newly created diagram
                    const newDiagram = await this.diagramService.loadDiagram(diagramId);
                    if (newDiagram) {
                        this.renderer.setDiagram(newDiagram);
                        console.log('AppController: Loaded new diagram successfully');
                        
                        // Update the toolbar to show the correct node/edge types for this diagram type
                        console.log('AppController: Updating toolbar for diagram type:', selectedType.type);
                        this.uiManager.updateToolbarContent(this.uiManager.getToolbarElement(), selectedType.type);
                    }
                }
                
                this.uiManager.updateStatus(`Created: ${diagramName}`);
                await this.uiManager.showSuccess(`Successfully created "${diagramName}"!`);
                
                // Refresh the diagram list to show the new diagram
                await this.refreshDiagramList();
            } else {
                throw new Error('Invalid response from create diagram');
            }
        } catch (error) {
            console.error('Failed to create new diagram:', error);
            await this.uiManager.showError(
                'Failed to create diagram', 
                error instanceof Error ? error.message : 'Unknown error'
            );
        }
    }
    
    private setupCanvasDragAndDrop(): void {
        console.log('Setting up canvas drag and drop...');
        
        // Prevent default drag behaviors
        this.canvas.addEventListener('dragover', (e) => {
            e.preventDefault();
            e.dataTransfer!.dropEffect = 'copy';
        });
        
        this.canvas.addEventListener('drop', async (e) => {
            e.preventDefault();
            
            // Get the drag data
            const dragDataString = e.dataTransfer?.getData('application/json');
            if (!dragDataString) {
                console.log('No drag data received');
                return;
            }
            
            try {
                const dragData = JSON.parse(dragDataString);
                console.log('Canvas drop received:', dragData);
                
                // Only handle WASM component drops
                if (dragData.type !== 'wasm-component') {
                    console.log('Not a WASM component drop, ignoring');
                    return;
                }
                
                // Get the drop position relative to the canvas
                const rect = this.canvas.getBoundingClientRect();
                const x = e.clientX - rect.left;
                const y = e.clientY - rect.top;
                
                // Convert to world coordinates (account for canvas transform)
                const worldPos = this.renderer.screenToWorld(x, y);
                
                // Create a WASM component node at the drop position
                const nodeData = {
                    type: 'wasm-component',
                    label: dragData.name,
                    properties: {
                        componentName: dragData.id,
                        componentPath: dragData.path || '',
                        category: dragData.category || 'WASM Components',
                        status: 'unloaded',
                        interfaces: dragData.interfaces || []
                    },
                    x: worldPos.x,
                    y: worldPos.y
                };
                
                console.log('Creating WASM node at position:', worldPos, 'with data:', nodeData);
                
                // Get current diagram ID
                const diagramId = this.diagramService.getCurrentDiagramId();
                if (!diagramId) {
                    console.error('No active diagram to add component to');
                    this.uiManager.updateStatus('Please create or load a diagram first');
                    return;
                }
                
                // Create the node using the diagram service
                try {
                    // Use the correct method signature with properties
                    await this.diagramService.createNode(
                        diagramId,
                        nodeData.type,  // nodeType
                        { x: nodeData.x, y: nodeData.y },  // position
                        nodeData.label,  // label
                        nodeData.properties  // properties (includes interfaces)
                    );
                    
                    console.log('WASM component node created successfully');
                    this.uiManager.updateStatus(`Added ${dragData.name} to diagram`);
                    
                    // The diagram service automatically refreshes the diagram after creating the node
                    // Now update the renderer to show the new component immediately
                    const currentDiagram = this.diagramService.getCurrentDiagram();
                    if (currentDiagram) {
                        console.log('Updating renderer with refreshed diagram');
                        this.renderer.setDiagram(currentDiagram);
                    }
                } catch (error) {
                    console.error('Failed to create WASM component node:', error);
                    this.uiManager.updateStatus('Failed to add component to diagram');
                }
            } catch (error) {
                console.error('Failed to parse drag data:', error);
            }
        });
        
        console.log('Canvas drag and drop setup complete');
    }
    
    public testSidebarIntegration(): void {
        console.log('=== SIDEBAR INTEGRATION TEST ===');
        console.log('Sidebar container:', document.querySelector('.sidebar'));
        console.log('UIManager sidebar:', this.uiManager);
        
        // Test adding another WASM component
        this.uiManager.addWasmComponentToLibrary({
            name: 'Test Component',
            description: 'Runtime test component',
            interfaces: ['test'],
            status: 'available',
            category: 'Testing'
        });
        
        console.log('Test component added');
        
        // Debug AI panel callback
        console.log('AI Panel events check:', this.uiManager.getAIPanelElement());
        console.log('Header icon manager:', this.uiManager.getHeaderIconManager());
    }
}