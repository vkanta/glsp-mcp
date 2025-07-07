/**
 * Base Dialog Class
 * Extended FloatingPanel with dialog-specific functionality
 */

import { FloatingPanel, FloatingPanelConfig, FloatingPanelEvents } from '../../FloatingPanel.js';
import { DialogResult } from '../DialogManager.js';

export interface DialogConfig extends Partial<FloatingPanelConfig> {
    modal?: boolean;
    showBackdrop?: boolean;
    closeOnBackdropClick?: boolean;
    closeOnEscape?: boolean;
    showFooter?: boolean;
    primaryButtonText?: string;
    secondaryButtonText?: string;
    cancelButtonText?: string;
}

export interface DialogEvents extends FloatingPanelEvents {
    onConfirm?: (value?: unknown) => void;
    onCancel?: () => void;
    onShow?: () => void;
    onClose?: () => void;
}

export abstract class BaseDialog extends FloatingPanel {
    private static currentDialogConfig: DialogConfig;
    protected dialogConfig: DialogConfig;
    protected dialogEvents: DialogEvents;
    protected backdrop?: HTMLElement;
    protected footerElement?: HTMLElement;
    protected flexboxContainer?: HTMLElement;
    protected isShown: boolean = false;

    constructor(config: DialogConfig = {}, events: DialogEvents = {}) {
        // Set default dialog configuration
        const defaultConfig: FloatingPanelConfig = {
            title: 'Dialog',
            width: 400,
            height: 300,
            minWidth: 300,
            minHeight: 200,
            maxWidth: 600,
            maxHeight: 500,
            initialPosition: { x: -1, y: -1 }, // Will be centered
            resizable: false,
            draggable: true,
            closable: true,
            collapsible: false,
            className: 'dialog-panel',
            zIndex: 10000,
            ...config
        };

        const dialogConfig: DialogConfig = {
            modal: true,
            showBackdrop: true,
            closeOnBackdropClick: true,
            closeOnEscape: true,
            showFooter: true,
            primaryButtonText: 'OK',
            secondaryButtonText: 'Cancel',
            cancelButtonText: 'Cancel',
            ...config
        };

        // Store config statically so createContent can access it
        BaseDialog.currentDialogConfig = dialogConfig;
        
        super(defaultConfig, events);
        
        this.dialogConfig = dialogConfig;
        this.dialogEvents = events;
        this.setupDialogStyling();
        this.setupFooter();
    }

    protected createContent(): string {
        const config = BaseDialog.currentDialogConfig;
        return `
            <div class="dialog-content">
                ${this.createDialogContent()}
            </div>
            ${config.showFooter ? '<div class="dialog-footer"></div>' : ''}
        `;
    }

    protected abstract createDialogContent(): string;

