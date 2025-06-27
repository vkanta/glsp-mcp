# ADAS Component Mapping

## Current Components (18 Total)

### Sensor Layer (6 components)
1. **camera-front-ecu** - Front-facing camera for general perception
2. **camera-surround-ecu** - 360° surround view for parking
3. **radar-front-ecu** - Long-range radar for ACC/AEB
4. **radar-corner-ecu** - Corner radars for blind spot/cross-traffic
5. **lidar-ecu** - High-resolution 3D point cloud
6. **ultrasonic-ecu** - Close-range parking sensors

### AI/Perception Layer (3 components)
7. **object-detection-ai** - Detects vehicles, pedestrians, cyclists
8. **behavior-prediction-ai** - Predicts other vehicle intentions
9. **perception-fusion** - Combines all perception inputs

### Fusion & Tracking Layer (2 components)
10. **sensor-fusion-ecu** - Low-level sensor data fusion
11. **tracking-prediction** - Multi-object tracking with Kalman filters

### Planning & Control Layer (2 components)
12. **planning-decision** - Path planning and behavior decisions
13. **vehicle-control-ecu** - Converts plans to vehicle commands

### Safety & Infrastructure Layer (5 components)
14. **safety-monitor** - Validates all commands, fail-safe
15. **can-gateway** - Interfaces with vehicle CAN bus
16. **hmi-interface** - Driver display and interaction
17. **adas-domain-controller** - System orchestration
18. **(empty component folder)** - To be removed

## Data Flow

```
Sensors → AI Processing → Fusion → Planning → Control → Vehicle
```

### Detailed Flow:
1. **Sensors** produce raw data (camera frames, radar targets, lidar clouds)
2. **AI components** process sensor data (object detection, lane detection)
3. **Fusion** combines all inputs into unified environment model
4. **Planning** decides what to do based on fused perception
5. **Control** executes the plan with vehicle commands
6. **Safety Monitor** validates everything
7. **CAN Gateway** sends commands to vehicle
8. **HMI** shows status to driver

## Component Dependencies

### Object Detection AI
- **Imports**: Camera data from camera ECUs
- **Exports**: Detected objects with bounding boxes

### Sensor Fusion
- **Imports**: All sensor data + AI detections
- **Exports**: Unified 3D environment model

### Planning & Decision
- **Imports**: Fused environment model + predictions
- **Exports**: Trajectory plans and maneuvers

### Vehicle Control
- **Imports**: Trajectory plans
- **Exports**: Steering, throttle, brake commands

### Safety Monitor
- **Imports**: All data for validation
- **Exports**: Safety status and overrides

## Current Status
- 12/18 components building successfully
- 6 components need WIT interface updates
- All components follow single-responsibility principle
- Ready for WebAssembly Component Model composition