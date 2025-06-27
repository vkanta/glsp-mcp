#!/bin/bash

echo "=== 🚀 WebAssembly Component Generation Test ==="
echo "Testing conversion from Rust modules to WebAssembly components..."
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
        
        echo -n "Testing $component_name... "
        
        # Build the component
        (cd "$component_dir" && cargo build --target wasm32-wasip2 --release &>/dev/null)
        
        if [ $? -eq 0 ]; then
            # Find the wasm file (convert component name to crate name)
            crate_name="adas_${component_name//-/_}"
            wasm_file="$component_dir/target/wasm32-wasip2/release/${crate_name}.wasm"
            
            if [ -f "$wasm_file" ]; then
                # Try to create WebAssembly component
                wasm-tools component new "$wasm_file" -o "wasm-components/${component_name}.wasm" &>/dev/null
                
                if [ $? -eq 0 ]; then
                    # Verify it's a valid component
                    wasm-tools validate "wasm-components/${component_name}.wasm" --features component-model &>/dev/null
                    
                    if [ $? -eq 0 ]; then
                        # Get component size
                        size=$(ls -lh "wasm-components/${component_name}.wasm" | awk '{print $5}')
                        echo "✅ SUCCESS (Size: $size)"
                        success_count=$((success_count + 1))
                    else
                        echo "❌ FAILED (Invalid component)"
                    fi
                else
                    echo "❌ FAILED (Component generation failed)"
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
echo "=== 📊 Component Generation Summary ==="
echo "Successfully generated: $success_count out of $total_count components"
echo "Success rate: $(echo "scale=1; $success_count * 100 / $total_count" | bc)%"

# List generated components
if [ $success_count -gt 0 ]; then
    echo
    echo "=== 📦 Generated WebAssembly Components ==="
    ls -lh wasm-components/*.wasm 2>/dev/null | awk '{print $9 ": " $5}'
fi

# Show component details for one example
if [ -f "wasm-components/camera-front-ecu.wasm" ]; then
    echo
    echo "=== 🔍 Example Component Analysis: camera-front-ecu ==="
    wasm-tools component wit "wasm-components/camera-front-ecu.wasm" 2>/dev/null | head -20
fi