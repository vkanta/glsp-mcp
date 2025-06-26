use wit_bindgen::generate;

// Generate bindings for the camera front ECU component
generate!({
    world: "camera-front-component",
    path: "../../wit/camera-front-ecu-standalone.wit"
});

use exports::adas::camera_front::camera_front::{
    Guest, CameraFrame, CameraConfig, CameraCalibration, CameraStatus, 
    CameraResolution, PixelFormat, DetectionResult, LaneDetection, 
    TrafficSign, DiagnosticResult
};

struct CameraFrontEcu {
    initialized: bool,
    status: CameraStatus,
    config: Option<CameraConfig>,
    calibration: Option<CameraCalibration>,
    frame_counter: u32,
}

static mut CAMERA: CameraFrontEcu = CameraFrontEcu {
    initialized: false,
    status: CameraStatus::Offline,
    config: None,
    calibration: None,
    frame_counter: 0,
};

impl Guest for CameraFrontEcu {
    fn initialize(config: CameraConfig, calibration: CameraCalibration) -> Result<(), String> {
        unsafe {
            CAMERA.config = Some(config);
            CAMERA.calibration = Some(calibration);
            CAMERA.initialized = true;
            CAMERA.status = CameraStatus::Active;
            CAMERA.frame_counter = 0;
        }
        Ok(())
    }

    fn start_capture() -> Result<(), String> {
        unsafe {
            if !CAMERA.initialized {
                return Err("Camera not initialized".to_string());
            }
            CAMERA.status = CameraStatus::Active;
        }
        Ok(())
    }

    fn stop_capture() -> Result<(), String> {
        unsafe {
            CAMERA.status = CameraStatus::Offline;
        }
        Ok(())
    }

    fn get_frame() -> Result<CameraFrame, String> {
        unsafe {
            if CAMERA.status != CameraStatus::Active {
                return Err("Camera not active".to_string());
            }

            CAMERA.frame_counter += 1;
            
            // Mock camera frame data
            let frame = CameraFrame {
                timestamp: CAMERA.frame_counter as u64 * 33_333_333, // ~30 FPS
                frame_id: CAMERA.frame_counter,
                width: 1920,
                height: 1080,
                pixel_format: PixelFormat::Rgb888,
                data: vec![0u8; 1920 * 1080 * 3], // Mock RGB data
                exposure_time: 16.67, // milliseconds
                gain: 1.0,
                temperature: 45.0, // Celsius
            };
            
            Ok(frame)
        }
    }

    fn detect_objects(_frame: CameraFrame) -> Result<Vec<DetectionResult>, String> {
        // Mock AI object detection results
        let detections = vec![
            DetectionResult {
                object_id: 1,
                object_type: exports::adas::camera_front::camera_front::ObjectType::Vehicle,
                confidence: 0.95,
                bounding_box: exports::adas::camera_front::camera_front::BoundingBox {
                    x: 100.0,
                    y: 150.0,
                    width: 200.0,
                    height: 120.0,
                },
                distance_estimate: 25.5,
                relative_velocity: -5.2,
            },
            DetectionResult {
                object_id: 2,
                object_type: exports::adas::camera_front::camera_front::ObjectType::Pedestrian,
                confidence: 0.87,
                bounding_box: exports::adas::camera_front::camera_front::BoundingBox {
                    x: 350.0,
                    y: 300.0,
                    width: 80.0,
                    height: 180.0,
                },
                distance_estimate: 15.2,
                relative_velocity: 0.0,
            },
        ];
        
        Ok(detections)
    }

    fn detect_lanes(_frame: CameraFrame) -> Result<LaneDetection, String> {
        // Mock lane detection
        let lane_detection = LaneDetection {
            left_lane: exports::adas::camera_front::camera_front::LaneMarking {
                points: vec![
                    exports::adas::camera_front::camera_front::Point2d { x: 200.0, y: 1080.0 },
                    exports::adas::camera_front::camera_front::Point2d { x: 300.0, y: 540.0 },
                    exports::adas::camera_front::camera_front::Point2d { x: 400.0, y: 0.0 },
                ],
                line_type: exports::adas::camera_front::camera_front::LaneType::Dashed,
                color: exports::adas::camera_front::camera_front::LaneColor::White,
                confidence: 0.92,
            },
            right_lane: exports::adas::camera_front::camera_front::LaneMarking {
                points: vec![
                    exports::adas::camera_front::camera_front::Point2d { x: 1400.0, y: 1080.0 },
                    exports::adas::camera_front::camera_front::Point2d { x: 1300.0, y: 540.0 },
                    exports::adas::camera_front::camera_front::Point2d { x: 1200.0, y: 0.0 },
                ],
                line_type: exports::adas::camera_front::camera_front::LaneType::Solid,
                color: exports::adas::camera_front::camera_front::LaneColor::White,
                confidence: 0.94,
            },
            ego_lane_center: exports::adas::camera_front::camera_front::Point2d { x: 960.0, y: 540.0 },
            lane_width: 3.5, // meters
            curvature: 0.001, // 1/radius
            confidence: 0.93,
        };
        
        Ok(lane_detection)
    }

    fn detect_traffic_signs(_frame: CameraFrame) -> Result<Vec<TrafficSign>, String> {
        // Mock traffic sign detection
        let signs = vec![
            TrafficSign {
                sign_type: exports::adas::camera_front::camera_front::SignType::SpeedLimit,
                value: Some("60".to_string()),
                confidence: 0.96,
                position: exports::adas::camera_front::camera_front::BoundingBox {
                    x: 1600.0,
                    y: 200.0,
                    width: 80.0,
                    height: 80.0,
                },
                distance: 35.0,
            },
        ];
        
        Ok(signs)
    }

    fn get_status() -> CameraStatus {
        unsafe { CAMERA.status.clone() }
    }

    fn update_config(config: CameraConfig) -> Result<(), String> {
        unsafe {
            CAMERA.config = Some(config);
        }
        Ok(())
    }

    fn get_calibration() -> CameraCalibration {
        unsafe {
            CAMERA.calibration.clone().unwrap_or(CameraCalibration {
                focal_length_x: 1000.0,
                focal_length_y: 1000.0,
                principal_point_x: 960.0,
                principal_point_y: 540.0,
                distortion_coefficients: vec![-0.1, 0.05, 0.0, 0.0, 0.0],
                rotation_matrix: vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
                translation_vector: vec![0.0, 0.0, 1.5],
            })
        }
    }

    fn run_diagnostic() -> Result<DiagnosticResult, String> {
        let diagnostic = DiagnosticResult {
            lens_clean: true,
            focus_quality: 0.95,
            exposure_stable: true,
            temperature_ok: true,
            vibration_level: 0.02,
        };
        
        Ok(diagnostic)
    }
}

export!(CameraFrontEcu);