    protected setupDialogStyling(): void {
        // Add dialog-specific CSS classes
        this.element.classList.add('base-dialog');
        
        // DISABLED: CSS animations with transforms can cause blur in dialog content
        // Simple opacity-only animations are safer for text rendering
        if (!document.querySelector('#dialog-animations')) {
            const style = document.createElement('style');
            style.id = 'dialog-animations';
            style.innerHTML = `
                @keyframes fadeIn {
                    from {
                        opacity: 0;
                    }
                    to {
                        opacity: 1;
                    }
                }
                
                @keyframes fadeOut {
                    from {
                        opacity: 1;
                    }
                    to {
                        opacity: 0;
                    }
                }
            `;
            document.head.appendChild(style);
        }
        
        // SIMPLE DIRECT RENDERING: No transforms, no GPU tricks
        this.element.style.cssText += `
            border-radius: var(--radius-md, 10px);
            border: 2px solid var(--border-bright, #3D444D);
            background: var(--bg-primary, #0A0E1A);
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            font-size: 14px;
            line-height: 1.5;
            /* Universal anti-blur text rendering (2024-2025 research) */
            text-rendering: auto;
            -webkit-font-smoothing: subpixel-antialiased;
            -moz-osx-font-smoothing: auto;
            font-smooth: never;
            /* Transform elimination (proven anti-blur) */
            transform: none !important;
            -webkit-transform: none !important;
            
            /* GPU control - prevent blur from hardware acceleration */
            will-change: auto;
            backface-visibility: visible;
            -webkit-backface-visibility: visible;
            perspective: none;
            -webkit-perspective: none;
            transform-style: flat;
            -webkit-transform-style: flat;
            
            /* Filter elimination */
            filter: none;
            -webkit-filter: none;
            backdrop-filter: none;
            -webkit-backdrop-filter: none;
            
            /* Isolation and containment (2024-2025 best practice) */
            isolation: isolate;
            contain: layout style;
            
            /* Sharp rendering */
            image-rendering: crisp-edges;
            image-rendering: -webkit-optimize-contrast;
            
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
        `;

        // Style the header
        if (this.headerElement) {
            this.headerElement.style.cssText += `
                background: var(--gradient-wasm, linear-gradient(135deg, #654FF0, #8B5CF6));
                color: white;
                border-radius: var(--radius-md, 10px) var(--radius-md, 10px) 0 0;
                font-weight: 600;
                border-bottom: 1px solid var(--border-bright, #3D444D);
            `;
        }

        // Style the content area - CPU RENDERING approach to avoid blur
        const contentArea = this.element.querySelector('.dialog-content') as HTMLElement;
        if (contentArea) {
            contentArea.style.cssText = `
                padding: 20px;
                flex: 1;
                overflow-y: auto;
                color: var(--text-primary, #E6EDF3);
                background: var(--bg-secondary, #151B2C);
                /* Universal anti-blur content rendering */
                text-rendering: auto;
                -webkit-font-smoothing: subpixel-antialiased;
                -moz-osx-font-smoothing: auto;
                font-smooth: never;
                
                /* No transforms or GPU acceleration */
                transform: none;
                -webkit-transform: none;
                will-change: auto;
                backface-visibility: visible;
                perspective: none;
                
                /* Sharp image rendering */
                image-rendering: crisp-edges;
                image-rendering: -webkit-optimize-contrast;
                
                /* Containment */
                contain: layout style;
            `;
        }
    }

    protected setupFooter(): void {
        if (!this.dialogConfig.showFooter) return;

        this.footerElement = this.element.querySelector('.dialog-footer') as HTMLElement;
        if (!this.footerElement) return;

        this.footerElement.style.cssText = `
            padding: 16px 20px;
            border-top: 1px solid var(--border-bright, #3D444D);
            display: flex;
            justify-content: flex-end;
            gap: 12px;
            background: var(--bg-tertiary, #1C2333);
            border-radius: 0 0 var(--radius-md, 10px) var(--radius-md, 10px);
        `;

        // Create footer buttons
        this.createFooterButtons();
    }

    protected createFooterButtons(): void {
        if (!this.footerElement) return;

        const buttons = [];

        // Cancel button (if not same as secondary)
        if (this.dialogConfig.cancelButtonText !== this.dialogConfig.secondaryButtonText) {
            buttons.push({
                text: this.dialogConfig.cancelButtonText!,
                className: 'cancel-btn',
                onClick: () => this.handleCancel()
            });
        }

        // Secondary button
        if (this.dialogConfig.secondaryButtonText) {
            buttons.push({
                text: this.dialogConfig.secondaryButtonText!,
                className: 'secondary-btn',
                onClick: () => this.handleCancel()
            });
        }

        // Primary button
        buttons.push({
            text: this.dialogConfig.primaryButtonText!,
            className: 'primary-btn',
            onClick: () => this.handleConfirm()
        });

        // Create button elements
        buttons.forEach(btn => {
            const button = document.createElement('button');
            button.textContent = btn.text;
            button.className = `dialog-btn ${btn.className}`;
            button.addEventListener('click', btn.onClick);
            
            this.styleButton(button, btn.className);
            this.footerElement!.appendChild(button);
        });
    }

    protected styleButton(button: HTMLElement, className: string): void {
        const baseStyle = `
            padding: 8px 16px;
            border: none;
            border-radius: 4px;
            font-size: 14px;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s ease;
            min-width: 80px;
        `;

        let specificStyle = '';
        if (className.includes('primary')) {
            specificStyle = `
                background: linear-gradient(90deg, #4A9EFF, #3A8EEF);
                color: white;
            `;
            button.addEventListener('mouseenter', () => {
                // Use box-shadow only for hover effect to avoid transform blur
                button.style.boxShadow = '0 4px 12px rgba(74, 158, 255, 0.4)';
                button.style.backgroundColor = 'linear-gradient(90deg, #5AAEFF, #4A9EFF)';
            });
        } else {
            specificStyle = `
                background: var(--bg-primary, #0F1419);
                color: var(--text-secondary, #A0A9BA);
                border: 1px solid var(--border-color, #2A3441);
            `;
            button.addEventListener('mouseenter', () => {
                button.style.backgroundColor = 'var(--bg-tertiary, #1C2333)';
                button.style.color = 'var(--text-primary, #E5E9F0)';
            });
        }

        button.addEventListener('mouseleave', () => {
            button.style.boxShadow = '';
            if (!className.includes('primary')) {
                button.style.backgroundColor = 'var(--bg-primary, #0F1419)';
                button.style.color = 'var(--text-secondary, #A0A9BA)';
            } else {
                button.style.backgroundColor = '';
            }
        });

        button.style.cssText = baseStyle + specificStyle;
    }

