import { detectEnvironment } from '../utils/environment.js';
// Import CSS styles
import './WorkspaceSelector.css';

interface WorkspaceInfo {
    path: string;
    name: string;
    last_used: string;
    diagrams_count: number;
    wasm_components_count: number;
}

declare global {
    interface Window {
        __TAURI__: {
            invoke: (command: string, args?: any) => Promise<any>;
        };
    }
}

export class WorkspaceSelector {
    private container: HTMLElement;
    private isOpen = false;
    private currentWorkspace: string | null = null;
    private onWorkspaceSelected: (workspace: string) => void;

    constructor(container: HTMLElement, onWorkspaceSelected: (workspace: string) => void) {
        this.container = container;
        this.onWorkspaceSelected = onWorkspaceSelected;
        this.initialize();
    }

    private initialize(): void {
        this.createUI();
        this.loadCurrentWorkspace();
    }

    public createSidebarSection(): any {
        const env = detectEnvironment();
        
        // Only show workspace selector in desktop mode
        if (!env.isDesktop) {
            return null;
        }

        // Create the workspace selector content
        const content = document.createElement('div');
        content.innerHTML = `
            <div class="workspace-selector-sidebar">
                <div class="current-workspace">
                    <div class="workspace-info">
                        <span class="workspace-icon">📁</span>
                        <div class="workspace-details">
                            <div id="workspace-name" class="workspace-name">Default Workspace</div>
                            <div id="workspace-path" class="workspace-path">/default/path</div>
                        </div>
                    </div>
                </div>
                <div class="workspace-actions">
                    <button id="browse-workspace" class="workspace-action-btn">
                        <span class="action-icon">🔍</span>
                        Browse...
                    </button>
                    <button id="create-workspace" class="workspace-action-btn">
                        <span class="action-icon">➕</span>
                        Create...
                    </button>
                    <button id="validate-workspace" class="workspace-action-btn">
                        <span class="action-icon">✓</span>
                        Validate
                    </button>
                </div>
                <div id="recent-workspaces" class="recent-workspaces">
                    <h4>Recent Workspaces</h4>
                    <div id="workspace-items" class="workspace-items">
                        <!-- Recent workspace items will be populated here -->
                    </div>
                </div>
            </div>
        `;

        // Set up container reference and attach event listeners
        this.container = content;
        this.attachEventListeners();
        this.loadRecentWorkspaces();
        this.loadCurrentWorkspace();

        return {
            id: 'workspace',
            title: 'Workspace',
            icon: '📁',
            collapsible: true,
            collapsed: false,
            order: 0, // Show at top
            content: content
        };
    }

    private createUI(): void {
        const env = detectEnvironment();
        console.log('WorkspaceSelector: Environment detected:', env);
        console.log('WorkspaceSelector: Tauri API available:', !!window.__TAURI__);
        
        // Only show workspace selector in desktop mode
        if (!env.isDesktop) {
            console.log('WorkspaceSelector: Not in desktop mode, skipping UI creation');
            return;
        }
        
        console.log('WorkspaceSelector: Creating UI for desktop mode');

        this.container.innerHTML = `
            <div class="workspace-selector">
                <button id="workspace-button" class="workspace-button">
                    <span class="workspace-icon">📁</span>
                    <span id="workspace-name">Default Workspace</span>
                    <span class="workspace-arrow">▼</span>
                </button>
                <div id="workspace-dropdown" class="workspace-dropdown hidden">
                    <div class="workspace-header">
                        <h3>Select Workspace</h3>
                        <div class="workspace-buttons">
                            <button id="browse-workspace" class="browse-button">
                                <span class="browse-icon">🔍</span>
                                Browse...
                            </button>
                            <button id="create-workspace" class="create-button">
                                <span class="create-icon">➕</span>
                                Create...
                            </button>
                            <button id="validate-workspace" class="validate-button">
                                <span class="validate-icon">✓</span>
                                Validate
                            </button>
                        </div>
                    </div>
                    <div class="workspace-list">
                        <div id="recent-workspaces" class="recent-workspaces">
                            <h4>Recent Workspaces</h4>
                            <div id="workspace-items" class="workspace-items">
                                <!-- Recent workspace items will be populated here -->
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        `;

        this.attachEventListeners();
        this.loadRecentWorkspaces();
    }

