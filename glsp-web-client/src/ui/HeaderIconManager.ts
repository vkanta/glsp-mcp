export interface HeaderIcon {
    id: string;
    title: string;
    icon: string;
    color?: string;
    onClick: () => void;
    onClose?: () => void;
}

export class HeaderIconManager {
    private container: HTMLElement;
    private icons: Map<string, HeaderIcon> = new Map();
    
    constructor() {
        this.container = this.createContainer();
        this.insertIntoHeader();
    }
    
    private createContainer(): HTMLElement {
        const container = document.createElement('div');
        container.id = 'header-icons';
        container.className = 'header-icons-container';
        container.style.cssText = `
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 0 12px;
            order: 1;
        `;
        return container;
    }
    
    private insertIntoHeader(): void {
        const headerActions = document.querySelector('.header-actions');
        if (headerActions) {
            // Insert before the status chip
            const statusChip = headerActions.querySelector('.status-chip');
            if (statusChip) {
                headerActions.insertBefore(this.container, statusChip);
            } else {
                headerActions.appendChild(this.container);
            }
        }
    }
    
    public addIcon(icon: HeaderIcon): void {
        this.icons.set(icon.id, icon);
        this.renderIcon(icon);
    }
    
    public removeIcon(id: string): void {
        console.log('HeaderIconManager: removeIcon called for id:', id);
        console.log('HeaderIconManager: Icon exists in map:', this.icons.has(id));
        
        // Always remove from Map
        this.icons.delete(id);
        
        // Find and remove ALL DOM elements with this id (in case there are duplicates)
        const iconElements = this.container.querySelectorAll(`[data-icon-id="${id}"]`);
        console.log('HeaderIconManager: Found DOM elements:', iconElements.length);
        
        iconElements.forEach((element, index) => {
            console.log(`HeaderIconManager: Removing DOM element ${index + 1}/${iconElements.length}`);
            element.remove();
        });
        
        if (iconElements.length > 0) {
            console.log('HeaderIconManager: All DOM elements removed');
        } else {
            console.log('HeaderIconManager: No DOM elements found to remove');
        }
    }
    
    public hasIcon(id: string): boolean {
        const mapHasIcon = this.icons.has(id);
        const domHasIcon = !!this.container.querySelector(`[data-icon-id="${id}"]`);
        console.log(`HeaderIconManager: hasIcon(${id}) - Map: ${mapHasIcon}, DOM: ${domHasIcon}`);
        return mapHasIcon || domHasIcon;
    }
    
