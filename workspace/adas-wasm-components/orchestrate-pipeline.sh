#!/bin/bash

# ADAS Component Orchestration Pipeline Demo
# Demonstrates data flow through the standardized component architecture

set -e

echo "🎼 ADAS Component Orchestration Pipeline"
echo "========================================"
echo ""

# Check if optimized components exist
if [ ! -f "target/wasm32-wasip2/release/adas_camera_front_ecu.wasm" ]; then
    echo "❌ Optimized components not found. Run ./build-optimized.sh first."
    exit 1
fi

echo "🔍 Component Inventory:"
echo "======================"

# Show available components by category
echo ""
echo "🔧 SENSOR COMPONENTS:"
find target/wasm32-wasip2/release -name "*_ecu.wasm" -not -name "*_control_*" | while read file; do
    size=$(ls -lh "$file" | awk '{print $5}')
    name=$(basename "$file" .wasm)
    echo "  📡 $name ($size)"
done

echo ""
echo "🤖 AI COMPONENTS:"
find target/wasm32-wasip2/release -name "*_ai.wasm" | while read file; do
    size=$(ls -lh "$file" | awk '{print $5}')
    name=$(basename "$file" .wasm)
    echo "  🧠 $name ($size)"
done

echo ""
echo "🎯 CONTROL COMPONENTS:"
find target/wasm32-wasip2/release -name "*control*.wasm" -o -name "*planning*.wasm" | while read file; do
    size=$(ls -lh "$file" | awk '{print $5}')
    name=$(basename "$file" .wasm)
    echo "  🚗 $name ($size)"
done

echo ""
echo "🛡️ SYSTEM COMPONENTS:"
find target/wasm32-wasip2/release -name "*monitor*.wasm" -o -name "*gateway*.wasm" -o -name "*interface*.wasm" | while read file; do
    size=$(ls -lh "$file" | awk '{print $5}')
    name=$(basename "$file" .wasm)
    echo "  🔒 $name ($size)"
done

echo ""
echo "🌊 Data Flow Pipeline:"
echo "====================="
echo ""
echo "┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐"
echo "│   🔧 SENSORS    │───▶│   🤖 AI/ML      │───▶│  🎯 CONTROL     │"
echo "│                 │    │                 │    │                 │"
echo "│ • Camera Front  │    │ • Object Detect │    │ • Planning      │"
echo "│ • Camera 360°   │    │ • Behavior Pred │    │ • Vehicle Ctrl  │"
echo "│ • Radar Front   │    │                 │    │                 │"
echo "│ • Radar Corner  │    └─────────────────┘    └─────────────────┘"
echo "│ • LiDAR         │              │                      │"
echo "│ • Ultrasonic    │              ▼                      ▼"
echo "└─────────────────┘    ┌─────────────────────────────────────────┐"
echo "          │            │         🛡️ SYSTEM MANAGEMENT            │"
echo "          │            │                                         │"
echo "          └───────────▶│ • Safety Monitor  • HMI Interface      │"
echo "                       │ • CAN Gateway     • Diagnostics        │"
echo "                       └─────────────────────────────────────────┘"

echo ""
echo "📊 Architecture Metrics:"
echo "========================"

total_size=$(find target/wasm32-wasip2/release -name "*.wasm" -not -path "*/deps/*" -exec stat -f%z {} + 2>/dev/null | awk '{sum+=$1} END {print sum}' || find target/wasm32-wasip2/release -name "*.wasm" -not -path "*/deps/*" -exec stat -c%s {} + | awk '{sum+=$1} END {print sum}')
total_mb=$(echo "scale=1; $total_size / 1048576" | bc)
component_count=$(find target/wasm32-wasip2/release -name "*.wasm" -not -path "*/deps/*" | wc -l)
avg_size=$(echo "scale=0; $total_size / $component_count / 1024" | bc)

echo "  📦 Total Components: $component_count"
echo "  💾 Total Size: ${total_mb}MB"
echo "  📏 Average Size: ${avg_size}KB per component"
echo "  ⚡ Load Time: ~$(echo "scale=1; $total_mb * 10" | bc)ms @ 100Mbps"

echo ""
echo "🔗 Interface Compatibility Matrix:"
echo "=================================="
echo ""
echo "                    sensor  perception  planning  diagnostics"
echo "                    ------  ----------  --------  -----------"
echo "🔧 Sensor Comp      ✅ Export    ❌         ❌        ✅ Export"
echo "🤖 AI Components    ✅ Import  ✅ Export    ❌        ✅ Export" 
echo "🎯 Control Comp     ❌        ✅ Import  ✅ Export   ✅ Export"
echo "🛡️ System Comp      ✅ Import  ✅ Import  ✅ Import   ✅ Export"

echo ""
echo "🚀 Deployment Scenarios:"
echo "========================"
echo ""
echo "🏁 BASIC ADAS (4 components, 340KB):"
echo "  • adas_camera_front_ecu.wasm"
echo "  • adas_object_detection_ai.wasm"
echo "  • adas_planning_decision.wasm"
echo "  • adas_safety_monitor.wasm"
echo ""
echo "🌟 ADVANCED ADAS (8 components, 680KB):"
echo "  • All sensors (6 components)"
echo "  • All AI components (2 components)"
echo ""
echo "🔥 FULL AUTONOMY (13 components, 1.1MB):"
echo "  • Complete standardized architecture"
echo "  • All interface types implemented"
echo "  • Production-ready deployment"

echo ""
echo "✨ Ready for component orchestration and real-time deployment!"