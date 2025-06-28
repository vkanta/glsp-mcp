/**
 * Interface Connection Dialog
 * Shows compatible interfaces when user clicks on an interface to link
 */

import { BaseDialog } from '../base/BaseDialog.js';
import { InterfaceCompatibilityChecker, WitInterface, InterfaceCompatibility } from '../../../diagrams/interface-compatibility.js';

export interface InterfaceConnectionOption {
    componentId: string;
    componentName: string;
    interface: WitInterface;
    compatibility: InterfaceCompatibility;
}

export interface InterfaceConnectionDialogConfig {
    sourceComponentId: string;
    sourceComponentName: string;
    sourceInterface: WitInterface;
    availableInterfaces: InterfaceConnectionOption[];
    position?: { x: number; y: number }; // Position near the clicked interface
}

export class InterfaceConnectionDialog extends BaseDialog {
    private config: InterfaceConnectionDialogConfig;
    private selectedOption?: InterfaceConnectionOption;
    private onConnectionCreate?: (option: InterfaceConnectionOption) => void;

    constructor(config: InterfaceConnectionDialogConfig) {
        super({
            title: `Connect "${config.sourceInterface.name}" Interface`,
            size: { width: 500, height: 400 },
            closable: true,
            modal: true
        });
        this.config = config;
    }

    public onConnection(callback: (option: InterfaceConnectionOption) => void): void {
        this.onConnectionCreate = callback;
    }

    protected createContent(): HTMLElement {
        const container = document.createElement('div');
        container.className = 'interface-connection-dialog';
        container.style.cssText = `
            display: flex;
            flex-direction: column;
            height: 100%;
            gap: 16px;
            padding: 20px;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        `;

        // Source interface info
        container.appendChild(this.createSourceInfo());

        // Available connections
        container.appendChild(this.createConnectionsList());

        // Action buttons
        container.appendChild(this.createActionButtons());

        return container;
    }

    private createSourceInfo(): HTMLElement {
        const section = document.createElement('div');
        section.className = 'source-info';
        section.style.cssText = `
            background: var(--bg-secondary, #f8f9fa);
            padding: 16px;
            border-radius: 8px;
            border-left: 4px solid var(--primary-color, #007acc);
        `;

        const interfaceTypeColor = this.config.sourceInterface.interface_type === 'export' ? '#28a745' : '#007acc';
        const interfaceIcon = this.config.sourceInterface.interface_type === 'export' ? '🟢' : '🔵';

        section.innerHTML = `
            <h3 style="margin: 0 0 8px 0; font-size: 16px; color: var(--text-primary, #333);">
                ${interfaceIcon} Source Interface
            </h3>
            <div style="font-weight: 600; color: ${interfaceTypeColor};">
                ${this.config.sourceInterface.name}
            </div>
            <div style="font-size: 14px; color: var(--text-secondary, #666); margin-top: 4px;">
                ${this.config.sourceInterface.interface_type.toUpperCase()} from "${this.config.sourceComponentName}"
            </div>
            <div style="font-size: 12px; color: var(--text-secondary, #666); margin-top: 8px;">
                ${this.config.sourceInterface.functions?.length || 0} function(s) available
            </div>
        `;

        return section;
    }

    private createConnectionsList(): HTMLElement {
        const section = document.createElement('div');
        section.className = 'connections-list';
        section.style.cssText = `
            flex: 1;
            display: flex;
            flex-direction: column;
            gap: 8px;
            min-height: 200px;
        `;

        const header = document.createElement('h3');
        header.textContent = 'Compatible Interfaces';
        header.style.cssText = `
            margin: 0 0 12px 0;
            font-size: 16px;
            color: var(--text-primary, #333);
        `;
        section.appendChild(header);

        if (this.config.availableInterfaces.length === 0) {
            const noOptions = document.createElement('div');
            noOptions.style.cssText = `
                text-align: center;
                color: var(--text-secondary, #666);
                font-style: italic;
                padding: 40px 20px;
            `;
            noOptions.textContent = 'No compatible interfaces found';
            section.appendChild(noOptions);
        } else {
            const scrollContainer = document.createElement('div');
            scrollContainer.style.cssText = `
                flex: 1;
                overflow-y: auto;
                border: 1px solid var(--border-color, #ddd);
                border-radius: 6px;
                max-height: 250px;
            `;

            this.config.availableInterfaces.forEach((option, index) => {
                const item = this.createConnectionItem(option, index);
                scrollContainer.appendChild(item);
            });

            section.appendChild(scrollContainer);
        }

        return section;
    }

