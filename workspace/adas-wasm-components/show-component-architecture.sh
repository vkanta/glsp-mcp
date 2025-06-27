#!/bin/bash

echo "=== 🏗️ ADAS WebAssembly Component Architecture ==="
echo "Demonstrating the component interfaces and data flow..."
echo

echo "📡 SENSOR COMPONENTS (Data Producers)"
echo "======================================"
for component in camera-front-ecu radar-front-ecu lidar-ecu ultrasonic-ecu; do
    echo
    echo "Component: $component"
    echo "Exports:"
    wasm-tools component wit "wasm-components/${component}.wasm" 2>/dev/null | grep -A3 "export.*-data:" | head -4
done

echo
echo
echo "🧠 AI PROCESSING COMPONENTS (Data Consumers & Producers)"
echo "========================================================"
for component in object-detection-ai behavior-prediction-ai; do
    echo
    echo "Component: $component"
    echo "Imports:"
    wasm-tools component wit "wasm-components/${component}.wasm" 2>/dev/null | grep -A2 "import.*-data:" | head -3
    echo "Exports:"
    wasm-tools component wit "wasm-components/${component}.wasm" 2>/dev/null | grep -A2 "export.*-data:" | head -3
done

echo
echo
echo "🔀 FUSION COMPONENTS (Multi-Sensor Integration)"
echo "=============================================="
for component in sensor-fusion-ecu perception-fusion tracking-prediction; do
    echo
    echo "Component: $component"
    echo "Exports:"
    wasm-tools component wit "wasm-components/${component}.wasm" 2>/dev/null | grep -A2 "export.*-data:" | head -3
done

echo
echo
echo "🎯 CONTROL COMPONENTS (Decision & Actuation)"
echo "==========================================="
for component in planning-decision vehicle-control-ecu; do
    echo
    echo "Component: $component"
    echo "Exports:"
    wasm-tools component wit "wasm-components/${component}.wasm" 2>/dev/null | grep -A2 "export" | grep -E "(planning|vehicle|control)" | head -3
done

echo
echo
echo "🛡️ SAFETY & MONITORING"
echo "====================="
echo
echo "Component: safety-monitor"
echo "Exports:"
wasm-tools component wit "wasm-components/safety-monitor.wasm" 2>/dev/null | grep -A2 "export safety" | head -6

echo
echo
echo "🚗 CAN GATEWAY (Vehicle Network Interface)"
echo "========================================="
echo "Component: can-gateway"
echo "Imports:"
wasm-tools component wit "wasm-components/can-gateway.wasm" 2>/dev/null | grep -A2 "import" | grep -v "^--" | head -4
echo "Exports:"
wasm-tools component wit "wasm-components/can-gateway.wasm" 2>/dev/null | grep -A2 "export can" | head -3

echo
echo
echo "📊 DATA FLOW ARCHITECTURE"
echo "========================"
echo "1. Sensors (Camera, Radar, Lidar, Ultrasonic) → Export sensor data"
echo "2. AI Components → Import sensor data, Export detections/predictions"
echo "3. Fusion Systems → Import multiple sources, Export unified model"
echo "4. Planning → Import fusion data, Export trajectory plans"
echo "5. Control → Import plans, Export vehicle commands"
echo "6. CAN Gateway → Import commands, Export to vehicle network"
echo "7. Safety Monitor → Monitors all critical data, Can override"
echo
echo "Total component size: 1.5MB (17 components)"
echo "Average component size: ~88KB"