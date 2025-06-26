/**
 * Main GLSP Web Client Application
 * Integrates MCP client, diagram state, and canvas renderer
 */

import { McpClient } from './mcp/client.js';
import { DiagramState, DiagramModel } from './model/diagram.js';
import { CanvasRenderer, InteractionEvent } from './renderer/canvas-renderer.js';
import { OllamaClient } from './ai/ollama-client.js';
import { DiagramAgent, DiagramRequest } from './ai/diagram-agent.js';
import { InteractionMode, DEFAULT_NODE_TYPES, DEFAULT_EDGE_TYPES } from './interaction/interaction-mode.js';

export class GLSPApp {
    private mcpClient: McpClient;
    private diagramState: DiagramState;
    private renderer: CanvasRenderer;
    private canvas: HTMLCanvasElement;
    private currentDiagramId?: string;

    // AI Components
    private ollamaClient: OllamaClient;
    private diagramAgent: DiagramAgent;

    // UI Elements
    private toolbarElement: HTMLElement;
    private statusElement: HTMLElement;
    private diagramListElement: HTMLElement;
    private aiPanelElement: HTMLElement;

    constructor(canvasElement: HTMLCanvasElement) {
        this.canvas = canvasElement;
        this.mcpClient = new McpClient();
        this.diagramState = new DiagramState();
        this.renderer = new CanvasRenderer(this.canvas);

        // Initialize AI components
        this.ollamaClient = new OllamaClient();
        this.diagramAgent = new DiagramAgent(this.ollamaClient, this.mcpClient);

        // Initialize UI elements
        this.toolbarElement = this.createToolbar();
        this.statusElement = this.createStatusBar();
        this.diagramListElement = this.createDiagramList();
        this.aiPanelElement = this.createAIPanel();

        this.setupEventHandlers();
        this.initialize();
    }

    private async initialize(): Promise<void> {
        try {
            this.updateStatus('Connecting to MCP server...');
            
            // Initialize MCP connection
            await this.mcpClient.initialize();
            this.updateStatus('Connected to MCP server');

            // Load available tools and resources
            await this.loadAvailableCapabilities();
            
            // Check AI connections
            await this.checkAIConnections();
            
            // Create a sample diagram
            await this.createSampleDiagram();

        } catch (error) {
            console.error('Failed to initialize GLSP app:', error);
            this.updateStatus('Failed to connect to server');
        }
    }

    private async checkAIConnections(): Promise<void> {
        const connections = await this.diagramAgent.checkConnections();
        
        const ollamaStatus = connections.ollama ? 'Connected' : 'Disconnected';
        const mcpStatus = connections.mcp ? 'Connected' : 'Disconnected';
        
        console.log(`AI Status - Ollama: ${ollamaStatus}, MCP: ${mcpStatus}`);
        
        // Auto-select available model if Ollama is connected
        if (connections.ollama) {
            try {
                const selectedModel = await this.ollamaClient.autoSelectModel();
                console.log(`Auto-selected model: ${selectedModel}`);
                await this.loadAvailableModels();
            } catch (error) {
                console.warn('Failed to load models:', error);
            }
        }
        
        this.updateAIStatus(connections.ollama, connections.mcp);
    }

    private async loadAvailableModels(): Promise<void> {
        try {
            const models = await this.ollamaClient.getAvailableModelNames();
            const currentModel = this.ollamaClient.getDefaultModel();
            
            const modelSelect = this.aiPanelElement.querySelector('#model-select') as HTMLSelectElement;
            if (modelSelect) {
                modelSelect.innerHTML = '';
                
                models.forEach(modelName => {
                    const option = document.createElement('option');
                    option.value = modelName;
                    option.textContent = modelName;
                    option.selected = modelName === currentModel;
                    modelSelect.appendChild(option);
                });
                
                // Add change handler
                modelSelect.addEventListener('change', () => {
                    this.ollamaClient.setDefaultModel(modelSelect.value);
                    console.log(`Switched to model: ${modelSelect.value}`);
                });
            }
        } catch (error) {
            console.error('Failed to load models:', error);
        }
    }

    private updateAIStatus(ollamaConnected: boolean, mcpConnected: boolean): void {
        const aiStatusElement = this.aiPanelElement.querySelector('#ai-status');
        if (aiStatusElement) {
            const ollamaIcon = ollamaConnected ? '🟢' : '🔴';
            const mcpIcon = mcpConnected ? '🟢' : '🔴';
            aiStatusElement.innerHTML = `
                <div>Ollama: ${ollamaIcon} ${ollamaConnected ? 'Connected' : 'Disconnected'}</div>
                <div>MCP: ${mcpIcon} ${mcpConnected ? 'Connected' : 'Disconnected'}</div>
            `;
        }
    }

