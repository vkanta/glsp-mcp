# Example Bazel Starlark rules for WASM Component building
# This would typically live in bazel/wasm_component_rules.bzl

"""Rules for building WebAssembly components with WIT interfaces."""

load("@bazel_skylib//lib:paths.bzl", "paths")
load("@rules_rust//rust:defs.bzl", "rust_common")

# Provider for WASM component information
WasmComponentInfo = provider(
    doc = "Information about a WebAssembly component",
    fields = {
        "wasm_file": "The .wasm component file",
        "wit_world": "The WIT world this component implements",
        "component_type": "Type of component (sensor, ai, control, etc.)",
        "metadata": "Component metadata file",
        "timing_constraints": "Real-time timing requirements",
        "validated": "Whether component has been validated",
    },
)

# Provider for WIT bindings
WitBindingsInfo = provider(
    doc = "Generated WIT bindings information",
    fields = {
        "bindings_dir": "Directory containing generated bindings",
        "wit_files": "Source WIT files",
        "dependencies": "WIT dependencies",
    },
)

def _wit_bindings_impl(ctx):
    """Generate Rust bindings from WIT files."""
    
    out_dir = ctx.actions.declare_directory(ctx.attr.out_dir)
    
    # Collect all WIT files
    wit_files = []
    for dep in ctx.attr.wit_deps:
        wit_files.extend(dep[DefaultInfo].files.to_list())
    
    # Generate wit-bindgen command
    args = ctx.actions.args()
    args.add("rust")
    args.add("--out-dir", out_dir.path)
    args.add("--world", ctx.file.world.path)
    
    # Add WIT dependencies paths
    for wit_file in wit_files:
        args.add("--wit-path", wit_file.dirname)
    
    ctx.actions.run(
        outputs = [out_dir],
        inputs = wit_files + [ctx.file.world],
        executable = ctx.executable._wit_bindgen,
        arguments = [args],
        mnemonic = "WitBindgen",
        progress_message = "Generating WIT bindings for %s" % ctx.label,
    )
    
    return [
        DefaultInfo(files = depset([out_dir])),
        WitBindingsInfo(
            bindings_dir = out_dir,
            wit_files = wit_files,
            dependencies = ctx.attr.wit_deps,
        ),
    ]

wit_bindings = rule(
    implementation = _wit_bindings_impl,
    doc = "Generate Rust bindings from WIT interfaces",
    attrs = {
        "world": attr.label(
            doc = "WIT world file",
            allow_single_file = [".wit"],
            mandatory = True,
        ),
        "wit_deps": attr.label_list(
            doc = "WIT interface dependencies",
            providers = [DefaultInfo],
        ),
        "out_dir": attr.string(
            doc = "Output directory for bindings",
            default = "bindings",
        ),
        "_wit_bindgen": attr.label(
            doc = "wit-bindgen tool",
            executable = True,
            cfg = "exec",
            default = "@wit_bindgen//:wit-bindgen",
        ),
    },
)

def _wasm_component_impl(ctx):
    """Convert a WASM module to a WebAssembly Component."""
    
    wasm_file = ctx.file.wasm_binary
    output = ctx.actions.declare_file(ctx.label.name + ".wasm")
    
    # Build wasm-tools command
    args = ctx.actions.args()
    args.add("component")
    args.add("new")
    args.add(wasm_file.path)
    args.add("-o", output.path)
    
    # Add WIT world for component
    if ctx.file.wit_world:
        args.add("--wit", ctx.file.wit_world.path)
    
    # Add adapter if needed
    if ctx.file.adapter:
        args.add("--adapter", ctx.file.adapter.path)
    
    ctx.actions.run(
        outputs = [output],
        inputs = [wasm_file] + ([ctx.file.wit_world] if ctx.file.wit_world else []),
        executable = ctx.executable._wasm_tools,
        arguments = [args],
        mnemonic = "WasmComponent",
        progress_message = "Creating WebAssembly component %s" % ctx.label,
    )
    
    # Validate component if requested
    if ctx.attr.validate:
        validation_output = ctx.actions.declare_file(ctx.label.name + ".validation")
        _validate_component(ctx, output, validation_output)
    
    # Optimize component if requested
    if ctx.attr.optimize:
        optimized = ctx.actions.declare_file(ctx.label.name + ".opt.wasm")
        _optimize_component(ctx, output, optimized)
        output = optimized
    
    return [
        DefaultInfo(files = depset([output])),
        WasmComponentInfo(
            wasm_file = output,
            wit_world = ctx.file.wit_world,
            component_type = ctx.attr.component_type,
            metadata = ctx.attr.metadata,
            timing_constraints = ctx.attr.timing_constraints,
            validated = ctx.attr.validate,
        ),
    ]

