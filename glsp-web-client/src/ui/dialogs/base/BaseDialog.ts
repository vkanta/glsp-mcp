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
    onConfirm?: (value?: any) => void;
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
        
        // Add CSS animations if not already present
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
                
                @keyframes slideIn {
                    from {
                        transform: translateY(-20px);
                        opacity: 0;
                    }
                    to {
                        transform: translateY(0);
                        opacity: 1;
                    }
                }
            `;
            document.head.appendChild(style);
        }
        
        // Set dialog-specific styles
        this.element.style.cssText += `
            border-radius: var(--radius-md, 10px);
            box-shadow: var(--shadow-lg, 0 8px 32px rgba(0, 0, 0, 0.5));
            border: 2px solid var(--border-bright, #3D444D);
            background: var(--bg-primary, #0A0E1A);
            animation: slideIn 0.3s ease-out;
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

        // Style the content area
        const contentArea = this.element.querySelector('.dialog-content') as HTMLElement;
        if (contentArea) {
            contentArea.style.cssText = `
                padding: 20px;
                flex: 1;
                overflow-y: auto;
                color: var(--text-primary, #E6EDF3);
                background: var(--bg-secondary, #151B2C);
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
                button.style.transform = 'translateY(-1px)';
                button.style.boxShadow = '0 4px 12px rgba(74, 158, 255, 0.4)';
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
            button.style.transform = '';
            button.style.boxShadow = '';
            if (!className.includes('primary')) {
                button.style.backgroundColor = 'var(--bg-primary, #0F1419)';
                button.style.color = 'var(--text-secondary, #A0A9BA)';
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

        // CRITICAL: Remove dialog from current parent and append to body to escape stacking context
        if (this.element.parentNode && this.element.parentNode !== document.body) {
            console.log('🐛 Moving dialog from', this.element.parentNode, 'to document.body');
            this.element.parentNode.removeChild(this.element);
        }
        document.body.appendChild(this.element);

        // Set very high z-index to ensure it's above backdrop
        this.element.style.zIndex = '100000';
        this.element.style.position = 'fixed';

        console.log('🐛 Dialog after positioning:', {
            zIndex: this.element.style.zIndex,
            position: this.element.style.position,
            parent: this.element.parentNode,
            computedStyles: window.getComputedStyle(this.element)
        });

        // Center the dialog if position is -1, -1
        if (this.config.initialPosition.x === -1 && this.config.initialPosition.y === -1) {
            this.centerDialog();
        }

        // Show the panel
        super.show();
        this.isShown = true;

        // Debug final state
        setTimeout(() => {
            console.log('🐛 Final dialog state:', {
                element: this.element,
                backdrop: this.backdrop,
                computedFilter: window.getComputedStyle(this.element).filter,
                computedBackdropFilter: window.getComputedStyle(this.element).backdropFilter,
                allStyles: this.element.style.cssText,
                boundingRect: this.element.getBoundingClientRect()
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

        // Clean up: remove dialog from DOM after animation
        setTimeout(() => {
            if (this.element && this.element.parentNode) {
                console.log('🐛 Removing dialog from DOM');
                this.element.parentNode.removeChild(this.element);
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
        // Find all direct children of body that aren't dialogs or backdrops
        const bodyChildren = Array.from(document.body.children);
        console.log('🐛 Body children found:', bodyChildren.length);
        
        bodyChildren.forEach(child => {
            const element = child as HTMLElement;
            console.log('🐛 Checking element:', element.tagName, element.className);
            
            // Skip dialog elements and backdrops
            if (!element.classList.contains('base-dialog') && 
                !element.classList.contains('dialog-backdrop') &&
                !element.classList.contains('floating-panel')) {
                
                console.log('🐛 Applying blur to:', element.tagName, element.className);
                // Apply blur to page content
                element.style.filter = 'blur(4px)';
                element.style.transition = 'filter 0.2s ease-out';
                element.setAttribute('data-dialog-blurred', 'true');
            } else {
                console.log('🐛 Skipping element (dialog/backdrop/panel):', element.tagName, element.className);
            }
        });
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

    protected centerDialog(): void {
        const rect = this.element.getBoundingClientRect();
        const x = (window.innerWidth - rect.width) / 2;
        const y = (window.innerHeight - rect.height) / 2;
        
        this.element.style.left = `${Math.max(20, x)}px`;
        this.element.style.top = `${Math.max(20, y)}px`;
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

    protected handleConfirm(value?: any): void {
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