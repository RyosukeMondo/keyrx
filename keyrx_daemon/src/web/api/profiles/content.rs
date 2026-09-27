//! Profile read/config endpoints: listing, active profile, config get/set,
//! and validation.

use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::DaemonError;
use crate::web::api::error::ApiError;
use crate::web::api::validation::{validate_config_source, validate_profile_name};
use crate::web::AppState;

use super::profile_error_to_api_error;
use super::types::{
    ProfileResponse, ProfilesListResponse, SetProfileConfigRequest, ValidationError,
    ValidationResponse,
};

/// GET /api/profiles - List all profiles
pub(super) async fn list_profiles(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ProfilesListResponse>, DaemonError> {
    use crate::error::ConfigError;

    // Use ProfileService to ensure consistent state across requests
    let profile_list = state
        .profile_service
        .list_profiles()
        .await
        .map_err(|e| ConfigError::Profile(e.to_string()))?;

    let profiles_dir = state.profile_service.profile_manager().profiles_dir();

    let profiles: Vec<ProfileResponse> = profile_list
        .iter()
        .map(|info| {
            // No blocking operations in map now!
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
                activated_at: info.activated_at,
                activated_by: info.activated_by.clone(),
            }
        })
        .collect();

    Ok(Json(ProfilesListResponse { profiles }))
}

/// GET /api/profiles/active - Get active profile
pub(super) async fn get_active_profile(State(state): State<Arc<AppState>>) -> Json<Value> {
    let active_profile = state.profile_service.get_active_profile().await;

    Json(json!({
        "active_profile": active_profile,
    }))
}

/// GET /api/profiles/:name/config - Get profile configuration
pub(super) async fn get_profile_config(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Validate profile name
    validate_profile_name(&name)?;

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

pub(super) async fn set_profile_config(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<SetProfileConfigRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate profile name
    validate_profile_name(&name)?;

    // Validate config source size
    validate_config_source(&payload.config)?;

    state
        .profile_service
        .set_profile_config(&name, &payload.config)
        .await
        .map_err(profile_error_to_api_error)?;

    Ok(Json(json!({
        "success": true,
    })))
}

pub(super) async fn validate_profile(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<ValidationResponse>, ApiError> {
    use crate::config::profile_compiler::ProfileCompiler;

    // Validate profile name
    validate_profile_name(&name)?;

    let pm = Arc::clone(state.profile_service.profile_manager());

    // Wrap all blocking operations in spawn_blocking
    tokio::task::spawn_blocking(move || {
        // Get profile metadata to find the .rhai file path
        let profile = pm
            .get(&name)
            .ok_or_else(|| format!("Profile '{}' not found", name))?;

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
                Ok::<ValidationResponse, String>(ValidationResponse {
                    valid: true,
                    errors: Vec::new(),
                })
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

                Ok(ValidationResponse {
                    valid: false,
                    errors,
                })
            }
        }
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Task join error: {}", e)))?
    .map_err(ApiError::InternalError)
    .map(Json)
}
