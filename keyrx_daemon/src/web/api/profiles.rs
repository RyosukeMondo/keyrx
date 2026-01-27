//! Profile management endpoints.

use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::config::profile_manager::{ProfileError, ProfileManager, ProfileTemplate};
use crate::error::DaemonError;
use crate::web::api::error::ApiError;
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/profiles", get(list_profiles).post(create_profile))
        .route("/profiles/active", get(get_active_profile))
        .route("/profiles/:name/activate", post(activate_profile))
        .route(
            "/profiles/:name/config",
            get(get_profile_config).put(set_profile_config),
        )
        .route("/profiles/:name", delete(delete_profile))
        .route("/profiles/:name/duplicate", post(duplicate_profile))
        .route("/profiles/:name/rename", put(rename_profile))
        .route("/profiles/:name/validate", post(validate_profile))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileResponse {
    name: String,
    rhai_path: String,
    krx_path: String,
    #[serde(serialize_with = "serialize_systemtime_as_rfc3339")]
    modified_at: std::time::SystemTime,
    #[serde(serialize_with = "serialize_systemtime_as_rfc3339")]
    created_at: std::time::SystemTime,
    layer_count: usize,
    #[serde(rename = "deviceCount")]
    device_count: usize,
    #[serde(rename = "keyCount")]
    key_count: usize,
    #[serde(rename = "isActive")]
    active: bool,
}

/// Serialize SystemTime as RFC 3339 / ISO 8601 string
fn serialize_systemtime_as_rfc3339<S>(
    time: &std::time::SystemTime,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    let datetime: DateTime<Utc> = (*time).into();
    datetime.to_rfc3339().serialize(serializer)
}

/// Convert ProfileError to ApiError with proper HTTP status codes
fn profile_error_to_api_error(err: ProfileError) -> ApiError {
    match err {
        ProfileError::NotFound(msg) => ApiError::NotFound(msg),
        ProfileError::InvalidName(msg) => ApiError::BadRequest(format!("Invalid name: {}", msg)),
        ProfileError::AlreadyExists(msg) => {
            ApiError::Conflict(format!("Profile already exists: {}", msg))
        }
        ProfileError::ProfileLimitExceeded => {
            ApiError::BadRequest("Profile limit exceeded".to_string())
        }
        _ => ApiError::InternalError(err.to_string()),
    }
}

#[derive(Serialize)]
struct ProfilesListResponse {
    profiles: Vec<ProfileResponse>,
}