    public show(): void {
        if (this.isShown) return;

        console.log('🐛 BaseDialog.show() called');
        console.log('🐛 Dialog element:', this.element);
        console.log('🐛 Current parent:', this.element.parentNode);
        console.log('🐛 Dialog classes:', this.element.className);
        console.log('🐛 Initial computed styles:', window.getComputedStyle(this.element));

        // Create backdrop first
        if (this.dialogConfig.showBackdrop) {
            console.log('🐛 Creating backdrop...');
            this.createBackdrop();
        }

        // Check if we need centering
        if (this.config.initialPosition.x === -1 && this.config.initialPosition.y === -1) {
            // ALWAYS use flexbox centering (2024-2025 best practice for blur prevention)
            console.log('🔧 Using universal flexbox centering for blur prevention');
            this.centerDialogWithFlexbox();
        } else {
            // Manual positioning: Append directly to body with fixed positioning
            if (this.element.parentNode) {
                this.element.parentNode.removeChild(this.element);
            }
            
            // Set fixed positioning BEFORE appending to body
            this.element.style.position = 'fixed';
            this.element.style.zIndex = '100000';
            
            document.body.appendChild(this.element);
        }

        console.log('🐛 Dialog after positioning:', {
            zIndex: this.element.style.zIndex,
            position: this.element.style.position,
            parent: this.element.parentNode,
            computedStyles: window.getComputedStyle(this.element)
        });

        // Show the panel (but prevent it from moving the element)
        this.element.style.display = 'block';
        
        // Manually handle z-index like bringToFront does
        const allPanels = document.querySelectorAll('.floating-panel');
        const maxZ = Math.max(...Array.from(allPanels).map(panel => {
            const z = parseInt(window.getComputedStyle(panel).zIndex) || 0;
            return isNaN(z) ? 0 : z;
        }));
        this.element.style.zIndex = (maxZ + 1).toString();
        
        // Skip super.show() to prevent moving element back to body
        this.isShown = true;

        // Debug final state and actively search for blur sources
        setTimeout(() => {
            const computedStyle = window.getComputedStyle(this.element);
            
            // Check all possible blur sources
            const possibleBlurSources = {
                filter: computedStyle.filter,
                backdropFilter: computedStyle.backdropFilter,
                transform: computedStyle.transform,
                transformStyle: computedStyle.transformStyle,
                perspective: computedStyle.perspective,
                willChange: computedStyle.willChange,
                isolation: computedStyle.isolation,
                mixBlendMode: computedStyle.mixBlendMode,
                opacity: computedStyle.opacity
            };

            console.log('🐛 Final dialog state:', {
                element: this.element,
                backdrop: this.backdrop,
                allStyles: this.element.style.cssText,
                computedStyles: possibleBlurSources,
                boundingRect: this.element.getBoundingClientRect()
            });

            // AGGRESSIVE PARENT CHAIN BLUR CLEANUP (2024-2025 approach)
            let parent = this.element.parentElement;
            let parentLevel = 0;
            const blurSources = ['filter', 'backdrop-filter', 'transform', 'will-change', 'perspective'];
            
            while (parent && parentLevel < 15) {
                const parentStyle = window.getComputedStyle(parent);
                const parentElement = parent as HTMLElement;
                
                // Check for blur-causing properties
                const issues = [];
                if (parentStyle.filter !== 'none') issues.push(`filter: ${parentStyle.filter}`);
                if (parentStyle.backdropFilter !== 'none') issues.push(`backdrop-filter: ${parentStyle.backdropFilter}`);
                if (parentStyle.transform !== 'none' && parentStyle.transform !== 'matrix(1, 0, 0, 1, 0, 0)') {
                    issues.push(`transform: ${parentStyle.transform}`);
                }
                if (parentStyle.willChange !== 'auto') issues.push(`will-change: ${parentStyle.willChange}`);
                if (parentStyle.perspective !== 'none') issues.push(`perspective: ${parentStyle.perspective}`);
                
                if (issues.length > 0) {
                    console.log(`🔧 NEUTRALIZING BLUR SOURCE IN PARENT ${parentLevel}:`, {
                        element: parent,
                        tagName: parent.tagName,
                        className: parent.className,
                        issues: issues
                    });
                    
                    // Aggressively neutralize blur sources (if safe to do so)
                    if (!parentElement.classList.contains('main-container') && 
                        !parentElement.classList.contains('canvas-container') &&
                        parentElement.tagName !== 'BODY' &&
                        parentElement.tagName !== 'HTML') {
                        
                        parentElement.style.setProperty('filter', 'none', 'important');
                        parentElement.style.setProperty('backdrop-filter', 'none', 'important');
                        parentElement.style.setProperty('will-change', 'auto', 'important');
                        parentElement.style.setProperty('perspective', 'none', 'important');
                        console.log(`🔧 Applied fixes to parent ${parentLevel}`);
                    }
                }
                
                parent = parent.parentElement;
                parentLevel++;
            }
            
            // Also check browser zoom level
            console.log('🐛 Browser state:', {
                devicePixelRatio: window.devicePixelRatio,
                zoom: Math.round(window.devicePixelRatio * 100) + '%',
                pageZoom: document.documentElement.style.zoom || '100%'
            });
        }, 100);

        // Focus first input or primary button
        this.focusFirst();

        // Trigger show event
        if (this.dialogEvents.onShow) {
            this.dialogEvents.onShow();
        }
    }

