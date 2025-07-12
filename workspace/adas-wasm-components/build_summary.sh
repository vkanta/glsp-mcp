#!/bin/bash

echo "==== ADAS WASM Components Build Summary ===="
echo ""

# Function to check if component built successfully
check_component() {
    local component_path=$1
    local wasm_file=$(find bazel-out -name "*_release.wasm" -path "*${component_path}*" 2>/dev/null | head -1)
    
    if [ -n "$wasm_file" ] && [ -f "$wasm_file" ]; then
        local size=$(ls -lh "$wasm_file" | awk '{print $5}')
        echo "✅ $component_path - Built successfully ($size)"
    else
        echo "❌ $component_path - Build failed or not attempted"
    fi
}

echo "Test Components:"
check_component "test/simple"

echo -e "\nSensor Components:"
check_component "sensors/camera-front"
check_component "sensors/camera-surround"
check_component "sensors/lidar"
check_component "sensors/radar-front"
check_component "sensors/radar-corner"
check_component "sensors/ultrasonic"

echo -e "\nAI Components:"
check_component "ai/object-detection"
check_component "ai/behavior-prediction"

echo -e "\nControl Components:"
check_component "control/planning-decision"
check_component "control/vehicle-control"

echo -e "\nFusion Components:"
check_component "fusion/sensor-fusion"
check_component "fusion/perception-fusion"
check_component "fusion/tracking-prediction"

echo -e "\nSystem Components:"
check_component "system/safety-monitor"
check_component "system/domain-controller"
check_component "system/can-gateway"
check_component "system/hmi-interface"
check_component "system/feo-demo"

echo -e "\nInput Components:"
check_component "input/video-decoder"

echo -e "\nIntegration Components:"
check_component "integration/video-ai-pipeline"

echo -e "\nGraphics Components:"
check_component "graphics/adas-visualizer"

echo -e "\nOrchestrator:"
check_component "orchestrator"

# Count summary
total=$(find components test -name "*.wasm" -path "*release*" 2>/dev/null | wc -l | xargs)
echo -e "\n==== Summary ===="
echo "Total components built: $total"
echo "Total expected: 24"