    private createConnectionItem(option: InterfaceConnectionOption, index: number): HTMLElement {
        const item = document.createElement('div');
        item.className = 'connection-item';
        item.style.cssText = `
            padding: 12px 16px;
            border-bottom: 1px solid var(--border-color, #eee);
            cursor: pointer;
            transition: background-color 0.2s;
            ${index === 0 ? 'border-top: none;' : ''}
        `;

        // Color coding based on compatibility score
        const scoreColor = option.compatibility.score >= 80 ? '#28a745' : 
                          option.compatibility.score >= 60 ? '#ffc107' : '#dc3545';
        
        const targetIcon = option.interface.interface_type === 'export' ? '🟢' : '🔵';

        item.innerHTML = `
            <div style="display: flex; justify-content: between; align-items: center; margin-bottom: 8px;">
                <div style="font-weight: 600; color: var(--text-primary, #333);">
                    ${targetIcon} ${option.interface.name}
                </div>
                <div style="color: ${scoreColor}; font-weight: 600; font-size: 14px;">
                    ${option.compatibility.score}% match
                </div>
            </div>
            <div style="font-size: 14px; color: var(--text-secondary, #666); margin-bottom: 4px;">
                ${option.interface.interface_type.toUpperCase()} from "${option.componentName}"
            </div>
            <div style="font-size: 12px; color: var(--text-secondary, #666);">
                ${option.interface.functions?.length || 0} function(s) • ${option.compatibility.matchedFunctions}/${option.compatibility.totalFunctions} functions match
            </div>
            ${option.compatibility.issues.length > 0 ? `
                <div style="font-size: 11px; color: #dc3545; margin-top: 6px; font-style: italic;">
                    ${option.compatibility.issues.slice(0, 2).join(', ')}${option.compatibility.issues.length > 2 ? '...' : ''}
                </div>
            ` : ''}
        `;

        // Hover effects
        item.addEventListener('mouseenter', () => {
            item.style.backgroundColor = 'var(--bg-hover, #f0f0f0)';
        });

        item.addEventListener('mouseleave', () => {
            item.style.backgroundColor = this.selectedOption === option ? 'var(--bg-selected, #e3f2fd)' : 'transparent';
        });

        // Selection
        item.addEventListener('click', () => {
            // Clear previous selection
            const allItems = item.parentElement?.querySelectorAll('.connection-item');
            allItems?.forEach(i => {
                (i as HTMLElement).style.backgroundColor = 'transparent';
            });

            // Select this item
            item.style.backgroundColor = 'var(--bg-selected, #e3f2fd)';
            this.selectedOption = option;

            // Enable connect button
            const connectBtn = this.element?.querySelector('.connect-button') as HTMLButtonElement;
            if (connectBtn) {
                connectBtn.disabled = false;
                connectBtn.style.opacity = '1';
            }
        });

        return item;
    }

    private createActionButtons(): HTMLElement {
        const actions = document.createElement('div');
        actions.className = 'dialog-actions';
        actions.style.cssText = `
            display: flex;
            justify-content: flex-end;
            gap: 12px;
            padding-top: 16px;
            border-top: 1px solid var(--border-color, #eee);
        `;

        // Cancel button
        const cancelBtn = document.createElement('button');
        cancelBtn.textContent = 'Cancel';
        cancelBtn.className = 'cancel-button';
        cancelBtn.style.cssText = `
            padding: 8px 16px;
            border: 1px solid var(--border-color, #ddd);
            background: var(--bg-primary, white);
            color: var(--text-primary, #333);
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
        `;

        cancelBtn.addEventListener('click', () => {
            this.close();
        });

        // Connect button
        const connectBtn = document.createElement('button');
        connectBtn.textContent = 'Create Connection';
        connectBtn.className = 'connect-button';
        connectBtn.disabled = true;
        connectBtn.style.cssText = `
            padding: 8px 16px;
            border: none;
            background: var(--primary-color, #007acc);
            color: white;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            opacity: 0.5;
            transition: opacity 0.2s;
        `;

        connectBtn.addEventListener('click', () => {
            if (this.selectedOption && this.onConnectionCreate) {
                this.onConnectionCreate(this.selectedOption);
                this.close();
            }
        });

        actions.appendChild(cancelBtn);
        actions.appendChild(connectBtn);

        return actions;
    }

    public show(): Promise<InterfaceConnectionOption | null> {
        return new Promise((resolve) => {
            this.onConnection((option) => {
                resolve(option);
            });

            // Resolve with null if dialog is closed without selection
            this.onClose(() => {
                if (!this.selectedOption) {
                    resolve(null);
                }
            });

            super.show();

            // Position dialog near the interface if position is provided
            if (this.config.position && this.element) {
                const dialogRect = this.element.getBoundingClientRect();
                const viewportWidth = window.innerWidth;
                const viewportHeight = window.innerHeight;

                let x = this.config.position.x + 20; // Offset to avoid covering the interface
                let y = this.config.position.y - dialogRect.height / 2;

                // Ensure dialog stays in viewport
                if (x + dialogRect.width > viewportWidth) {
                    x = this.config.position.x - dialogRect.width - 20;
                }
                if (y < 0) {
                    y = 10;
                }
                if (y + dialogRect.height > viewportHeight) {
                    y = viewportHeight - dialogRect.height - 10;
                }

                this.element.style.left = x + 'px';
                this.element.style.top = y + 'px';
                this.element.style.position = 'fixed';
                this.element.style.transform = 'none'; // Override centering
            }
        });
    }
}