    public close(): void {
        if (!this.isShown) return;

        console.log('🐛 BaseDialog.close() called');

        // Remove backdrop first
        if (this.backdrop) {
            this.removeBackdrop();
        }

        // Hide the panel
        super.hide();
        this.isShown = false;

        // Trigger close event
        if (this.dialogEvents.onClose) {
            this.dialogEvents.onClose();
        }

        // Clean up: remove dialog and flexbox container from DOM after animation
        setTimeout(() => {
            if (this.element && this.element.parentNode) {
                console.log('🔧 Removing dialog from DOM');
                
                // Check if dialog is in a flexbox container
                if (this.flexboxContainer && this.flexboxContainer.parentNode) {
                    console.log('🔧 Removing stored flexbox container');
                    this.flexboxContainer.parentNode.removeChild(this.flexboxContainer);
                    this.flexboxContainer = undefined;
                } else {
                    const parent = this.element.parentNode as HTMLElement;
                    if (parent && parent.classList.contains('dialog-flexbox-container')) {
                        console.log('🔧 Removing detected flexbox container');
                        // Remove the entire flexbox container
                        if (parent.parentNode) {
                            parent.parentNode.removeChild(parent);
                        }
                    } else {
                        // Remove just the dialog element
                        parent.removeChild(this.element);
                    }
                }
            }
        }, 300); // Wait for animations to complete
    }

    public setZIndex(zIndex: number): void {
        this.element.style.zIndex = zIndex.toString();
        if (this.backdrop) {
            this.backdrop.style.zIndex = (zIndex - 1).toString();
        }
    }

    public setOnShowCallback(callback: () => void): void {
        this.dialogEvents.onShow = callback;
    }

    public setOnCloseCallback(callback: () => void): void {
        this.dialogEvents.onClose = callback;
    }

