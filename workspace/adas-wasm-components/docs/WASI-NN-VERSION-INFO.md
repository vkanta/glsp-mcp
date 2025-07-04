# WASI-NN Version Information

## Current Version Used

Your components use: **`wasi:nn@0.2.0-rc-2024-10-28`**

This is the latest release candidate version of WASI-NN from October 28, 2024.

## Version History

- `wasi:nn@0.1.0` - Original version
- `wasi:nn@0.2.0-rc-2024-10-28` - Current RC with improved resource handling

## Why This Version?

The `0.2.0-rc-2024-10-28` version includes:
- Better resource management for tensors and graphs
- Improved error handling
- Support for more ML frameworks
- Component Model compatibility

## Runtime Support Status

| Runtime | WASI-NN Support | Version Match | Notes |
|---------|-----------------|---------------|-------|
| wasmtime v26-27 | ✅ Yes | ✅ Same version | Needs to be built with `--features wasi-nn` |
| WasmEdge | ✅ Yes | ❌ Different API | Doesn't support Component Model |
| WAMR | ✅ Yes | ❓ Unknown | Good for embedded |

## The Issue

The pre-built wasmtime binaries don't include WASI-NN backends (ONNX, OpenVINO, etc.) even though they have the right WASI-NN version. You need to:

1. Build wasmtime from source with WASI-NN
2. Install the ML backend (ONNX Runtime, OpenVINO, etc.)
3. Run with proper flags

## Quick Fix Options

### Option 1: Use Docker with Pre-configured Runtime
```bash
# (Future solution - create Docker image with wasmtime + ONNX)
docker run -v $(pwd):/app wasmtime-wasi-nn \
  wasmtime run -S nn=y /app/target/wasm32-wasip2/debug/adas_object_detection_ai.wasm
```

### Option 2: Downgrade to Older WASI-NN (Not Recommended)
Would require rebuilding all components with older wit-bindgen.

### Option 3: Wait for Official Support
wasmtime will eventually ship with WASI-NN backends in pre-built binaries.

## Your Components Are Correct!

The version `wasi:nn@0.2.0-rc-2024-10-28` is the right one to use. It's the same version that wasmtime's source code uses. The issue is just that pre-built binaries don't include the ML backends.