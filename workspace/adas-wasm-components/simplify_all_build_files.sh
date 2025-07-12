#!/bin/bash

# Simplify all BUILD.bazel files to remove complex dependencies

simplify_build_file() {
    local build_file=$1
    local component_name=$2
    local target_name=$3
    local package_name=$4
    
    echo "Simplifying BUILD file: $build_file"
    
    # Read the existing file to extract the profiles if they exist
    local profiles="[\"debug\", \"release\"]"
    if grep -q "profiles = " "$build_file" 2>/dev/null; then
        profiles=$(grep "profiles = " "$build_file" | head -1 | sed 's/.*profiles = //')
    fi
    
    cat > "$build_file" << EOF
"""${component_name} Component - Bazel Build"""

load("@rules_wasm_component//wit:defs.bzl", "wit_library")
load("@rules_wasm_component//rust:defs.bzl", "rust_wasm_component_bindgen", "rust_wasm_component_test")

package(default_visibility = ["//visibility:public"])

# WIT interfaces for component
wit_library(
    name = "${target_name}_interfaces",
    srcs = ["wit/world.wit"],
    package_name = "${package_name}",
)

# Build component
rust_wasm_component_bindgen(
    name = "${target_name}",
    srcs = ["src/lib.rs"],
    wit = ":${target_name}_interfaces",
    profiles = ${profiles},
)

# Test the component
rust_wasm_component_test(
    name = "${target_name}_test",
    component = ":${target_name}",
)
EOF
}

# Sensor components
simplify_build_file "components/sensors/camera-surround/BUILD.bazel" "Camera Surround ECU" "camera_surround_ecu" "adas:camera-surround"
simplify_build_file "components/sensors/lidar/BUILD.bazel" "Lidar ECU" "lidar_ecu" "adas:lidar"
simplify_build_file "components/sensors/radar-corner/BUILD.bazel" "Radar Corner ECU" "radar_corner_ecu" "adas:radar-corner"
simplify_build_file "components/sensors/ultrasonic/BUILD.bazel" "Ultrasonic ECU" "ultrasonic_ecu" "adas:ultrasonic"

# AI components
simplify_build_file "components/ai/object-detection/BUILD.bazel" "Object Detection AI" "object_detection_ai" "adas:object-detection"
simplify_build_file "components/ai/behavior-prediction/BUILD.bazel" "Behavior Prediction AI" "behavior_prediction_ai" "adas:behavior-prediction"

# Control components
simplify_build_file "components/control/planning-decision/BUILD.bazel" "Planning Decision" "planning_decision_ecu" "adas:planning-decision"
simplify_build_file "components/control/vehicle-control/BUILD.bazel" "Vehicle Control" "vehicle_control_ecu" "adas:vehicle-control"

# Fusion components
simplify_build_file "components/fusion/sensor-fusion/BUILD.bazel" "Sensor Fusion" "sensor_fusion_ecu" "adas:sensor-fusion"
simplify_build_file "components/fusion/perception-fusion/BUILD.bazel" "Perception Fusion" "perception_fusion_ecu" "adas:perception-fusion"
simplify_build_file "components/fusion/tracking-prediction/BUILD.bazel" "Tracking Prediction" "tracking_prediction_ecu" "adas:tracking-prediction"

# System components
simplify_build_file "components/system/safety-monitor/BUILD.bazel" "Safety Monitor" "safety_monitor_ecu" "adas:safety-monitor"
simplify_build_file "components/system/domain-controller/BUILD.bazel" "Domain Controller" "domain_controller_ecu" "adas:domain-controller"
simplify_build_file "components/system/can-gateway/BUILD.bazel" "CAN Gateway" "can_gateway_ecu" "adas:can-gateway"
simplify_build_file "components/system/hmi-interface/BUILD.bazel" "HMI Interface" "hmi_interface_ecu" "adas:hmi-interface"
simplify_build_file "components/system/feo-demo/BUILD.bazel" "FEO Demo" "feo_demo_ecu" "adas:feo-demo"

# Input components
simplify_build_file "components/input/video-decoder/BUILD.bazel" "Video Decoder" "video_decoder_ecu" "adas:video-decoder"

# Integration components
simplify_build_file "components/integration/video-ai-pipeline/BUILD.bazel" "Video AI Pipeline" "video_ai_pipeline_ecu" "adas:video-ai-pipeline"

echo "All BUILD files have been simplified!"