    private renderIcon(icon: HeaderIcon): void {
        // Remove existing icon if it exists
        this.removeIcon(icon.id);
        
        const iconElement = document.createElement('div');
        iconElement.className = 'header-icon';
        iconElement.dataset.iconId = icon.id;
        iconElement.style.cssText = `
            position: relative;
            padding: 8px 12px;
            background: var(--bg-tertiary);
            border: 1px solid var(--border);
            border-radius: var(--radius-sm);
            cursor: pointer;
            transition: all 0.3s ease;
            display: flex;
            align-items: center;
            gap: 6px;
            font-size: 13px;
            color: var(--text-primary);
            user-select: none;
            animation: slideInFromRight 0.4s ease;
        `;
        
        // Icon
        const iconSpan = document.createElement('span');
        iconSpan.textContent = icon.icon;
        iconSpan.style.cssText = `
            font-size: 16px;
            ${icon.color ? `color: ${icon.color};` : ''}
        `;
        iconElement.appendChild(iconSpan);
        
        // Title
        const titleSpan = document.createElement('span');
        titleSpan.textContent = icon.title;
        titleSpan.style.cssText = `
            font-weight: 500;
            max-width: 80px;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        `;
        iconElement.appendChild(titleSpan);
        
        // Close button (optional)
        if (icon.onClose) {
            const closeBtn = document.createElement('button');
            closeBtn.innerHTML = '×';
            closeBtn.type = 'button'; // Prevent form submission
            closeBtn.style.cssText = `
                background: none;
                border: none;
                color: var(--text-secondary);
                cursor: pointer;
                padding: 2px;
                margin-left: 4px;
                border-radius: 2px;
                font-size: 14px;
                line-height: 1;
                transition: all 0.2s ease;
                position: relative;
                z-index: 10;
                pointer-events: auto;
            `;
            
            // Add debugging for all events
            closeBtn.addEventListener('click', (e) => {
                console.log('HeaderIconManager: Close button clicked - event details:', {
                    target: e.target,
                    currentTarget: e.currentTarget,
                    timeStamp: e.timeStamp,
                    isTrusted: e.isTrusted,
                    bubbles: e.bubbles
                });
                e.stopPropagation();
                e.preventDefault();
                console.log('HeaderIconManager: Calling onClose handler');
                icon.onClose!();
                this.removeIcon(icon.id);
            });
            
            // Add debugging for other events that might trigger clicks
            closeBtn.addEventListener('focus', () => {
                console.log('HeaderIconManager: Close button received focus');
            });
            
            closeBtn.addEventListener('blur', () => {
                console.log('HeaderIconManager: Close button lost focus');
            });
            
            closeBtn.addEventListener('mousedown', (e) => {
                console.log('HeaderIconManager: Close button mousedown');
                e.stopPropagation();
            });
            
            closeBtn.addEventListener('mouseup', (e) => {
                console.log('HeaderIconManager: Close button mouseup');
                e.stopPropagation();
            });
            
            closeBtn.addEventListener('mouseenter', () => {
                closeBtn.style.background = 'var(--accent-error)';
                closeBtn.style.color = 'white';
            });
            
            closeBtn.addEventListener('mouseleave', () => {
                closeBtn.style.background = 'none';
                closeBtn.style.color = 'var(--text-secondary)';
            });
            
            iconElement.appendChild(closeBtn);
            console.log('HeaderIconManager: Close button created and added to icon:', icon.id);
        }
        
        // Click handler
        iconElement.addEventListener('click', (e) => {
            console.log('HeaderIconManager: Main icon clicked, target:', e.target);
            console.log('HeaderIconManager: Event currentTarget:', e.currentTarget);
            icon.onClick();
            // Add click animation
            iconElement.style.transform = 'scale(0.95)';
            setTimeout(() => {
                iconElement.style.transform = 'scale(1)';
            }, 150);
        });
        
        // Hover effects
        iconElement.addEventListener('mouseenter', () => {
            iconElement.style.background = icon.color || 'var(--accent-wasm)';
            iconElement.style.color = 'white';
            iconElement.style.borderColor = icon.color || 'var(--accent-wasm)';
            iconElement.style.transform = 'translateY(-2px)';
            iconElement.style.boxShadow = `0 4px 12px ${icon.color ? icon.color + '40' : 'rgba(101, 79, 240, 0.4)'}`;
        });
        
        iconElement.addEventListener('mouseleave', () => {
            iconElement.style.background = 'var(--bg-tertiary)';
            iconElement.style.color = 'var(--text-primary)';
            iconElement.style.borderColor = 'var(--border)';
            iconElement.style.transform = 'translateY(0)';
            iconElement.style.boxShadow = 'none';
        });
        
        this.container.appendChild(iconElement);
    }
    
    public updateIcon(id: string, updates: Partial<HeaderIcon>): void {
        const icon = this.icons.get(id);
        if (icon) {
            const updatedIcon = { ...icon, ...updates };
            this.icons.set(id, updatedIcon);
            
            // Find existing DOM element
            const iconElement = this.container.querySelector(`[data-icon-id="${id}"]`) as HTMLElement;
            if (iconElement) {
                // Update only the visual parts without recreating the entire element
                const iconSpan = iconElement.querySelector('span:first-child');
                const titleSpan = iconElement.querySelector('span:last-of-type');
                
                if (iconSpan && updates.icon) {
                    iconSpan.textContent = updates.icon;
                    if (updates.color) {
                        iconSpan.style.color = updates.color;
                    }
                }
                
                if (titleSpan && updates.title) {
                    titleSpan.textContent = updates.title;
                }
                
                // Update hover colors if color changed
                if (updates.color) {
                    const originalMouseEnter = iconElement.onmouseenter;
                    const originalMouseLeave = iconElement.onmouseleave;
                    
                    iconElement.onmouseenter = () => {
                        iconElement.style.background = updates.color || 'var(--accent-wasm)';
                        iconElement.style.color = 'white';
                        iconElement.style.borderColor = updates.color || 'var(--accent-wasm)';
                        iconElement.style.transform = 'translateY(-2px)';
                        iconElement.style.boxShadow = `0 4px 12px ${updates.color ? updates.color + '40' : 'rgba(101, 79, 240, 0.4)'}`;
                    };
                }
            } else {
                // If DOM element doesn't exist, fall back to full render
                this.renderIcon(updatedIcon);
            }
        }
    }
    
    public getIconCount(): number {
        return this.icons.size;
    }
    
    public clear(): void {
        this.icons.clear();
        this.container.innerHTML = '';
    }
    
    public forceRemoveAllDiagramIcons(): void {
        console.log('HeaderIconManager: Force removing all diagram icons');
        // Remove from map
        this.icons.delete('current-diagram');
        // Remove all DOM elements that might be diagram icons
        const elements = this.container.querySelectorAll('[data-icon-id="current-diagram"]');
        console.log('HeaderIconManager: Force found DOM elements:', elements.length);
        elements.forEach((element, index) => {
            console.log(`HeaderIconManager: Force removing element ${index + 1}`);
            element.remove();
        });
    }
}