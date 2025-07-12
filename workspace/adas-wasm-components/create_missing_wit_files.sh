#!/bin/bash

# Create simple WIT files for all components missing them

create_wit_file() {
    local component_path=$1
    local component_name=$2
    local package_name=$3
    
    local wit_dir="${component_path}/wit"
    local wit_file="${wit_dir}/world.wit"
    
    if [ ! -f "$wit_file" ]; then
        echo "Creating WIT file for $component_name at $wit_file"
        mkdir -p "$wit_dir"
        
        cat > "$wit_file" << EOF
package ${package_name}@0.1.0;

world ${component_name} {
    export process-frame: func() -> string;
}
EOF
    fi
}

# Sensor components
create_wit_file "components/sensors/camera-surround" "camera-surround" "adas:camera-surround"
create_wit_file "components/sensors/lidar" "lidar" "adas:lidar"
create_wit_file "components/sensors/radar-corner" "radar-corner" "adas:radar-corner"
create_wit_file "components/sensors/ultrasonic" "ultrasonic" "adas:ultrasonic"

# AI components
create_wit_file "components/ai/object-detection" "object-detection" "adas:object-detection"
create_wit_file "components/ai/behavior-prediction" "behavior-prediction" "adas:behavior-prediction"

# Control components
create_wit_file "components/control/planning-decision" "planning-decision" "adas:planning-decision"
create_wit_file "components/control/vehicle-control" "vehicle-control" "adas:vehicle-control"

# Fusion components
create_wit_file "components/fusion/sensor-fusion" "sensor-fusion" "adas:sensor-fusion"
create_wit_file "components/fusion/perception-fusion" "perception-fusion" "adas:perception-fusion"
create_wit_file "components/fusion/tracking-prediction" "tracking-prediction" "adas:tracking-prediction"

# System components
create_wit_file "components/system/safety-monitor" "safety-monitor" "adas:safety-monitor"
create_wit_file "components/system/domain-controller" "domain-controller" "adas:domain-controller"
create_wit_file "components/system/can-gateway" "can-gateway" "adas:can-gateway"
create_wit_file "components/system/hmi-interface" "hmi-interface" "adas:hmi-interface"
create_wit_file "components/system/feo-demo" "feo-demo" "adas:feo-demo"

# Input components
create_wit_file "components/input/video-decoder" "video-decoder" "adas:video-decoder"

# Integration components
create_wit_file "components/integration/video-ai-pipeline" "video-ai-pipeline" "adas:video-ai-pipeline"

echo "All missing WIT files have been created!"