    protected createBackdrop(): void {
        // Create simple backdrop without backdrop-filter
        this.backdrop = document.createElement('div');
        this.backdrop.className = 'dialog-backdrop';
        this.backdrop.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0, 0, 0, 0.6);
            z-index: 50000;
            animation: fadeIn 0.2s ease-out;
        `;

        // Instead of backdrop-filter, blur the actual page content
        this.blurPageContent();

        if (this.dialogConfig.closeOnBackdropClick) {
            this.backdrop.addEventListener('click', (e) => {
                if (e.target === this.backdrop) {
                    this.handleCancel();
                }
            });
        }

        document.body.appendChild(this.backdrop);
    }

    private blurPageContent(): void {
        // Disabled blur effect to prevent dialog content from appearing blurred
        // The backdrop provides sufficient visual separation
        console.log('🐛 Blur effect disabled - using backdrop only');
        return;
        
        // Original blur code commented out:
        // const bodyChildren = Array.from(document.body.children);
        // bodyChildren.forEach(child => {
        //     const element = child as HTMLElement;
        //     if (!element.classList.contains('base-dialog') && 
        //         !element.classList.contains('dialog-backdrop') &&
        //         !element.classList.contains('floating-panel')) {
        //         element.style.filter = 'blur(4px)';
        //         element.style.transition = 'filter 0.2s ease-out';
        //         element.setAttribute('data-dialog-blurred', 'true');
        //     }
        // });
    }

    private unblurPageContent(): void {
        console.log('🐛 Unblurring page content...');
        
        // Remove blur from all marked elements
        const blurredElements = document.querySelectorAll('[data-dialog-blurred="true"]');
        console.log('🐛 Found blurred elements:', blurredElements.length);
        
        blurredElements.forEach(element => {
            const htmlElement = element as HTMLElement;
            htmlElement.style.filter = '';
            htmlElement.style.transition = '';
            htmlElement.removeAttribute('data-dialog-blurred');
        });
        
        // Safety check: also remove blur from any main containers that might have been missed
        const mainContainers = ['#app', '.main-container', '.sidebar', '.canvas-container', '#diagram-canvas'];
        mainContainers.forEach(selector => {
            const element = document.querySelector(selector) as HTMLElement;
            if (element && element.style.filter) {
                console.log('🐛 Removing blur from missed element:', selector);
                element.style.filter = '';
                element.style.transition = '';
            }
        });
    }

    protected removeBackdrop(): void {
        if (!this.backdrop) return;

        // Restore page content blur
        this.unblurPageContent();

        this.backdrop.style.animation = 'fadeOut 0.2s ease-out';
        setTimeout(() => {
            if (this.backdrop && this.backdrop.parentNode) {
                this.backdrop.parentNode.removeChild(this.backdrop);
                this.backdrop = undefined;
            }
        }, 200);
    }

    protected centerDialogWithFlexbox(): void {
        // 2024-2025 ANTI-BLUR: Clean up any existing containers first
        const existingContainers = document.querySelectorAll('.dialog-flexbox-container');
        existingContainers.forEach(container => {
            if (container.parentNode && container.children.length === 0) {
                container.parentNode.removeChild(container);
                console.log('🔧 Cleaned up empty flexbox container');
            }
        });

        // Create new flexbox container with comprehensive blur prevention
        const container = document.createElement('div');
        container.className = 'dialog-flexbox-container';
        container.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            width: 100vw;
            height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 100001;
            pointer-events: none;
            
            /* Anti-blur properties for container */
            transform: none;
            -webkit-transform: none;
            filter: none;
            -webkit-filter: none;
            backdrop-filter: none;
            -webkit-backdrop-filter: none;
            will-change: auto;
            backface-visibility: visible;
            perspective: none;
            isolation: isolate;
            contain: layout;
        `;
        
        // Reset dialog positioning to work within flexbox
        this.element.style.position = 'relative';
        this.element.style.left = 'auto';
        this.element.style.top = 'auto';
        this.element.style.zIndex = 'auto';
        this.element.style.pointerEvents = 'auto';
        
        // CRITICAL: Ensure dialog is properly moved into container
        if (this.element.parentNode) {
            this.element.parentNode.removeChild(this.element);
        }
        
        // Add dialog to container FIRST, then container to body
        container.appendChild(this.element);
        document.body.appendChild(container);
        
        // Store reference for cleanup
        this.flexboxContainer = container;
        
        // Verify the dialog is properly nested
        setTimeout(() => {
            if (this.element.parentNode === container) {
                console.log('🔧 Dialog successfully placed in flexbox container');
            } else {
                console.error('🚨 Dialog not properly nested in flexbox container!');
            }
        }, 10);
        
