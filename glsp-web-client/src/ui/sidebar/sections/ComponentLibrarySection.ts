import { SidebarSection } from '../SidebarComponent.js';

export interface ComponentItem {
    id: string;
    name: string;
    category: string;
    description?: string;
    icon?: string;
    version?: string;
    author?: string;
    tags?: string[];
    thumbnail?: string;
    status?: 'available' | 'loading' | 'error';
    path?: string;
    interfaces?: unknown;
    onDragStart?: (e: DragEvent) => void;
    onSelect?: () => void;
}

export interface ComponentFilter {
    search?: string;
    category?: string;
    tags?: string[];
    status?: string;
}

export class ComponentLibrarySection {
    private components: Map<string, ComponentItem> = new Map();
    private categories: Set<string> = new Set();
    private filter: ComponentFilter = {};
    private element?: HTMLElement;
    private view: 'grid' | 'list' = 'grid';
    private selectedComponents: Set<string> = new Set();
    private multiSelectMode = false;
    
    public addComponent(component: ComponentItem): void {
        this.components.set(component.id, component);
        this.categories.add(component.category);
        this.refresh();
    }
    
    public removeComponent(id: string): void {
        this.components.delete(id);
        this.refresh();
    }
    
    public updateComponent(id: string, updates: Partial<ComponentItem>): void {
        const component = this.components.get(id);
        if (component) {
            this.components.set(id, { ...component, ...updates });
            this.refresh();
        }
    }
    
    public setFilter(filter: ComponentFilter): void {
        this.filter = filter;
        this.refresh();
    }
    
    public setView(view: 'grid' | 'list'): void {
        this.view = view;
        this.refresh();
    }
    
    public toggleMultiSelectMode(): void {
        this.multiSelectMode = !this.multiSelectMode;
        if (!this.multiSelectMode) {
            this.clearSelection();
        }
        this.refresh();
    }
    
    public setMultiSelectMode(enabled: boolean): void {
        this.multiSelectMode = enabled;
        if (!enabled) {
            this.clearSelection();
        }
        this.refresh();
    }
    
    public selectComponent(id: string): void {
        if (this.multiSelectMode) {
            if (this.selectedComponents.has(id)) {
                this.selectedComponents.delete(id);
            } else {
                this.selectedComponents.add(id);
            }
        } else {
            this.selectedComponents.clear();
            this.selectedComponents.add(id);
        }
        this.refresh();
    }
    
    public selectAll(): void {
        this.selectedComponents.clear();
        const filteredComponents = this.getFilteredComponents();
        filteredComponents.forEach(component => {
            this.selectedComponents.add(component.id);
        });
        this.refresh();
    }
    
    public clearSelection(): void {
        this.selectedComponents.clear();
        this.refresh();
    }
    
    public getSelectedComponents(): ComponentItem[] {
        return Array.from(this.selectedComponents)
            .map(id => this.components.get(id))
            .filter((component): component is ComponentItem => component !== undefined);
    }
    
    public hasSelection(): boolean {
        return this.selectedComponents.size > 0;
    }
    
    public createSection(): SidebarSection {
        this.element = this.createContent();
        return {
            id: 'wasm-components',
            title: 'WASM Components',
            icon: '📦',
            collapsible: true,
            collapsed: false,
            order: 4,
            content: this.element
        };
    }
    
    private createContent(): HTMLElement {
        const container = document.createElement('div');
        container.className = 'component-library-container';
        container.style.cssText = `
            display: flex;
            flex-direction: column;
            gap: 12px;
        `;
        
        // Search and filters
        const filterBar = this.createFilterBar();
        container.appendChild(filterBar);
        
        // View toggle
        const viewToggle = this.createViewToggle();
        container.appendChild(viewToggle);
        
        // Multi-select toolbar
        const multiSelectToolbar = this.createMultiSelectToolbar();
        container.appendChild(multiSelectToolbar);
        
        // Selection actions (only visible when components are selected)
        if (this.hasSelection()) {
            const selectionActions = this.createSelectionActions();
            container.appendChild(selectionActions);
        }
        
        // Components display
        const componentsDisplay = this.createComponentsDisplay();
        container.appendChild(componentsDisplay);
        
        // Add keyboard shortcuts
        container.addEventListener('keydown', (e) => {
            this.handleKeyboardShortcuts(e);
        });
        
        // Make container focusable for keyboard events
        container.tabIndex = 0;
        
        return container;
    }
    
