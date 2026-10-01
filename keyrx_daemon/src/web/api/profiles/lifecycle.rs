//! Profile lifecycle endpoints: create, activate, reload, delete, duplicate,
//! and rename.

use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::config::profile_manager::ProfileTemplate;
use crate::web::api::error::ApiError;
use crate::web::api::validation::validate_profile_name;
use crate::web::AppState;

use super::profile_error_to_api_error;
use super::types::{CreateProfileRequest, DuplicateProfileRequest, RenameProfileRequest};

pub(super) async fn create_profile(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateProfileRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate profile name
    validate_profile_name(&payload.name)?;

    let template = ProfileTemplate::from_name(&payload.template).map_err(ApiError::BadRequest)?;

    // Use ProfileService to ensure consistent state
    let profile_info = state
        .profile_service
        .create_profile(&payload.name, template)
        .await
        .map_err(profile_error_to_api_error)?;

    // Build paths from name
    let profiles_dir = state.profile_service.profile_manager().profiles_dir();
    let rhai_path_str = profiles_dir
        .join(format!("{}.rhai", profile_info.name))
        .display()
        .to_string();
    let krx_path_str = profiles_dir
        .join(format!("{}.krx", profile_info.name))
        .display()
        .to_string();

    Ok(Json(json!({
        "success": true,
        "profile": {
            "name": profile_info.name,
            "rhaiPath": rhai_path_str,
            "krxPath": krx_path_str,
        }
    })))
}

/// POST /api/profiles/:name/activate - Activate profile
pub(super) async fn activate_profile(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Validate profile name
    validate_profile_name(&name)?;

    // Use ProfileService directly for both test and production modes.
    // Test mode has no running daemon to reload via IPC, so direct activation
    // through ProfileService is the correct path.
    {
        let result = state
            .profile_service
            .activate_profile(&name)
            .await
            .map_err(profile_error_to_api_error)?;

        if !result.success {
            return Err(ApiError::InternalError(
                result.error.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        // Reload simulation service with the new profile
        if let Err(e) = state.simulation_service.load_profile(&name) {
            log::warn!("Failed to load profile into simulation service: {}", e);
            // Don't fail the activation if simulation service load fails - it's not critical
        }

        // Broadcast event to WebSocket subscribers
        use crate::web::rpc_types::ServerMessage;
        let event = ServerMessage::Event {
            channel: "profiles".to_string(),
            data: serde_json::json!({
                "action": "activated",
                "profile": name.clone()
            }),
        };
        if let Err(e) = state.event_broadcaster.send(event) {
            log::warn!("Failed to broadcast profile activated event: {}", e);
        }

        Ok(Json(json!({
            "success": true,
            "profile": name,
            "compile_time_ms": result.compile_time_ms,
            "reload_time_ms": result.reload_time_ms,
        })))
    }
}

/// POST /api/profiles/active/reload - Reload active profile (recompile if .rhai newer than .krx)
pub(super) async fn reload_active_profile(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .profile_service
        .reload_active_profile()
        .await
        .map_err(profile_error_to_api_error)?;

    if !result.success {
        return Err(ApiError::InternalError(
            result
                .error
                .unwrap_or_else(|| "Compilation failed".to_string()),
        ));
    }

    // If recompiled (ProfileService already asked the daemon to reload)
    if result.recompiled {
        // Broadcast event to WebSocket subscribers
        use crate::web::rpc_types::ServerMessage;
        let active = state.profile_service.get_active_profile().await;
        let event = ServerMessage::Event {
            channel: "profiles".to_string(),
            data: serde_json::json!({
                "action": "reloaded",
                "profile": active,
                "recompiled": true,
            }),
        };
        if let Err(e) = state.event_broadcaster.send(event) {
            log::warn!("Failed to broadcast profile reload event: {}", e);
        }
    }

    Ok(Json(json!({
        "success": true,
        "recompiled": result.recompiled,
        "compile_time_ms": result.compile_time_ms,
    })))
}

/// DELETE /api/profiles/:name - Delete profile
pub(super) async fn delete_profile(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Validate profile name
    validate_profile_name(&name)?;

    state
        .profile_service
        .delete_profile(&name)
        .await
        .map_err(profile_error_to_api_error)?;

    Ok(Json(json!({ "success": true })))
}

pub(super) async fn duplicate_profile(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<DuplicateProfileRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate both profile names
    validate_profile_name(&name)?;
    validate_profile_name(&payload.new_name)?;

    let profile_info = state
        .profile_service
        .duplicate_profile(&name, &payload.new_name)
        .await
        .map_err(profile_error_to_api_error)?;

    // Build rhai_path from name
    let profiles_dir = state.profile_service.profile_manager().profiles_dir();
    let rhai_path_str = profiles_dir
        .join(format!("{}.rhai", profile_info.name))
        .display()
        .to_string();

    Ok(Json(json!({
        "success": true,
        "profile": {
            "name": profile_info.name,
            "rhaiPath": rhai_path_str,
        }
    })))
}

pub(super) async fn rename_profile(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<RenameProfileRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate both profile names
    validate_profile_name(&name)?;
    validate_profile_name(&payload.new_name)?;

    let profile_info = state
        .profile_service
        .rename_profile(&name, &payload.new_name)
        .await
        .map_err(profile_error_to_api_error)?;

    // Build paths from name
    let profiles_dir = state.profile_service.profile_manager().profiles_dir();
    let rhai_path_str = profiles_dir
        .join(format!("{}.rhai", profile_info.name))
        .display()
        .to_string();
    let krx_path_str = profiles_dir
        .join(format!("{}.krx", profile_info.name))
        .display()
        .to_string();

    Ok(Json(json!({
        "success": true,
        "profile": {
            "name": profile_info.name,
            "rhaiPath": rhai_path_str,
            "krxPath": krx_path_str,
        }
    })))
}
