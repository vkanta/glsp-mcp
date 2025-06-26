/**
 * Main entry point for GLSP Web Client
 */

import { GLSPApp } from './app.js';

// Initialize the application when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    const canvas = document.getElementById('diagram-canvas') as HTMLCanvasElement;
    
    if (!canvas) {
        console.error('Canvas element not found');
        return;
    }

    // Create the GLSP application
    const app = new GLSPApp(canvas);

    // Mount UI components
    const toolbarContainer = document.getElementById('toolbar-container');
    const statusContainer = document.getElementById('status-container');
    const diagramListContainer = document.getElementById('diagram-list-container');
    const wasmPaletteContainer = document.getElementById('wasm-palette-container');

    if (toolbarContainer) {
        toolbarContainer.appendChild(app.getToolbar());
    }

    if (statusContainer) {
        statusContainer.appendChild(app.getStatus());
    }

    if (diagramListContainer) {
        diagramListContainer.appendChild(app.getDiagramList());
        diagramListContainer.appendChild(app.getAIPanel());
    }

    if (wasmPaletteContainer) {
        wasmPaletteContainer.appendChild(app.getWasmPalette());
    }

    // Make app globally available for HTML onclick handlers
    (window as any).app = app;

    console.log('🚀 MCP-GLSP Web Client initialized');
});

// Handle global errors
window.addEventListener('error', (event) => {
    console.error('Global error:', event.error);
});

window.addEventListener('unhandledrejection', (event) => {
    console.error('Unhandled promise rejection:', event.reason);
});