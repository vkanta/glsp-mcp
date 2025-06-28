#!/bin/bash

# ADAS WebAssembly Components - Cleanup Script
# Removes build artifacts, outdated files, and dangling directories

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🧹 ADAS Workspace Cleanup${NC}"
echo -e "${CYAN}=========================${NC}"
echo ""

# Function to safely remove files/directories
safe_remove() {
    local target=$1
    local description=$2
    
    if [ -e "$target" ]; then
        echo -e "${YELLOW}🗑️  Removing $description${NC}"
        rm -rf "$target"
        echo -e "  ${GREEN}✓ Deleted: $target${NC}"
    else
        echo -e "  ${BLUE}ℹ️ Not found: $target${NC}"
    fi
}

# Function to clean cargo build artifacts
clean_cargo_artifacts() {
    echo -e "${BLUE}🔧 Cleaning Cargo build artifacts...${NC}"
    
    # Find all Cargo.toml files and clean their targets
    local cleaned=0
    while IFS= read -r -d '' cargo_file; do
        local dir=$(dirname "$cargo_file")
        if [ -d "$dir/target" ]; then
            echo -e "  Cleaning: $dir"
            (cd "$dir" && cargo clean --quiet)
            ((cleaned++))
        fi
    done < <(find . -name "Cargo.toml" -print0)
    
    echo -e "  ${GREEN}✓ Cleaned $cleaned component build caches${NC}"
}

# Function to remove dangling WASM files
clean_dangling_wasm() {
    echo -e "${BLUE}🗂️ Removing dangling WASM files...${NC}"
    
    # Remove small test/dummy WASM files
    safe_remove "polling-test.wasm" "empty test file"
    safe_remove "new-component.wasm" "test component file"
    safe_remove "test/test.wasm" "test directory WASM file"
    safe_remove "test/" "test directory"
}

# Function to remove outdated build outputs
clean_outdated_outputs() {
    echo -e "${BLUE}📦 Removing outdated build outputs...${NC}"
    
    safe_remove "wasm-outputs/" "old wasm-outputs directory"
    safe_remove "build/" "old build directory"
    safe_remove "dist/" "current dist directory"
}

# Function to remove duplicate documentation
clean_duplicate_docs() {
    echo -e "${BLUE}📄 Removing duplicate/outdated documentation...${NC}"
    
    # Keep only docs/ directory, remove duplicates in root
    safe_remove "ADAS_ARCHITECTURE.md" "duplicate architecture doc"
    safe_remove "BUILD_STATUS.md" "outdated build status"
    safe_remove "FINAL_REPORT.md" "outdated final report"
}

# Function to remove unused metadata
clean_unused_metadata() {
    echo -e "${BLUE}🏷️ Removing unused metadata files...${NC}"
    
    safe_remove "metadata/" "unused metadata directory"
}

# Function to remove outdated/unused components
clean_unused_components() {
    echo -e "${BLUE}🔧 Removing unused components...${NC}"
    
    # Remove the complex pipeline orchestrator (replaced by simple FEO demo)
    safe_remove "components/system/pipeline-orchestrator/" "complex pipeline orchestrator"
    
    # Remove old integration component (replaced by FEO approach)
    safe_remove "components/integration/video-ai-pipeline/" "old tightly-coupled integration"
}

# Function to clean duplicate video files
clean_duplicate_videos() {
    echo -e "${BLUE}🎬 Cleaning duplicate video files...${NC}"
    
    # Remove duplicate video in video-decoder component (use main models/ directory)
    safe_remove "components/input/video-decoder/models/" "duplicate video files in component"
}

# Function to remove Rust cache and lock files
clean_rust_cache() {
    echo -e "${BLUE}🦀 Cleaning Rust cache files...${NC}"
    
    # Remove Cargo.lock files from components (keep workspace lock)
    find components -name "Cargo.lock" -delete 2>/dev/null && echo -e "  ${GREEN}✓ Removed component Cargo.lock files${NC}" || true
    
    # Clean global Cargo cache if requested
    if [ "$1" = "--deep" ]; then
        echo -e "${YELLOW}🧽 Deep cleaning Cargo cache...${NC}"
        cargo clean --quiet 2>/dev/null || true
        echo -e "  ${GREEN}✓ Cleaned global Cargo cache${NC}"
    fi
}

