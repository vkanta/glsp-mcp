# WASM Graphics Implementation Plan

## Current Decision: Server-Side Rendering (Phase 1)

Based on security analysis, we'll start with server-side rendering for all WASM graphics components.

## Integration with MCP

### New MCP Tool: `render_wasm_graphics`

```rust
Tool {
    name: "render_wasm_graphics",
    description: "Execute WASM component graphics rendering on server",
    inputSchema: {
        type: "object",
        properties: {
            componentId: { type: "string" },
            method: { type: "string", default: "render" },
            input: { type: "object" },
            format: { 
                type: "string", 
                enum: ["png", "svg", "canvas_commands"],
                default: "svg"
            }
        },
        required: ["componentId"]
    }
}
```

### New MCP Resource: `wasm://graphics/{componentId}/render`

Streaming graphics updates via HTTP/2 Server-Sent Events:

```typescript
// Client-side consumption
const graphicsStream = await mcpClient.subscribeResource(
    `wasm://graphics/${componentId}/render`
);

graphicsStream.on('data', (frame: GraphicsOutput) => {
    switch (frame.type) {
        case 'svg':
            updateSvgElement(frame.data);
            break;
        case 'image':
            updateImageElement(frame.data);
            break;
        case 'canvas_commands':
            replayCanvasCommands(frame.data);
            break;
    }
});
```

## Example WASM Component with Graphics

```rust
// example_visualization.rs
use wasi_gfx::{Canvas, Color, Font};

#[export_name = "render"]
pub extern "C" fn render(input_ptr: *const u8, input_len: usize) -> *mut u8 {
    // Parse input data
    let input = unsafe { 
        std::slice::from_raw_parts(input_ptr, input_len) 
    };
    
    // Create canvas
    let mut canvas = Canvas::new(400, 300);
    
    // Draw visualization
    canvas.fill_rect(10, 10, 380, 280, Color::rgb(240, 240, 240));
    canvas.stroke_rect(10, 10, 380, 280, Color::rgb(51, 51, 51), 2.0);
    
    // Draw data visualization based on input
    let data: Vec<f32> = parse_input(input);
    draw_chart(&mut canvas, &data);
    
    // Export as SVG
    let svg = canvas.to_svg();
    Box::into_raw(Box::new(svg))
}
```

## Security Measures

1. **Resource Limits**
   ```rust
   const MAX_RENDER_TIME: Duration = Duration::from_secs(5);
   const MAX_MEMORY: usize = 100 * 1024 * 1024; // 100MB
   const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024; // 10MB
   ```

2. **Output Sanitization**
   - SVG: Remove scripts, event handlers, external references
   - Images: Validate format, re-encode if necessary
   - Canvas commands: Whitelist allowed operations

3. **Rate Limiting**
   - Per-component: 30 renders/second max
   - Per-client: 100 renders/minute max
   - Global: 1000 renders/second max

## Performance Optimization

1. **Caching**
   - LRU cache for rendered outputs
   - Cache key: hash(componentId + method + input)
   - TTL: 5 minutes for static, 1 second for animated

2. **Streaming**
   - Use HTTP/2 multiplexing
   - Compress output with Brotli
   - Delta encoding for animations

3. **Parallel Rendering**
   - Thread pool for WASM execution
   - GPU acceleration where available
   - Batch similar requests

## Future Phases

### Phase 2: Component Signing (Q2 2025)
- Implement PKI infrastructure
- Sign trusted components
- Verify signatures on load

### Phase 3: Selective Client Rendering (Q3 2025)
- Enable for signed components only
- User consent required
- Fallback to server rendering

### Phase 4: Hybrid Intelligence (Q4 2025)
- ML-based risk assessment
- Automatic routing decisions
- Performance prediction

## Testing Strategy

1. **Security Tests**
   - Malicious SVG injection
   - Resource exhaustion
   - Output size limits

2. **Performance Tests**
   - Latency measurements
   - Throughput benchmarks
   - Scaling tests

3. **Compatibility Tests**
   - Different WASM components
   - Various output formats
   - Client rendering fallbacks