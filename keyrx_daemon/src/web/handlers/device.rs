//! Device RPC method handlers.
//!
//! This module implements all device-related RPC methods for WebSocket communication.
//! Each method accepts parameters as serde_json::Value, validates them, and delegates
//! to the DeviceService for business logic execution.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use typeshare::typeshare;

use crate::services::DeviceService;
use crate::web::rpc_types::{RpcError, ServerMessage, INTERNAL_ERROR};
use crate::web::AppState;

/// Parameters for get_devices query
#[derive(Debug, Deserialize)]
struct GetDevicesParams {
    // No parameters needed - returns all devices
}

/// Parameters for rename_device command
#[derive(Debug, Deserialize)]
struct RenameDeviceParams {
    id: String,
    name: String,
}

/// Parameters for forget_device command
#[derive(Debug, Deserialize)]
struct ForgetDeviceParams {
    id: String,
}

/// Device information returned by RPC methods
#[typeshare]
#[derive(Debug, Serialize)]
pub struct DeviceRpcInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub serial: Option<String>,
    pub active: bool,
    pub layout: Option<String>,
}

/// Validate device ID to prevent injection attacks
fn validate_device_id(id: &str) -> Result<(), RpcError> {
    if id.is_empty() {
        return Err(RpcError::invalid_params("Device ID cannot be empty"));
    }

    // Device IDs should be reasonable length (max 256 chars as per DeviceEntry)
    if id.len() > 256 {
        return Err(RpcError::invalid_params(
            "Device ID too long (max 256 chars)",
        ));
    }

    // Check for path traversal attempts
    if id.contains("..") {
        return Err(RpcError::invalid_params("Device ID cannot contain '..'"));
    }

    if id.contains('/') || id.contains('\\') {
        return Err(RpcError::invalid_params(
            "Device ID cannot contain path separators",
        ));
    }

    // Check for null bytes or other control characters
    if id.chars().any(|c| c.is_control()) {
        return Err(RpcError::invalid_params(
            "Device ID cannot contain control characters",
        ));
    }

    Ok(())
}

/// Get all devices
pub async fn get_devices(device_service: &DeviceService, params: Value) -> Result<Value, RpcError> {
    // Validate params (should be empty object or null)
    let _params: Option<GetDevicesParams> = if params.is_null() {
        None
    } else {
        Some(
            serde_json::from_value(params)
                .map_err(|e| RpcError::invalid_params(format!("Invalid parameters: {}", e)))?,
        )
    };

    log::debug!("RPC: get_devices");

    // Call device service
    let devices = device_service
        .list_devices()
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, format!("Failed to list devices: {}", e)))?;

    // Convert to RPC format
    let rpc_devices: Vec<DeviceRpcInfo> = devices
        .iter()
        .map(|d| DeviceRpcInfo {
            id: d.id.clone(),
            name: d.name.clone(),
            path: d.path.clone(),
            serial: d.serial.clone(),
            active: d.active,
            layout: d.layout.clone(),
        })
        .collect();

    serde_json::to_value(rpc_devices)
        .map_err(|e| RpcError::internal_error(format!("Serialization failed: {}", e)))
}

/// Rename a device
pub async fn rename_device(state: &AppState, params: Value) -> Result<Value, RpcError> {
    let params: RenameDeviceParams = serde_json::from_value(params)
        .map_err(|e| RpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

    log::info!("RPC: rename_device id={} name={}", params.id, params.name);

    // Validate device ID
    validate_device_id(&params.id)?;

    // Device name validation is handled by DeviceRegistry

    // Call device service
    state
        .device_service
        .rename_device(&params.id, &params.name)
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, format!("Failed to rename device: {}", e)))?;

    // Broadcast event to WebSocket subscribers
    let event = ServerMessage::Event {
        channel: "devices".to_string(),
        data: serde_json::json!({
            "action": "renamed",
            "id": params.id,
            "name": params.name
        }),
    };
    if let Err(e) = state.event_broadcaster.send(event) {
        log::warn!("Failed to broadcast device renamed event: {}", e);
    }

    // Return success
    Ok(serde_json::json!({
        "renamed": true,
        "id": params.id,
        "name": params.name
    }))
}

/// Forget a device
pub async fn forget_device(
    device_service: &DeviceService,
    params: Value,
) -> Result<Value, RpcError> {
    let params: ForgetDeviceParams = serde_json::from_value(params)
        .map_err(|e| RpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

    log::info!("RPC: forget_device id={}", params.id);

    // Validate device ID
    validate_device_id(&params.id)?;

    // Call device service
    device_service
        .forget_device(&params.id)
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, format!("Failed to forget device: {}", e)))?;

    // Return success
    Ok(serde_json::json!({
        "forgotten": true,
        "id": params.id
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_device_id_valid() {
        assert!(validate_device_id("device-123").is_ok());
        assert!(validate_device_id("keyboard_usb_001").is_ok());
        assert!(validate_device_id("Test-Device_456").is_ok());
    }

    #[test]
    fn test_validate_device_id_empty() {
        assert!(validate_device_id("").is_err());
    }

    #[test]
    fn test_validate_device_id_too_long() {
        let long_id = "a".repeat(257);
        assert!(validate_device_id(&long_id).is_err());
    }

    #[test]
    fn test_validate_device_id_path_traversal() {
        assert!(validate_device_id("../etc/passwd").is_err());
        assert!(validate_device_id("..").is_err());
        assert!(validate_device_id("test/../device").is_err());
    }

    #[test]
    fn test_validate_device_id_path_separators() {
        assert!(validate_device_id("test/device").is_err());
        assert!(validate_device_id("test\\device").is_err());
        assert!(validate_device_id("/root").is_err());
    }

    #[test]
    fn test_validate_device_id_control_characters() {
        assert!(validate_device_id("device\0id").is_err());
        assert!(validate_device_id("device\nid").is_err());
        assert!(validate_device_id("device\tid").is_err());
    }
}
