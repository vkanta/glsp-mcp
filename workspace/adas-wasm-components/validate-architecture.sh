#!/bin/bash

# ADAS Architecture Validation Script
# Validates the standardized component architecture

set -e

echo "🔍 ADAS Architecture Validation"
echo "==============================="
echo ""

# Check build artifacts
echo "📦 Checking Build Artifacts:"
wasm_count=$(find ./target/wasm32-wasip2/debug -name "*.wasm" -not -path "*/deps/*" 2>/dev/null | wc -l)
if [ "$wasm_count" -eq 13 ]; then
    echo "  ✅ All 13 components built successfully"
else
    echo "  ❌ Expected 13 components, found $wasm_count"
fi

# Check interface implementations
echo ""
echo "🔗 Checking Interface Implementations:"

sensor_count=$(find components/sensors -name "lib.rs" -exec grep -l "sensor_control\|sensor-data" {} \; 2>/dev/null | wc -l)
echo "  🔧 Sensor interfaces: $sensor_count/6 components"

ai_count=$(find components/ai -name "lib.rs" -exec grep -l "ai_control\|perception-data" {} \; 2>/dev/null | wc -l)
echo "  🤖 AI interfaces: $ai_count/2 components"

control_count=$(find components/control -name "lib.rs" -exec grep -l "planning-data" {} \; 2>/dev/null | wc -l)
echo "  🎯 Control interfaces: $control_count/2 components"

system_count=$(find components/system -name "lib.rs" -exec grep -l "health_monitoring\|performance_monitoring" {} \; 2>/dev/null | wc -l)
echo "  🛡️ System interfaces: $system_count/3 components"

# Check WIT structure
echo ""
echo "📋 Checking WIT Structure:"

wit_worlds=$(find wit/components -name "component.wit" 2>/dev/null | wc -l)
echo "  📄 Component worlds defined: $wit_worlds"

wit_interfaces=$(find wit/interfaces -name "*.wit" 2>/dev/null | wc -l)
echo "  🔌 Interface packages: $wit_interfaces"

# Check component dependencies
echo ""
echo "🔧 Checking Component Dependencies:"

deps_ok=0
total_components=0

for component_dir in components/*/* ; do
    if [ -d "$component_dir/wit/deps" ]; then
        total_components=$((total_components + 1))
        if [ -d "$component_dir/wit/deps/adas-common-types" ] && [ -d "$component_dir/wit/deps/adas-diagnostics" ]; then
            deps_ok=$((deps_ok + 1))
        fi
    fi
done

echo "  📦 Components with proper dependencies: $deps_ok/$total_components"

# Architecture summary
echo ""
echo "🏗️ Architecture Summary:"
echo "========================"
echo "  🔧 sensor-component:  6 components"
echo "  🤖 ai-component:      2 components"
echo "  🎯 control-component: 2 components"
echo "  🛡️ system-component:  3 components"
echo "  📊 Total:             13 components"

# Validation result
echo ""
total_checks=5
passed_checks=0

[ "$wasm_count" -eq 13 ] && passed_checks=$((passed_checks + 1))
[ "$sensor_count" -eq 6 ] && passed_checks=$((passed_checks + 1))
[ "$ai_count" -eq 2 ] && passed_checks=$((passed_checks + 1))
[ "$control_count" -eq 2 ] && passed_checks=$((passed_checks + 1))
[ "$system_count" -eq 3 ] && passed_checks=$((passed_checks + 1))

if [ "$passed_checks" -eq "$total_checks" ]; then
    echo "✅ Architecture Validation: PASSED ($passed_checks/$total_checks checks)"
    echo ""
    echo "🎉 Standardized ADAS architecture is fully operational!"
    exit 0
else
    echo "❌ Architecture Validation: FAILED ($passed_checks/$total_checks checks)"
    exit 1
fi