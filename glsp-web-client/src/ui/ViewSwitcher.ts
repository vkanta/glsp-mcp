/**
 * View Switcher Component
 * Provides UI for switching between different diagram view modes
 */

export interface ViewMode {
    id: string;
    label: string;
    icon: string;
    tooltip: string;
}

export class ViewSwitcher {
    private container: HTMLElement;
    private currentMode: string = 'component';
    private onModeChange?: (mode: string) => void;
    
    private viewModes: ViewMode[] = [
        {
            id: 'component',
            label: 'Component View',
            icon: '📦',
            tooltip: 'View WASM components and their connections'
        },
        {
            id: 'uml-interface',
            label: 'UML View',
            icon: '📐',
            tooltip: 'View components in UML-style class diagram format'
        },
        {
            id: 'wit-interface',
            label: 'WIT Interface',
            icon: '🔗',
            tooltip: 'View WIT interfaces with packages, functions, and types'
        },
        {
            id: 'wit-dependencies',
            label: 'Dependencies',
            icon: '🕸️',
            tooltip: 'View interface dependencies and relationships'
        }
    ];
    
    constructor() {
        this.container = this.createViewSwitcher();
    }
    
    private createViewSwitcher(): HTMLElement {
        const container = document.createElement('div');
        container.className = 'view-switcher';
        
        // Add view mode indicator
        const indicator = document.createElement('div');
        indicator.className = 'view-mode-indicator';
        indicator.innerHTML = `
            <span class="indicator-label">View:</span>
            <span class="indicator-mode">${this.getViewModeLabel(this.currentMode)}</span>
        `;
        container.appendChild(indicator);
        
        // Add mode buttons
        this.viewModes.forEach(mode => {
            const button = document.createElement('button');
            button.className = `view-mode-btn ${mode.id === this.currentMode ? 'active' : ''}`;
            button.title = mode.tooltip;
            button.innerHTML = `
                <span class="view-mode-icon">${mode.icon}</span>
                <span class="view-mode-label">${mode.label}</span>
            `;
            
            button.onclick = () => this.switchMode(mode.id);
            container.appendChild(button);
        });
        
        this.addStyles();
        return container;
    }
    
    private addStyles(): void {
        const style = document.createElement('style');
        style.textContent = `
            .view-switcher {
                display: flex;
                align-items: center;
                gap: 8px;
                background: var(--bg-tertiary);
                border: 1px solid var(--border);
                border-radius: var(--radius-md);
                padding: 4px 8px;
                margin: 0 16px;
            }

            .view-mode-indicator {
                display: flex;
                align-items: center;
                gap: 4px;
                padding: 4px 8px;
                background: var(--bg-secondary);
                border-radius: var(--radius-sm);
                border-left: 3px solid var(--accent-wasm);
                margin-right: 4px;
            }

            .indicator-label {
                font-size: 11px;
                color: var(--text-secondary);
                font-weight: 600;
                text-transform: uppercase;
                letter-spacing: 0.5px;
            }

            .indicator-mode {
                font-size: 12px;
                color: var(--text-primary);
                font-weight: 500;
            }
            
            .view-mode-btn {
                display: flex;
                align-items: center;
                gap: 6px;
                padding: 6px 12px;
                background: transparent;
                border: 1px solid transparent;
                border-radius: var(--radius-sm);
                color: var(--text-secondary);
                cursor: pointer;
                transition: all 0.2s ease;
                font-size: 13px;
                font-weight: 500;
                white-space: nowrap;
            }
            
            .view-mode-btn:hover {
                background: var(--bg-secondary);
                color: var(--text-primary);
                border-color: var(--border);
            }
            
            .view-mode-btn.active {
                background: var(--accent-wasm);
                color: white;
                border-color: var(--accent-wasm);
            }
            
            .view-mode-icon {
                font-size: 16px;
            }
            
            .view-mode-label {
                display: none;
            }
            
            @media (min-width: 1200px) {
                .view-mode-label {
                    display: inline;
                }
            }
            
            /* Compact mode for smaller screens */
            @media (max-width: 768px) {
                .view-switcher {
                    gap: 2px;
                    padding: 2px;
                }
                
                .view-mode-btn {
                    padding: 4px 8px;
                    font-size: 12px;
                }
                
                .view-mode-icon {
                    font-size: 14px;
                }
            }
        `;
        document.head.appendChild(style);
    }
    
    private switchMode(modeId: string): void {
        if (modeId === this.currentMode) return;
        
        console.log(`ViewSwitcher: Switching from ${this.currentMode} to ${modeId}`);
        
        this.currentMode = modeId;
        
        // Update button states
        this.container.querySelectorAll('.view-mode-btn').forEach(btn => {
            const mode = this.viewModes.find(m => 
                btn.querySelector('.view-mode-label')?.textContent === m.label
            );
            btn.classList.toggle('active', mode?.id === modeId);
        });

        // Update view mode indicator
        const indicatorMode = this.container.querySelector('.indicator-mode');
        if (indicatorMode) {
            indicatorMode.textContent = this.getViewModeLabel(modeId);
        }
        
        // Show visual feedback that mode is changing
        const activeBtn = this.container.querySelector('.view-mode-btn.active') as HTMLElement;
        if (activeBtn) {
            activeBtn.style.opacity = '0.6';
            setTimeout(() => {
                activeBtn.style.opacity = '1';
            }, 300);
        }
        
        // Notify listener (AppController.handleViewModeChange)
        if (this.onModeChange) {
            console.log(`ViewSwitcher: Notifying mode change handler for ${modeId}`);
            this.onModeChange(modeId);
        }
    }

    private getViewModeLabel(modeId: string): string {
        const mode = this.viewModes.find(m => m.id === modeId);
        return mode ? mode.label : modeId;
    }
    
    public setModeChangeHandler(handler: (mode: string) => void): void {
        this.onModeChange = handler;
    }
    
    public getElement(): HTMLElement {
        return this.container;
    }
    
    public getCurrentMode(): string {
        return this.currentMode;
    }
    
    public setMode(modeId: string): void {
        if (this.viewModes.find(m => m.id === modeId)) {
            this.switchMode(modeId);
        }
    }
    
    public showForDiagramType(diagramType: string): void {
        // Show/hide view switcher based on diagram type
        if (diagramType === 'wasm-component' || diagramType === 'wit-interface') {
            this.container.style.display = 'flex';
        } else {
            this.container.style.display = 'none';
        }
    }
}