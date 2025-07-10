// Comprehensive UI element search - run this in the browser console
console.log('=== Comprehensive UI Element Search ===');

// Search for all possible toolbar-like elements
const possibleToolbars = [
    '.glsp-toolbar',
    '.toolbar',
    '[class*="toolbar"]',
    '[id*="toolbar"]',
    'header',
    'nav',
    '.top-bar',
    '.menu-bar'
];

console.log('Searching for toolbar-like elements...');
possibleToolbars.forEach(selector => {
    const elements = document.querySelectorAll(selector);
    if (elements.length > 0) {
        console.log(`Found ${elements.length} elements matching "${selector}":`, elements);
        elements.forEach((el, i) => {
            console.log(`  ${i + 1}:`, el.tagName, el.className, el.id, el);
        });
    }
});

// Search for workspace-related elements
const workspaceSelectors = [
    '#workspace-selector-container',
    '.workspace-selector',
    '[id*="workspace"]',
    '[class*="workspace"]'
];

console.log('Searching for workspace-related elements...');
workspaceSelectors.forEach(selector => {
    const elements = document.querySelectorAll(selector);
    if (elements.length > 0) {
        console.log(`Found ${elements.length} elements matching "${selector}":`, elements);
    }
});

// Get all elements and look for structure
console.log('Document body structure:');
const body = document.body;
if (body) {
    console.log('Body children:');
    Array.from(body.children).forEach((child, i) => {
        console.log(`  ${i + 1}:`, child.tagName, child.className, child.id);
        
        // Look for main app container
        if (child.children.length > 0) {
            console.log(`    Children of ${child.tagName}:`);
            Array.from(child.children).forEach((grandchild, j) => {
                console.log(`      ${j + 1}:`, grandchild.tagName, grandchild.className, grandchild.id);
            });
        }
    });
}

// Search for elements with specific content
console.log('Searching for elements containing "Diagram Type" or "Mode"...');
const allElements = document.querySelectorAll('*');
let foundToolbarElements = [];

allElements.forEach(el => {
    if (el.textContent && (
        el.textContent.includes('Diagram Type') ||
        el.textContent.includes('Mode:') ||
        el.textContent.includes('Create Node') ||
        el.textContent.includes('Create Edge')
    )) {
        foundToolbarElements.push(el);
    }
});

if (foundToolbarElements.length > 0) {
    console.log('Found potential toolbar elements by content:');
    foundToolbarElements.forEach((el, i) => {
        console.log(`  ${i + 1}:`, el.tagName, el.className, el.id, el.textContent.substring(0, 100));
    });
} else {
    console.log('No toolbar elements found by content search');
}

// Look for any elements with toolbar-group class
const toolbarGroups = document.querySelectorAll('.toolbar-group');
console.log(`Found ${toolbarGroups.length} .toolbar-group elements:`, toolbarGroups);

// Check if there are any hidden elements
console.log('Checking for hidden elements...');
const hiddenElements = document.querySelectorAll('[style*="display: none"], [style*="visibility: hidden"], .hidden');
console.log(`Found ${hiddenElements.length} potentially hidden elements:`, hiddenElements);

// Final check - dump the entire page structure to see what we're working with
console.log('Full page HTML structure (first 2000 chars):');
console.log(document.documentElement.innerHTML.substring(0, 2000));

console.log('=== End Comprehensive Search ===');