    private attachEventListeners(): void {
        const workspaceButton = this.container.querySelector('#workspace-button') as HTMLButtonElement;
        const workspaceDropdown = this.container.querySelector('#workspace-dropdown') as HTMLElement;
        const browseButton = this.container.querySelector('#browse-workspace') as HTMLButtonElement;
        const createButton = this.container.querySelector('#create-workspace') as HTMLButtonElement;
        const validateButton = this.container.querySelector('#validate-workspace') as HTMLButtonElement;

        if (workspaceButton) {
            workspaceButton.addEventListener('click', () => {
                this.toggleDropdown();
            });
        }

        if (browseButton) {
            browseButton.addEventListener('click', () => {
                this.openWorkspaceDialog();
            });
        }

        if (createButton) {
            createButton.addEventListener('click', () => {
                this.createWorkspaceDialog();
            });
        }

        if (validateButton) {
            validateButton.addEventListener('click', () => {
                this.validateCurrentWorkspace();
            });
        }

        // Close dropdown when clicking outside
        document.addEventListener('click', (event) => {
            if (!this.container.contains(event.target as Node)) {
                this.closeDropdown();
            }
        });
    }

    private toggleDropdown(): void {
        const dropdown = this.container.querySelector('#workspace-dropdown') as HTMLElement;
        if (dropdown) {
            if (this.isOpen) {
                this.closeDropdown();
            } else {
                this.openDropdown();
            }
        }
    }

    private openDropdown(): void {
        const dropdown = this.container.querySelector('#workspace-dropdown') as HTMLElement;
        if (dropdown) {
            dropdown.classList.remove('hidden');
            this.isOpen = true;
            this.loadRecentWorkspaces(); // Refresh the list
        }
    }

    private closeDropdown(): void {
        const dropdown = this.container.querySelector('#workspace-dropdown') as HTMLElement;
        if (dropdown) {
            dropdown.classList.add('hidden');
            this.isOpen = false;
        }
    }

    private async openWorkspaceDialog(): Promise<void> {
        try {
            const result = await window.__TAURI__.invoke('select_workspace_directory');
            if (result) {
                await this.selectWorkspace(result);
            }
        } catch (error) {
            console.error('Error opening workspace dialog:', error);
            this.showError('Failed to open workspace dialog');
        }
    }

    private async selectWorkspace(workspacePath: string): Promise<void> {
        try {
            // Set workspace directory using MCP (no server restart needed)
            const result = await window.__TAURI__.invoke('set_workspace_directory', { 
                workspacePath: workspacePath, 
                createIfMissing: true 
            });
            
            // Add to recent workspaces
            await window.__TAURI__.invoke('add_recent_workspace', { workspacePath: workspacePath });
            
            // Update UI
            this.currentWorkspace = workspacePath;
            this.updateWorkspaceDisplay();
            this.updateWindowTitle();
            
            // Close dropdown
            this.closeDropdown();
            
            // Notify parent component
            this.onWorkspaceSelected(workspacePath);
            
            this.showSuccess(`Workspace changed to: ${workspacePath}`);
        } catch (error) {
            console.error('Error selecting workspace:', error);
            this.showError('Failed to select workspace: ' + (error as any).message || error);
        }
    }

    private async loadRecentWorkspaces(): Promise<void> {
        try {
            const workspaces: WorkspaceInfo[] = await window.__TAURI__.invoke('get_recent_workspaces');
            this.displayRecentWorkspaces(workspaces);
        } catch (error) {
            console.error('Error loading recent workspaces:', error);
        }
    }

