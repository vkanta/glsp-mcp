#!/bin/bash
set -e

# ADAS WebAssembly Components - wac Composition Build Script
# This script builds all ADAS components and composes them into a single
# wasmtime-compatible component using WebAssembly Composition (wac)

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TARGET="wasm32-wasip2"
BUILD_MODE="release"
COMPOSE_OUTPUT="target/adas-complete-system.wasm"
TEMP_DIR="target/wac-temp"

# Component lists (compatible with older bash versions)
SENSOR_COMPONENTS="adas_camera_front_ecu adas_camera_surround_ecu adas_radar_front_ecu adas_radar_corner_ecu adas_lidar_ecu adas_ultrasonic_ecu"
AI_COMPONENTS="adas_object_detection_ai adas_behavior_prediction_ai"
FUSION_COMPONENTS="adas_sensor_fusion_ecu adas_perception_fusion adas_tracking_prediction_ai"
CONTROL_COMPONENTS="adas_planning_decision adas_vehicle_control_ecu"
INPUT_COMPONENTS="adas_video_decoder"
INTEGRATION_COMPONENTS="adas_video_ai_pipeline"
SYSTEM_COMPONENTS="adas_safety_monitor adas_can_gateway adas_hmi_interface adas_domain_controller adas_feo_demo"
ORCHESTRATION_COMPONENTS="adas_orchestrator"

ALL_COMPONENTS="$SENSOR_COMPONENTS $AI_COMPONENTS $FUSION_COMPONENTS $CONTROL_COMPONENTS $INPUT_COMPONENTS $INTEGRATION_COMPONENTS $SYSTEM_COMPONENTS $ORCHESTRATION_COMPONENTS"

# Functions
print_header() {
    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE}🏗️  ADAS WebAssembly Components - wac Composition${NC}"
    echo -e "${BLUE}================================================${NC}"
    echo ""
}

