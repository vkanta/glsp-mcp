# ADAS Component Architecture

## Component Connection Graph

```mermaid
graph TB
    %% Sensor Layer - Data Producers
    CF[Camera Front<br/>EXPORTS: camera-data]
    CR[Camera Rear<br/>EXPORTS: camera-data]
    CS1[Camera Side L<br/>EXPORTS: camera-data]
    CS2[Camera Side R<br/>EXPORTS: camera-data]
    
    RF[Radar Front<br/>EXPORTS: radar-data]
    RC1[Radar Corner FL<br/>EXPORTS: radar-data]
    RC2[Radar Corner FR<br/>EXPORTS: radar-data]
    RC3[Radar Corner RL<br/>EXPORTS: radar-data]
    RC4[Radar Corner RR<br/>EXPORTS: radar-data]
    
    LF[LiDAR Front<br/>EXPORTS: lidar-data]
    
    US[Ultrasonic Array<br/>EXPORTS: ultrasonic-data]
    
    %% AI Processing Layer
    OD[Object Detection AI<br/>IMPORTS: camera-data, wasi-nn<br/>EXPORTS: detection-data]
    LD[Lane Detection AI<br/>IMPORTS: camera-data, wasi-nn<br/>EXPORTS: lane-data]
    TSR[Traffic Sign Recognition<br/>IMPORTS: camera-data, wasi-nn<br/>EXPORTS: traffic-sign-data]
    
    %% Fusion Layer
    SF[Sensor Fusion<br/>IMPORTS: camera-data, radar-data, lidar-data, detection-data<br/>EXPORTS: fusion-data, object-tracks]
    
    %% Prediction Layer
    TP[Tracking & Prediction<br/>IMPORTS: fusion-data, wasi-nn<br/>EXPORTS: tracked-objects, predictions]
    BP[Behavior Prediction<br/>IMPORTS: tracked-objects, wasi-nn<br/>EXPORTS: behavior-predictions]
    
    %% Planning Layer
    PD[Planning & Decision<br/>IMPORTS: fusion-data, predictions, behavior-predictions<br/>EXPORTS: trajectory-plan, maneuvers]
    
    %% Control Layer
    VC[Vehicle Control<br/>IMPORTS: trajectory-plan, fusion-data<br/>EXPORTS: control-commands]
    
    %% Safety Layer
    SM[Safety Monitor<br/>IMPORTS: fusion-data, control-commands<br/>EXPORTS: safety-status, override-commands]
    
    %% Actuation Layer
    CG[CAN Gateway<br/>IMPORTS: control-commands, safety-status<br/>EXPORTS: can-messages]
    
    %% HMI Layer
    HMI[HMI Interface<br/>IMPORTS: fusion-data, trajectory-plan, safety-status<br/>EXPORTS: display-data, user-feedback]
    
    %% Domain Controller
    DC[ADAS Domain Controller<br/>IMPORTS: all-status-data<br/>EXPORTS: system-health, diagnostics]
    
    %% Connections - Sensor to AI
    CF --> OD
    CF --> LD
    CF --> TSR
    CR --> OD
    CS1 --> OD
    CS2 --> OD
    
    %% Connections - Sensors to Fusion
    CF --> SF
    CR --> SF
    CS1 --> SF
    CS2 --> SF
    RF --> SF
    RC1 --> SF
    RC2 --> SF
    RC3 --> SF
    RC4 --> SF
    LF --> SF
    
    %% Connections - AI to Fusion
    OD --> SF
    LD --> SF
    TSR --> SF
    
    %% Connections - Fusion to Prediction
    SF --> TP
    SF --> BP
    TP --> BP
    
    %% Connections - Prediction to Planning
    TP --> PD
    BP --> PD
    SF --> PD
    
    %% Connections - Planning to Control
    PD --> VC
    SF --> VC
    
    %% Connections - Control to Safety
    VC --> SM
    SF --> SM
    
    %% Connections - Safety to CAN
    VC --> CG
    SM --> CG
    
    %% Connections - To HMI
    SF --> HMI
    PD --> HMI
    SM --> HMI
    
    %% Connections - To Domain Controller
    SF --> DC
    SM --> DC
    VC --> DC
```

## Data Flow Architecture