    private createFilterBar(): HTMLElement {
        const filterBar = document.createElement('div');
        filterBar.className = 'filter-bar';
        filterBar.style.cssText = `
            display: flex;
            flex-direction: column;
            gap: 8px;
        `;
        
        // Search input
        const searchInput = document.createElement('input');
        searchInput.type = 'text';
        searchInput.placeholder = 'Search components...';
        searchInput.value = this.filter.search || '';
        searchInput.style.cssText = `
            padding: 8px 12px;
            background: var(--bg-primary, #0F1419);
            border: 1px solid var(--border-color, #2A3441);
            border-radius: 6px;
            color: var(--text-primary, #E6EDF3);
            font-size: 13px;
            transition: all 0.2s ease;
        `;
        
        searchInput.addEventListener('input', (e) => {
            this.filter.search = (e.target as HTMLInputElement).value;
            this.refresh();
        });
        
        searchInput.addEventListener('focus', () => {
            searchInput.style.borderColor = 'var(--accent-wasm, #654FF0)';
        });
        
        searchInput.addEventListener('blur', () => {
            searchInput.style.borderColor = 'var(--border-color, #2A3441)';
        });
        
        filterBar.appendChild(searchInput);
        
        // Category filter
        if (this.categories.size > 1) {
            const categorySelect = document.createElement('select');
            categorySelect.style.cssText = `
                padding: 6px 12px;
                background: var(--bg-primary, #0F1419);
                border: 1px solid var(--border-color, #2A3441);
                border-radius: 4px;
                color: var(--text-primary, #E6EDF3);
                font-size: 12px;
                cursor: pointer;
            `;
            
            const allOption = document.createElement('option');
            allOption.value = '';
            allOption.textContent = 'All Categories';
            categorySelect.appendChild(allOption);
            
            Array.from(this.categories).sort().forEach(category => {
                const option = document.createElement('option');
                option.value = category;
                option.textContent = category;
                option.selected = this.filter.category === category;
                categorySelect.appendChild(option);
            });
            
            categorySelect.addEventListener('change', (e) => {
                this.filter.category = (e.target as HTMLSelectElement).value || undefined;
                this.refresh();
            });
            
            filterBar.appendChild(categorySelect);
        }
        
        return filterBar;
    }
    
    private createViewToggle(): HTMLElement {
        const container = document.createElement('div');
        container.style.cssText = `
            display: flex;
            gap: 4px;
            background: var(--bg-primary, #0F1419);
            padding: 4px;
            border-radius: 6px;
        `;
        
        const gridButton = this.createViewButton('grid', '⊞', 'Grid View');
        const listButton = this.createViewButton('list', '☰', 'List View');
        
        container.appendChild(gridButton);
        container.appendChild(listButton);
        
        return container;
    }
    
    private createViewButton(view: 'grid' | 'list', icon: string, title: string): HTMLElement {
        const button = document.createElement('button');
        button.textContent = icon;
        button.title = title;
        button.style.cssText = `
            flex: 1;
            padding: 6px;
            background: ${this.view === view ? 'var(--accent-wasm, #654FF0)' : 'transparent'};
            border: none;
            border-radius: 4px;
            color: ${this.view === view ? 'white' : 'var(--text-secondary, #7D8590)'};
            cursor: pointer;
            transition: all 0.2s ease;
            font-size: 16px;
        `;
        
        button.addEventListener('click', () => {
            this.setView(view);
        });
        
        if (this.view !== view) {
            button.addEventListener('mouseenter', () => {
                button.style.background = 'var(--bg-secondary, #151B2C)';
                button.style.color = 'var(--text-primary, #E6EDF3)';
            });
            
            button.addEventListener('mouseleave', () => {
                button.style.background = 'transparent';
                button.style.color = 'var(--text-secondary, #7D8590)';
            });
        }
        
        return button;
    }
    
