#!/bin/bash

# ADAS WebAssembly Components - Modern Build System
# Builds all components with FEO (Fixed Execution Order) support

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Build configuration
TARGET="wasm32-wasip2"
BUILD_MODE="${1:-debug}"  # debug or release
OUTPUT_DIR="dist"
PARALLEL_JOBS=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

echo -e "${CYAN}🚗 ADAS WebAssembly Components Build System${NC}"
echo -e "${CYAN}==============================================${NC}"
echo "Build mode: $BUILD_MODE"
echo "Target: $TARGET"
echo "Parallel jobs: $PARALLEL_JOBS"
echo "Output directory: $OUTPUT_DIR"
echo ""

# Function to find all component directories
find_components() {
    find components -name "Cargo.toml" -exec dirname {} \; | sort
}

# Function to build a single component
build_component() {
    local comp_path=$1
    local comp_name=$(basename "$comp_path")
    local category=$(basename "$(dirname "$comp_path")")
    local original_dir=$(pwd)
    
    echo -e "${BLUE}🔧 Building $category/$comp_name${NC}"
    
    if ! cd "$comp_path" 2>/dev/null; then
        echo -e "  ${RED}✗ Component directory not found: $comp_path${NC}"
        return 1
    fi
    
    # Build with cargo
    local build_flags="--target $TARGET"
    if [ "$BUILD_MODE" = "release" ]; then
        build_flags="$build_flags --release"
    fi
    
    if cargo build $build_flags --quiet 2>/dev/null; then
        # Find the generated WASM file
        local wasm_file="target/$TARGET/$BUILD_MODE"
        local wasm_name=""
        
        # Find the actual WASM file (handle name variations)
        if [ -f "$wasm_file/adas_${comp_name//-/_}.wasm" ]; then
            wasm_name="adas_${comp_name//-/_}.wasm"
        elif [ -f "$wasm_file/${comp_name//-/_}.wasm" ]; then
            wasm_name="${comp_name//-/_}.wasm"
        else
            # Find any .wasm file in the directory
            wasm_name=$(find "$wasm_file" -name "*.wasm" -not -path "*/deps/*" | head -1 | xargs basename 2>/dev/null)
        fi
        
        if [ -n "$wasm_name" ] && [ -f "$wasm_file/$wasm_name" ]; then
            # Copy to output directory with clean name
            local output_name="${category}-${comp_name}.wasm"
            mkdir -p "$original_dir/$OUTPUT_DIR"
            cp "$wasm_file/$wasm_name" "$original_dir/$OUTPUT_DIR/$output_name"
            
            # Get file size
            local size=$(du -h "$original_dir/$OUTPUT_DIR/$output_name" | cut -f1)
            echo -e "  ${GREEN}✓ Built successfully${NC} ($size)"
            
            cd "$original_dir"
            return 0
        else
            echo -e "  ${RED}✗ WASM file not found${NC}"
            cd "$original_dir"
            return 1
        fi
    else
        echo -e "  ${RED}✗ Compilation failed${NC}"
        cd "$original_dir"
        return 1
    fi
}

