#!/bin/bash

# Generate all ADAS component Cargo.toml and lib.rs files

set -e

# Component definitions: name:description:world-name
COMPONENT_DEFINITIONS=(
    "camera-surround-ecu:360° surround-view camera system:camera-surround-component"
    "radar-front-ecu:Front long-range radar ECU:radar-front-component" 
    "radar-corner-ecu:Corner radars for blind spot detection:radar-corner-component"
    "lidar-ecu:LiDAR point cloud processing ECU:lidar-component"
    "ultrasonic-ecu:Ultrasonic sensors for parking:ultrasonic-component"
    "object-detection-ai:AI object detection using CNNs:object-detection-ai-component"
    "tracking-prediction-ai:Kalman tracking and prediction:tracking-prediction-ai-component"
    "computer-vision-ai:Computer vision processing:computer-vision-ai-component"
    "behavior-prediction-ai:Behavior prediction AI:behavior-prediction-ai-component"
    "perception-fusion:High-level perception fusion:perception-fusion-component"
    "planning-decision:Path planning and decisions:planning-decision-component"
    "safety-monitor:ISO 26262 safety monitoring:safety-monitor-component"
    "adas-domain-controller:Central ADAS coordinator:adas-domain-controller-component"
    "vehicle-control-ecu:Vehicle actuation control:vehicle-control-ecu-component"
    "can-gateway:CAN/Ethernet communication:can-gateway-component"
    "hmi-interface:Human-machine interface:hmi-interface-component"
)

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPONENTS_DIR="${PROJECT_ROOT}/components"

echo "Generating ADAS component stubs..."

for comp_def in "${COMPONENT_DEFINITIONS[@]}"; do
    IFS=':' read -r name description world <<< "$comp_def"
    
    echo "Creating $name..."
    
    # Create Cargo.toml
    cat > "${COMPONENTS_DIR}/${name}/Cargo.toml" << EOF
[package]
name = "adas-${name//-/_}"
version = "0.1.0"
edition = "2021"
description = "${description}"
license = "Apache-2.0"

[workspace]

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

    # Create lib.rs stub
    interface_name=$(echo "${name//-/_}" | sed 's/_ecu$//' | sed 's/_ai$//')
    
    cat > "${COMPONENTS_DIR}/${name}/src/lib.rs" << EOF
use wit_bindgen::generate;

// Generate bindings for ${name}
generate!({
    world: "${world}",
    path: "../../wit/${name}-standalone.wit"
});

// Component implementation
struct Component;

impl Component {
    fn new() -> Self {
        Component
    }
}

// TODO: Implement proper WIT interface once WIT file is created
// For now, export a basic component
export!(Component);
EOF

done

echo "Generated ${#COMPONENT_DEFINITIONS[@]} component stubs"
echo "Note: WIT interfaces need to be created for complete functionality"