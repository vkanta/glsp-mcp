// Test script to verify WASM component visual indicators
// Run this in the browser console when the app is loaded

console.log('=== WASM Visual Status Test ===');

// Check if the app controller is available
if (window.appController) {
    console.log('✅ App controller found');
    
    // Get the current diagram
    const diagram = window.appController.diagramService.getCurrentDiagram();
    if (diagram) {
        console.log('✅ Current diagram found:', diagram.id);
        
        // Find WASM components in the diagram
        const wasmComponents = Object.entries(diagram.elements).filter(([id, element]) => 
            element.type === 'wasm-component' || element.type === 'WASM Component'
        );
        
        console.log(`Found ${wasmComponents.length} WASM components:`);
        
        wasmComponents.forEach(([id, component]) => {
            const name = component.properties?.componentName || component.properties?.name || 'Unknown';
            const isLoaded = component.properties?.isLoaded || false;
            console.log(`- ${name} (${id}): ${isLoaded ? '✅ Loaded' : '⭕ Not loaded'}`);
        });
        
        // Check the canvas renderer
        if (window.appController.renderer) {
            console.log('✅ Canvas renderer available');
            console.log('Triggering canvas refresh...');
            window.appController.renderer.render();
            console.log('Canvas refreshed! Check visual indicators.');
        }
        
        // Check WASM runtime manager
        if (window.wasmRuntime) {
            console.log('✅ WASM runtime manager available');
            const loadedComponents = window.wasmRuntime.getLoadedComponents();
            console.log(`Loaded components in memory: ${loadedComponents.size}`);
        }
        
    } else {
        console.log('❌ No diagram loaded');
    }
} else {
    console.log('❌ App controller not found - make sure the app is fully loaded');
}

console.log('\n=== Visual Indicator Guide ===');
console.log('Components should show:');
console.log('- Purple/blue color when loaded (with green status dot)');
console.log('- Gray color when unloaded (with gray status dot)');
console.log('- Red overlay when missing/error (with red status dot)');
console.log('\nUse the floating panel buttons to change component states and see visual updates!');