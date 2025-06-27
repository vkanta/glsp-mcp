import { getNodeTypesForDiagram, getEdgeTypesForDiagram } from '../diagrams/diagram-type-registry.js';
import { diagramTypeRegistry } from '../diagrams/diagram-type-registry.js';
import { statusManager, ConnectionStatus } from '../services/StatusManager.js';

export class UIManager {
    private toolbarElement: HTMLElement;
    private statusElement: HTMLElement;
    private diagramListElement: HTMLElement;
    private aiPanelElement: HTMLElement;
    private _wasmPaletteElement: HTMLElement;
    private currentMode: string = 'select';
    private currentNodeType: string = '';
    private currentEdgeType: string = '';

    constructor() {
        console.log('UIManager: Creating UI elements');
        this.toolbarElement = this.createToolbar();
        this.statusElement = this.createStatusBar();
        this.diagramListElement = this.createDiagramList();
        this.aiPanelElement = this.createAIPanel();
        this._wasmPaletteElement = document.createElement('div'); // Placeholder
        
        // Setup unified status listening
        statusManager.addListener((status: ConnectionStatus) => {
            this.updateUnifiedStatus(status);
        });
        
        // Setup keyboard shortcuts
        this.setupKeyboardShortcuts();
        
        console.log('UIManager: UI elements created');
    }

    public getToolbarElement(): HTMLElement { return this.toolbarElement; }
    public getStatusElement(): HTMLElement { return this.statusElement; }
    public getDiagramListElement(): HTMLElement { return this.diagramListElement; }
    public getAIPanelElement(): HTMLElement { return this.aiPanelElement; }

    private createToolbar(): HTMLElement {
        const toolbar = document.createElement('div');
        toolbar.className = 'glsp-toolbar';
        this.updateToolbarContent(toolbar, 'workflow');
        return toolbar;
    }
    
    public updateToolbarContent(toolbar: HTMLElement, diagramType: string): void {
        const nodeTypes = getNodeTypesForDiagram(diagramType);
        const edgeTypes = getEdgeTypesForDiagram(diagramType);
        const availableTypes = diagramTypeRegistry.getAvailableTypes();
        
        console.log('=== UPDATING TOOLBAR ===');
        console.log('Diagram type:', diagramType);
        console.log('Node type labels:', nodeTypes.map(n => n.label));
        console.log('Edge type labels:', edgeTypes.map(e => e.label));
        
        // Store current values before updating innerHTML
        const currentMode = this.currentMode;
        const currentNodeType = this.currentNodeType;
        const currentEdgeType = this.currentEdgeType;
        
        const newHTML = `
            <div class="toolbar-group">
                <label>Diagram Type: (${new Date().getSeconds()}s)</label>
                <select id="diagram-type-select">
                    ${availableTypes.map(type => 
                        `<option value="${type.type}" ${type.type === diagramType ? 'selected' : ''}>${type.label}</option>`
                    ).join('')}
                </select>
            </div>
            <div class="toolbar-group">
                <label>Mode:</label>
                <button id="mode-select" class="active">Select</button>
                <button id="mode-pan">Pan</button>
            </div>
            <div class="toolbar-group">
                <label>Create Node:</label>
                ${nodeTypes.map(nodeType => 
                    `<button class="node-type" data-type="${nodeType.type}" title="${nodeType.icon || ''}">
                        ${nodeType.icon || ''} ${nodeType.label}
                    </button>`
                ).join('')}
            </div>
            <div class="toolbar-group">
                <label>Create Edge:</label>
                ${edgeTypes.map(edgeType => 
                    `<button class="edge-type" data-type="${edgeType.type}">${edgeType.label}</button>`
                ).join('')}
            </div>
            <div class="toolbar-group">
                <button id="apply-layout">Apply Layout</button>
                <button id="zoom-in">Zoom In</button>
                <button id="zoom-out">Zoom Out</button>
                <button id="fit-content">Fit</button>
                <button id="reset-view">Reset</button>
            </div>
        `;
        
        // console.log('Generated HTML for toolbar:', newHTML);
        toolbar.innerHTML = newHTML;
        
        // Restore current values
        this.currentMode = currentMode;
        this.currentNodeType = currentNodeType;
        this.currentEdgeType = currentEdgeType;
        
        // Re-setup event handlers for the newly created elements
        console.log('Re-setting up toolbar button handlers after content update');
        this.setupToolbarButtonHandlers(toolbar);
        
        // Re-setup diagram type change handler if needed
        this.setupDiagramTypeChangeHandler(toolbar);
    }

