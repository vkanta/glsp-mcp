#!/bin/bash

# ADAS Standardized Components Build Script
# Builds all successfully migrated components for wasm32-wasip2

set -e

echo "🚗 Building ADAS Standardized Components"
echo "========================================"

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
    
    # Video Processing Components (2)
    "adas-video_decoder"
    "adas-video_ai_pipeline"
)

echo "Building ${#COMPONENTS[@]} standardized components..."
echo ""

# Clean previous builds
echo "🧹 Cleaning previous builds..."
find . -name "*.wasm" -path "*/target/wasm32-wasip2/*" -delete 2>/dev/null || true

# Build all components
echo "🔨 Building components for wasm32-wasip2..."
for component in "${COMPONENTS[@]}"; do
    echo "  Building: $component"
    cargo build --target wasm32-wasip2 --package "$component" --quiet
    if [ $? -eq 0 ]; then
        echo "    ✅ Success"
    else
        echo "    ❌ Failed"
    fi
done

echo ""
echo "📦 Build Results:"
echo "=================="

# Show generated WASM files
wasm_files=$(find ./target/wasm32-wasip2/debug -name "*.wasm" -not -path "*/deps/*" | sort)
count=$(echo "$wasm_files" | wc -l)

echo "Generated $count WASM components:"
echo "$wasm_files" | while read file; do
    size=$(ls -lh "$file" | awk '{print $5}')
    basename=$(basename "$file")
    echo "  📁 $basename ($size)"
done

echo ""
echo "🏗️ Architecture Summary:"
echo "========================"
echo "  🔧 sensor-component:  7 components (includes video-decoder)"
echo "  🤖 ai-component:      2 components" 
echo "  🎯 control-component: 2 components"
echo "  🛡️  system-component:  4 components (includes video-ai-pipeline)"
echo ""
echo "✨ Total: ${#COMPONENTS[@]} standardized components successfully built!"