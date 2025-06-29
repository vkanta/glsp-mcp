#!/bin/bash

# ADAS Optimized Release Build Script
# Builds optimized release versions of all standardized components

set -e

echo "🚀 Building ADAS Optimized Components"
echo "======================================"

# List of successfully migrated components
COMPONENTS=(
    # Sensor Components (6)
    "adas-camera-front-ecu"
    "adas-camera_surround_ecu" 
    "adas-radar_front_ecu"
    "adas-radar_corner_ecu"
    "adas-lidar_ecu"
    "adas-ultrasonic_ecu"
    
    # AI Components (2)
    "adas-object_detection_ai"
    "adas-behavior_prediction_ai"
    
    # Control Components (2)
    "adas-vehicle_control_ecu"
    "adas-planning_decision"
    
    # System Components (3)
    "adas-safety_monitor"
    "adas-hmi_interface"
    "adas-can_gateway"
)

echo "Building ${#COMPONENTS[@]} components with optimizations..."
echo ""

# Clean previous builds
echo "🧹 Cleaning previous release builds..."
find ./target/wasm32-wasip2/release -name "*.wasm" -not -path "*/deps/*" -delete 2>/dev/null || true

# Build optimized components
echo "🔨 Building optimized components for wasm32-wasip2..."
for component in "${COMPONENTS[@]}"; do
    echo "  Building: $component (release + optimizations)"
    RUSTFLAGS="-C opt-level=s -C lto=fat -C codegen-units=1 -C panic=abort" \
    cargo build --target wasm32-wasip2 --package "$component" --release --quiet
    if [ $? -eq 0 ]; then
        echo "    ✅ Success"
    else
        echo "    ❌ Failed"
    fi
done

echo ""
echo "📊 Size Comparison:"
echo "==================="

# Show size comparison
debug_total=0
release_total=0

echo "Component                          Debug    Release   Reduction"
echo "----------------------------------------------------------------"

for component in "${COMPONENTS[@]}"; do
    # Convert package name to expected wasm file name
    wasm_name=$(echo "$component" | sed 's/-/_/g')
    
    debug_file="./target/wasm32-wasip2/debug/${wasm_name}.wasm"
    release_file="./target/wasm32-wasip2/release/${wasm_name}.wasm"
    
    if [ -f "$debug_file" ] && [ -f "$release_file" ]; then
        debug_size=$(stat -f%z "$debug_file" 2>/dev/null || stat -c%s "$debug_file" 2>/dev/null)
        release_size=$(stat -f%z "$release_file" 2>/dev/null || stat -c%s "$release_file" 2>/dev/null)
        
        debug_mb=$(echo "scale=1; $debug_size / 1048576" | bc)
        release_mb=$(echo "scale=1; $release_size / 1048576" | bc)
        reduction=$(echo "scale=1; ($debug_size - $release_size) * 100 / $debug_size" | bc)
        
        printf "%-30s %6.1fM   %6.1fM     %4.1f%%\n" "${wasm_name}" "$debug_mb" "$release_mb" "$reduction"
        
        debug_total=$((debug_total + debug_size))
        release_total=$((release_total + release_size))
    fi
done

echo "----------------------------------------------------------------"
debug_total_mb=$(echo "scale=1; $debug_total / 1048576" | bc)
release_total_mb=$(echo "scale=1; $release_total / 1048576" | bc)
total_reduction=$(echo "scale=1; ($debug_total - $release_total) * 100 / $debug_total" | bc)

printf "%-30s %6.1fM   %6.1fM     %4.1f%%\n" "TOTAL" "$debug_total_mb" "$release_total_mb" "$total_reduction"

echo ""
echo "🏗️ Optimization Summary:"
echo "========================"
echo "  📦 Components built: ${#COMPONENTS[@]}"
echo "  💾 Total size reduction: ${total_reduction}%"
echo "  ⚡ Optimizations applied:"
echo "    - Size optimization (-C opt-level=s)"
echo "    - Link-time optimization (-C lto=fat)"
echo "    - Single codegen unit (-C codegen-units=1)"
echo "    - Panic abort mode (-C panic=abort)"
echo ""
echo "✨ Optimized components ready for production deployment!"