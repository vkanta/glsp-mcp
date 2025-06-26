#!/bin/bash

# ADAS WebAssembly Components Build Script
# This script builds all ADAS components and embeds metadata using wasm-tools

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPONENTS_DIR="${PROJECT_ROOT}/components"
METADATA_DIR="${PROJECT_ROOT}/metadata"
OUTPUT_DIR="${PROJECT_ROOT}/build"
WIT_DIR="${PROJECT_ROOT}/wit"

# Component list - 18 ADAS components
COMPONENTS=(
    # Sensor Layer
    "camera-front-ecu" "camera-surround-ecu" "radar-front-ecu" 
    "radar-corner-ecu" "lidar-ecu" "ultrasonic-ecu"
    
    # AI/ML Processing Layer  
    "object-detection-ai" "tracking-prediction-ai" "computer-vision-ai" 
    "behavior-prediction-ai"
    
    # Fusion & Decision Layer
    "sensor-fusion-ecu" "perception-fusion" "planning-decision" 
    "safety-monitor"
    
    # Control & Communication Layer
    "adas-domain-controller" "vehicle-control-ecu" "can-gateway" 
    "hmi-interface"
)

echo -e "${BLUE}=== ADAS WebAssembly Components Build Script ===${NC}"
echo "Project Root: ${PROJECT_ROOT}"
echo "Components: ${COMPONENTS[*]}"
echo ""

# Check dependencies
echo -e "${YELLOW}Checking dependencies...${NC}"
check_dependency() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}Error: $1 is not installed${NC}"
        exit 1
    fi
}

check_dependency "cargo"
check_dependency "wasm-tools"

# Check if wasm32-wasip2 target is installed
if ! rustup target list --installed | grep -q wasm32-wasip2; then
    echo -e "${YELLOW}Installing wasm32-wasip2 target...${NC}"
    rustup target add wasm32-wasip2
fi

echo -e "${GREEN}All dependencies are available${NC}"
echo ""

# Create output directory
mkdir -p "${OUTPUT_DIR}"

# Function to build a single component
build_component() {
    local component_name="$1"
    local component_dir="${COMPONENTS_DIR}/${component_name}"
    local metadata_file="${METADATA_DIR}/${component_name}.toml"
    local output_file="${OUTPUT_DIR}/${component_name}.wasm"
    
    echo -e "${BLUE}Building component: ${component_name}${NC}"
    
    # Check if component directory exists
    if [ ! -d "${component_dir}" ]; then
        echo -e "${YELLOW}Warning: Component directory ${component_dir} does not exist, creating stub...${NC}"
        create_component_stub "${component_name}"
    fi
    
    # Check if metadata file exists
    if [ ! -f "${metadata_file}" ]; then
        echo -e "${RED}Error: Metadata file ${metadata_file} not found${NC}"
        return 1
    fi
    
    # Build the component using standard Rust compilation
    cd "${component_dir}"
    echo "  - Compiling Rust component to WASM..."
    cargo build --target wasm32-wasip2 --release
    
    # Find the generated wasm file
    local wasm_file=$(find target/wasm32-wasip2/release -name "*.wasm" -type f | head -n 1)
    if [ -z "${wasm_file}" ]; then
        echo -e "${RED}Error: No WASM file generated for ${component_name}${NC}"
        return 1
    fi
    
    # Check if the WASM file is already a component (wasm32-wasip2 generates components)
    if wasm-tools component wit "${wasm_file}" &>/dev/null; then
        echo "  - WASM file is already a component, copying directly..."
        cp "${wasm_file}" "${output_file}"
    else
        # Create temporary file for component creation process
        local temp_embedded="${output_file}.embedded.wasm"
        
        # Embed WIT metadata using wasm-tools
        echo "  - Embedding WIT metadata..."
        embed_wit_metadata "${wasm_file}" "${temp_embedded}" "${component_name}"
        
        # Create component from embedded WASM
        echo "  - Creating WASM component..."
        create_component "${temp_embedded}" "${output_file}"
        
        # Clean up temporary file
        rm -f "${temp_embedded}"
    fi
    
    # Validate the component
    echo "  - Validating component..."
    validate_component "${output_file}"
    
    echo -e "${GREEN}  ✓ Component ${component_name} built successfully${NC}"
    echo ""
}

