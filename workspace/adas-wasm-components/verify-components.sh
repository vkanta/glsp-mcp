#!/bin/bash

echo "=== 🚀 WebAssembly Component Verification ==="
echo "Verifying all components are valid WebAssembly Component Model modules..."
echo

# Create output directory
mkdir -p wasm-components

# Counter for successes
success_count=0
total_count=0

# Test each component
for component_dir in components/*/; do
    if [ -f "$component_dir/Cargo.toml" ]; then
        component_name=$(basename "$component_dir")
        total_count=$((total_count + 1))
        
        echo -n "Verifying $component_name... "
        
        # Build the component in release mode
        (cd "$component_dir" && cargo build --target wasm32-wasip2 --release &>/dev/null)
        
        if [ $? -eq 0 ]; then
            # Find the wasm file
            crate_name="adas_${component_name//-/_}"
            wasm_file="$component_dir/target/wasm32-wasip2/release/${crate_name}.wasm"
            
            if [ -f "$wasm_file" ]; then
                # Validate it's a WebAssembly component
                wasm-tools validate "$wasm_file" --features component-model &>/dev/null
                
                if [ $? -eq 0 ]; then
                    # Copy to output directory
                    cp "$wasm_file" "wasm-components/${component_name}.wasm"
                    
                    # Get component info
                    size=$(ls -lh "$wasm_file" | awk '{print $5}')
                    
                    # Extract export count
                    exports=$(wasm-tools component wit "$wasm_file" 2>/dev/null | grep -E "export|import" | wc -l)
                    
                    echo "✅ SUCCESS (Size: $size, Interfaces: $exports)"
                    success_count=$((success_count + 1))
                else
                    echo "❌ FAILED (Invalid component model)"
                fi
            else
                echo "❌ FAILED (WASM file not found)"
            fi
        else
            echo "❌ FAILED (Build failed)"
        fi
    fi
done

echo
echo "=== 📊 Component Verification Summary ==="
echo "Valid components: $success_count out of $total_count"
echo "Success rate: $(echo "scale=1; $success_count * 100 / $total_count" | bc)%"

# Show detailed analysis of one component
if [ -f "wasm-components/vehicle-control-ecu.wasm" ]; then
    echo
    echo "=== 🔍 Example Component Interface: vehicle-control-ecu ==="
    echo "Exported interfaces:"
    wasm-tools component wit "wasm-components/vehicle-control-ecu.wasm" 2>/dev/null | grep -A5 "export" | head -15
fi

# List all components with sizes
if [ $success_count -gt 0 ]; then
    echo
    echo "=== 📦 WebAssembly Component Sizes ==="
    ls -lhS wasm-components/*.wasm 2>/dev/null | awk '{printf "%-30s %s\n", $9, $5}'
    
    # Total size
    echo
    total_size=$(du -sh wasm-components 2>/dev/null | awk '{print $1}')
    echo "Total size of all components: $total_size"
fi