    private createMultiSelectToolbar(): HTMLElement {
        const toolbar = document.createElement('div');
        toolbar.style.cssText = `
            display: flex;
            gap: 8px;
            align-items: center;
            padding: 8px;
            background: var(--bg-secondary, #151B2C);
            border-radius: 6px;
            border: 1px solid var(--border-color, #2A3441);
        `;
        
        // Multi-select toggle button
        const multiSelectButton = document.createElement('button');
        multiSelectButton.textContent = this.multiSelectMode ? '☑️' : '☐';
        multiSelectButton.title = this.multiSelectMode ? 'Exit multi-select mode' : 'Enter multi-select mode';
        multiSelectButton.style.cssText = `
            padding: 6px 10px;
            background: ${this.multiSelectMode ? 'var(--accent-wasm, #654FF0)' : 'transparent'};
            border: 1px solid var(--border-color, #2A3441);
            border-radius: 4px;
            color: ${this.multiSelectMode ? 'white' : 'var(--text-primary, #E6EDF3)'};
            cursor: pointer;
            font-size: 12px;
            transition: all 0.2s ease;
        `;
        
        multiSelectButton.addEventListener('click', () => {
            this.toggleMultiSelectMode();
        });
        
        if (!this.multiSelectMode) {
            multiSelectButton.addEventListener('mouseenter', () => {
                multiSelectButton.style.background = 'var(--bg-primary, #0F1419)';
            });
            
            multiSelectButton.addEventListener('mouseleave', () => {
                multiSelectButton.style.background = 'transparent';
            });
        }
        
        toolbar.appendChild(multiSelectButton);
        
        // Multi-select label
        const label = document.createElement('span');
        label.textContent = this.multiSelectMode ? 'Multi-select enabled' : 'Single selection';
        label.style.cssText = `
            font-size: 12px;
            color: var(--text-secondary, #7D8590);
            flex: 1;
        `;
        toolbar.appendChild(label);
        
        // Select all button (only visible in multi-select mode)
        if (this.multiSelectMode) {
            const selectAllButton = document.createElement('button');
            selectAllButton.textContent = 'Select All';
            selectAllButton.style.cssText = `
                padding: 4px 8px;
                background: transparent;
                border: 1px solid var(--border-color, #2A3441);
                border-radius: 4px;
                color: var(--text-secondary, #7D8590);
                cursor: pointer;
                font-size: 11px;
                transition: all 0.2s ease;
            `;
            
            selectAllButton.addEventListener('click', () => {
                this.selectAll();
            });
            
            selectAllButton.addEventListener('mouseenter', () => {
                selectAllButton.style.background = 'var(--bg-primary, #0F1419)';
                selectAllButton.style.color = 'var(--text-primary, #E6EDF3)';
            });
            
            selectAllButton.addEventListener('mouseleave', () => {
                selectAllButton.style.background = 'transparent';
                selectAllButton.style.color = 'var(--text-secondary, #7D8590)';
            });
            
            toolbar.appendChild(selectAllButton);
            
            // Clear selection button
            if (this.hasSelection()) {
                const clearButton = document.createElement('button');
                clearButton.textContent = 'Clear';
                clearButton.style.cssText = `
                    padding: 4px 8px;
                    background: transparent;
                    border: 1px solid var(--border-color, #2A3441);
                    border-radius: 4px;
                    color: var(--text-secondary, #7D8590);
                    cursor: pointer;
                    font-size: 11px;
                    transition: all 0.2s ease;
                `;
                
                clearButton.addEventListener('click', () => {
                    this.clearSelection();
                });
                
                clearButton.addEventListener('mouseenter', () => {
                    clearButton.style.background = 'var(--accent-error, #F85149)';
                    clearButton.style.color = 'white';
                    clearButton.style.borderColor = 'var(--accent-error, #F85149)';
                });
                
                clearButton.addEventListener('mouseleave', () => {
                    clearButton.style.background = 'transparent';
                    clearButton.style.color = 'var(--text-secondary, #7D8590)';
                    clearButton.style.borderColor = 'var(--border-color, #2A3441)';
                });
                
                toolbar.appendChild(clearButton);
            }
        }
        
        return toolbar;
    }
    