print_section() {
    echo -e "${YELLOW}📋 $1${NC}"
    echo "----------------------------------------"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

check_prerequisites() {
    print_section "Checking Prerequisites"
    
    # Check if cargo is installed
    if ! command -v cargo &> /dev/null; then
        print_error "cargo is not installed. Please install Rust toolchain."
    fi
    print_success "cargo found"
    
    # Check if wasm32-wasip2 target is installed
    if ! rustup target list --installed | grep -q "wasm32-wasip2"; then
        print_info "Installing wasm32-wasip2 target..."
        rustup target add wasm32-wasip2
    fi
    print_success "wasm32-wasip2 target available"
    
    # Check if wasm-tools is installed
    if ! command -v wasm-tools &> /dev/null; then
        print_info "Installing wasm-tools..."
        cargo install wasm-tools
    fi
    print_success "wasm-tools found"
    
    # Check if wac is installed
    if ! command -v wac &> /dev/null; then
        print_info "Installing wac (WebAssembly Composition tool)..."
        cargo install wac-cli
    fi
    print_success "wac found"
    
    # Check if wit-bindgen is installed
    if ! command -v wit-bindgen &> /dev/null; then
        print_info "Installing wit-bindgen..."
        cargo install wit-bindgen-cli
    fi
    print_success "wit-bindgen found"
    
    echo ""
}

build_components() {
    print_section "Building Individual Components"
    
    print_info "Building all ADAS components in $BUILD_MODE mode..."
    
    # Build all components
    cargo build --$BUILD_MODE --target $TARGET
    
    if [ $? -eq 0 ]; then
        print_success "All components built successfully"
    else
        print_error "Component build failed"
    fi
    
    echo ""
}

convert_to_components() {
    print_section "Preparing WebAssembly Components"
    
    # Create temporary directory for component preparation
    mkdir -p $TEMP_DIR
    
    local build_dir="target/$TARGET/$BUILD_MODE"
    local prepared_count=0
    
    print_info "Preparing WebAssembly components for composition..."
    
    # Check each component
    for component in $ALL_COMPONENTS; do
        local wasm_file="$build_dir/${component}.wasm"
        local component_file="$TEMP_DIR/${component}-component.wasm"
        
        if [ -f "$wasm_file" ]; then
            print_info "  Checking $component..."
            
            # Check if already a component using wasm-tools
            if wasm-tools component wit "$wasm_file" > /dev/null 2>&1; then
                print_info "    Already a component, copying..."
                cp "$wasm_file" "$component_file"
                print_success "  ✓ $component prepared (already component)"
                prepared_count=$((prepared_count + 1))
            else
                print_info "    Converting core module to component..."
                # Convert to component using wasm-tools
                if wasm-tools component new "$wasm_file" -o "$component_file" 2>/dev/null; then
                    print_success "  ✓ $component converted"
                    prepared_count=$((prepared_count + 1))
                else
                    print_error "  ✗ Failed to convert $component"
                fi
            fi
        else
            print_info "  ⚠ WASM file not found: $wasm_file (component may not be built)"
        fi
    done
    
    print_success "Prepared $prepared_count components for composition"
    echo ""
}

validate_components() {
    print_section "Validating WebAssembly Components"
    
    local validation_count=0
    
    print_info "Validating converted components..."
    
    for component_file in $TEMP_DIR/*-component.wasm; do
        if [ -f "$component_file" ]; then
            local component_name=$(basename "$component_file" -component.wasm)
            
            # Validate component
            if wasm-tools validate "$component_file" 2>/dev/null; then
                print_success "  ✓ $component_name validated"
                ((validation_count++))
            else
                print_error "  ✗ $component_name validation failed"
            fi
        fi
    done
    
    print_success "Validated $validation_count components"
    echo ""
}

compose_system() {
    print_section "Composing ADAS Complete System"
    
    print_info "Using wac to compose all components..."
    
    # Create output directory
    mkdir -p $(dirname "$COMPOSE_OUTPUT")
    
    # Update wac.toml to use temporary component files
    local temp_wac_config="$TEMP_DIR/wac.toml"
    cp wac.toml "$temp_wac_config"
    
    # Update paths in wac.toml to point to converted components
    sed -i.bak "s|target/wasm32-wasip2/release/|$TEMP_DIR/|g" "$temp_wac_config"
    sed -i.bak "s|\.wasm|-component.wasm|g" "$temp_wac_config"
    
    # Compose using wac
    print_info "Running wac compose..."
    wac compose -c "$temp_wac_config" -o "$COMPOSE_OUTPUT"
    
    if [ $? -eq 0 ]; then
        print_success "System composed successfully"
    else
        print_error "System composition failed"
    fi
    
    echo ""
}

validate_composed_system() {
    print_section "Validating Composed System"
    
    if [ ! -f "$COMPOSE_OUTPUT" ]; then
        print_error "Composed system file not found: $COMPOSE_OUTPUT"
    fi
    
    print_info "Validating composed system..."
    
    # Validate the composed component
    if wasm-tools validate "$COMPOSE_OUTPUT"; then
        print_success "Composed system is valid"
    else
        print_error "Composed system validation failed"
    fi
    
    # Get component info
    local file_size=$(du -h "$COMPOSE_OUTPUT" | cut -f1)
    print_info "Composed system size: $file_size"
    
    # Inspect the component
    print_info "Component inspection:"
    wasm-tools component wit "$COMPOSE_OUTPUT" | head -20
    
    echo ""
}

generate_usage_examples() {
    print_section "Generating Usage Examples"
    
    local examples_dir="examples"
    mkdir -p "$examples_dir"
    
    # Create wasmtime usage example
    cat > "$examples_dir/run-with-wasmtime.sh" << 'EOF'
#!/bin/bash
# Run the composed ADAS system with wasmtime

# Install wasmtime with WASI-NN support if not already installed
if ! command -v wasmtime &> /dev/null; then
    echo "Installing wasmtime..."
    curl https://wasmtime.dev/install.sh -sSf | bash
fi

# Set up WASI-NN (requires additional setup for AI components)
export WASMTIME_WASI_NN=1

# Run the composed system
echo "🚀 Running ADAS Complete System..."
wasmtime run \
    --wasi-modules=experimental-wasi-nn \
    --allow-unknown-exports \
    --invoke=init-system \
    target/adas-complete-system.wasm

echo "✅ ADAS system execution completed"
EOF

    chmod +x "$examples_dir/run-with-wasmtime.sh"
    
    # Create Rust host example
    cat > "$examples_dir/rust-host-example.rs" << 'EOF'
use wasmtime::{Config, Engine, Store, Component, Linker};
use wasmtime_wasi::WasiCtx;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure wasmtime with component model support
    let mut config = Config::new();
    config.wasm_component_model(true);
    
    // Create engine and store
    let engine = Engine::new(&config)?;
    let mut store = Store::new(&engine, WasiCtx::new());
    
    // Load the composed ADAS system
    let component_bytes = std::fs::read("target/adas-complete-system.wasm")?;
    let component = Component::new(&engine, component_bytes)?;
    
    // Create linker and instantiate
    let mut linker = Linker::new(&engine);
    let instance = linker.instantiate(&mut store, &component)?;
    
    // Get the complete system interface
    let system_interface = instance.get_typed_func::<(), ()>(&mut store, "init-system")?;
    
    // Initialize and run the system
    println!("🚀 Initializing ADAS Complete System...");
    system_interface.call(&mut store, ())?;
    
    println!("✅ ADAS system initialized successfully");
    
    Ok(())
}
EOF
    
    # Create Cargo.toml for the host example
    cat > "$examples_dir/Cargo.toml" << 'EOF'
[package]
name = "adas-host-example"
version = "0.1.0"
edition = "2021"

[dependencies]
wasmtime = { version = "25.0", features = ["component-model"] }
wasmtime-wasi = "25.0"
tokio = { version = "1.0", features = ["full"] }
EOF
    
    print_success "Usage examples generated in $examples_dir/"
    echo ""
}

cleanup() {
    print_section "Cleaning Up"
    
    # Remove temporary files
    if [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
        print_success "Temporary files cleaned up"
    fi
    
    echo ""
}

print_summary() {
    print_section "Build Summary"
    
    if [ -f "$COMPOSE_OUTPUT" ]; then
        local file_size=$(du -h "$COMPOSE_OUTPUT" | cut -f1)
        print_success "✅ ADAS Complete System built successfully!"
        print_info "📍 Output file: $COMPOSE_OUTPUT"
        print_info "📏 File size: $file_size"
        print_info "🎯 Target: $TARGET"
        print_info "🔧 Build mode: $BUILD_MODE"
        
        echo ""
        echo -e "${GREEN}🎉 Ready to deploy with wasmtime!${NC}"
        echo -e "${BLUE}📖 Usage examples available in examples/ directory${NC}"
        echo ""
        echo -e "${YELLOW}Next steps:${NC}"
        echo "1. Run: ./examples/run-with-wasmtime.sh"
        echo "2. Or use the Rust host example for custom integration"
        echo "3. Test the system with your specific ADAS requirements"
        echo ""
    else
        print_error "Build failed - composed system not found"
    fi
}

# Main execution
main() {
    print_header
    
    # Check if we should run specific steps
    case "${1:-all}" in
        "prereq")
            check_prerequisites
            ;;
        "build")
            build_components
            ;;
        "convert")
            convert_to_components
            ;;
        "compose")
            compose_system
            ;;
        "validate")
            validate_composed_system
            ;;
        "clean")
            cleanup
            ;;
        "all"|"")
            check_prerequisites
            build_components
            convert_to_components
            validate_components
            compose_system
            validate_composed_system
            generate_usage_examples
            cleanup
            print_summary
            ;;
        *)
            echo "Usage: $0 [prereq|build|convert|compose|validate|clean|all]"
            echo ""
            echo "Commands:"
            echo "  prereq   - Check and install prerequisites"
            echo "  build    - Build all components"
            echo "  convert  - Convert to WebAssembly components"
            echo "  compose  - Compose system with wac"
            echo "  validate - Validate composed system"
            echo "  clean    - Clean temporary files"
            echo "  all      - Run complete build pipeline (default)"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"