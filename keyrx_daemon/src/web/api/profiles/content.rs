//! Profile read/config endpoints: listing, active profile, config get/set,
//! and validation.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
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
    validate_profile_name(&name)?;
    let pm = Arc::clone(state.profile_service.profile_manager());
    tokio::task::spawn_blocking(move || {
        let profile = pm
            .get(&name)
            .ok_or_else(|| ApiError::NotFound(format!("Profile '{name}' not found")))?;
        validate_rhai_file(&profile.rhai_path)
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Task join error: {e}")))?
    .map(Json)
}

/// Body of POST /api/profiles/validate.
#[derive(Deserialize)]
pub(super) struct ValidateSourceRequest {
    config: String,
}

/// POST /api/profiles/validate - Compile unsaved source (the editor's buffer)
/// and report errors with their line/column, without touching any profile.
pub(super) async fn validate_source(
    Json(payload): Json<ValidateSourceRequest>,
) -> Result<Json<ValidationResponse>, ApiError> {
    validate_config_source(&payload.config)?;
    tokio::task::spawn_blocking(move || {
        let dir = ScratchDir::new()?;
        let rhai = dir.0.join("buffer.rhai");
        std::fs::write(&rhai, payload.config)
            .map_err(|e| ApiError::InternalError(format!("write temp source: {e}")))?;
        validate_rhai_file(&rhai)
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Task join error: {e}")))?
    .map(Json)
}

/// Compiles `rhai` to a throwaway `.krx` and reports the result.
fn validate_rhai_file(rhai: &std::path::Path) -> Result<ValidationResponse, ApiError> {
    use crate::config::profile_compiler::ProfileCompiler;

    let out = ScratchDir::new()?;
    match ProfileCompiler::new().compile_profile(rhai, &out.0.join("check.krx")) {
        Ok(_) => Ok(ValidationResponse {
            valid: true,
            errors: Vec::new(),
        }),
        Err(e) => {
            let (line, column) = e.location().unwrap_or((1, 1));
            Ok(ValidationResponse {
                valid: false,
                errors: vec![ValidationError {
                    line,
                    column,
                    length: 1,
                    message: e.short_message(),
                }],
            })
        }
    }
}

/// A unique temporary directory, removed on drop.
struct ScratchDir(std::path::PathBuf);

impl ScratchDir {
    fn new() -> Result<Self, ApiError> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "keyrx-validate-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir)
            .map_err(|e| ApiError::InternalError(format!("temp dir {}: {e}", dir.display())))?;
        Ok(Self(dir))
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