    private async createDiagramFromAI(description: string): Promise<void> {
        const outputElement = this.aiPanelElement.querySelector('#ai-output') as HTMLElement;
        
        try {
            outputElement.innerHTML = '<div class="ai-thinking">🤖 AI is thinking...</div>';
            
            const request: DiagramRequest = {
                description,
                diagramType: 'workflow'
            };
            
            const response = await this.diagramAgent.createDiagramFromDescription(request);
            
            // Display step-by-step progress
            let output = '<div class="ai-response">';
            output += `<h4>${response.success ? '✅' : '❌'} ${response.message}</h4>`;
            
            output += '<div class="ai-steps">';
            response.steps.forEach(step => {
                output += `<div class="step">${step}</div>`;
            });
            output += '</div>';
            
            if (response.errors && response.errors.length > 0) {
                output += '<div class="ai-errors">';
                response.errors.forEach(error => {
                    output += `<div class="error">❌ ${error}</div>`;
                });
                output += '</div>';
            }
            
            output += '</div>';
            outputElement.innerHTML = output;
            
            // Load the created diagram
            if (response.success && response.diagramId) {
                this.currentDiagramId = response.diagramId;
                await this.loadDiagram(response.diagramId);
            }
            
        } catch (error) {
            outputElement.innerHTML = `<div class="ai-error">❌ AI Error: ${error}</div>`;
        }
    }

    private async loadAvailableCapabilities(): Promise<void> {
        try {
            const [tools, resources, prompts] = await Promise.all([
                this.mcpClient.listTools(),
                this.mcpClient.listResources(),
                this.mcpClient.listPrompts()
            ]);

            console.log('Available tools:', tools);
            console.log('Available resources:', resources);
            console.log('Available prompts:', prompts);

            this.updateToolbar(tools, prompts);
        } catch (error) {
            console.error('Failed to load capabilities:', error);
        }
    }

    private setupEventHandlers(): void {
        // Diagram state events
        this.diagramState.addEventHandler((event) => {
            if (event.type === 'model-updated' && event.model) {
                this.renderer.setDiagram(event.model);
                this.updateDiagramList();
            }
        });

        // Renderer interaction events
        this.renderer.addInteractionHandler((event: InteractionEvent) => {
            this.handleRendererInteraction(event);
        });
        
        // Selection change events
        this.renderer.getSelectionManager().addChangeHandler((change) => {
            this.handleSelectionChange(change);
        });

        // Keyboard shortcuts
        document.addEventListener('keydown', (event) => {
            this.handleKeyboardShortcut(event);
        });
    }

    private handleRendererInteraction(event: InteractionEvent): void {
        switch (event.type) {
            case 'click':
                console.log('Element clicked:', event.element?.id);
                break;
            case 'drag-end':
                if (this.currentDiagramId) {
                    this.updateSelectedElementPositions();
                }
                break;
            case 'hover':
                if (this.currentDiagramId && event.element) {
                    this.mcpClient.callTool('hover_element', {
                        diagramId: this.currentDiagramId,
                        elementId: event.element.id
                    }).catch(console.error);
                }
                break;
            case 'canvas-click':
                if (this.currentDiagramId) {
                    this.createNodeAtPosition(event.position);
                }
                break;
            case 'edge-end':
                if (this.currentDiagramId && event.sourceElement && event.element) {
                    this.createEdgeBetween(event.sourceElement.id, event.element.id);
                }
                break;
        }
    }

    private handleKeyboardShortcut(event: KeyboardEvent): void {
        if (event.ctrlKey || event.metaKey) {
            switch (event.key) {
                case 'n':
                    event.preventDefault();
                    this.createNewDiagram();
                    break;
                case 's':
                    event.preventDefault();
                    this.saveDiagram();
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
                case 'a':
                    event.preventDefault();
                    this.selectAllElements();
                    break;
            }
        }

        switch (event.key) {
            case 'Delete':
            case 'Backspace':
                event.preventDefault();
                this.deleteSelectedElements();
                break;
            case 'Escape':
                event.preventDefault();
                this.clearSelection();
                break;
        }
    }
    