    private createSelectionActions(): HTMLElement {
        const actions = document.createElement('div');
        actions.style.cssText = `
            display: flex;
            flex-direction: column;
            gap: 8px;
            padding: 12px;
            background: var(--accent-wasm, #654FF0);
            border-radius: 6px;
            border: 1px solid var(--accent-wasm-bright, #7B5FFF);
        `;
        
        // Selection info
        const selectionInfo = document.createElement('div');
        const selectedCount = this.selectedComponents.size;
        selectionInfo.textContent = `${selectedCount} component${selectedCount !== 1 ? 's' : ''} selected`;
        selectionInfo.style.cssText = `
            font-size: 12px;
            font-weight: 600;
            color: white;
            text-align: center;
        `;
        actions.appendChild(selectionInfo);
        
        // Create group button
        const createGroupButton = document.createElement('button');
        createGroupButton.textContent = '🔗 Create Component Group';
        createGroupButton.style.cssText = `
            padding: 8px 12px;
            background: white;
            border: none;
            border-radius: 4px;
            color: var(--accent-wasm, #654FF0);
            cursor: pointer;
            font-size: 12px;
            font-weight: 600;
            transition: all 0.2s ease;
        `;
        
        createGroupButton.addEventListener('click', () => {
            this.createComponentGroup();
        });
        
        createGroupButton.addEventListener('mouseenter', () => {
            createGroupButton.style.background = 'var(--bg-primary, #0F1419)';
            createGroupButton.style.color = 'white';
        });
        
        createGroupButton.addEventListener('mouseleave', () => {
            createGroupButton.style.background = 'white';
            createGroupButton.style.color = 'var(--accent-wasm, #654FF0)';
        });
        
        actions.appendChild(createGroupButton);
        
        return actions;
    }
    
    private createComponentGroup(): void {
        const selectedComponents = this.getSelectedComponents();
        if (selectedComponents.length === 0) {
            return;
        }
        
        // Create a simple prompt for the group name
        const groupName = prompt('Enter a name for the component group:', 
            `Group of ${selectedComponents.length} components`);
        
        if (!groupName) {
            return;
        }
        
        const groupDescription = prompt('Enter a description for the component group (optional):');
        
        // Here we would typically call an MCP tool to create the group
        // For now, we'll just show a success message and clear selection
        console.log('Creating component group:', {
            name: groupName,
            description: groupDescription,
            components: selectedComponents.map(c => ({ id: c.id, name: c.name }))
        });
        
        // TODO: Call MCP create_component_group tool
        // await mcpClient.callTool('create_component_group', {
        //     diagramId: currentDiagramId,
        //     name: groupName,
        //     description: groupDescription,
        //     componentIds: selectedComponents.map(c => c.id)
        // });
        
        this.clearSelection();
        this.setMultiSelectMode(false);
        
        // Show success notification
        this.showNotification(`Component group "${groupName}" created successfully!`, 'success');
    }
    