# Function to create a component stub if it doesn't exist
create_component_stub() {
    local component_name="$1"
    local component_dir="${COMPONENTS_DIR}/${component_name}"
    
    mkdir -p "${component_dir}/src"
    
    # Create basic Cargo.toml for wasm32 target
    cat > "${component_dir}/Cargo.toml" << EOF
[package]
name = "adas-${component_name}"
version = "0.1.0"
edition = "2021"
description = "${component_name} component for ADAS systems"
license = "Apache-2.0"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.33"

# Configuration for building WASM components
[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"
EOF

    # Create basic lib.rs stub
    cat > "${component_dir}/src/lib.rs" << EOF
// ADAS ${component_name} component implementation
// This is a placeholder stub for the ${component_name} component

use wit_bindgen::generate;

// Generate bindings for the component world
generate!({
    world: "adas-system",
    path: "../../wit"
});

// Component implementation
struct Component;

// TODO: Implement the actual ${component_name} functionality
impl Guest for Component {
    // Add implementation methods here based on the WIT interface
}

// Export the component
export!(Component);
EOF
    
    echo -e "${GREEN}Created stub for ${component_name} component${NC}"
}

# Function to embed WIT metadata into WASM using wasm-tools
embed_wit_metadata() {
    local wasm_file="$1"
    local output_file="$2"
    local component_name="$3"
    local wit_file="${WIT_DIR}/${component_name}.wit"
    
    # Check if specific WIT file exists, try standalone version, otherwise use world.wit
    if [ ! -f "${wit_file}" ]; then
        local standalone_wit="${WIT_DIR}/${component_name}-standalone.wit"
        if [ -f "${standalone_wit}" ]; then
            wit_file="${standalone_wit}"
            echo "    Using ${component_name}-standalone.wit"
        else
            wit_file="${WIT_DIR}/world.wit"
            echo "    Using world.wit as ${component_name}.wit not found"
        fi
    fi
    
    # Embed WIT metadata using wasm-tools
    if wasm-tools component embed "${wit_file}" "${wasm_file}" -o "${output_file}"; then
        echo "    ✓ WIT metadata embedded successfully"
    else
        echo -e "${RED}    ✗ Failed to embed WIT metadata${NC}"
        return 1
    fi
}

# Function to create component from embedded WASM
create_component() {
    local embedded_wasm="$1"
    local output_file="$2"
    
    # Create component using wasm-tools
    if wasm-tools component new "${embedded_wasm}" -o "${output_file}"; then
        echo "    ✓ Component created successfully"
    else
        echo -e "${RED}    ✗ Failed to create component${NC}"
        return 1
    fi
}

# Function to validate component
validate_component() {
    local wasm_file="$1"
    
    # Validate using wasm-tools
    if wasm-tools validate "${wasm_file}"; then
        echo "    ✓ WASM validation passed"
    else
        echo -e "${RED}    ✗ WASM validation failed${NC}"
        return 1
    fi
    
    # Check if it's a component and display WIT interface
    if wasm-tools component wit "${wasm_file}" &>/dev/null; then
        echo "    ✓ Component format validated"
        
        # Try to display the WIT interface (truncated for readability)
        echo "    WIT Interface:"
        wasm-tools component wit "${wasm_file}" 2>/dev/null | head -10 | sed 's/^/      /'
        if [ ${PIPESTATUS[0]} -eq 0 ]; then
            local wit_lines=$(wasm-tools component wit "${wasm_file}" 2>/dev/null | wc -l)
            if [ $wit_lines -gt 10 ]; then
                echo "      ... (${wit_lines} total lines)"
            fi
        fi
    else
        echo -e "${YELLOW}    ! Not a WebAssembly component (core module)${NC}"
        return 1
    fi
    
    # Display component info
    local size=$(ls -lh "${wasm_file}" | awk '{print $5}')
    echo "    Component size: ${size}"
}

# Function to build all components
build_all_components() {
    echo -e "${BLUE}Building all ADAS components...${NC}"
    echo ""
    
    local success_count=0
    local total_count=${#COMPONENTS[@]}
    
    for component in "${COMPONENTS[@]}"; do
        if build_component "${component}"; then
            ((success_count++))
        else
            echo -e "${RED}Failed to build ${component}${NC}"
        fi
    done
    
    echo -e "${BLUE}=== Build Summary ===${NC}"
    echo "Successfully built: ${success_count}/${total_count} components"
    
    if [ ${success_count} -eq ${total_count} ]; then
        echo -e "${GREEN}All components built successfully!${NC}"
        list_built_components
    else
        echo -e "${YELLOW}Some components failed to build${NC}"
        return 1
    fi
}

# Function to list built components
list_built_components() {
    echo ""
    echo -e "${BLUE}Built components:${NC}"
    for file in "${OUTPUT_DIR}"/*.wasm; do
        if [ -f "${file}" ]; then
            local basename=$(basename "${file}")
            local size=$(ls -lh "${file}" | awk '{print $5}')
            echo "  - ${basename} (${size})"
            
            # Check for metadata file
            if [ -f "${file}.metadata.json" ]; then
                echo "    + metadata: ${basename}.metadata.json"
            fi
        fi
    done
}

# Function to clean build artifacts
clean_build() {
    echo -e "${YELLOW}Cleaning build artifacts...${NC}"
    rm -rf "${OUTPUT_DIR}"
    
    for component in "${COMPONENTS[@]}"; do
        local component_dir="${COMPONENTS_DIR}/${component}"
        if [ -d "${component_dir}/target" ]; then
            echo "  Cleaning ${component} target directory..."
            rm -rf "${component_dir}/target"
        fi
    done
    
    echo -e "${GREEN}Build artifacts cleaned${NC}"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [COMMAND] [COMPONENT]"
    echo ""
    echo "Commands:"
    echo "  build [component]  - Build specific component or all components"
    echo "  clean             - Clean build artifacts"
    echo "  list              - List available components"
    echo "  validate          - Validate WIT interfaces"
    echo "  help              - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 build                 # Build all components"
    echo "  $0 build localization    # Build only localization component"
    echo "  $0 clean                 # Clean all build artifacts"
}

# Function to validate WIT interfaces
validate_wit_interfaces() {
    echo -e "${BLUE}Validating WIT interfaces...${NC}"
    
    for wit_file in "${WIT_DIR}"/*.wit; do
        if [ -f "${wit_file}" ]; then
            local filename=$(basename "${wit_file}")
            echo -n "  Validating ${filename}... "
            
            # Basic syntax validation (check if wasm-tools can parse it)
            if wasm-tools component wit "${wit_file}" --dry-run &>/dev/null; then
                echo -e "${GREEN}✓${NC}"
            else
                echo -e "${RED}✗${NC}"
                echo -e "${RED}    Error in ${filename}${NC}"
            fi
        fi
    done
}

# Main script logic
case "${1:-build}" in
    "build")
        if [ -n "$2" ]; then
            # Build specific component
            if [[ " ${COMPONENTS[*]} " =~ " $2 " ]]; then
                build_component "$2"
            else
                echo -e "${RED}Error: Unknown component '$2'${NC}"
                echo "Available components: ${COMPONENTS[*]}"
                exit 1
            fi
        else
            # Build all components
            build_all_components
        fi
        ;;
    "clean")
        clean_build
        ;;
    "list")
        echo "Available components:"
        for component in "${COMPONENTS[@]}"; do
            echo "  - ${component}"
        done
        ;;
    "validate")
        validate_wit_interfaces
        ;;
    "help"|"--help"|"-h")
        show_usage
        ;;
    *)
        echo -e "${RED}Error: Unknown command '$1'${NC}"
        show_usage
        exit 1
        ;;
esac