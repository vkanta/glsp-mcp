/**
 * WASM Component Palette
 * Shows available WASM components that can be dragged into diagrams
 */

import { McpClient } from '../mcp/client.js';

export interface WasmComponentInfo {
    name: string;
    path: string;
    description: string;
    status: 'available' | 'missing';
    interfaces: number;
    lastSeen?: string;
    removedAt?: string;
}

export interface DragEventData {
    componentName: string;
    componentInfo: WasmComponentInfo;
}

export class WasmComponentPalette {
    private element: HTMLElement;
    private mcpClient: McpClient;
    private components: WasmComponentInfo[] = [];
    private dragData?: DragEventData;

    constructor(mcpClient: McpClient) {
        this.mcpClient = mcpClient;
        this.element = this.createElement();
        this.setupEventHandlers();
    }

    private createElement(): HTMLElement {
        const palette = document.createElement('div');
        palette.className = 'wasm-component-palette';
        palette.innerHTML = `
            <div class="palette-header">
                <h3>📦 WASM Components</h3>
                <button id="refresh-components" class="refresh-btn" title="Refresh component list">🔄</button>
            </div>
            <div class="palette-search">
                <input type="text" id="component-search" placeholder="Search components..." />
            </div>
            <div class="palette-status">
                <span id="component-count">Loading...</span>
                <span id="component-health"></span>
            </div>
            <div class="palette-content" id="component-list">
                <div class="loading">Loading WASM components...</div>
            </div>
        `;
        return palette;
    }

    private setupEventHandlers(): void {
        // Refresh button
        this.element.querySelector('#refresh-components')?.addEventListener('click', () => {
            this.loadComponents();
        });

        // Search functionality
        this.element.querySelector('#component-search')?.addEventListener('input', (e) => {
            const searchTerm = (e.target as HTMLInputElement).value.toLowerCase();
            this.filterComponents(searchTerm);
        });
    }

    public async loadComponents(): Promise<void> {
        try {
            // First trigger a scan to ensure we have latest data
            await this.mcpClient.callTool('scan_wasm_components', {});
            
            // Get components list
            const listResource = await this.mcpClient.readResource('wasm://components/list');
            const data = JSON.parse(listResource.text || '{}');
            
            this.components = data.components || [];
            this.renderComponents();
            this.updateStatus(data);
            
        } catch (error) {
            console.error('Failed to load WASM components:', error);
            this.showError('Failed to load WASM components');
        }
    }

    private renderComponents(): void {
        const listElement = this.element.querySelector('#component-list');
        if (!listElement) return;

        if (this.components.length === 0) {
            listElement.innerHTML = '<div class="no-components">No WASM components found</div>';
            return;
        }

        listElement.innerHTML = this.components.map(component => this.renderComponent(component)).join('');
        
        // Add drag event handlers to component items
        listElement.querySelectorAll('.component-item').forEach((item, index) => {
            this.setupComponentDragHandlers(item as HTMLElement, this.components[index]);
        });
    }

    private renderComponent(component: WasmComponentInfo): string {
        const statusIcon = component.status === 'available' ? '✅' : '❌';
        const statusClass = component.status === 'available' ? 'available' : 'missing';
        
        return `
            <div class="component-item ${statusClass}" draggable="true" data-component="${component.name}">
                <div class="component-header">
                    <span class="component-status">${statusIcon}</span>
                    <span class="component-name">${component.name}</span>
                    <span class="interface-count">${component.interfaces} interfaces</span>
                </div>
                <div class="component-description">${component.description}</div>
                <div class="component-details">
                    <span class="component-path">${component.path}</span>
                    ${component.status === 'missing' ? 
                        `<span class="missing-info">Missing since: ${component.removedAt || 'Unknown'}</span>` : 
                        ''
                    }
                </div>
            </div>
        `;
    }

    private setupComponentDragHandlers(element: HTMLElement, component: WasmComponentInfo): void {
        element.addEventListener('dragstart', (e) => {
            if (component.status !== 'available') {
                e.preventDefault();
                this.showNotification(`Cannot drag missing component: ${component.name}`);
                return;
            }

            this.dragData = {
                componentName: component.name,
                componentInfo: component
            };

            e.dataTransfer!.effectAllowed = 'copy';
            e.dataTransfer!.setData('application/wasm-component', JSON.stringify(this.dragData));
            
            // Visual feedback
            element.classList.add('dragging');
            this.showNotification(`Dragging ${component.name} - drop on diagram to add`);
        });

        element.addEventListener('dragend', () => {
            element.classList.remove('dragging');
            this.dragData = undefined;
        });

        // Double-click to add to center of diagram
        element.addEventListener('dblclick', () => {
            if (component.status === 'available') {
                this.addComponentToCenter(component);
            }
        });
    }

    private filterComponents(searchTerm: string): void {
        const items = this.element.querySelectorAll('.component-item');
        items.forEach((item) => {
            const componentName = item.querySelector('.component-name')?.textContent?.toLowerCase() || '';
            const description = item.querySelector('.component-description')?.textContent?.toLowerCase() || '';
            
            const matches = componentName.includes(searchTerm) || description.includes(searchTerm);
            (item as HTMLElement).style.display = matches ? 'block' : 'none';
        });
    }

    private updateStatus(data: any): void {
        const countElement = this.element.querySelector('#component-count');
        const healthElement = this.element.querySelector('#component-health');
        
        if (countElement) {
            countElement.textContent = `${data.total || 0} components`;
        }
        
        if (healthElement) {
            const available = data.available || 0;
            const missing = data.missing || 0;
            const healthIcon = missing === 0 ? '🟢' : missing < available ? '🟡' : '🔴';
            healthElement.innerHTML = `${healthIcon} ${available} available, ${missing} missing`;
        }
    }

    private showError(message: string): void {
        const listElement = this.element.querySelector('#component-list');
        if (listElement) {
            listElement.innerHTML = `<div class="error">${message}</div>`;
        }
    }

    private showNotification(message: string): void {
        // Simple notification - in a real app this would be a proper toast/notification system
        console.log(`WASM Palette: ${message}`);
        
        // You could also update a status area or show a temporary message
        const statusElement = this.element.querySelector('#component-count');
        if (statusElement) {
            const originalText = statusElement.textContent;
            statusElement.textContent = message;
            setTimeout(() => {
                statusElement.textContent = originalText;
            }, 3000);
        }
    }

    private async addComponentToCenter(component: WasmComponentInfo): Promise<void> {
        // This would be called by the parent app when a component is double-clicked
        // For now, just log it - the actual implementation will be in the main app
        console.log(`Add component ${component.name} to center of diagram`);
        this.showNotification(`Double-click detected for ${component.name} - implement in main app`);
    }

    public getElement(): HTMLElement {
        return this.element;
    }

    public getDragData(): DragEventData | undefined {
        return this.dragData;
    }

    public async show(): Promise<void> {
        this.element.style.display = 'block';
        await this.loadComponents();
    }

    public hide(): void {
        this.element.style.display = 'none';
    }

    public isVisible(): boolean {
        return this.element.style.display !== 'none';
    }

    // Called by external code when a component is successfully added to diagram
    public onComponentAdded(componentName: string): void {
        this.showNotification(`Successfully added ${componentName} to diagram`);
    }

    // Called by external code when a component add fails
    public onComponentAddFailed(componentName: string, error: string): void {
        this.showNotification(`Failed to add ${componentName}: ${error}`);
    }
}