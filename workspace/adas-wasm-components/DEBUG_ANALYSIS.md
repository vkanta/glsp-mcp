# Deep Debugging Analysis: WASI SDK Toolchain Issue

## Root Cause Identified ✅

The WASI SDK is properly downloaded and exists, but there's a **path mismatch** between what the rust_toolchain expects and what actually exists in the sandbox.

## Critical Path Analysis

### What the rust_toolchain is trying to execute:
```
external/rules_wasm_component+/toolchains/rules_wasm_component++wasi_sdk+wasi_sdk/bin/ar
```

### What actually exists in the sandbox:
```
external/rules_wasm_component++wasi_sdk+wasi_sdk/bin/ar -> llvm-ar ✅
```

**Issue**: The rust_toolchain includes an extra `/toolchains/` segment in the path that doesn't exist.

## WASI SDK Status: ✅ WORKING

### WASI SDK Download Status:
- ✅ Successfully downloaded to: `/private/var/tmp/_bazel_r/.../external/rules_wasm_component++wasi_sdk+wasi_sdk/`
- ✅ Contains all required tools: `ar`, `llvm-ar`, `clang`, etc.
- ✅ Tools are executable and properly linked
- ✅ Mapped into sandbox correctly

### WASI SDK Configuration in MODULE.bazel.lock:
```json
{
  "strategy": "download",
  "version": "25", 
  "url": "",
  "wasi_sdk_root": ""  // ⚠️ Empty but tools still downloaded
}
```

## Toolchain Configuration Analysis

### rules_wasm_component Toolchain: ✅ CORRECT
```starlark
filegroup(
  name = "wasi_sdk_tools",
  srcs = ["@@rules_wasm_component++wasi_sdk+wasi_sdk//:ar", ...]  // ✅ Correct path
)
```

### rust_toolchain Configuration: ❌ INCORRECT PATH
- The rust_toolchain is somehow referencing a path with `/toolchains/` inserted
- This suggests the rules_rust fork may have incorrect WASI SDK path construction

## Environment Analysis

### Platform Configuration: ✅
- **Host**: macOS ARM64 (darwin_arm64) 
- **Target**: wasm32-wasip2
- **Bazel**: 8.3.1 (Modern Bazel 2025)
- **rules_rust**: commit `1945773a` (avrabe/rules_rust fork)

### Sandbox Analysis: ✅
- WASI SDK tools properly mapped into sandbox
- Symlinks correctly created: `ar -> llvm-ar`
- All binaries executable and accessible

### Configuration Files: ✅
- No conflicting `.bazelrc` files
- No legacy `WORKSPACE` files
- Proper MODULE.bazel bzlmod configuration
- No environment variable conflicts

## Comparison with Working Examples

The rules_wasm_component maintainer reports this works in their examples. Key differences likely:

1. **Build target structure**: They may be using different rust_wasm_component_bindgen configurations
2. **Toolchain registration order**: Different sequence of toolchain registrations
3. **Platform constraints**: Different platform/constraint configurations
4. **rules_rust version**: Potentially using a different commit/branch

## Potential Solutions

### Option 1: Path Correction in rules_rust Fork
The avrabe/rules_rust fork needs to fix the WASI SDK path construction to remove the extra `/toolchains/` segment.

### Option 2: Toolchain Reference Update
Update our MODULE.bazel to use a different toolchain registration that matches the expected path format.

### Option 3: Custom Toolchain Override
Create a custom toolchain configuration that corrects the path mismatch.

## Next Steps

1. **Report to rules_rust fork**: The path construction bug needs to be fixed
2. **Test with different commit**: Try a different rules_rust commit that might have the correct path
3. **Custom toolchain**: Implement a workaround with correct paths

## Verification Commands

```bash
# Check WASI SDK exists and is executable
ls -la /private/var/tmp/_bazel_r/.../external/rules_wasm_component++wasi_sdk+wasi_sdk/bin/ar

# Check sandbox mapping
ls -la /private/var/tmp/_bazel_r/.../sandbox/.../external/rules_wasm_component++wasi_sdk+wasi_sdk/bin/ar

# Debug build with sandbox details
bazel build //test/simple:simple_component_release --sandbox_debug --verbose_failures
```

## Conclusion

The issue is **NOT** with WASI SDK download or configuration. The WASI SDK is working perfectly. The issue is a **path construction bug** in the rules_rust fork where it's inserting an extra `/toolchains/` segment that doesn't exist in the actual repository structure.