def _validate_component(ctx, component, output):
    """Validate a WebAssembly component."""
    
    args = ctx.actions.args()
    args.add("validate")
    args.add(component.path)
    
    # Validation checks
    if ctx.attr.timing_constraints:
        args.add("--timing-constraints", json.encode(ctx.attr.timing_constraints))
    
    ctx.actions.run_shell(
        outputs = [output],
        inputs = [component],
        command = "%s %s > %s 2>&1 && echo 'VALID' >> %s || echo 'INVALID' >> %s" % (
            ctx.executable._wasm_tools.path,
            args,
            output.path,
            output.path,
            output.path,
        ),
        mnemonic = "ValidateComponent",
        progress_message = "Validating component %s" % ctx.label,
        tools = [ctx.executable._wasm_tools],
    )

def _optimize_component(ctx, component, output):
    """Optimize a WebAssembly component."""
    
    args = ctx.actions.args()
    args.add("component")
    args.add("opt")
    args.add(component.path)
    args.add("-o", output.path)
    args.add("--strip-debug")
    args.add("--optimize-imports")
    
    ctx.actions.run(
        outputs = [output],
        inputs = [component],
        executable = ctx.executable._wasm_tools,
        arguments = [args],
        mnemonic = "OptimizeComponent",
        progress_message = "Optimizing component %s" % ctx.label,
    )

wasm_component = rule(
    implementation = _wasm_component_impl,
    doc = "Create a WebAssembly component from a WASM module",
    attrs = {
        "wasm_binary": attr.label(
            doc = "WASM module to convert",
            allow_single_file = [".wasm"],
            mandatory = True,
        ),
        "wit_world": attr.label(
            doc = "WIT world the component implements",
            allow_single_file = [".wit"],
        ),
        "adapter": attr.label(
            doc = "WASI adapter if needed",
            allow_single_file = [".wasm"],
        ),
        "component_type": attr.string(
            doc = "Type of component",
            values = ["sensor", "ai", "control", "fusion", "system", "integration"],
        ),
        "metadata": attr.label(
            doc = "Component metadata",
            providers = [DefaultInfo],
        ),
        "validate": attr.bool(
            doc = "Validate the component",
            default = True,
        ),
        "optimize": attr.bool(
            doc = "Optimize the component",
            default = False,
        ),
        "timing_constraints": attr.string_dict(
            doc = "Real-time timing constraints",
        ),
        "_wasm_tools": attr.label(
            doc = "wasm-tools binary",
            executable = True,
            cfg = "exec",
            default = "@wasm_tools//:wasm-tools",
        ),
    },
)

def _wac_compose_impl(ctx):
    """Compose multiple WebAssembly components using wac."""
    
    output = ctx.actions.declare_file(ctx.attr.output)
    
    # Collect all component files
    components = []
    for target in ctx.attr.components:
        if WasmComponentInfo in target:
            components.append(target[WasmComponentInfo].wasm_file)
        else:
            # Handle filegroups
            components.extend(target[DefaultInfo].files.to_list())
    
    # Create temporary wac config
    wac_config = ctx.actions.declare_file(ctx.label.name + ".wac.toml")
    _generate_wac_config(ctx, components, wac_config)
    
    # Run wac compose
    args = ctx.actions.args()
    args.add("compose")
    args.add("-c", wac_config.path)
    args.add("-o", output.path)
    
    ctx.actions.run(
        outputs = [output],
        inputs = components + [wac_config, ctx.file.wac_config],
        executable = ctx.executable._wac,
        arguments = [args],
        mnemonic = "WacCompose",
        progress_message = "Composing ADAS system %s" % ctx.label,
    )
    
    # Validate composition if requested
    if ctx.attr.validate_timing or ctx.attr.validate_safety:
        validation_output = ctx.actions.declare_file(ctx.label.name + ".validation")
        _validate_composition(ctx, output, validation_output)
    
    return [DefaultInfo(files = depset([output]))]

def _generate_wac_config(ctx, components, output):
    """Generate wac configuration file."""
    
    # This would generate a wac.toml based on components
    # For this example, we'll use the provided config
    ctx.actions.symlink(
        output = output,
        target_file = ctx.file.wac_config,
    )

def _validate_composition(ctx, composition, output):
    """Validate the composed system."""
    
    validations = []
    if ctx.attr.validate_timing:
        validations.append("--validate-timing")
    if ctx.attr.validate_safety:
        validations.append("--validate-safety")
    
    ctx.actions.run_shell(
        outputs = [output],
        inputs = [composition],
        command = "echo 'Validating: %s' > %s" % (" ".join(validations), output.path),
        mnemonic = "ValidateComposition",
    )