# Function to build components in parallel
build_all_components() {
    local components=($(find_components))
    local total=${#components[@]}
    local success=0
    local failed=0
    local failed_list=()
    
    echo -e "${BLUE}📦 Found $total components to build${NC}"
    echo ""
    
    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    
    # Build components (with limited parallelism to avoid overwhelming the system)
    for comp_path in "${components[@]}"; do
        if build_component "$comp_path"; then
            ((success++))
        else
            ((failed++))
            failed_list+=("$comp_path")
        fi
    done
    
    echo ""
    echo -e "${CYAN}📊 Build Summary${NC}"
    echo -e "${CYAN}================${NC}"
    echo -e "Total components: $total"
    echo -e "${GREEN}✓ Successful: $success${NC}"
    
    if [ $failed -gt 0 ]; then
        echo -e "${RED}✗ Failed: $failed${NC}"
        echo -e "${YELLOW}Failed components:${NC}"
        for comp in "${failed_list[@]}"; do
            echo "  - $comp"
        done
        return 1
    else
        echo -e "${GREEN}🎉 All components built successfully!${NC}"
        return 0
    fi
}

# Function to show component architecture
show_architecture() {
    echo ""
    echo -e "${CYAN}🏗️ Component Architecture${NC}"
    echo -e "${CYAN}========================${NC}"
    echo ""
    echo "📊 Categories:"
    for category in sensors ai fusion control input integration system; do
        if [ -d "components/$category" ]; then
            local count=$(find "components/$category" -name "Cargo.toml" | wc -l)
            echo "  $category: $count components"
        fi
    done
    echo ""
    echo "🔄 Data Flow:"
    echo "  Sensors → AI Processing → Fusion → Planning → Control"
    echo "  Input → Integration → System"
    echo ""
    echo "⚡ FEO (Fixed Execution Order) Components:"
    echo "  - Video Decoder (input/video-decoder)"
    echo "  - Object Detection AI (ai/object-detection)"  
    echo "  - FEO Demo (system/feo-demo)"
}

# Function to validate built components
validate_components() {
    echo ""
    echo -e "${BLUE}🔍 Validating built components...${NC}"
    
    if ! command -v wasm-tools &> /dev/null; then
        echo -e "${YELLOW}⚠️ wasm-tools not found - skipping validation${NC}"
        return 0
    fi
    
    local valid=0
    local invalid=0
    
    for wasm_file in "$OUTPUT_DIR"/*.wasm; do
        if [ -f "$wasm_file" ]; then
            local name=$(basename "$wasm_file")
            if wasm-tools validate "$wasm_file" &>/dev/null; then
                echo -e "  ${GREEN}✓ $name${NC}"
                ((valid++))
            else
                echo -e "  ${RED}✗ $name${NC}"
                ((invalid++))
            fi
        fi
    done
    
    echo ""
    if [ $invalid -eq 0 ]; then
        echo -e "${GREEN}✅ All $valid components are valid WASM${NC}"
    else
        echo -e "${YELLOW}⚠️ $invalid components failed validation${NC}"
    fi
}

# Main execution
main() {
    # Check prerequisites
    echo -e "${BLUE}🔍 Checking prerequisites...${NC}"
    
    for tool in cargo rustc; do
        if ! command -v $tool &> /dev/null; then
            echo -e "${RED}❌ $tool is required but not installed${NC}"
            exit 1
        fi
    done
    
    # Check for wasm32-wasip2 target
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo -e "${YELLOW}📥 Installing $TARGET target...${NC}"
        rustup target add "$TARGET"
    fi
    
    echo -e "${GREEN}✅ Prerequisites satisfied${NC}"
    echo ""
    
    # Build all components
    if build_all_components; then
        validate_components
        show_architecture
        
        echo ""
        echo -e "${GREEN}🚀 Build completed successfully!${NC}"
        echo "📁 Components available in: $OUTPUT_DIR/"
        echo "📊 Total size: $(du -sh $OUTPUT_DIR | cut -f1)"
        echo ""
        echo "🔧 Usage:"
        echo "  ./build.sh           # Debug build"
        echo "  ./build.sh release   # Release build"
        echo "  ./clean.sh           # Clean workspace"
        echo ""
        exit 0
    else
        echo ""
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    fi
}

# Show help if requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "ADAS WebAssembly Components Build System"
    echo ""
    echo "Usage: $0 [MODE]"
    echo ""
    echo "Modes:"
    echo "  debug    Build in debug mode (default)"
    echo "  release  Build in release mode (optimized)"
    echo ""
    echo "Output:"
    echo "  All components are built to: $OUTPUT_DIR/"
    echo ""
    echo "Prerequisites:"
    echo "  - Rust toolchain with cargo"
    echo "  - wasm32-wasip2 target (auto-installed)"
    echo "  - wasm-tools (optional, for validation)"
    exit 0
fi

# Run main function
main