    private async handleSelectionChange(change: any): Promise<void> {
        if (!this.currentDiagramId) return;
        
        try {
            // Sync selection with backend
            await this.mcpClient.callTool('select_elements', {
                diagramId: this.currentDiagramId,
                elementIds: change.current,
                mode: 'multiple',
                append: false
            });
        } catch (error) {
            console.error('Failed to sync selection:', error);
        }
    }

    private async createSampleDiagram(): Promise<void> {
        try {
            // Create a new workflow diagram
            const result = await this.mcpClient.callTool('create_diagram', {
                diagramType: 'workflow',
                name: 'Sample Workflow'
            });

            console.log('Created diagram:', result);

            // Get the diagram ID from the result text
            const match = result.content[0]?.text.match(/ID: ([a-f0-9-]+)/);
            if (match) {
                const diagramId = match[1];
                this.currentDiagramId = diagramId;

                // Add some sample nodes
                await this.addSampleNodes(diagramId);
                
                // Load and display the diagram
                await this.loadDiagram(diagramId);
            }
        } catch (error) {
            console.error('Failed to create sample diagram:', error);
        }
    }

    private async addSampleNodes(diagramId: string): Promise<void> {
        try {
            // Create start node
            await this.mcpClient.callTool('create_node', {
                diagramId,
                nodeType: 'start-event',
                position: { x: 50, y: 100 },
                label: 'Start'
            });

            // Create task node
            await this.mcpClient.callTool('create_node', {
                diagramId,
                nodeType: 'task',
                position: { x: 200, y: 100 },
                label: 'Process Order'
            });

            // Create gateway
            await this.mcpClient.callTool('create_node', {
                diagramId,
                nodeType: 'gateway',
                position: { x: 350, y: 100 },
                label: 'Valid?'
            });

            // Create end node
            await this.mcpClient.callTool('create_node', {
                diagramId,
                nodeType: 'end-event',
                position: { x: 500, y: 100 },
                label: 'End'
            });

            console.log('Sample nodes created');
        } catch (error) {
            console.error('Failed to create sample nodes:', error);
        }
    }

    private async loadDiagram(diagramId: string): Promise<void> {
        try {
            const modelResource = await this.mcpClient.readResource(`diagram://model/${diagramId}`);
            
            if (modelResource.text) {
                const diagram: DiagramModel = JSON.parse(modelResource.text);
                this.diagramState.updateDiagram(diagram);
                this.currentDiagramId = diagramId;
                this.updateStatus(`Loaded diagram: ${diagram.diagramType}`);
            }
        } catch (error) {
            console.error('Failed to load diagram:', error);
            this.updateStatus('Failed to load diagram');
        }
    }

    private async updateSelectedElementPositions(): Promise<void> {
        if (!this.currentDiagramId) return;
        
        const selectedIds = this.renderer.getSelectionManager().getSelectedIds();
        const diagram = this.diagramState.getDiagram(this.currentDiagramId);
        if (!diagram) return;
        
        try {
            // Update positions for all selected elements
            for (const elementId of selectedIds) {
                const element = diagram.elements[elementId];
                if (element?.bounds) {
                    await this.mcpClient.callTool('update_element', {
                        diagramId: this.currentDiagramId,
                        elementId,
                        position: {
                            x: element.bounds.x,
                            y: element.bounds.y
                        }
                    });
                }
            }
            
            // Reload the diagram to ensure consistency
            await this.loadDiagram(this.currentDiagramId);
            this.updateStatus(`Moved ${selectedIds.length} element(s)`);
        } catch (error) {
            console.error('Failed to update element positions:', error);
        }
    }

    private async createNewDiagram(): Promise<void> {
        const diagramType = prompt('Enter diagram type (workflow, bpmn, uml):') || 'workflow';
        
        try {
            const result = await this.mcpClient.callTool('create_diagram', {
                diagramType,
                name: `New ${diagramType}`
            });

            const match = result.content[0]?.text.match(/ID: ([a-f0-9-]+)/);
            if (match) {
                const diagramId = match[1];
                await this.loadDiagram(diagramId);
            }
        } catch (error) {
            console.error('Failed to create new diagram:', error);
        }
    }

    private async saveDiagram(): Promise<void> {
        if (!this.currentDiagramId) return;

        try {
            const result = await this.mcpClient.callTool('export_diagram', {
                diagramId: this.currentDiagramId,
                format: 'json'
            });

            console.log('Diagram saved:', result);
            this.updateStatus('Diagram saved');
        } catch (error) {
            console.error('Failed to save diagram:', error);
        }
    }

