#!/bin/bash

# ADAS WebAssembly Components Build Script
# Uses wasm-tools for component generation

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Component list - actual components that exist
COMPONENTS=(
    # Sensor Layer (ECUs)
    "camera-front-ecu"
    "camera-surround-ecu"
    "radar-front-ecu"
    "radar-corner-ecu"
    "lidar-ecu"
    "ultrasonic-ecu"
    
    # AI Processing Layer
    "object-detection-ai"
    "behavior-prediction-ai"
    
    # Fusion & Perception Layer
    "sensor-fusion-ecu"
    "perception-fusion"
    "tracking-prediction"
    
    # Planning & Control Layer
    "planning-decision"
    "vehicle-control-ecu"
    
    # Safety & Infrastructure Layer
    "safety-monitor"
    "can-gateway"
    "hmi-interface"
    "adas-domain-controller"
)

# Build directory
BUILD_DIR="build"
mkdir -p "$BUILD_DIR"

# Function to build a component
build_component() {
    local component=$1
    echo -e "${BLUE}Building component: $component${NC}"
    
    # Component directory
    COMP_DIR="components/$component"
    
    if [ ! -d "$COMP_DIR" ]; then
        echo -e "${RED}Error: Component directory $COMP_DIR not found${NC}"
        return 1
    fi
    
    # Build with cargo (targeting wasm32-wasip2)
    echo "  - Compiling Rust component to WASM..."
    cd "$COMP_DIR"
    
    if ! cargo build --target wasm32-wasip2 --release 2>&1; then
        echo -e "${RED}Error: Failed to compile $component${NC}"
        cd ../..
        return 1
    fi
    
    cd ../..
    
    # Find the generated WASM file
    WASM_FILE="target/wasm32-wasip2/release/${component//-/_}.wasm"
    
    if [ ! -f "$WASM_FILE" ]; then
        echo -e "${RED}Error: No WASM file generated for $component${NC}"
        return 1
    fi
    
    # Check if WASM is already a component (wasm32-wasip2 generates components)
    echo "  - Checking WASM format..."
    if wasm-tools validate "$WASM_FILE" 2>&1 | grep -q "component"; then
        echo "  - WASM file is already a component, copying directly..."
        cp "$WASM_FILE" "$BUILD_DIR/$component.wasm"
    else
        echo "  - Converting to component..."
        wasm-tools component new "$WASM_FILE" -o "$BUILD_DIR/$component.wasm"
    fi
    
    # Validate the component
    echo "  - Validating component..."
    if wasm-tools validate "$BUILD_DIR/$component.wasm" 2>&1 | grep -q "component"; then
        echo "    ✓ WASM validation passed"
    else
        echo -e "${RED}    ✗ WASM validation failed${NC}"
        return 1
    fi
    
    # Check component format
    if wasm-tools component wit "$BUILD_DIR/$component.wasm" > /dev/null 2>&1; then
        echo "    ✓ Component format validated"
        
        # Show component info
        echo "    WIT Interface:"
        wasm-tools component wit "$BUILD_DIR/$component.wasm" 2>/dev/null | head -20 | sed 's/^/      /'
        echo "      ... ($(wasm-tools component wit "$BUILD_DIR/$component.wasm" 2>/dev/null | wc -l) total lines)"
    else
        echo -e "${YELLOW}    ! Could not extract WIT interface${NC}"
    fi
    
    # Show component size
    SIZE=$(du -h "$BUILD_DIR/$component.wasm" | cut -f1)
    echo "    Component size: $SIZE"
    
    echo -e "${GREEN}  ✓ Component $component built successfully${NC}"
    return 0
}

# Main build process
echo "ADAS WebAssembly Components Build System"
echo "========================================"
echo ""

# Check for required tools
echo "Checking build requirements..."
for tool in cargo wasm-tools; do
    if ! command -v $tool &> /dev/null; then
        echo -e "${RED}Error: $tool is not installed${NC}"
        exit 1
    fi
done

# Check for wasm32-wasip2 target
if ! rustup target list --installed | grep -q "wasm32-wasip2"; then
    echo -e "${YELLOW}Installing wasm32-wasip2 target...${NC}"
    rustup target add wasm32-wasip2
fi

echo -e "${GREEN}✓ All requirements satisfied${NC}"
echo ""

# Build all components
SUCCESSFUL=0
FAILED=0
FAILED_COMPONENTS=()

for component in "${COMPONENTS[@]}"; do
    if build_component "$component"; then
        ((SUCCESSFUL++))
    else
        ((FAILED++))
        FAILED_COMPONENTS+=("$component")
        echo -e "${RED}Failed to build $component${NC}"
    fi
    echo ""
done

# Summary
echo -e "${BLUE}=== Build Summary ===${NC}"
echo "Successfully built: $SUCCESSFUL/${#COMPONENTS[@]} components"

if [ $FAILED -gt 0 ]; then
    echo -e "${YELLOW}Failed components:${NC}"
    for comp in "${FAILED_COMPONENTS[@]}"; do
        echo "  - $comp"
    done
    echo -e "${YELLOW}Some components failed to build${NC}"
else
    echo -e "${GREEN}All components built successfully!${NC}"
fi

echo ""
echo "Build artifacts in: $BUILD_DIR/"

# Show component architecture if all built successfully
if [ $FAILED -eq 0 ]; then
    echo ""
    echo -e "${BLUE}Component Architecture:${NC}"
    echo "  Sensors → AI Processing → Fusion → Planning → Control → Actuation"
    echo "  Total: ${#COMPONENTS[@]} components"
fi