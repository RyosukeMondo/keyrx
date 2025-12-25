use axum::{routing::get, Json, Router};
use serde::Serialize;
use serde_json::{json, Value};

/// Device information returned by the /api/devices endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceResponse {
    /// Unique device identifier (serial-based or path-based).
    pub id: String,
    /// Human-readable device name.
    pub name: String,
    /// Device node path (e.g., "/dev/input/event0").
    pub path: String,
    /// USB serial number if available.
    pub serial: Option<String>,
    /// Whether the device is currently active (being managed).
    pub active: bool,
}

/// Response wrapper for the devices list.
#[derive(Debug, Clone, Serialize)]
pub struct DevicesListResponse {
    /// List of discovered keyboard devices.
    pub devices: Vec<DeviceResponse>,
}

#[allow(dead_code)]
pub fn create_router() -> Router {
    Router::new()
        .route("/status", get(get_status))
        .route("/config", get(get_config))
        .route("/devices", get(get_devices))
}

#[allow(dead_code)]
async fn get_status() -> Json<Value> {
    Json(json!({
        "status": "running",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[allow(dead_code)]
async fn get_config() -> Json<Value> {
    Json(json!({
        "config": "placeholder"
    }))
}

/// GET /api/devices - Returns list of connected keyboard devices.
///
/// This endpoint enumerates all keyboard input devices available on the system.
/// Each device includes its unique ID, name, path, serial number (if available),
/// and active status.
///
/// # Response Format
///
/// ```json
/// {
///   "devices": [
///     {
///       "id": "serial-ABC123",
///       "name": "USB Keyboard",
///       "path": "/dev/input/event0",
///       "serial": "ABC123",
///       "active": true
///     }
///   ]
/// }
/// ```
#[cfg(any(target_os = "linux", target_os = "windows"))]
#[allow(dead_code)]
async fn get_devices() -> Json<DevicesListResponse> {
    use crate::device_manager::enumerate_keyboards;

    let devices = match enumerate_keyboards() {
        Ok(keyboards) => keyboards
            .into_iter()
            .map(|kb| {
                // Generate device ID following the same pattern as ManagedDevice::device_id()
                let id = if let Some(ref serial) = kb.serial {
                    if !serial.is_empty() {
                        format!("serial-{}", serial)
                    } else {
                        format!("path-{}", kb.path.display())
                    }
                } else {
                    format!("path-{}", kb.path.display())
                };

                DeviceResponse {
                    id,
                    name: kb.name,
                    path: kb.path.display().to_string(),
                    serial: kb.serial,
                    active: true, // All enumerated devices are considered available
                }
            })
            .collect(),
        Err(e) => {
            log::warn!("Failed to enumerate devices for API: {}", e);
            Vec::new()
        }
    };

    Json(DevicesListResponse { devices })
}

/// Fallback for unsupported platforms.
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
#[allow(dead_code)]
async fn get_devices() -> Json<DevicesListResponse> {
    // On unsupported platforms, return empty device list
    Json(DevicesListResponse {
        devices: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_router() {
        let router = create_router();
        // Just verify router can be created
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[tokio::test]
    async fn test_get_status() {
        let result = get_status().await;
        let value = result.0;
        assert_eq!(value["status"], "running");
    }

    #[tokio::test]
    async fn test_get_config() {
        let result = get_config().await;
        let value = result.0;
        assert_eq!(value["config"], "placeholder");
    }

    #[tokio::test]
    async fn test_get_devices_returns_valid_response() {
        let result = get_devices().await;
        let response = result.0;

        // Response should have a devices field (may be empty if no permissions)
        // We're just testing the response structure, not device enumeration
        // The test passes if we get a valid DevicesListResponse (no panic)
        let _ = response.devices; // Access devices to verify structure
    }

    #[test]
    fn test_device_response_serialization() {
        let device = DeviceResponse {
            id: "serial-ABC123".to_string(),
            name: "USB Keyboard".to_string(),
            path: "/dev/input/event0".to_string(),
            serial: Some("ABC123".to_string()),
            active: true,
        };

        let json = serde_json::to_value(&device).expect("Failed to serialize");
        assert_eq!(json["id"], "serial-ABC123");
        assert_eq!(json["name"], "USB Keyboard");
        assert_eq!(json["path"], "/dev/input/event0");
        assert_eq!(json["serial"], "ABC123");
        assert_eq!(json["active"], true);
    }

    #[test]
    fn test_device_response_without_serial() {
        let device = DeviceResponse {
            id: "path-/dev/input/event1".to_string(),
            name: "AT Translated Set 2 keyboard".to_string(),
            path: "/dev/input/event1".to_string(),
            serial: None,
            active: true,
        };

        let json = serde_json::to_value(&device).expect("Failed to serialize");
        assert_eq!(json["serial"], serde_json::Value::Null);
    }

    #[test]
    fn test_devices_list_response_serialization() {
        let response = DevicesListResponse {
            devices: vec![
                DeviceResponse {
                    id: "serial-ABC".to_string(),
                    name: "Keyboard 1".to_string(),
                    path: "/dev/input/event0".to_string(),
                    serial: Some("ABC".to_string()),
                    active: true,
                },
                DeviceResponse {
                    id: "path-/dev/input/event1".to_string(),
                    name: "Keyboard 2".to_string(),
                    path: "/dev/input/event1".to_string(),
                    serial: None,
                    active: true,
                },
            ],
        };

        let json = serde_json::to_value(&response).expect("Failed to serialize");
        assert!(json["devices"].is_array());
        assert_eq!(json["devices"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_devices_list_response_empty() {
        let response = DevicesListResponse { devices: vec![] };

        let json = serde_json::to_value(&response).expect("Failed to serialize");
        assert!(json["devices"].is_array());
        assert_eq!(json["devices"].as_array().unwrap().len(), 0);
    }
}
