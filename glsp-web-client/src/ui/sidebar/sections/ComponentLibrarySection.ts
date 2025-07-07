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
        
        // Components display
        const componentsDisplay = this.createComponentsDisplay();
        container.appendChild(componentsDisplay);
        
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
        item.draggable = true;
        item.style.cssText = `
            background: var(--bg-secondary, #151B2C);
            border: 1px solid var(--border-color, #2A3441);
            border-radius: 8px;
            padding: 12px;
            cursor: grab;
            transition: all 0.2s ease;
            display: flex;
            flex-direction: column;
            gap: 8px;
        `;
        
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
        item.draggable = true;
        item.style.cssText = `
            background: var(--bg-secondary, #151B2C);
            border: 1px solid var(--border-color, #2A3441);
            border-radius: 6px;
            padding: 10px 12px;
            cursor: grab;
            transition: all 0.2s ease;
            display: flex;
            align-items: center;
            gap: 12px;
        `;
        
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
        
        // Drag events
        element.addEventListener('dragstart', (e) => {
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
        element.addEventListener('click', () => {
            component.onSelect?.();
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
    
    private refresh(): void {
        if (this.element) {
            const newContent = this.createContent();
            this.element.replaceWith(newContent);
            this.element = newContent;
        }
    }
}