    private async deleteSelectedElements(): Promise<void> {
        if (!this.currentDiagramId) return;
        
        const selectedIds = this.renderer.getSelectionManager().getSelectedIds();
        if (selectedIds.length === 0) return;
        
        try {
            for (const elementId of selectedIds) {
                await this.mcpClient.callTool('delete_element', {
                    diagramId: this.currentDiagramId,
                    elementId
                });
            }
            
            // Clear selection after deletion
            this.renderer.getSelectionManager().clearSelection();
            
            // Reload the diagram to reflect changes
            await this.loadDiagram(this.currentDiagramId);
            this.updateStatus(`Deleted ${selectedIds.length} element(s)`);
        } catch (error) {
            console.error('Failed to delete elements:', error);
        }
    }
    
    private async selectAllElements(): Promise<void> {
        if (!this.currentDiagramId) return;
        
        const diagram = this.diagramState.getDiagram(this.currentDiagramId);
        if (!diagram) return;
        
        const allIds = Object.keys(diagram.elements)
            .filter(id => id !== diagram.root.id);
        
        this.renderer.getSelectionManager().selectAll(allIds);
    }
    
    private clearSelection(): void {
        this.renderer.getSelectionManager().clearSelection();
    }
    
    private async applyLayout(): Promise<void> {
        if (!this.currentDiagramId) return;
        
        const algorithm = prompt('Layout algorithm (grid, hierarchical):') || 'grid';
        
        try {
            await this.mcpClient.callTool('apply_layout', {
                diagramId: this.currentDiagramId,
                algorithm
            });
            
            // Reload diagram to show new layout
            await this.loadDiagram(this.currentDiagramId);
            this.updateStatus(`Applied ${algorithm} layout`);
        } catch (error) {
            console.error('Failed to apply layout:', error);
        }
    }
    
    private async createNodeAtPosition(position: { x: number; y: number }): Promise<void> {
        if (!this.currentDiagramId) return;
        
        const nodeType = this.renderer.getModeManager().getSelectedNodeType();
        const label = prompt(`Enter label for ${nodeType}:`) || `New ${nodeType}`;
        
        try {
            const result = await this.mcpClient.callTool('create_node', {
                diagramId: this.currentDiagramId,
                nodeType,
                position,
                label
            });
            
            console.log('Created node:', result);
            
            // Reload diagram to show new node
            await this.loadDiagram(this.currentDiagramId);
            
            // Switch back to select mode
            this.renderer.getModeManager().setMode(InteractionMode.Select);
            this.updateStatus(`Created ${nodeType} node`);
        } catch (error) {
            console.error('Failed to create node:', error);
        }
    }
    
    private async createEdgeBetween(sourceId: string, targetId: string): Promise<void> {
        if (!this.currentDiagramId) return;
        
        const edgeType = this.renderer.getModeManager().getSelectedEdgeType();
        const label = prompt('Enter edge label (optional):') || undefined;
        
        try {
            const result = await this.mcpClient.callTool('create_edge', {
                diagramId: this.currentDiagramId,
                edgeType,
                sourceId,
                targetId,
                label
            });
            
            console.log('Created edge:', result);
            
            // Reload diagram to show new edge
            await this.loadDiagram(this.currentDiagramId);
            
            // Stay in edge creation mode for multiple edges
            this.updateStatus(`Created ${edgeType} edge`);
        } catch (error) {
            console.error('Failed to create edge:', error);
        }
    }

