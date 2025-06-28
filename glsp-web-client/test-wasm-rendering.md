# WASM Component Rendering Fix

## Problem
When dragging WASM components from the sidebar to the canvas, they showed error indicators ("!" and "...") instead of the component name.

## Root Causes
1. **Component name extraction**: The renderer was looking for `componentName` in properties, but the MCP `create_node` tool sets it as the `label` field
2. **Missing indicator logic**: Components were incorrectly marked as "missing" when they were just unloaded

## Fixes Applied

### 1. Fixed component name extraction in WasmComponentRendererV2.ts:
```typescript
// Before:
const componentName = element.properties?.label?.toString() || 
                    element.properties?.componentName?.toString() || 
                    'Component';

// After:
const componentName = element.label?.toString() || 
                    element.properties?.label?.toString() || 
                    element.properties?.componentName?.toString() || 
                    'Component';
```

### 2. Fixed missing indicator logic in canvas-renderer.ts:
```typescript
// Before:
isMissing: isMissing || !isLoaded, // Show as missing if not loaded

// After:
isMissing: isMissing, // Only show as missing if file is actually missing
```

## Testing
1. Start the dev server: `npm run dev`
2. Open the application
3. Ensure you're in a workflow diagram (or create a new one)
4. Drag a WASM component from the sidebar to the canvas
5. The component should now display its name correctly without error indicators

## Expected Result
WASM components should render with:
- Component name in the header
- Proper styling (no red error overlay)
- Status indicator showing "unloaded" state (not error state)