    private setupToolbarButtonHandlers(toolbar?: HTMLElement): void {
        const toolbarEl = toolbar || this.toolbarElement;
        
        // Mode buttons
        toolbarEl.querySelector('#mode-select')?.addEventListener('click', () => {
            this.setMode('select');
        });
        
        toolbarEl.querySelector('#mode-pan')?.addEventListener('click', () => {
            this.setMode('pan');
        });
        
        // Node creation buttons
        toolbarEl.querySelectorAll('.node-type').forEach(button => {
            button.addEventListener('click', (e) => {
                const btn = e.currentTarget as HTMLButtonElement;
                const nodeType = btn.getAttribute('data-type');
                console.log('Node type button clicked:', nodeType);
                if (nodeType) {
                    this.setMode('create-node');
                    this.currentNodeType = nodeType;
                    this.updateActiveButton(btn, '.node-type');
                    console.log('Set mode to create-node, nodeType:', this.currentNodeType);
                }
            });
        });
        
        // Edge creation buttons
        toolbarEl.querySelectorAll('.edge-type').forEach(button => {
            button.addEventListener('click', (e) => {
                const btn = e.currentTarget as HTMLButtonElement;
                const edgeType = btn.getAttribute('data-type');
                console.log('Edge type button clicked:', edgeType);
                if (edgeType) {
                    this.setMode('create-edge');
                    this.currentEdgeType = edgeType;
                    this.updateActiveButton(btn, '.edge-type');
                    console.log('Set mode to create-edge, edgeType:', this.currentEdgeType);
                }
            });
        });
        
        // View control buttons
        toolbarEl.querySelector('#zoom-in')?.addEventListener('click', () => {
            window.dispatchEvent(new CustomEvent('toolbar-zoom', { detail: { direction: 'in' } }));
        });
        
        toolbarEl.querySelector('#zoom-out')?.addEventListener('click', () => {
            window.dispatchEvent(new CustomEvent('toolbar-zoom', { detail: { direction: 'out' } }));
        });
        
        toolbarEl.querySelector('#fit-content')?.addEventListener('click', () => {
            window.dispatchEvent(new CustomEvent('toolbar-fit-content'));
        });
        
        toolbarEl.querySelector('#reset-view')?.addEventListener('click', () => {
            window.dispatchEvent(new CustomEvent('toolbar-reset-view'));
        });
        
        toolbarEl.querySelector('#apply-layout')?.addEventListener('click', () => {
            window.dispatchEvent(new CustomEvent('toolbar-apply-layout'));
        });
    }
    
    private setMode(mode: string): void {
        this.currentMode = mode;
        window.dispatchEvent(new CustomEvent('toolbar-mode-change', { detail: { mode } }));
        
        // Update active button styling
        if (mode === 'select') {
            const selectBtn = this.toolbarElement?.querySelector('#mode-select');
            if (selectBtn) this.updateActiveButton(selectBtn, '#mode-select, #mode-pan');
        } else if (mode === 'pan') {
            const panBtn = this.toolbarElement?.querySelector('#mode-pan');
            if (panBtn) this.updateActiveButton(panBtn, '#mode-select, #mode-pan');
        }
    }
    
    private updateActiveButton(activeBtn: Element | null, selector: string): void {
        if (!activeBtn) return;
        
        // Find the toolbar element that contains the button
        const toolbar = activeBtn.closest('.glsp-toolbar') || this.toolbarElement;
        
        toolbar.querySelectorAll(selector).forEach(btn => {
            btn.classList.remove('active');
        });
        activeBtn.classList.add('active');
    }
    
    public getCurrentMode(): string {
        return this.currentMode;
    }
    
    public getCurrentNodeType(): string {
        return this.currentNodeType;
    }
    
    public getCurrentEdgeType(): string {
        return this.currentEdgeType;
    }
    
    private onDiagramTypeChangeCallback?: (newType: string) => void;