    private createToolbar(): HTMLElement {
        const toolbar = document.createElement('div');
        toolbar.className = 'glsp-toolbar';
        toolbar.innerHTML = `
            <div class="toolbar-group">
                <label>Mode:</label>
                <button id="mode-select" class="active">Select</button>
                <button id="mode-pan">Pan</button>
            </div>
            <div class="toolbar-group">
                <label>Create Node:</label>
                ${DEFAULT_NODE_TYPES.map(nodeType => 
                    `<button class="node-type" data-type="${nodeType.type}">${nodeType.label}</button>`
                ).join('')}
            </div>
            <div class="toolbar-group">
                <label>Create Edge:</label>
                ${DEFAULT_EDGE_TYPES.map(edgeType => 
                    `<button class="edge-type" data-type="${edgeType.type}">${edgeType.label}</button>`
                ).join('')}
            </div>
            <div class="toolbar-group">
                <button id="apply-layout">Apply Layout</button>
                <button id="zoom-in">Zoom In</button>
                <button id="zoom-out">Zoom Out</button>
                <button id="fit-content">Fit</button>
            </div>
        `;

        const modeManager = this.renderer.getModeManager();
        
        // Mode buttons
        toolbar.querySelector('#mode-select')?.addEventListener('click', (e) => {
            this.setActiveButton(e.target as HTMLElement, '.toolbar-group button');
            modeManager.setMode(InteractionMode.Select);
        });
        
        toolbar.querySelector('#mode-pan')?.addEventListener('click', (e) => {
            this.setActiveButton(e.target as HTMLElement, '.toolbar-group button');
            modeManager.setMode(InteractionMode.Pan);
        });
        
        // Node type buttons
        toolbar.querySelectorAll('.node-type').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const button = e.target as HTMLElement;
                const nodeType = button.dataset.type!;
                this.setActiveButton(button, '.toolbar-group button');
                modeManager.setSelectedNodeType(nodeType);
            });
        });
        
        // Edge type buttons
        toolbar.querySelectorAll('.edge-type').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const button = e.target as HTMLElement;
                const edgeType = button.dataset.type!;
                this.setActiveButton(button, '.toolbar-group button');
                modeManager.setSelectedEdgeType(edgeType);
            });
        });
        
        // Other buttons
        toolbar.querySelector('#apply-layout')?.addEventListener('click', () => this.applyLayout());
        toolbar.querySelector('#zoom-in')?.addEventListener('click', () => this.renderer.zoom(1.2));
        toolbar.querySelector('#zoom-out')?.addEventListener('click', () => this.renderer.zoom(0.8));
        toolbar.querySelector('#fit-content')?.addEventListener('click', () => this.renderer.fitToContent());

        return toolbar;
    }
    
    private setActiveButton(button: HTMLElement, selector: string): void {
        const parent = button.closest('.glsp-toolbar');
        parent?.querySelectorAll(selector).forEach(btn => btn.classList.remove('active'));
        button.classList.add('active');
    }

    private createStatusBar(): HTMLElement {
        const status = document.createElement('div');
        status.className = 'glsp-status';
        status.innerHTML = '<span id="status-text">Initializing...</span>';
        return status;
    }

    private createDiagramList(): HTMLElement {
        const list = document.createElement('div');
        list.className = 'glsp-diagram-list';
        list.innerHTML = '<h3>Diagrams</h3><ul id="diagram-list"></ul>';
        return list;
    }

    private createAIPanel(): HTMLElement {
        const panel = document.createElement('div');
        panel.className = 'glsp-ai-panel';
        panel.innerHTML = `
            <h3>🤖 AI Diagram Assistant</h3>
            <div id="ai-status" class="ai-status">
                <div>Checking connections...</div>
            </div>
            <div class="ai-model-section">
                <label for="model-select">AI Model:</label>
                <select id="model-select">
                    <option>Loading models...</option>
                </select>
            </div>
            <div class="ai-input-section">
                <label for="ai-description">Describe your diagram:</label>
                <textarea 
                    id="ai-description" 
                    placeholder="e.g., Create a workflow for processing customer orders with payment validation and inventory check"
                    rows="3"
                ></textarea>
                <button id="ai-create-btn">Create Diagram</button>
            </div>
            <div class="ai-actions">
                <button id="ai-analyze-btn">Analyze Current Diagram</button>
                <button id="ai-optimize-btn">Optimize Layout</button>
            </div>
            <div id="ai-output" class="ai-output">
                <div class="ai-placeholder">Use the AI assistant to create diagrams from natural language descriptions!</div>
            </div>
        `;

        // Add event listeners
        panel.querySelector('#ai-create-btn')?.addEventListener('click', async () => {
            const textarea = panel.querySelector('#ai-description') as HTMLTextAreaElement;
            const description = textarea.value.trim();
            if (description) {
                await this.createDiagramFromAI(description);
                textarea.value = ''; // Clear after creation
            }
        });

        panel.querySelector('#ai-analyze-btn')?.addEventListener('click', async () => {
            if (this.currentDiagramId) {
                await this.analyzeDiagramWithAI();
            } else {
                const outputElement = panel.querySelector('#ai-output') as HTMLElement;
                outputElement.innerHTML = '<div class="ai-error">❌ No diagram selected for analysis</div>';
            }
        });

        panel.querySelector('#ai-optimize-btn')?.addEventListener('click', async () => {
            if (this.currentDiagramId) {
                await this.optimizeDiagramWithAI();
            } else {
                const outputElement = panel.querySelector('#ai-output') as HTMLElement;
                outputElement.innerHTML = '<div class="ai-error">❌ No diagram selected for optimization</div>';
            }
        });

        return panel;
    }

    private updateStatus(message: string): void {
        const statusText = this.statusElement.querySelector('#status-text');
        if (statusText) {
            statusText.textContent = message;
        }
    }

    private updateToolbar(_tools: any[], prompts: any[]): void {
        // Add dynamic buttons for available prompts
        const promptGroup = document.createElement('div');
        promptGroup.className = 'toolbar-group';
        promptGroup.innerHTML = '<label>AI Prompts:</label>';

        prompts.forEach(prompt => {
            const button = document.createElement('button');
            button.textContent = prompt.name.replace(/_/g, ' ');
            button.addEventListener('click', () => this.runPrompt(prompt.name));
            promptGroup.appendChild(button);
        });

        this.toolbarElement.appendChild(promptGroup);
    }

    private async updateDiagramList(): Promise<void> {
        try {
            const listResource = await this.mcpClient.readResource('diagram://list');
            if (listResource.text) {
                const data = JSON.parse(listResource.text);
                const listElement = this.diagramListElement.querySelector('#diagram-list');
                
                if (listElement) {
                    listElement.innerHTML = '';
                    data.diagrams.forEach((diagram: any) => {
                        const li = document.createElement('li');
                        li.innerHTML = `
                            <span>${diagram.type} (${diagram.elementCount} elements)</span>
                            <button onclick="app.loadDiagramPublic('${diagram.id}')">Load</button>
                        `;
                        listElement.appendChild(li);
                    });
                }
            }
        } catch (error) {
            console.error('Failed to update diagram list:', error);
        }
    }

    private async runPrompt(promptName: string): Promise<void> {
        try {
            const prompt = await this.mcpClient.getPrompt(promptName, {
                diagram_id: this.currentDiagramId || ''
            });

            console.log('AI Prompt:', prompt);
            alert('AI prompt generated! Check console for details. In a real implementation, this would be sent to an AI agent.');
        } catch (error) {
            console.error('Failed to run prompt:', error);
        }
    }

    private async analyzeDiagramWithAI(): Promise<void> {
        if (!this.currentDiagramId) return;
        
        const outputElement = this.aiPanelElement.querySelector('#ai-output') as HTMLElement;
        
        try {
            outputElement.innerHTML = '<div class="ai-thinking">🤖 Analyzing diagram...</div>';
            
            const response = await this.diagramAgent.analyzeDiagram(this.currentDiagramId, 'general');
            
            let output = '<div class="ai-response">';
            output += `<h4>📊 Diagram Analysis</h4>`;
            output += `<div class="ai-analysis">${response.message}</div>`;
            
            if (response.steps.length > 0) {
                output += '<div class="ai-steps">';
                response.steps.forEach(step => {
                    output += `<div class="step">${step}</div>`;
                });
                output += '</div>';
            }
            
            output += '</div>';
            outputElement.innerHTML = output;
            
        } catch (error) {
            outputElement.innerHTML = `<div class="ai-error">❌ Analysis Error: ${error}</div>`;
        }
    }

    private async optimizeDiagramWithAI(): Promise<void> {
        if (!this.currentDiagramId) return;
        
        const outputElement = this.aiPanelElement.querySelector('#ai-output') as HTMLElement;
        
        try {
            outputElement.innerHTML = '<div class="ai-thinking">🤖 Optimizing diagram...</div>';
            
            const response = await this.diagramAgent.optimizeDiagram(this.currentDiagramId, 'readability');
            
            let output = '<div class="ai-response">';
            output += `<h4>${response.success ? '✅' : '❌'} ${response.message}</h4>`;
            
            output += '<div class="ai-steps">';
            response.steps.forEach(step => {
                output += `<div class="step">${step}</div>`;
            });
            output += '</div>';
            
            output += '</div>';
            outputElement.innerHTML = output;
            
            // Reload the diagram to show optimizations
            if (response.success) {
                await this.loadDiagram(this.currentDiagramId);
            }
            
        } catch (error) {
            outputElement.innerHTML = `<div class="ai-error">❌ Optimization Error: ${error}</div>`;
        }
    }

    // Public API for HTML
    getToolbar(): HTMLElement { return this.toolbarElement; }
    getStatus(): HTMLElement { return this.statusElement; }
    getDiagramList(): HTMLElement { return this.diagramListElement; }
    getAIPanel(): HTMLElement { return this.aiPanelElement; }
    async loadDiagramPublic(diagramId: string): Promise<void> { return this.loadDiagram(diagramId); }
}