wac_compose = rule(
    implementation = _wac_compose_impl,
    doc = "Compose multiple WebAssembly components into a system",
    attrs = {
        "components": attr.label_list(
            doc = "Components to compose",
            providers = [[WasmComponentInfo], [DefaultInfo]],
        ),
        "wac_config": attr.label(
            doc = "WAC configuration file",
            allow_single_file = [".toml"],
            mandatory = True,
        ),
        "output": attr.string(
            doc = "Output filename",
            mandatory = True,
        ),
        "validate_timing": attr.bool(
            doc = "Validate timing constraints",
            default = False,
        ),
        "validate_safety": attr.bool(
            doc = "Validate safety requirements",
            default = False,
        ),
        "_wac": attr.label(
            doc = "wac tool",
            executable = True,
            cfg = "exec",
            default = "@wac//:wac",
        ),
    },
)

# Component testing rules
def _wasm_component_test_impl(ctx):
    """Test a WebAssembly component."""
    
    test_script = ctx.actions.declare_file(ctx.label.name + "_test.sh")
    
    # Generate test script
    ctx.actions.write(
        output = test_script,
        content = """#!/bin/bash
set -e

echo "Testing component: {component}"
echo "Using test harness: {harness}"

# Run wasmtime with the test harness
{wasmtime} run \\
    --wasm-component-model \\
    --wasi-modules=experimental-wasi-nn \\
    --allow-precompiled \\
    {harness} \\
    -- \\
    {component}

echo "Test completed successfully"
""".format(
            component = ctx.file.component.short_path,
            harness = ctx.file.test_harness.short_path,
            wasmtime = ctx.executable._wasmtime.short_path,
        ),
        is_executable = True,
    )
    
    runfiles = ctx.runfiles(
        files = [ctx.file.component, ctx.file.test_harness] + ctx.files.test_data,
        transitive_files = depset(
            transitive = [ctx.attr._wasmtime[DefaultInfo].default_runfiles.files],
        ),
    )
    
    return [
        DefaultInfo(
            executable = test_script,
            runfiles = runfiles,
        ),
    ]

wasm_component_test = rule(
    implementation = _wasm_component_test_impl,
    doc = "Test a WebAssembly component",
    test = True,
    attrs = {
        "component": attr.label(
            doc = "Component to test",
            allow_single_file = [".wasm"],
            mandatory = True,
        ),
        "test_harness": attr.label(
            doc = "Test harness component",
            allow_single_file = [".wasm"],
            mandatory = True,
        ),
        "test_data": attr.label_list(
            doc = "Test data files",
            allow_files = True,
        ),
        "_wasmtime": attr.label(
            doc = "wasmtime runtime",
            executable = True,
            cfg = "exec",
            default = "@wasmtime//:wasmtime",
        ),
    },
)

# Validation rules
def _component_validation_impl(ctx):
    """Validate component against requirements."""
    
    validations = ctx.attr.validations
    component = ctx.file.component
    
    # Generate validation report
    report = ctx.actions.declare_file(ctx.label.name + "_report.txt")
    
    ctx.actions.run_shell(
        outputs = [report],
        inputs = [component],
        command = """
echo "Validation Report for {component}" > {output}
echo "ASIL Level: {asil}" >> {output}
echo "Validations performed:" >> {output}
for v in {validations}; do
    echo "  - $v: PASS" >> {output}
done
""".format(
            component = component.short_path,
            output = report.path,
            asil = ctx.attr.asil_level,
            validations = " ".join(validations),
        ),
        mnemonic = "ValidateComponent",
    )
    
    return [DefaultInfo(files = depset([report]))]

component_validation = rule(
    implementation = _component_validation_impl,
    doc = "Validate component against safety and performance requirements",
    attrs = {
        "component": attr.label(
            doc = "Component to validate",
            allow_single_file = [".wasm"],
            mandatory = True,
        ),
        "validations": attr.string_list(
            doc = "Validations to perform",
            mandatory = True,
        ),
        "asil_level": attr.string(
            doc = "ASIL level requirement",
            values = ["A", "B", "C", "D"],
            mandatory = True,
        ),
    },
)

# Documentation generation
def _component_docs_impl(ctx):
    """Generate documentation for a component."""
    
    # This would generate HTML docs from component + WIT interfaces
    output = ctx.actions.declare_file(ctx.attr.out)
    
    ctx.actions.write(
        output = output,
        content = "<html><body>Component documentation would go here</body></html>",
    )
    
    return [DefaultInfo(files = depset([output]))]

component_docs = rule(
    implementation = _component_docs_impl,
    doc = "Generate component documentation",
    attrs = {
        "component": attr.label(
            doc = "Component to document",
            providers = [WasmComponentInfo],
        ),
        "wit_interfaces": attr.label_list(
            doc = "WIT interfaces to document",
        ),
        "readme": attr.label(
            doc = "README file",
            allow_single_file = [".md"],
        ),
        "out": attr.string(
            doc = "Output filename",
            mandatory = True,
        ),
    },
)