/// GET /api/profiles - List all profiles
async fn list_profiles(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ProfilesListResponse>, DaemonError> {
    use crate::error::ConfigError;

    // Use ProfileService to ensure consistent state across requests
    let profile_list = state
        .profile_service
        .list_profiles()
        .await
        .map_err(|e| ConfigError::Profile(e.to_string()))?;

    let profiles: Vec<ProfileResponse> = profile_list
        .iter()
        .map(|info| {
            // Build paths from name (ProfileService doesn't return paths)
            let config_dir = get_config_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let profiles_dir = config_dir.join("profiles");
            let rhai_path = profiles_dir.join(format!("{}.rhai", info.name));
            let krx_path = profiles_dir.join(format!("{}.krx", info.name));

            ProfileResponse {
                name: info.name.clone(),
                rhai_path: rhai_path.display().to_string(),
                krx_path: krx_path.display().to_string(),
                modified_at: info.modified_at,
                created_at: info.modified_at, // Use modified_at as created_at for now
                layer_count: info.layer_count,
                device_count: 0, // TODO: Track device count per profile
                key_count: 0,    // TODO: Parse Rhai config to count key mappings
                active: info.active,
            }
        })
        .collect();

    Ok(Json(ProfilesListResponse { profiles }))
}

/// POST /api/profiles - Create new profile
#[derive(Deserialize)]
struct CreateProfileRequest {
    name: String,
    template: String, // "blank", "simple_remap", "capslock_escape", "vim_navigation", "gaming"
}

async fn create_profile(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateProfileRequest>,
) -> Result<Json<Value>, DaemonError> {
    use crate::error::{ConfigError, WebError};

    let template = match payload.template.as_str() {
        "blank" => ProfileTemplate::Blank,
        "simple_remap" => ProfileTemplate::SimpleRemap,
        "capslock_escape" => ProfileTemplate::CapslockEscape,
        "vim_navigation" => ProfileTemplate::VimNavigation,
        "gaming" => ProfileTemplate::Gaming,
        _ => {
            return Err(WebError::InvalidRequest {
                reason: format!("Invalid template: '{}'. Valid templates: blank, simple_remap, capslock_escape, vim_navigation, gaming", payload.template),
            }
            .into())
        }
    };

    // Use ProfileService to ensure consistent state
    let profile_info = state
        .profile_service
        .create_profile(&payload.name, template)
        .await
        .map_err(|e| ConfigError::Profile(e.to_string()))?;

    // Build paths from name
    let config_dir = get_config_dir()?;
    let profiles_dir = config_dir.join("profiles");
    let rhai_path = profiles_dir.join(format!("{}.rhai", profile_info.name));
    let krx_path = profiles_dir.join(format!("{}.krx", profile_info.name));

    Ok(Json(json!({
        "success": true,
        "profile": {
            "name": profile_info.name,
            "rhai_path": rhai_path.display().to_string(),
            "krx_path": krx_path.display().to_string(),
        }
    })))
}

/// POST /api/profiles/:name/activate - Activate profile
async fn activate_profile(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, DaemonError> {
    use crate::error::ConfigError;

    // Check if test mode is enabled
    if let Some(socket_path) = &state.test_mode_socket {
        // Test mode: use IPC to activate profile
        use crate::ipc::{unix_socket::UnixSocketIpc, DaemonIpc, IpcRequest, IpcResponse};
        use std::time::Duration;

        let mut ipc = UnixSocketIpc::new(socket_path.clone());

        // Send activation request with 5 second timeout
        let request = IpcRequest::ActivateProfile { name: name.clone() };

        let response = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::task::spawn_blocking(move || ipc.send_request(&request)).await
        })
        .await
        .map_err(|_| {
            ConfigError::Profile(
                "IPC timeout: profile activation took longer than 5 seconds".to_string(),
            )
        })?
        .map_err(|e| ConfigError::Profile(format!("Failed to join IPC task: {}", e)))?
        .map_err(|e| ConfigError::Profile(format!("IPC error: {}", e)))?;

        match response {
            IpcResponse::ProfileActivated { name: profile_name } => {
                // Reload simulation service with the new profile
                if let Err(e) = state.simulation_service.load_profile(&profile_name) {
                    log::warn!("Failed to load profile into simulation service: {}", e);
                    // Don't fail the activation if simulation service load fails - it's not critical
                }

                // Broadcast event to WebSocket subscribers
                use crate::web::rpc_types::ServerMessage;
                let event = ServerMessage::Event {
                    channel: "profiles".to_string(),
                    data: serde_json::json!({
                        "action": "activated",
                        "profile": profile_name.clone()
                    }),
                };
                if let Err(e) = state.event_broadcaster.send(event) {
                    log::warn!("Failed to broadcast profile activated event: {}", e);
                }

                Ok(Json(json!({
                    "success": true,
                    "profile": profile_name,
                    "compile_time_ms": 0,
                    "reload_time_ms": 0,
                })))
            }
            IpcResponse::Error { code, message } => Err(ConfigError::Profile(format!(
                "Profile activation failed (code {}): {}",
                code, message
            ))
            .into()),
            _ => Err(ConfigError::Profile("Unexpected IPC response".to_string()).into()),
        }
    } else {
        // Production mode: use ProfileService to ensure consistent state
        let result = state
            .profile_service
            .activate_profile(&name)
            .await
            .map_err(|e| ConfigError::Profile(e.to_string()))?;

        if !result.success {
            return Err(ConfigError::CompilationFailed {
                reason: result.error.unwrap_or_else(|| "Unknown error".to_string()),
            }
            .into());
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

/// DELETE /api/profiles/:name - Delete profile
async fn delete_profile(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, DaemonError> {
    use crate::error::ConfigError;

    state
        .profile_service
        .delete_profile(&name)
        .await
        .map_err(|e| ConfigError::Profile(e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

/// POST /api/profiles/:name/duplicate - Duplicate profile
#[derive(Deserialize)]
struct DuplicateProfileRequest {
    new_name: String,
}

async fn duplicate_profile(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<DuplicateProfileRequest>,
) -> Result<Json<Value>, ApiError> {
    let profile_info = state
        .profile_service
        .duplicate_profile(&name, &payload.new_name)
        .await
        .map_err(profile_error_to_api_error)?;

    // Build rhai_path from name
    let config_dir = get_config_dir().map_err(|e| ApiError::InternalError(e.to_string()))?;
    let profiles_dir = config_dir.join("profiles");
    let rhai_path = profiles_dir.join(format!("{}.rhai", profile_info.name));

    Ok(Json(json!({
        "success": true,
        "profile": {
            "name": profile_info.name,
            "rhai_path": rhai_path.display().to_string(),
        }
    })))
}

/// PUT /api/profiles/:name/rename - Rename profile
#[derive(Deserialize)]
struct RenameProfileRequest {
    new_name: String,
}

async fn rename_profile(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<RenameProfileRequest>,
) -> Result<Json<Value>, ApiError> {
    let profile_info = state
        .profile_service
        .rename_profile(&name, &payload.new_name)
        .await
        .map_err(profile_error_to_api_error)?;

    // Build paths from name
    let config_dir = get_config_dir().map_err(|e| ApiError::InternalError(e.to_string()))?;
    let profiles_dir = config_dir.join("profiles");
    let rhai_path = profiles_dir.join(format!("{}.rhai", profile_info.name));
    let krx_path = profiles_dir.join(format!("{}.krx", profile_info.name));

    Ok(Json(json!({
        "success": true,
        "profile": {
            "name": profile_info.name,
            "rhai_path": rhai_path.display().to_string(),
            "krx_path": krx_path.display().to_string(),
        }
    })))
}

/// GET /api/profiles/active - Get active profile
async fn get_active_profile(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, DaemonError> {
    let active_profile = state.profile_service.get_active_profile().await;

    Ok(Json(json!({
        "active_profile": active_profile,
    })))
}

/// GET /api/profiles/:name/config - Get profile configuration
async fn get_profile_config(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let config = state
        .profile_service
        .get_profile_config(&name)
        .await
        .map_err(profile_error_to_api_error)?;

    Ok(Json(json!({
        "name": name,
        "source": config,
    })))
}

/// PUT /api/profiles/:name/config - Set profile configuration
#[derive(Deserialize)]
struct SetProfileConfigRequest {
    config: String,
}

async fn set_profile_config(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<SetProfileConfigRequest>,
) -> Result<Json<Value>, ApiError> {
    state
        .profile_service
        .set_profile_config(&name, &payload.config)
        .await
        .map_err(profile_error_to_api_error)?;

    Ok(Json(json!({
        "success": true,
    })))
}

/// POST /api/profiles/:name/validate - Validate profile configuration
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidationError {
    line: usize,
    column: Option<usize>,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidationResponse {
    valid: bool,
    errors: Vec<ValidationError>,
}

async fn validate_profile(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<ValidationResponse>, ApiError> {
    use crate::config::profile_compiler::ProfileCompiler;

    // For validation, we still need to access ProfileManager directly to get file paths
    // This is read-only so it doesn't affect state consistency
    let config_dir = get_config_dir().map_err(|e| ApiError::InternalError(e.to_string()))?;
    let pm = ProfileManager::new(config_dir).map_err(profile_error_to_api_error)?;

    // Get profile metadata to find the .rhai file path
    let profile = pm
        .get(&name)
        .ok_or_else(|| ApiError::NotFound(format!("Profile '{}' not found", name)))?;

    // Compile the profile to validate it
    let compiler = ProfileCompiler::new();
    // Use timestamp + profile name for temporary file to avoid collisions
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_krx = std::env::temp_dir().join(format!("{}_{}.krx", name, timestamp));

    let validation_result = compiler.compile_profile(&profile.rhai_path, &temp_krx);

    // Clean up temporary file
    let _ = std::fs::remove_file(&temp_krx);

    match validation_result {
        Ok(_) => {
            // Compilation succeeded - profile is valid
            Ok(Json(ValidationResponse {
                valid: true,
                errors: Vec::new(),
            }))
        }
        Err(e) => {
            // Compilation failed - extract error information
            let error_message = e.to_string();

            // Parse error message to extract line/column information
            // The error format from the compiler is user-friendly and may include line numbers
            let errors = vec![ValidationError {
                line: 1, // TODO: Parse actual line number from error message
                column: None,
                message: error_message,
            }];

            Ok(Json(ValidationResponse {
                valid: false,
                errors,
            }))
        }
    }
}

/// Get config directory path (cross-platform)
fn get_config_dir() -> Result<std::path::PathBuf, DaemonError> {
    use crate::error::ConfigError;

    let config_dir = dirs::config_dir().ok_or_else(|| ConfigError::ParseError {
        path: std::path::PathBuf::from("~"),
        reason: "Cannot determine config directory".to_string(),
    })?;

    Ok(config_dir.join("keyrx"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn test_serialize_systemtime_as_rfc3339() {
        // Create a known timestamp: 2024-01-01T00:00:00Z
        let timestamp = UNIX_EPOCH + Duration::from_secs(1704067200);

        // Serialize to JSON
        let json_value = serde_json::to_value(&ProfileResponse {
            name: "test".to_string(),
            rhai_path: "/test.rhai".to_string(),
            krx_path: "/test.krx".to_string(),
            modified_at: timestamp,
            created_at: timestamp,
            layer_count: 1,
            device_count: 0,
            key_count: 0,
            active: false,
        })
        .unwrap();

        // Check that modifiedAt is a string in ISO 8601 / RFC 3339 format
        let modified_at_str = json_value["modifiedAt"].as_str().unwrap();

        // Should be in format: YYYY-MM-DDTHH:MM:SS.sssZ or similar RFC 3339
        assert!(
            modified_at_str.contains('T'),
            "Timestamp should contain 'T' separator: {}",
            modified_at_str
        );
        assert!(
            modified_at_str.ends_with('Z')
                || modified_at_str.contains('+')
                || modified_at_str.contains('-'),
            "Timestamp should have timezone (Z or offset): {}",
            modified_at_str
        );

        // Verify it can be parsed back by JavaScript Date constructor
        // RFC 3339 format is guaranteed to be parseable by new Date()
        assert!(
            modified_at_str.len() >= 20, // Minimum length for ISO 8601
            "Timestamp too short: {}",
            modified_at_str
        );
    }

    #[test]
    fn test_profile_response_camel_case_fields() {
        let timestamp = UNIX_EPOCH + Duration::from_secs(1704067200);

        let response = ProfileResponse {
            name: "gaming".to_string(),
            rhai_path: "/profiles/gaming.rhai".to_string(),
            krx_path: "/profiles/gaming.krx".to_string(),
            modified_at: timestamp,
            created_at: timestamp,
            layer_count: 3,
            device_count: 2,
            key_count: 127,
            active: true,
        };

        let json_value = serde_json::to_value(&response).unwrap();

        // Verify camelCase field names
        assert!(
            json_value["modifiedAt"].is_string(),
            "modifiedAt should be a string"
        );
        assert!(
            json_value["createdAt"].is_string(),
            "createdAt should be a string"
        );
        assert!(
            json_value["layerCount"].is_number(),
            "layerCount should be a number"
        );
        assert!(
            json_value["deviceCount"].is_number(),
            "deviceCount should be a number"
        );
        assert!(
            json_value["keyCount"].is_number(),
            "keyCount should be a number"
        );
        assert!(
            json_value["active"].is_boolean(),
            "active should be a boolean"
        );

        // Verify snake_case fields are NOT present
        assert!(
            json_value.get("modified_at").is_none(),
            "Should not have snake_case modified_at"
        );
        assert!(
            json_value.get("created_at").is_none(),
            "Should not have snake_case created_at"
        );
        assert!(
            json_value.get("layer_count").is_none(),
            "Should not have snake_case layer_count"
        );
        // active is the correct field name, not is_active
        assert!(
            json_value.get("is_active").is_none(),
            "Should not have snake_case is_active"
        );
    }
}
