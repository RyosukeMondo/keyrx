//! Profile management endpoints.
//!
//! Split by concern:
//! - `types` - Request/response DTOs
//! - `lifecycle` - Create, activate, reload, delete, duplicate, rename
//! - `content` - List, active profile, config get/set, validation

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::config::profile_manager::ProfileError;
use crate::web::api::error::ApiError;
use crate::web::AppState;

mod content;
mod lifecycle;
mod types;

use content::{
    get_active_profile, get_profile_config, list_profiles, set_profile_config, validate_profile,
};
use lifecycle::{
    activate_profile, create_profile, delete_profile, duplicate_profile, reload_active_profile,
    rename_profile,
};

#[cfg(test)]
use types::ProfileResponse;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/profiles", get(list_profiles).post(create_profile))
        .route("/profiles/active", get(get_active_profile))
        .route("/profiles/active/reload", post(reload_active_profile))
        // More specific routes first (with path suffix)
        .route("/profiles/:name/activate", post(activate_profile))
        .route("/profiles/:name/validate", post(validate_profile))
        .route("/profiles/:name/duplicate", post(duplicate_profile))
        .route(
            "/profiles/:name/config",
            get(get_profile_config).put(set_profile_config),
        )
        .route("/profiles/:name/rename", put(rename_profile))
        // Less specific route last (matches any :name)
        .route("/profiles/:name", delete(delete_profile))
}

/// Convert ProfileError to ApiError with proper HTTP status codes
/// PROF-003: Enhanced error conversion with more detailed error messages.
fn profile_error_to_api_error(err: ProfileError) -> ApiError {
    match err {
        ProfileError::NotFound(msg) => ApiError::NotFound(format!("Profile not found: {}", msg)),
        ProfileError::InvalidName(msg) => {
            ApiError::BadRequest(format!("Invalid profile name: {}", msg))
        }
        ProfileError::AlreadyExists(msg) => {
            ApiError::Conflict(format!("Profile already exists: {}", msg))
        }
        ProfileError::ProfileLimitExceeded => {
            ApiError::BadRequest(format!("Profile limit exceeded (maximum {} profiles)", 100))
        }
        ProfileError::Compilation(e) => {
            ApiError::BadRequest(format!("Configuration compilation failed: {}", e))
        }
        ProfileError::LockError(msg) => {
            ApiError::InternalError(format!("Lock acquisition failed: {}", msg))
        }
        ProfileError::ActivationInProgress(name) => {
            ApiError::Conflict(format!("Profile '{}' is already being activated", name))
        }
        ProfileError::InvalidMetadata(msg) => {
            ApiError::BadRequest(format!("Invalid metadata: {}", msg))
        }
        ProfileError::IoError(e) => ApiError::InternalError(format!("IO error: {}", e)),
        _ => ApiError::InternalError(format!("Profile operation failed: {}", err)),
    }
}

#[cfg(test)]
mod tests;
