//! Device management endpoints.

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, patch, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use validator::Validate;

use crate::config::device_registry::DeviceEntry;
use crate::error::DaemonError;
use crate::services::device_service::DeviceEditError;
use crate::web::api::error::ApiError;
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/devices", get(list_devices))
        .route("/devices/:id/name", put(rename_device))
        .route("/devices/:id/layout", put(set_device_layout))
        .route("/devices/:id/layout", get(get_device_layout))
        .route("/devices/:id", patch(update_device_config))
        .route("/devices/:id", delete(forget_device))
}

#[derive(Serialize)]
struct DeviceResponse {
    id: String,
    name: String,
    path: String,
    serial: Option<String>,
    active: bool,
    layout: Option<String>,
    is_virtual: bool,
    is_keyrx_output: bool,
}

#[derive(Deserialize)]
struct ListQuery {
    /// Also list software devices and keyrx's own output keyboards.
    #[serde(default)]
    include_virtual: bool,
}

#[derive(Serialize)]
struct DevicesListResponse {
    devices: Vec<DeviceResponse>,
}

/// GET /api/devices - List all connected devices.
///
/// Delegates to `DeviceService`, the one device list for every transport
/// (REST, WS-RPC): `active` is what the daemon actually grabbed, not
/// "the OS can see it".
async fn list_devices(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<DevicesListResponse>, DaemonError> {
    use crate::error::PlatformError;

    let devices = state
        .device_service
        .list_all_devices(query.include_virtual)
        .await
        .map_err(PlatformError::DeviceError)?
        .into_iter()
        .map(|d| DeviceResponse {
            id: d.id,
            name: d.name,
            path: d.path,
            serial: d.serial,
            active: d.active,
            layout: d.layout,
            is_virtual: d.is_virtual,
            is_keyrx_output: d.is_keyrx_output,
        })
        .collect();
    Ok(Json(DevicesListResponse { devices }))
}

/// PUT /api/devices/:id/name - Rename a device
#[derive(Deserialize, Validate)]
struct RenameDeviceRequest {
    #[validate(length(min = 1, max = 100))]
    name: String,
}

async fn rename_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<RenameDeviceRequest>,
) -> Result<Json<DeviceEntry>, ApiError> {
    payload
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation failed: {}", e)))?;
    let entry = state
        .device_service
        .rename_device(&id, &payload.name)
        .await
        .map_err(api_error)?;
    broadcast(
        &state,
        json!({ "action": "renamed", "id": id, "name": payload.name }),
    );
    Ok(Json(entry))
}

/// Maps a `DeviceService` edit error to its HTTP status.
fn api_error(e: DeviceEditError) -> ApiError {
    match e {
        DeviceEditError::NotFound(msg) => ApiError::NotFound(msg),
        DeviceEditError::Invalid(msg) => ApiError::BadRequest(msg),
        DeviceEditError::Storage(msg) => ApiError::InternalError(msg),
    }
}

/// Tells WebSocket subscribers of the `devices` channel about a change.
fn broadcast(state: &AppState, data: Value) {
    use crate::web::rpc_types::ServerMessage;
    let event = ServerMessage::Event {
        channel: "devices".to_string(),
        data,
    };
    if let Err(e) = state.event_broadcaster.send(event) {
        log::warn!("Failed to broadcast device event: {}", e);
    }
}

/// PUT /api/devices/:id/layout - Set device layout
#[derive(Deserialize, Validate)]
struct SetDeviceLayoutRequest {
    #[validate(length(min = 1, max = 50))]
    layout: String,
}

async fn set_device_layout(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<SetDeviceLayoutRequest>,
) -> Result<Json<Value>, ApiError> {
    payload
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation failed: {}", e)))?;
    state
        .device_service
        .set_layout(&id, &payload.layout)
        .await
        .map_err(api_error)?;
    Ok(Json(json!({ "success": true })))
}

/// GET /api/devices/:id/layout - Get device layout
#[derive(Serialize)]
struct GetDeviceLayoutResponse {
    layout: Option<String>,
}

async fn get_device_layout(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<GetDeviceLayoutResponse>, ApiError> {
    let layout = state
        .device_service
        .get_layout(&id)
        .await
        .map_err(api_error)?;
    Ok(Json(GetDeviceLayoutResponse { layout }))
}

/// PATCH /api/devices/:id - Update device configuration
#[derive(Deserialize, Validate)]
struct UpdateDeviceConfigRequest {
    #[validate(length(min = 1, max = 50))]
    layout: Option<String>,
}

async fn update_device_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDeviceConfigRequest>,
) -> Result<Json<Value>, ApiError> {
    payload
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation failed: {}", e)))?;
    if let Some(layout) = &payload.layout {
        state
            .device_service
            .set_layout(&id, layout)
            .await
            .map_err(api_error)?;
    }
    broadcast(
        &state,
        json!({ "action": "updated", "id": id, "layout": payload.layout }),
    );
    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/devices/:id - Forget device
async fn forget_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    state
        .device_service
        .forget_device(&id)
        .await
        .map_err(api_error)?;
    Ok(Json(json!({ "success": true })))
}
