// Force workspace selector to be visible - run this in the browser console
console.log('=== Force Workspace Selector Visible ===');

// Find the workspace container and make it highly visible
const container = document.querySelector('#workspace-selector-container');
if (container) {
    console.log('Found workspace container, forcing visibility...');
    
    // Add temporary styling to make it super visible
    container.style.cssText = `
        background: red !important;
        border: 3px solid yellow !important;
        padding: 10px !important;
        margin: 10px !important;
        min-width: 200px !important;
        min-height: 50px !important;
        display: block !important;
        visibility: visible !important;
        opacity: 1 !important;
        z-index: 9999 !important;
        position: relative !important;
    `;
    
    console.log('Applied visibility styles to workspace container');
    console.log('Container content:', container.innerHTML);
    
    // Also style any workspace selector inside
    const selector = container.querySelector('.workspace-selector');
    if (selector) {
        console.log('Found workspace selector, styling it too...');
        selector.style.cssText = `
            background: blue !important;
            color: white !important;
            border: 2px solid white !important;
            padding: 5px !important;
            display: block !important;
            visibility: visible !important;
            opacity: 1 !important;
        `;
    }
    
    // Add test content if empty
    if (!container.innerHTML.trim()) {
        console.log('Container is empty, adding test content...');
        container.innerHTML = '<div style="background: green; color: white; padding: 10px;">TEST WORKSPACE SELECTOR</div>';
    }
    
} else {
    console.log('❌ Workspace container not found');
    
    // Try to find the toolbar and add workspace selector manually
    const toolbar = document.querySelector('.glsp-toolbar');
    if (toolbar) {
        console.log('Found toolbar, adding workspace selector manually...');
        const testWorkspace = document.createElement('div');
        testWorkspace.innerHTML = `
            <div style="background: purple; color: white; padding: 10px; margin: 5px; border: 2px solid white;">
                <strong>TEST WORKSPACE SELECTOR</strong>
                <br>This should be visible now!
            </div>
        `;
        toolbar.appendChild(testWorkspace);
        console.log('Added test workspace selector to toolbar');
    } else {
        console.log('❌ Toolbar not found either');
    }
}

// List all toolbar content
const toolbar = document.querySelector('.glsp-toolbar');
if (toolbar) {
    console.log('Toolbar content:');
    console.log(toolbar.innerHTML.substring(0, 500) + '...');
    
    console.log('Toolbar children:');
    Array.from(toolbar.children).forEach((child, i) => {
        console.log(`${i + 1}:`, child.tagName, child.className, child.id);
    });
}

console.log('=== End Force Visibility Test ===');