# Function to remove old script files
clean_old_scripts() {
    echo -e "${BLUE}📜 Removing outdated scripts...${NC}"
    
    # Remove old/outdated scripts
    safe_remove "scripts/build-components.sh" "outdated build script"
    safe_remove "scripts/clean-all.sh" "outdated clean script"
    safe_remove "scripts/test-wasm-generation.sh" "outdated test script"
    safe_remove "scripts/verify-components.sh" "outdated verify script"
    safe_remove "scripts/compose-adas-system.sh" "outdated compose script"
    safe_remove "scripts/generate-components.sh" "outdated generate script"
    safe_remove "scripts/reorganize-components.sh" "outdated reorganize script"
    
    # Keep useful scripts
    echo -e "  ${BLUE}ℹ️ Keeping: fix-gitignore.sh, show-component-architecture.sh${NC}"
}

# Function to show cleanup summary
show_cleanup_summary() {
    echo ""
    echo -e "${CYAN}📊 Cleanup Summary${NC}"
    echo -e "${CYAN}==================${NC}"
    
    # Calculate remaining component count
    local component_count=$(find components -name "Cargo.toml" | wc -l)
    echo "📦 Active components: $component_count"
    
    # Show remaining important files
    echo "📁 Key directories:"
    for dir in components wit models docs; do
        if [ -d "$dir" ]; then
            echo "  ✓ $dir/"
        fi
    done
    
    echo "🔧 Build scripts:"
    for script in build.sh clean.sh; do
        if [ -f "$script" ]; then
            echo "  ✓ $script"
        fi
    done
    
    echo ""
    echo -e "${GREEN}🎉 Workspace cleanup completed!${NC}"
    echo ""
    echo "🚀 Next steps:"
    echo "  ./build.sh          # Build all components"
    echo "  ./build.sh release  # Build optimized components"
    echo "  ./clean.sh --deep   # Deep clean including Cargo cache"
}

# Main cleanup execution
main() {
    # Parse arguments
    local deep_clean=false
    if [ "$1" = "--deep" ]; then
        deep_clean=true
        echo -e "${YELLOW}⚠️ Deep cleaning mode enabled${NC}"
        echo ""
    fi
    
    # Confirm destructive operations
    if [ "$1" != "--force" ] && [ "$2" != "--force" ]; then
        echo -e "${YELLOW}⚠️ This will remove build artifacts and outdated files${NC}"
        echo "Continue? (y/N)"
        read -r response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            echo "Cleanup cancelled"
            exit 0
        fi
        echo ""
    fi
    
    # Perform cleanup operations
    clean_dangling_wasm
    clean_outdated_outputs
    clean_duplicate_docs
    clean_unused_metadata
    clean_unused_components
    clean_duplicate_videos
    clean_old_scripts
    
    if [ "$deep_clean" = true ]; then
        clean_rust_cache --deep
    else
        clean_rust_cache
    fi
    
    clean_cargo_artifacts
    
    show_cleanup_summary
}

# Show help if requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "ADAS Workspace Cleanup Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --deep    Deep clean including Cargo cache"
    echo "  --force   Skip confirmation prompt"
    echo "  --help    Show this help message"
    echo ""
    echo "What gets cleaned:"
    echo "  ✓ Build artifacts (target/ directories)"
    echo "  ✓ Dangling WASM files"
    echo "  ✓ Outdated documentation"
    echo "  ✓ Unused metadata"
    echo "  ✓ Obsolete components"
    echo "  ✓ Old scripts"
    echo "  ✓ Rust cache (with --deep)"
    echo ""
    echo "What gets preserved:"
    echo "  ✓ Source code (components/)"
    echo "  ✓ WIT interfaces (wit/)"
    echo "  ✓ Models (models/)"
    echo "  ✓ Documentation (docs/)"
    echo "  ✓ Build scripts (build.sh, clean.sh)"
    exit 0
fi

# Run main function
main "$@"