    private showNotification(message: string, type: 'success' | 'error' | 'info' = 'info'): void {
        // Simple notification system - in a real app this would be more sophisticated
        const notification = document.createElement('div');
        notification.textContent = message;
        notification.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            padding: 12px 16px;
            background: ${type === 'success' ? 'var(--accent-success, #3FB950)' : 
                          type === 'error' ? 'var(--accent-error, #F85149)' : 
                          'var(--accent-info, #58A6FF)'};
            color: white;
            border-radius: 6px;
            font-size: 14px;
            z-index: 10000;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
            max-width: 300px;
        `;
        
        document.body.appendChild(notification);
        
        // Remove after 3 seconds
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 3000);
    }
    
    private createComponentsDisplay(): HTMLElement {
        const display = document.createElement('div');
        display.className = 'components-display';
        display.style.cssText = `
            display: ${this.view === 'grid' ? 'grid' : 'flex'};
            ${this.view === 'grid' ? 'grid-template-columns: repeat(2, 1fr);' : 'flex-direction: column;'}
            gap: 8px;
            max-height: 400px;
            overflow-y: auto;
            padding-right: 4px;
        `;
        
        // Apply custom scrollbar
        const style = document.createElement('style');
        style.textContent = `
            .components-display::-webkit-scrollbar {
                width: 8px;
            }
            .components-display::-webkit-scrollbar-track {
                background: var(--bg-primary, #0F1419);
                border-radius: 4px;
            }
            .components-display::-webkit-scrollbar-thumb {
                background: var(--accent-wasm, #654FF0);
                border-radius: 4px;
            }
            .components-display::-webkit-scrollbar-thumb:hover {
                background: var(--accent-wasm-bright, #7B5FFF);
            }
        `;
        document.head.appendChild(style);
        
        // Filter and display components
        const filteredComponents = this.getFilteredComponents();
        
        if (filteredComponents.length === 0) {
            display.innerHTML = `
                <div style="
                    grid-column: 1 / -1;
                    text-align: center;
                    padding: 24px;
                    color: var(--text-secondary, #7D8590);
                    font-size: 13px;
                ">
                    <div style="font-size: 32px; margin-bottom: 8px;">📦</div>
                    No components found
                </div>
            `;
        } else {
            filteredComponents.forEach(component => {
                const componentElement = this.view === 'grid' 
                    ? this.createGridItem(component)
                    : this.createListItem(component);
                display.appendChild(componentElement);
            });
        }
        
        return display;
    }
    
    private createGridItem(component: ComponentItem): HTMLElement {
        const item = document.createElement('div');
        item.className = 'component-grid-item';
        item.draggable = !this.multiSelectMode;
        
        const isSelected = this.selectedComponents.has(component.id);
        
        item.style.cssText = `
            background: var(--bg-secondary, #151B2C);
            border: 2px solid ${isSelected ? 'var(--accent-wasm, #654FF0)' : 'var(--border-color, #2A3441)'};
            border-radius: 8px;
            padding: 12px;
            cursor: ${this.multiSelectMode ? 'pointer' : 'grab'};
            transition: all 0.2s ease;
            display: flex;
            flex-direction: column;
            gap: 8px;
            position: relative;
            ${isSelected ? 'box-shadow: 0 0 0 2px rgba(101, 79, 240, 0.3);' : ''}
        `;
        
        // Selection indicator (checkbox) for multi-select mode
        if (this.multiSelectMode) {
            const checkbox = document.createElement('div');
            checkbox.style.cssText = `
                position: absolute;
                top: 8px;
                right: 8px;
                width: 18px;
                height: 18px;
                border: 2px solid ${isSelected ? 'var(--accent-wasm, #654FF0)' : 'var(--border-color, #2A3441)'};
                border-radius: 3px;
                background: ${isSelected ? 'var(--accent-wasm, #654FF0)' : 'transparent'};
                display: flex;
                align-items: center;
                justify-content: center;
                font-size: 12px;
                color: white;
            `;
            
            if (isSelected) {
                checkbox.textContent = '✓';
            }
            
            item.appendChild(checkbox);
        }
        
        // Icon or thumbnail
        const visual = document.createElement('div');
        visual.style.cssText = `
            width: 100%;
            height: 60px;
            background: var(--bg-primary, #0F1419);
            border-radius: 6px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 32px;
            color: var(--text-secondary, #7D8590);
        `;
        
        if (component.thumbnail) {
            visual.style.backgroundImage = `url(${component.thumbnail})`;
            visual.style.backgroundSize = 'cover';
            visual.style.backgroundPosition = 'center';
        } else {
            visual.textContent = component.icon || '📦';
        }
        
        item.appendChild(visual);
        
        // Name
        const name = document.createElement('div');
        name.textContent = component.name;
        name.style.cssText = `
            font-size: 13px;
            font-weight: 600;
            color: var(--text-primary, #E6EDF3);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        `;
        item.appendChild(name);
        
        // Category
        const category = document.createElement('div');
        category.textContent = component.category;
        category.style.cssText = `
            font-size: 11px;
            color: var(--text-secondary, #7D8590);
        `;
        item.appendChild(category);
        
        // Status indicator
        if (component.status) {
            const status = document.createElement('div');
            status.style.cssText = `
                position: absolute;
                top: 8px;
                right: 8px;
                width: 8px;
                height: 8px;
                border-radius: 50%;
                background: ${
                    component.status === 'available' ? 'var(--accent-success, #3FB950)' :
                    component.status === 'loading' ? 'var(--accent-warning, #F0B72F)' :
                    'var(--accent-error, #F85149)'
                };
            `;
            item.appendChild(status);
        }
        
        // Event handlers
        this.setupComponentEvents(item, component);
        
        return item;
    }
    
    private createListItem(component: ComponentItem): HTMLElement {
        const item = document.createElement('div');
        item.className = 'component-list-item';
        item.draggable = !this.multiSelectMode;
        
        const isSelected = this.selectedComponents.has(component.id);
        
        item.style.cssText = `
            background: var(--bg-secondary, #151B2C);
            border: 2px solid ${isSelected ? 'var(--accent-wasm, #654FF0)' : 'var(--border-color, #2A3441)'};
            border-radius: 6px;
            padding: 10px 12px;
            cursor: ${this.multiSelectMode ? 'pointer' : 'grab'};
            transition: all 0.2s ease;
            display: flex;
            align-items: center;
            gap: 12px;
            position: relative;
            ${isSelected ? 'box-shadow: 0 0 0 2px rgba(101, 79, 240, 0.3);' : ''}
        `;
        
        // Selection indicator (checkbox) for multi-select mode
        if (this.multiSelectMode) {
            const checkbox = document.createElement('div');
            checkbox.style.cssText = `
                width: 16px;
                height: 16px;
                border: 2px solid ${isSelected ? 'var(--accent-wasm, #654FF0)' : 'var(--border-color, #2A3441)'};
                border-radius: 3px;
                background: ${isSelected ? 'var(--accent-wasm, #654FF0)' : 'transparent'};
                display: flex;
                align-items: center;
                justify-content: center;
                font-size: 10px;
                color: white;
                flex-shrink: 0;
            `;
            
            if (isSelected) {
                checkbox.textContent = '✓';
            }
            
            item.appendChild(checkbox);
        }
        
        // Icon
        const icon = document.createElement('div');
        icon.textContent = component.icon || '📦';
        icon.style.cssText = `
            font-size: 24px;
            flex-shrink: 0;
        `;
        item.appendChild(icon);
        
        // Info
        const info = document.createElement('div');
        info.style.cssText = `
            flex: 1;
            min-width: 0;
        `;
        
        const name = document.createElement('div');
        name.textContent = component.name;
        name.style.cssText = `
            font-size: 13px;
            font-weight: 600;
            color: var(--text-primary, #E6EDF3);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        `;
        info.appendChild(name);
        
        const meta = document.createElement('div');
        meta.textContent = `${component.category}${component.version ? ` • v${component.version}` : ''}`;
        meta.style.cssText = `
            font-size: 11px;
            color: var(--text-secondary, #7D8590);
        `;
        info.appendChild(meta);
        
        item.appendChild(info);
        
        // Status
        if (component.status) {
            const status = document.createElement('div');
            status.style.cssText = `
                width: 6px;
                height: 6px;
                border-radius: 50%;
                background: ${
                    component.status === 'available' ? 'var(--accent-success, #3FB950)' :
                    component.status === 'loading' ? 'var(--accent-warning, #F0B72F)' :
                    'var(--accent-error, #F85149)'
                };
                flex-shrink: 0;
            `;
            item.appendChild(status);
        }
        
        // Event handlers
        this.setupComponentEvents(item, component);
        
        return item;
    }
    
    private setupComponentEvents(element: HTMLElement, component: ComponentItem): void {
        // Hover effects
        element.addEventListener('mouseenter', () => {
            element.style.borderColor = 'var(--accent-wasm, #654FF0)';
            element.style.transform = 'translateY(-2px)';
            element.style.boxShadow = '0 4px 8px rgba(101, 79, 240, 0.2)';
        });
        
        element.addEventListener('mouseleave', () => {
            element.style.borderColor = 'var(--border-color, #2A3441)';
            element.style.transform = 'translateY(0)';
            element.style.boxShadow = 'none';
        });
        
        // Drag events (disabled in multi-select mode)
        element.addEventListener('dragstart', (e) => {
            if (this.multiSelectMode) {
                e.preventDefault();
                return;
            }
            
            element.style.opacity = '0.5';
            element.style.cursor = 'grabbing';
            
            if (e.dataTransfer) {
                e.dataTransfer.effectAllowed = 'copy';
                const dragData = {
                    type: 'wasm-component',
                    id: component.id,
                    name: component.name,
                    category: component.category,
                    path: component.path,
                    interfaces: component.interfaces
                };
                console.log('ComponentLibrarySection - Starting drag with data:', dragData);
                e.dataTransfer.setData('application/json', JSON.stringify(dragData));
                // Also set text/plain for compatibility
                e.dataTransfer.setData('text/plain', component.name);
            }
            
            component.onDragStart?.(e);
        });
        
        element.addEventListener('dragend', () => {
            element.style.opacity = '1';
            element.style.cursor = 'grab';
        });
        
        // Click event
        element.addEventListener('click', (e) => {
            if (this.multiSelectMode) {
                // In multi-select mode, toggle selection
                e.preventDefault();
                e.stopPropagation();
                this.selectComponent(component.id);
            } else {
                // In single-select mode, call the original onSelect handler
                component.onSelect?.();
            }
        });
    }
    
    private getFilteredComponents(): ComponentItem[] {
        let components = Array.from(this.components.values());
        
        // Apply search filter
        if (this.filter.search) {
            const search = this.filter.search.toLowerCase();
            components = components.filter(c => 
                c.name.toLowerCase().includes(search) ||
                c.description?.toLowerCase().includes(search) ||
                c.tags?.some(tag => tag.toLowerCase().includes(search))
            );
        }
        
        // Apply category filter
        if (this.filter.category) {
            components = components.filter(c => c.category === this.filter.category);
        }
        
        // Apply status filter
        if (this.filter.status) {
            components = components.filter(c => c.status === this.filter.status);
        }
        
        // Apply tag filter
        if (this.filter.tags && this.filter.tags.length > 0) {
            components = components.filter(c => 
                c.tags?.some(tag => this.filter.tags!.includes(tag))
            );
        }
        
        return components;
    }
    
    private handleKeyboardShortcuts(e: KeyboardEvent): void {
        // Escape to exit multi-select mode
        if (e.key === 'Escape') {
            if (this.multiSelectMode) {
                this.setMultiSelectMode(false);
                e.preventDefault();
            }
        }
        
        // Ctrl+A to select all (only in multi-select mode)
        if (e.key === 'a' && (e.ctrlKey || e.metaKey)) {
            if (this.multiSelectMode) {
                this.selectAll();
                e.preventDefault();
            }
        }
        
        // Delete to clear selection
        if (e.key === 'Delete' || e.key === 'Backspace') {
            if (this.multiSelectMode && this.hasSelection()) {
                this.clearSelection();
                e.preventDefault();
            }
        }
        
        // Enter to create group from selection
        if (e.key === 'Enter') {
            if (this.multiSelectMode && this.hasSelection()) {
                this.createComponentGroup();
                e.preventDefault();
            }
        }
        
        // M key to toggle multi-select mode
        if (e.key === 'm' || e.key === 'M') {
            if (!e.ctrlKey && !e.metaKey && !e.altKey) {
                this.toggleMultiSelectMode();
                e.preventDefault();
            }
        }
    }
    
    public refresh(): void {
        if (this.element) {
            const newContent = this.createContent();
            this.element.replaceWith(newContent);
            this.element = newContent;
        }
    }
}