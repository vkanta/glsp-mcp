# WASI SDK Path Construction Bug in rust_toolchain

## Summary
The rust_toolchain target is constructing incorrect paths to WASI SDK tools, adding an extra `/toolchains/` segment that doesn't exist in the actual repository structure.

## Environment
- **rules_rust**: commit `1945773a` from `avrabe/rules_rust` feature/wasi-p2-support branch
- **Platform**: macOS ARM64
- **Target**: wasm32-wasip2
- **Bazel**: 8.3.1

## Issue Details

### Expected Path:
```
external/rules_wasm_component++wasi_sdk+wasi_sdk/bin/ar
```
*(This path exists and works correctly)*

### Actual Path Being Used by rust_toolchain:
```
external/rules_wasm_component+/toolchains/rules_wasm_component++wasi_sdk+wasi_sdk/bin/ar
```
*(This path does not exist - note the extra `/toolchains/` segment)*

## Error Message
```
src/main/tools/process-wrapper-legacy.cc:80: "execvp(external/rules_wasm_component+/toolchains/rules_wasm_component++wasi_sdk+wasi_sdk/bin/ar, ...)": No such file or directory
```

## Verification
The WASI SDK is properly downloaded and all tools exist:
```bash
$ ls -la /private/var/tmp/_bazel_r/.../external/rules_wasm_component++wasi_sdk+wasi_sdk/bin/
total 768
-rwxr-xr-x  1 r  wheel  97816 Dec 12  2024 ar -> llvm-ar ✅
-rwxr-xr-x  1 r  wheel  98344 Dec 12  2024 clang ✅
-rwxr-xr-x  1 r  wheel  97816 Dec 12  2024 llvm-ar ✅
# ... all tools present and executable
```

## Root Cause
The rust_toolchain target appears to be incorrectly constructing WASI SDK tool paths by inserting `/toolchains/` where it shouldn't exist. This suggests a bug in how the rules_rust fork references external WASI SDK tools.

## Impact
This prevents successful compilation of WebAssembly components using `rust_wasm_component_bindgen` rules, as the final linking step fails when trying to execute the `ar` tool.

## Requested Fix
Please investigate and fix the path construction logic in the rust_toolchain to reference WASI SDK tools using the correct path format without the extra `/toolchains/` segment.