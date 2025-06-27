#!/bin/bash

# ADAS System Composition Script
# Shows how WebAssembly components connect together

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}ADAS WebAssembly Component System${NC}"
echo "=================================="
echo ""

# Function to show component connections
show_connections() {
    echo -e "${GREEN}Component Connection Graph:${NC}"
    echo ""
    
    echo "🎥 SENSOR LAYER (Data Producers)"
    echo "├── camera-front-ecu     [EXPORTS: camera-data]"
    echo "├── camera-rear-ecu      [EXPORTS: camera-data]"
    echo "├── camera-surround-ecu  [EXPORTS: camera-data]"
    echo "├── radar-front-ecu      [EXPORTS: radar-data]"
    echo "├── radar-corner-ecu     [EXPORTS: radar-data]"
    echo "├── lidar-ecu            [EXPORTS: lidar-data]"
    echo "└── ultrasonic-ecu       [EXPORTS: ultrasonic-data]"
    echo ""
    
    echo "🤖 AI PROCESSING LAYER"
    echo "├── object-detection-ai"
    echo "│   ├── IMPORTS: camera-data, wasi-nn"
    echo "│   └── EXPORTS: detection-data"
    echo "├── lane-detection-ai"
    echo "│   ├── IMPORTS: camera-data, wasi-nn"
    echo "│   └── EXPORTS: lane-data"
    echo "└── traffic-sign-recognition-ai"
    echo "    ├── IMPORTS: camera-data, wasi-nn"
    echo "    └── EXPORTS: traffic-sign-data"
    echo ""
    
    echo "🔀 FUSION & PREDICTION LAYER"
    echo "├── sensor-fusion-ecu"
    echo "│   ├── IMPORTS: camera-data, radar-data, lidar-data, detection-data"
    echo "│   └── EXPORTS: fusion-data, object-tracks"
    echo "├── tracking-prediction"
    echo "│   ├── IMPORTS: fusion-data, wasi-nn"
    echo "│   └── EXPORTS: tracked-objects, predictions"
    echo "└── behavior-prediction-ai"
    echo "    ├── IMPORTS: tracked-objects, wasi-nn"
    echo "    └── EXPORTS: behavior-predictions"
    echo ""
    
    echo "📋 PLANNING & CONTROL LAYER"
    echo "├── planning-decision"
    echo "│   ├── IMPORTS: fusion-data, predictions, behavior-predictions"
    echo "│   └── EXPORTS: trajectory-plan, maneuvers"
    echo "└── vehicle-control-ecu"
    echo "    ├── IMPORTS: trajectory-plan, fusion-data"
    echo "    └── EXPORTS: control-commands"
    echo ""
    
    echo "🛡️ SAFETY & INFRASTRUCTURE"
    echo "├── safety-monitor"
    echo "│   ├── IMPORTS: fusion-data, control-commands"
    echo "│   └── EXPORTS: safety-status, override-commands"
    echo "├── can-gateway"
    echo "│   ├── IMPORTS: control-commands, safety-status"
    echo "│   └── EXPORTS: can-messages"
    echo "├── hmi-interface"
    echo "│   ├── IMPORTS: fusion-data, trajectory-plan, safety-status"
    echo "│   └── EXPORTS: display-data, user-feedback"
    echo "└── adas-domain-controller"
    echo "    ├── IMPORTS: all-status-data"
    echo "    └── EXPORTS: system-health, diagnostics"
}

# Function to show data flow
show_data_flow() {
    echo ""
    echo -e "${GREEN}Data Flow Example - Object Detection to Control:${NC}"
    echo ""
    echo "1. Camera captures frame (30 fps)"
    echo "   camera-front-ecu → [camera-data stream]"
    echo ""
    echo "2. AI processes frame (10-30 Hz)"
    echo "   [camera-data] → object-detection-ai → [detection-data]"
    echo ""
    echo "3. Sensor fusion combines all inputs (20 Hz)"
    echo "   [camera + radar + lidar + detections] → sensor-fusion → [fusion-data]"
    echo ""
    echo "4. Tracking & prediction (10 Hz)"
    echo "   [fusion-data] → tracking-prediction → [tracked-objects + predictions]"
    echo ""
    echo "5. Planning creates trajectory (10 Hz)"
    echo "   [predictions + fusion-data] → planning-decision → [trajectory-plan]"
    echo ""
    echo "6. Control executes plan (50 Hz)"
    echo "   [trajectory-plan] → vehicle-control → [control-commands]"
    echo ""
    echo "7. Safety validates & CAN transmits (100 Hz)"
    echo "   [control-commands] → safety-monitor → can-gateway → [CAN bus]"
}

# Function to show composition example
show_composition_example() {
    echo ""
    echo -e "${GREEN}WebAssembly Component Composition Example:${NC}"
    echo ""
    echo "# Using wasm-tools compose (hypothetical example)"
    echo "wasm-tools compose \\"
    echo "  --component camera-front=build/camera-front-ecu.wasm \\"
    echo "  --component detector=build/object-detection-ai.wasm \\"
    echo "  --component fusion=build/sensor-fusion-ecu.wasm \\"
    echo "  --link camera-front:camera-data → detector:camera-data \\"
    echo "  --link detector:detection-data → fusion:detection-data \\"
    echo "  --output adas-perception.wasm"
    echo ""
    echo "# The composed component would:"
    echo "# - Take camera input"
    echo "# - Run AI detection"
    echo "# - Output fused perception data"
}

# Function to show benefits
show_benefits() {
    echo ""
    echo -e "${GREEN}Benefits of This Architecture:${NC}"
    echo ""
    echo "✅ Modular: Each component has single responsibility"
    echo "✅ Testable: Components can be tested independently"
    echo "✅ Scalable: Easy to add/remove sensors or features"
    echo "✅ Safe: Safety monitor can override any command"
    echo "✅ Efficient: Data flows in one direction"
    echo "✅ Standard: Uses WebAssembly Component Model"
    echo "✅ Portable: Runs on any WASM runtime"
}

# Main execution
show_connections
show_data_flow
show_composition_example
show_benefits

echo ""
echo -e "${YELLOW}Note: This is a reference architecture. Actual component"
echo -e "composition requires a WebAssembly runtime with Component Model support.${NC}"
echo ""
echo "See docs/ADAS_ARCHITECTURE.md for detailed documentation."