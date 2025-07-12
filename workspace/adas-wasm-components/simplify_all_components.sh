#!/bin/bash

# Simplify all component lib.rs files to match simple WIT interface

simplify_component() {
    local component_path=$1
    local binding_name=$2
    local component_display_name=$3
    
    local lib_file="${component_path}/src/lib.rs"
    
    echo "Simplifying $component_display_name at $lib_file"
    
    cat > "$lib_file" << EOF
// ${component_display_name} Component Implementation

// The bindings are generated as a separate crate based on the BUILD target name
use ${binding_name}::Guest;

struct Component;

impl Guest for Component {
    fn process_frame() -> String {
        format!("${component_display_name} - Frame processed")
    }
}

// Export the component using the generated macro with proper path
${binding_name}::export!(Component with_types_in ${binding_name});
EOF
}

# Sensor components
simplify_component "components/sensors/camera-surround" "camera_surround_ecu_bindings" "Camera Surround ECU"
simplify_component "components/sensors/lidar" "lidar_ecu_bindings" "Lidar ECU"
simplify_component "components/sensors/radar-corner" "radar_corner_ecu_bindings" "Radar Corner ECU"
simplify_component "components/sensors/ultrasonic" "ultrasonic_ecu_bindings" "Ultrasonic ECU"

# AI components
simplify_component "components/ai/object-detection" "object_detection_ai_bindings" "Object Detection AI"
simplify_component "components/ai/behavior-prediction" "behavior_prediction_ai_bindings" "Behavior Prediction AI"

# Control components
simplify_component "components/control/planning-decision" "planning_decision_ecu_bindings" "Planning Decision ECU"
simplify_component "components/control/vehicle-control" "vehicle_control_ecu_bindings" "Vehicle Control ECU"

# Fusion components
simplify_component "components/fusion/sensor-fusion" "sensor_fusion_ecu_bindings" "Sensor Fusion ECU"
simplify_component "components/fusion/perception-fusion" "perception_fusion_ecu_bindings" "Perception Fusion ECU"
simplify_component "components/fusion/tracking-prediction" "tracking_prediction_ecu_bindings" "Tracking Prediction ECU"

# System components
simplify_component "components/system/safety-monitor" "safety_monitor_ecu_bindings" "Safety Monitor ECU"
simplify_component "components/system/domain-controller" "domain_controller_ecu_bindings" "Domain Controller ECU"
simplify_component "components/system/can-gateway" "can_gateway_ecu_bindings" "CAN Gateway ECU"
simplify_component "components/system/hmi-interface" "hmi_interface_ecu_bindings" "HMI Interface ECU"
simplify_component "components/system/feo-demo" "feo_demo_ecu_bindings" "FEO Demo ECU"

# Input components
simplify_component "components/input/video-decoder" "video_decoder_ecu_bindings" "Video Decoder ECU"

# Integration components
simplify_component "components/integration/video-ai-pipeline" "video_ai_pipeline_ecu_bindings" "Video AI Pipeline ECU"

# Graphics components (keep complex implementation)
# simplify_component "components/graphics/adas-visualizer" "adas_visualizer_ecu_bindings" "ADAS Visualizer ECU"

# Orchestrator (keep complex implementation)
# simplify_component "components/orchestrator" "adas_orchestrator_ecu_bindings" "ADAS Orchestrator ECU"

echo "All components have been simplified!"