    public setupToolbarEventHandlers(onDiagramTypeChange: (newType: string) => void): void {
        // Store the callback for later use
        this.onDiagramTypeChangeCallback = onDiagramTypeChange;
        
        // Setup diagram type change handler
        this.setupDiagramTypeChangeHandler();
        
        // Setup all other toolbar button handlers
        this.setupToolbarButtonHandlers();
    }

    private setupDiagramTypeChangeHandler(toolbar?: HTMLElement): void {
        const toolbarEl = toolbar || this.toolbarElement;
        if (!this.onDiagramTypeChangeCallback) return;

        const selectElement = toolbarEl.querySelector('#diagram-type-select');
        if (selectElement) {
            selectElement.addEventListener('change', (e) => {
                const select = e.target as HTMLSelectElement;
                if (this.onDiagramTypeChangeCallback) {
                    this.onDiagramTypeChangeCallback(select.value);
                }
            });
        }
    }
    
    public setupAIPanelEventHandlers(
        onCreateDiagram: (prompt: string) => void,
        onTestAIDiagram: () => void,
        onAnalyzeDiagram: () => void,
        onOptimizeLayout: () => void
    ): void {
        // Main send button
        this.aiPanelElement.querySelector('#ai-send-btn')?.addEventListener('click', () => {
            this.handleAIMessage(onCreateDiagram);
        });
        
        // Quick action buttons
        this.aiPanelElement.querySelectorAll('.quick-action-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const action = (e.target as HTMLElement).getAttribute('data-action');
                switch (action) {
                    case 'create':
                        const input = this.aiPanelElement.querySelector('#ai-prompt') as HTMLInputElement;
                        if (input.value.trim()) {
                            this.handleAIMessage(onCreateDiagram);
                        } else {
                            this.addAIMessage('AI', 'Please describe what kind of diagram you\'d like me to create!');
                        }
                        break;
                    case 'analyze':
                        onAnalyzeDiagram();
                        break;
                    case 'optimize':
                        onOptimizeLayout();
                        break;
                    case 'test':
                        onTestAIDiagram();
                        break;
                }
            });
        });
        
        // Handle Enter key in prompt input
        const promptInput = this.aiPanelElement.querySelector('#ai-prompt') as HTMLInputElement;
        if (promptInput) {
            promptInput.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') {
                    e.preventDefault();
                    this.handleAIMessage(onCreateDiagram);
                }
            });
        }
    }

    private handleAIMessage(onCreateDiagram: (prompt: string) => void): void {
        const input = this.aiPanelElement.querySelector('#ai-prompt') as HTMLInputElement;
        if (input && input.value.trim()) {
            const userMessage = input.value.trim();
            this.addAIMessage('User', userMessage);
            input.value = '';
            
            // Process the message
            onCreateDiagram(userMessage);
        }
    }

    public addAIMessage(sender: 'AI' | 'User', content: string): void {
        const chatContainer = this.aiPanelElement.querySelector('#ai-chat');
        if (!chatContainer) return;

        // Simple markdown-style formatting
        const formattedContent = content
            .replace(/\n\n/g, '<br><br>')
            .replace(/\n/g, '<br>')
            .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
            .replace(/^• /gm, '• ');

        const messageDiv = document.createElement('div');
        messageDiv.className = `ai-message ${sender === 'User' ? 'user' : ''}`;
        messageDiv.innerHTML = `
            <div class="message-avatar">${sender === 'User' ? 'U' : 'AI'}</div>
            <div class="message-content">${formattedContent}</div>
        `;

        chatContainer.appendChild(messageDiv);
        chatContainer.scrollTop = chatContainer.scrollHeight;
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
        list.innerHTML = `
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
                <h3 style="margin: 0;">Diagrams</h3>
                <button id="create-new-diagram-btn" style="
                    background: var(--accent-wasm); 
                    color: white; 
                    border: none; 
                    padding: 6px 12px; 
                    border-radius: var(--radius-sm); 
                    cursor: pointer; 
                    font-size: 12px;
                    transition: all 0.2s ease;
                " title="Create New Diagram">+ New</button>
            </div>
            <ul id="diagram-list"></ul>
        `;
        return list;
    }

    private createAIPanel(): HTMLElement {
        const panel = document.createElement('div');
        panel.className = 'ai-assistant';
        panel.innerHTML = `
            <div class="ai-header">
                <div class="ai-title">
                    <div class="ai-icon">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="white">
                            <path d="M12 2L2 7L12 12L22 7L12 2Z"/>
                            <path d="M2 17L12 22L22 17"/>
                            <path d="M2 12L12 17L22 12"/>
                        </svg>
                    </div>
                    WASM Assistant
                </div>
                <div class="ai-header-actions">
                    <button class="ai-minimize-btn" title="Minimize">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M19 9l-7 7-7-7"/>
                        </svg>
                    </button>
                </div>
            </div>
            <div class="ai-status-bar" id="ai-status-display">
                <span id="ai-connection-status" style="color: var(--accent-error);">Offline</span>
                <div class="ai-model-section">
                    <select id="ai-model-select">
                        <option value="">Loading models...</option>
                    </select>
                </div>
            </div>
            
            <div class="ai-chat" id="ai-chat">
                <div class="ai-message">
                    <div class="message-avatar">AI</div>
                    <div class="message-content">
                        Hello! I can help you design and optimize your WebAssembly component architecture. What would you like to build?
                    </div>
                </div>
            </div>
            
            <div class="ai-input">
                <input type="text" class="ai-prompt" id="ai-prompt" placeholder="Ask about WASM components, optimization, or architecture...">
                <button id="ai-send-btn" style="background: var(--accent-wasm); border: none; color: white; padding: 10px 20px; border-radius: var(--radius-sm); cursor: pointer;">
                    Send
                </button>
            </div>
            
            <div class="ai-quick-actions" style="padding: 12px 16px; border-top: 1px solid var(--border); display: flex; gap: 8px; flex-wrap: wrap;">
                <button class="quick-action-btn" data-action="create" style="background: var(--bg-tertiary); border: 1px solid var(--border); color: var(--text-secondary); padding: 6px 12px; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px;">
                    📝 Create Diagram
                </button>
                <button class="quick-action-btn" data-action="analyze" style="background: var(--bg-tertiary); border: 1px solid var(--border); color: var(--text-secondary); padding: 6px 12px; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px;">
                    🔍 Analyze
                </button>
                <button class="quick-action-btn" data-action="optimize" style="background: var(--bg-tertiary); border: 1px solid var(--border); color: var(--text-secondary); padding: 6px 12px; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px;">
                    ⚡ Optimize
                </button>
                <button class="quick-action-btn" data-action="test" style="background: var(--bg-tertiary); border: 1px solid var(--border); color: var(--text-secondary); padding: 6px 12px; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px;">
                    🧪 Test
                </button>
            </div>
        `;
        
        // Setup drag functionality
        this.setupAIPanelDragging(panel);
        
        return panel;
    }

    private setupAIPanelDragging(panel: HTMLElement): void {
        const header = panel.querySelector('.ai-header') as HTMLElement;
        const minimizeBtn = panel.querySelector('.ai-minimize-btn') as HTMLElement;
        let isDragging = false;
        let startX = 0;
        let startY = 0;
        let startLeft = 0;
        let startTop = 0;

        // Make header draggable (but not the minimize button)
        header.style.cursor = 'move';
        
        // Minimize button handler
        minimizeBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            panel.classList.toggle('minimized');
            const icon = minimizeBtn.querySelector('svg');
            if (panel.classList.contains('minimized')) {
                icon?.setAttribute('style', 'transform: rotate(180deg);');
            } else {
                icon?.setAttribute('style', 'transform: rotate(0deg);');
            }
        });

        // Drag handlers
        header.addEventListener('mousedown', (e) => {
            // Don't drag if clicking on minimize button
            if ((e.target as HTMLElement).closest('.ai-minimize-btn')) {
                return;
            }
            
            isDragging = true;
            startX = e.clientX;
            startY = e.clientY;
            
            const rect = panel.getBoundingClientRect();
            startLeft = rect.left;
            startTop = rect.top;
            
            // Change cursor for the whole document while dragging
            document.body.style.cursor = 'move';
            document.body.style.userSelect = 'none';
            
            e.preventDefault();
        });

        document.addEventListener('mousemove', (e) => {
            if (!isDragging) return;
            
            const deltaX = e.clientX - startX;
            const deltaY = e.clientY - startY;
            
            let newLeft = startLeft + deltaX;
            let newTop = startTop + deltaY;
            
            // Constrain to viewport
            const maxLeft = window.innerWidth - panel.offsetWidth;
            const maxTop = window.innerHeight - panel.offsetHeight;
            
            newLeft = Math.max(0, Math.min(newLeft, maxLeft));
            newTop = Math.max(0, Math.min(newTop, maxTop));
            
            panel.style.left = newLeft + 'px';
            panel.style.top = newTop + 'px';
            panel.style.right = 'auto';
            panel.style.bottom = 'auto';
        });

        document.addEventListener('mouseup', () => {
            if (isDragging) {
                isDragging = false;
                document.body.style.cursor = '';
                document.body.style.userSelect = '';
            }
        });
    }

    private setupKeyboardShortcuts(): void {
        document.addEventListener('keydown', (e) => {
            // Only trigger if not in an input field
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement || e.target instanceof HTMLSelectElement) {
                return;
            }
            
            if (e.key.toLowerCase() === 'h') {
                e.preventDefault();
                this.showShortcutsPopup();
            }
        });
    }

    private showShortcutsPopup(): void {
        // Remove existing popup if any
        const existing = document.getElementById('shortcuts-popup');
        if (existing) {
            existing.remove();
            return;
        }

        const popup = this.createShortcutsPopup();
        document.body.appendChild(popup);
        
        // Focus for keyboard events
        popup.focus();
    }

    private createShortcutsPopup(): HTMLElement {
        const popup = document.createElement('div');
        popup.id = 'shortcuts-popup';
        popup.className = 'shortcuts-popup';
        popup.tabIndex = -1; // Make focusable
        
        popup.innerHTML = `
            <div class="shortcuts-header">
                <div class="shortcuts-title">
                    <div class="shortcuts-icon">⌨️</div>
                    Keyboard Shortcuts
                </div>
                <div class="shortcuts-header-actions">
                    <button class="shortcuts-close-btn" title="Close (Esc)">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                        </svg>
                    </button>
                </div>
            </div>
            
            <div class="shortcuts-content">
                <div class="shortcuts-section">
                    <h4>General</h4>
                    <div class="shortcut-item">
                        <kbd>H</kbd>
                        <span>Show/Hide this help</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Ctrl+N</kbd>
                        <span>New Diagram</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Ctrl+S</kbd>
                        <span>Save Diagram</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Delete</kbd>
                        <span>Delete Selected Element</span>
                    </div>
                </div>
                
                <div class="shortcuts-section">
                    <h4>Navigation</h4>
                    <div class="shortcut-item">
                        <kbd>Ctrl++</kbd>
                        <span>Zoom In</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Ctrl+-</kbd>
                        <span>Zoom Out</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Ctrl+0</kbd>
                        <span>Fit to Content</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Ctrl+R</kbd>
                        <span>Reset View</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Space + Drag</kbd>
                        <span>Pan Canvas</span>
                    </div>
                </div>
                
                <div class="shortcuts-section">
                    <h4>Selection & Editing</h4>
                    <div class="shortcut-item">
                        <kbd>Click</kbd>
                        <span>Select Element</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Ctrl+Click</kbd>
                        <span>Multi-select</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Drag</kbd>
                        <span>Move Element</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Ctrl+A</kbd>
                        <span>Select All</span>
                    </div>
                </div>
                
                <div class="shortcuts-section">
                    <h4>WASM Components</h4>
                    <div class="shortcut-item">
                        <kbd>Click Switch</kbd>
                        <span>Load/Unload Component</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Click Loaded</kbd>
                        <span>View Execution Examples</span>
                    </div>
                </div>
            </div>
            
            <div class="shortcuts-footer">
                <span>Press <kbd>Esc</kbd> or <kbd>Enter</kbd> to close</span>
            </div>
        `;

        // Setup event handlers
        this.setupShortcutsPopupHandlers(popup);
        
        return popup;
    }

    private setupShortcutsPopupHandlers(popup: HTMLElement): void {
        // Close button
        const closeBtn = popup.querySelector('.shortcuts-close-btn');
        if (closeBtn) {
            closeBtn.addEventListener('click', () => {
                popup.remove();
            });
        }

        // Keyboard handlers
        popup.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' || e.key === 'Enter') {
                e.preventDefault();
                popup.remove();
            }
        });

        // Click outside to close
        popup.addEventListener('click', (e) => {
            if (e.target === popup) {
                popup.remove();
            }
        });

        // Make draggable
        this.setupPopupDragging(popup);
    }

    private setupPopupDragging(popup: HTMLElement): void {
        const header = popup.querySelector('.shortcuts-header') as HTMLElement;
        let isDragging = false;
        let startX = 0;
        let startY = 0;
        let startLeft = 0;
        let startTop = 0;

        header.style.cursor = 'move';
        
        header.addEventListener('mousedown', (e) => {
            // Don't drag if clicking on close button
            if ((e.target as HTMLElement).closest('.shortcuts-close-btn')) {
                return;
            }
            
            isDragging = true;
            startX = e.clientX;
            startY = e.clientY;
            
            const rect = popup.getBoundingClientRect();
            startLeft = rect.left;
            startTop = rect.top;
            
            document.body.style.cursor = 'move';
            document.body.style.userSelect = 'none';
            
            e.preventDefault();
        });

        document.addEventListener('mousemove', (e) => {
            if (!isDragging) return;
            
            const deltaX = e.clientX - startX;
            const deltaY = e.clientY - startY;
            
            let newLeft = startLeft + deltaX;
            let newTop = startTop + deltaY;
            
            // Constrain to viewport
            const maxLeft = window.innerWidth - popup.offsetWidth;
            const maxTop = window.innerHeight - popup.offsetHeight;
            
            newLeft = Math.max(0, Math.min(newLeft, maxLeft));
            newTop = Math.max(0, Math.min(newTop, maxTop));
            
            popup.style.left = newLeft + 'px';
            popup.style.top = newTop + 'px';
            popup.style.right = 'auto';
            popup.style.bottom = 'auto';
        });

        document.addEventListener('mouseup', () => {
            if (isDragging) {
                isDragging = false;
                document.body.style.cursor = '';
                document.body.style.userSelect = '';
            }
        });
    }

    private updateUnifiedStatus(status: ConnectionStatus): void {
        // Update header status
        const headerIndicator = document.querySelector('#connection-indicator');
        const headerSpan = headerIndicator?.parentElement?.querySelector('span');
        if (headerIndicator && headerSpan) {
            headerIndicator.className = `status-indicator ${status.mcp ? '' : 'disconnected'}`;
            headerSpan.textContent = status.mcp ? 'MCP Connected' : 'MCP Disconnected';
        }

        // Update footer status  
        const footerIndicator = document.querySelector('#connection-indicator-status');
        const footerSpan = footerIndicator?.parentElement?.querySelector('span');
        if (footerIndicator && footerSpan) {
            footerIndicator.className = `status-indicator ${status.mcp ? '' : 'disconnected'}`;
            footerSpan.textContent = status.message;
        }

        // Update main status bar
        const statusText = this.statusElement.querySelector('#status-text');
        if (statusText) {
            statusText.textContent = status.message;
        }

        // Update AI panel connection status
        const aiConnectionElement = this.aiPanelElement.querySelector('#ai-connection-status');
        if (aiConnectionElement) {
            const statusText = status.ai ? 'Online' : 'Offline';
            const statusColor = status.ai ? 'var(--accent-success)' : 'var(--accent-error)';
            aiConnectionElement.textContent = statusText;
            aiConnectionElement.style.color = statusColor;
        }

        // Enable/disable model selection based on AI connection
        const aiModelSelect = this.aiPanelElement.querySelector('#ai-model-select') as HTMLSelectElement;
        if (aiModelSelect) {
            aiModelSelect.disabled = !status.ai;
            if (!status.ai) {
                aiModelSelect.innerHTML = '<option value="">Offline</option>';
            }
        }
    }

    public updateStatus(message: string): void {
        // Legacy method - now just updates the main status text
        const statusText = this.statusElement.querySelector('#status-text');
        if (statusText) {
            statusText.textContent = message;
        }
    }

    public updateDiagramList(diagrams: any[], loadDiagramCallback: (diagramId: string) => void, deleteDiagramCallback?: (diagramId: string, diagramName: string) => void): void {
        console.log('UIManager: updateDiagramList called with', diagrams.length, 'diagrams');
        const listElement = this.diagramListElement.querySelector('#diagram-list');
        console.log('UIManager: diagram list element found:', !!listElement);
        if (listElement) {
            listElement.innerHTML = '';
            diagrams.forEach((diagram: any) => {
                console.log('UIManager: Adding diagram to list:', diagram.name, diagram.id);
                const li = document.createElement('li');
                li.innerHTML = `
                    <div style="flex: 1;">
                        <div style="font-weight: 500;">${diagram.name}</div>
                        <div style="font-size: 0.8em; color: var(--text-dim);">${diagram.diagramType}</div>
                    </div>
                    <div style="display: flex; gap: 4px;">
                        <button class="load-btn" style="
                            background: var(--accent-wasm); 
                            color: white; 
                            border: none; 
                            padding: 4px 8px; 
                            border-radius: var(--radius-sm); 
                            cursor: pointer; 
                            font-size: 11px;
                        ">Load</button>
                        <button class="delete-btn" style="
                            background: var(--accent-error); 
                            color: white; 
                            border: none; 
                            padding: 4px 8px; 
                            border-radius: var(--radius-sm); 
                            cursor: pointer; 
                            font-size: 11px;
                        " title="Delete diagram">×</button>
                    </div>
                `;
                
                // Update li styling for flex layout
                li.style.display = 'flex';
                li.style.alignItems = 'center';
                li.style.gap = '8px';
                
                // Add load event listener
                li.querySelector('.load-btn')!.addEventListener('click', () => {
                    console.log('UIManager: Load button clicked for diagram:', diagram.id);
                    loadDiagramCallback(diagram.id);
                });
                
                // Add delete event listener if callback provided
                if (deleteDiagramCallback) {
                    li.querySelector('.delete-btn')!.addEventListener('click', (e) => {
                        e.stopPropagation();
                        if (confirm(`Are you sure you want to delete "${diagram.name}"? This action cannot be undone.`)) {
                            console.log('UIManager: Delete button clicked for diagram:', diagram.id);
                            deleteDiagramCallback(diagram.id, diagram.name);
                        }
                    });
                }
                
                listElement.appendChild(li);
            });
        } else {
            console.error('UIManager: diagram-list element not found in diagramListElement');
        }
    }
    
    public setupCreateDiagramButton(createDiagramCallback: () => void): void {
        const createBtn = this.diagramListElement.querySelector('#create-new-diagram-btn');
        if (createBtn) {
            createBtn.addEventListener('click', () => {
                console.log('UIManager: Create new diagram button clicked');
                createDiagramCallback();
            });
            
            // Add hover effect
            createBtn.addEventListener('mouseenter', () => {
                (createBtn as HTMLElement).style.background = 'var(--accent-info)';
            });
            createBtn.addEventListener('mouseleave', () => {
                (createBtn as HTMLElement).style.background = 'var(--accent-wasm)';
            });
        } else {
            console.error('UIManager: Create new diagram button not found');
        }
    }

    // Removed - now handled by updateUnifiedStatus

    public updateAIModelSelect(models: string[], currentModel: string, onModelChange: (modelName: string) => void): void {
        const selectElement = this.aiPanelElement.querySelector('#ai-model-select') as HTMLSelectElement;
        if (selectElement && models.length > 0) {
            // Clear existing options
            selectElement.innerHTML = '';
            
            // Add model options
            models.forEach(model => {
                const option = document.createElement('option');
                option.value = model;
                option.textContent = model;
                option.selected = model === currentModel;
                selectElement.appendChild(option);
            });
            
            // Add change listener
            selectElement.addEventListener('change', () => {
                if (selectElement.value) {
                    onModelChange(selectElement.value);
                }
            });
        }
    }

    public updateAIOutput(content: string): void {
        // For the chat interface, we'll add messages instead of replacing content
        if (content.includes('ai-thinking')) {
            this.addAIMessage('AI', '🤖 Thinking...');
        } else if (content.includes('ai-error')) {
            const errorMatch = content.match(/ai-error[^>]*>([^<]+)</);
            if (errorMatch) {
                this.addAIMessage('AI', `❌ ${errorMatch[1]}`);
            }
        } else if (content.includes('ai-response')) {
            // Extract the response content
            const responseMatch = content.match(/<h4[^>]*>(.*?)<\/h4>/);
            if (responseMatch) {
                this.addAIMessage('AI', responseMatch[1]);
            }
        }
    }
}