        console.log('🔧 Using enhanced flexbox centering for blur prevention');
    }

    protected centerDialog(): void {
        // FALLBACK: Use pixel centering for manual positioning
        const x = Math.max(20, (window.innerWidth - this.element.offsetWidth) / 2);
        const y = Math.max(20, (window.innerHeight - this.element.offsetHeight) / 2);
        
        // Use integer pixel values to prevent subpixel rendering
        this.element.style.left = `${Math.round(x)}px`;
        this.element.style.top = `${Math.round(y)}px`;
    }

    protected focusFirst(): void {
        // Focus first input element or primary button
        const firstInput = this.element.querySelector('input, select, textarea') as HTMLElement;
        if (firstInput) {
            firstInput.focus();
        } else {
            const primaryBtn = this.element.querySelector('.primary-btn') as HTMLElement;
            if (primaryBtn) {
                primaryBtn.focus();
            }
        }
    }

    protected handleConfirm(value?: unknown): void {
        if (this.dialogEvents.onConfirm) {
            this.dialogEvents.onConfirm(value);
        }
    }

    protected handleCancel(): void {
        if (this.dialogEvents.onCancel) {
            this.dialogEvents.onCancel();
        }
    }

    // Get dialog result - to be implemented by specific dialog types
    public abstract getResult(): DialogResult;

    // Validate dialog input - to be implemented by specific dialog types
    public abstract validate(): boolean;

    // Note: Dialogs maintain their own z-index based on dialog manager
    // The dialog manager handles z-index for dialogs
    public ensureDialogZIndex(): void {
        // Dialogs should always stay at very high z-index
        this.element.style.zIndex = '100000';
        this.element.style.position = 'fixed';
        this.element.style.isolation = 'isolate';
        console.log('🐛 Dialog ensureDialogZIndex called, maintaining z-index:', this.element.style.zIndex);
    }

    // Force sharp rendering - call this if experiencing blur
    public forceSharpRendering(): void {
        // Apply anti-blur styles
        this.element.style.textRendering = 'optimizeLegibility';
        this.element.style.webkitFontSmoothing = 'antialiased';
        this.element.style.mozOsxFontSmoothing = 'grayscale';
        // Don't use transform as it can cause blur
        this.element.style.transform = 'none';
        this.element.style.backfaceVisibility = 'hidden';
        this.element.style.webkitBackfaceVisibility = 'hidden';
        this.element.style.perspective = '1000';
        this.element.style.webkitPerspective = '1000';
        
        // Round position to nearest pixel
        const rect = this.element.getBoundingClientRect();
        this.element.style.left = `${Math.round(rect.left)}px`;
        this.element.style.top = `${Math.round(rect.top)}px`;
        
        // Apply to all child elements
        this.element.querySelectorAll('*').forEach(el => {
            const element = el as HTMLElement;
            element.style.textRendering = 'optimizeLegibility';
            element.style.webkitFontSmoothing = 'antialiased';
            element.style.transform = 'none';
        });
        
        console.log('🔧 Applied sharp rendering fixes to dialog');
    }

    // Debug helper - call from console
    public static debugDialogState(): void {
        console.group('🐛 Dialog Debug Information');
        
        // Find all dialogs
        const dialogs = document.querySelectorAll('.base-dialog');
        console.log('📊 Total dialogs found:', dialogs.length);
        
        dialogs.forEach((dialog, index) => {
            const element = dialog as HTMLElement;
            const computed = window.getComputedStyle(element);
            
            console.group(`📋 Dialog ${index + 1}`);
            console.log('Element:', element);
            console.log('Classes:', element.className);
            console.log('Parent:', element.parentNode);
            console.log('Z-Index:', computed.zIndex);
            console.log('Position:', computed.position);
            console.log('Filter:', computed.filter);
            console.log('Backdrop-Filter:', computed.backdropFilter);
            console.log('Background:', computed.background);
            console.log('Opacity:', computed.opacity);
            console.log('Transform:', computed.transform);
            console.log('Bounding Rect:', element.getBoundingClientRect());
            console.groupEnd();
        });

        // Find all backdrops
        const backdrops = document.querySelectorAll('.dialog-backdrop');
        console.log('🎭 Total backdrops found:', backdrops.length);
        
        backdrops.forEach((backdrop, index) => {
            const element = backdrop as HTMLElement;
            const computed = window.getComputedStyle(element);
            
            console.group(`🎭 Backdrop ${index + 1}`);
            console.log('Element:', element);
            console.log('Z-Index:', computed.zIndex);
            console.log('Background:', computed.background);
            console.log('Filter:', computed.filter);
            console.log('Backdrop-Filter:', computed.backdropFilter);
            console.groupEnd();
        });

        // Check for blurred elements
        const blurredElements = document.querySelectorAll('[data-dialog-blurred="true"]');
        console.log('🌫️ Blurred elements found:', blurredElements.length);
        
        blurredElements.forEach(element => {
            const htmlElement = element as HTMLElement;
            const computed = window.getComputedStyle(htmlElement);
            console.log('Blurred element:', htmlElement.tagName, htmlElement.className, 'Filter:', computed.filter);
        });

        console.groupEnd();
    }
}