### Layer 1: Sensor Data Production (100Hz)
- **Camera ECUs**: Export raw image frames at 30-60 fps
- **Radar ECUs**: Export point cloud targets at 20 Hz  
- **LiDAR ECU**: Export 3D point clouds at 10-20 Hz
- **Ultrasonic**: Export distance measurements at 40 Hz

### Layer 2: AI Perception (10-30Hz)
- **Object Detection**: Processes camera frames, detects vehicles/pedestrians/cyclists
- **Lane Detection**: Identifies lane markings, road boundaries
- **Traffic Sign Recognition**: Detects and classifies traffic signs/lights

### Layer 3: Sensor Fusion (20Hz)
- Combines all sensor inputs into unified 3D environment model
- Associates detections across sensors
- Maintains consistent object tracks
- Generates free-space map

### Layer 4: Prediction (10Hz)
- **Tracking**: Maintains temporal consistency of objects
- **Trajectory Prediction**: Predicts future paths of detected objects
- **Behavior Prediction**: Predicts intentions (lane change, turning, etc.)

### Layer 5: Planning & Decision (10Hz)
- Generates safe trajectory plans
- Makes tactical decisions (lane change, overtake, follow)
- Considers predictions and traffic rules

### Layer 6: Control (50Hz)
- Converts plans to vehicle control commands
- Manages longitudinal control (speed/acceleration)
- Manages lateral control (steering)

### Layer 7: Safety & Actuation (100Hz)
- **Safety Monitor**: Validates all commands, can override
- **CAN Gateway**: Translates to vehicle-specific CAN messages

### Layer 8: Human Machine Interface (30Hz)
- Visualizes environment and system status
- Handles user inputs and preferences
- Provides warnings and alerts

## Component Details

### Sensor Components (Data Producers)

1. **Camera ECUs** (4 components)
   - Front: Wide FOV for general perception
   - Rear: Backup and rear cross-traffic
   - Side Left/Right: Blind spot and lane change

2. **Radar ECUs** (5 components)
   - Front Long Range: ACC, AEB
   - Corner (4x): Cross-traffic, blind spot

3. **LiDAR ECU** (1 component)
   - 360° or front-facing high-resolution 3D

4. **Ultrasonic Array** (1 component)
   - Close-range parking assistance

### AI Components (Import sensors + WASI-NN)

1. **Object Detection AI**
   - YOLOv8 or EfficientDet model
   - Detects: vehicles, pedestrians, cyclists, etc.

2. **Lane Detection AI**
   - Semantic segmentation model
   - Lane lines, road edges, crosswalks

3. **Traffic Sign Recognition**
   - Classification + OCR models
   - Speed limits, traffic lights, signs

### Fusion & Tracking Components

1. **Sensor Fusion**
   - Multi-sensor Kalman filtering
   - Data association across sensors
   - Occupancy grid generation

2. **Tracking & Prediction**
   - Multi-object tracking (MOT)
   - Trajectory prediction using LSTM/Transformer

3. **Behavior Prediction**
   - Intent prediction neural networks
   - Lane change, turn, stop predictions

### Planning & Control Components

1. **Planning & Decision**
   - Behavior planning state machine
   - Trajectory optimization
   - Cost function evaluation

2. **Vehicle Control**
   - MPC (Model Predictive Control)
   - PID controllers for actuators
   - Vehicle dynamics model

### Safety & Infrastructure

1. **Safety Monitor**
   - Redundant safety checks
   - Fail-safe mechanisms
   - ISO 26262 compliance

2. **CAN Gateway**
   - Protocol translation
   - Message routing
   - Diagnostic support

3. **HMI Interface**
   - Real-time visualization
   - Warning generation
   - User interaction

4. **ADAS Domain Controller**
   - System orchestration
   - Health monitoring
   - OTA updates

## Key Design Principles

1. **Decoupled Architecture**: Each component has clear interfaces
2. **Data Flow**: Unidirectional data flow from sensors to actuators
3. **Fail-Safe**: Safety monitor can override at any time
4. **Scalable**: Easy to add/remove sensors or features
5. **Real-Time**: Designed for deterministic execution
6. **Standards**: ISO 26262, AUTOSAR Adaptive compatible