    private displayRecentWorkspaces(workspaces: WorkspaceInfo[]): void {
        const workspaceItems = this.container.querySelector('#workspace-items') as HTMLElement;
        if (!workspaceItems) return;

        if (workspaces.length === 0) {
            workspaceItems.innerHTML = '<p class="no-workspaces">No recent workspaces</p>';
            return;
        }

        workspaceItems.innerHTML = workspaces.map(workspace => `
            <div class="workspace-item" data-path="${workspace.path}">
                <div class="workspace-item-main">
                    <div class="workspace-item-name">${workspace.name}</div>
                    <div class="workspace-item-path">${workspace.path}</div>
                </div>
                <div class="workspace-item-stats">
                    <span class="stat">📊 ${workspace.diagrams_count} diagrams</span>
                    <span class="stat">📦 ${workspace.wasm_components_count} components</span>
                </div>
            </div>
        `).join('');

        // Add click listeners to workspace items
        workspaceItems.querySelectorAll('.workspace-item').forEach(item => {
            item.addEventListener('click', () => {
                const path = item.getAttribute('data-path');
                if (path) {
                    this.selectWorkspace(path);
                }
            });
        });
    }

    private async loadCurrentWorkspace(): Promise<void> {
        try {
            // Get current workspace info from MCP server
            const workspaceInfo = await window.__TAURI__.invoke('get_workspace_info');
            this.currentWorkspace = workspaceInfo.path;
            this.updateWorkspaceDisplay();
            this.updateWindowTitle();
        } catch (error) {
            console.error('Error loading current workspace:', error);
            // Fallback to app data directory if MCP call fails
            try {
                const appDataDir = await window.__TAURI__.invoke('get_app_data_dir');
                this.currentWorkspace = appDataDir;
                this.updateWorkspaceDisplay();
                this.updateWindowTitle();
            } catch (fallbackError) {
                console.error('Error loading app data directory:', fallbackError);
            }
        }
    }

    private updateWorkspaceDisplay(): void {
        const workspaceName = this.container.querySelector('#workspace-name') as HTMLElement;
        const workspacePath = this.container.querySelector('#workspace-path') as HTMLElement;
        
        if (workspaceName && this.currentWorkspace) {
            const name = this.currentWorkspace.split('/').pop() || 'Default Workspace';
            workspaceName.textContent = name;
        }
        
        if (workspacePath && this.currentWorkspace) {
            // Show a shortened path for the sidebar
            const pathParts = this.currentWorkspace.split('/');
            const shortPath = pathParts.length > 3 ? 
                `.../${pathParts.slice(-2).join('/')}` : 
                this.currentWorkspace;
            workspacePath.textContent = shortPath;
            workspacePath.title = this.currentWorkspace; // Full path on hover
        }
    }

    private updateWindowTitle(): void {
        if (this.currentWorkspace) {
            const workspaceName = this.currentWorkspace.split('/').pop() || 'Default Workspace';
            document.title = `WASM Component Designer - ${workspaceName}`;
        } else {
            document.title = 'WASM Component Designer';
        }
    }

    private showError(message: string): void {
        // Simple error display - in a real app, you might want to use a proper notification system
        console.error(message);
        // You could also show a toast notification or update the UI
    }

    private showSuccess(message: string): void {
        // Simple success display - in a real app, you might want to use a proper notification system
        console.log(message);
        // You could also show a toast notification or update the UI
    }

    private async createWorkspaceDialog(): Promise<void> {
        try {
            const result = await window.__TAURI__.invoke('select_workspace_directory');
            if (result) {
                // Create workspace structure
                await window.__TAURI__.invoke('create_workspace_structure', { workspacePath: result });
                await this.selectWorkspace(result);
                this.showSuccess(`Created workspace structure at: ${result}`);
            }
        } catch (error) {
            console.error('Error creating workspace:', error);
            this.showError('Failed to create workspace: ' + (error as any).message || error);
        }
    }

    private async validateCurrentWorkspace(): Promise<void> {
        if (!this.currentWorkspace) {
            this.showError('No workspace selected');
            return;
        }

        try {
            const validationResult = await window.__TAURI__.invoke('validate_workspace', { 
                workspacePath: this.currentWorkspace 
            });
            
            if (validationResult.valid) {
                this.showSuccess('Workspace validation passed');
            } else {
                this.showError('Workspace validation failed: ' + validationResult.issues?.join(', '));
            }
        } catch (error) {
            console.error('Error validating workspace:', error);
            this.showError('Failed to validate workspace: ' + (error as any).message || error);
        }
    }
}