//! Profile RPC method handlers.
//!
//! This module implements all profile-related RPC methods for WebSocket communication.
//! Each method accepts parameters as serde_json::Value, validates them, and delegates
//! to the ProfileService for business logic execution.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use typeshare::typeshare;
use validator::Validate;

use crate::config::ProfileTemplate;
use crate::services::ProfileService;
use crate::validation;
use crate::web::rpc_types::{RpcError, ServerMessage, INTERNAL_ERROR};
use crate::web::AppState;

/// Parameters for get_profiles query
#[derive(Debug, Deserialize)]
struct GetProfilesParams {
    // No parameters needed - returns all profiles
}

/// Parameters for create_profile command
#[derive(Debug, Deserialize, Validate)]
struct CreateProfileParams {
    #[validate(length(min = 1, max = 100))]
    name: String,
    #[serde(default = "default_template")]
    #[validate(length(min = 1, max = 50))]
    template: String,
}

fn default_template() -> String {
    "blank".to_string()
}

/// Parameters for activate_profile command
#[derive(Debug, Deserialize, Validate)]
struct ActivateProfileParams {
    #[validate(length(min = 1, max = 100))]
    name: String,
}

/// Parameters for delete_profile command
#[derive(Debug, Deserialize, Validate)]
struct DeleteProfileParams {
    #[validate(length(min = 1, max = 100))]
    name: String,
}

/// Parameters for duplicate_profile command
#[derive(Debug, Deserialize, Validate)]
struct DuplicateProfileParams {
    #[validate(length(min = 1, max = 100))]
    src_name: String,
    #[validate(length(min = 1, max = 100))]
    dest_name: String,
}

/// Parameters for rename_profile command
#[derive(Debug, Deserialize, Validate)]
struct RenameProfileParams {
    #[validate(length(min = 1, max = 100))]
    old_name: String,
    #[validate(length(min = 1, max = 100))]
    new_name: String,
}

/// Parameters for get_profile_config query
#[derive(Debug, Deserialize, Validate)]
struct GetProfileConfigParams {
    #[validate(length(min = 1, max = 100))]
    name: String,
}

/// Parameters for set_profile_config command
#[derive(Debug, Deserialize, Validate)]
struct SetProfileConfigParams {
    #[validate(length(min = 1, max = 100))]
    name: String,
    #[validate(length(min = 1, max = 1048576))] // 1MB limit
    source: String,
}

/// Profile information returned by RPC methods
#[typeshare]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRpcInfo {
    pub name: String,
    #[typeshare(serialized_as = "number")]
    pub layer_count: usize,
    pub active: bool,
    #[typeshare(serialized_as = "number")]
    pub modified_at_secs: u64,
}

/// Activation result returned by activate_profile
#[typeshare]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationRpcResult {
    pub success: bool,
    #[typeshare(serialized_as = "number")]
    pub compile_time_ms: u64,
    #[typeshare(serialized_as = "number")]
    pub reload_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Profile configuration returned by get_profile_config
#[typeshare]
#[derive(Debug, Serialize)]
pub struct ProfileConfigRpc {
    pub name: String,
    pub source: String,
}

/// Validate profile name to prevent path traversal attacks
///
/// This delegates to `crate::validation::profile_name::validate_profile_name`
/// and converts the result to an RpcError.
fn validate_profile_name(name: &str) -> Result<(), RpcError> {
    validation::profile_name::validate_profile_name(name)
        .map_err(|e| RpcError::invalid_params(e.to_string()))
}

/// Get all profiles
pub async fn get_profiles(
    profile_service: &ProfileService,
    params: Value,
) -> Result<Value, RpcError> {
    // Validate params (should be empty object or null)
    let _params: Option<GetProfilesParams> = if params.is_null() {
        None
    } else {
        Some(
            serde_json::from_value(params)
                .map_err(|e| RpcError::invalid_params(format!("Invalid parameters: {}", e)))?,
        )
    };

    log::debug!("RPC: get_profiles");

    // Call profile service
    let profiles = profile_service
        .list_profiles()
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, format!("Failed to list profiles: {}", e)))?;

    // Convert to RPC format
    let rpc_profiles: Vec<ProfileRpcInfo> = profiles
        .iter()
        .map(|p| ProfileRpcInfo {
            name: p.name.clone(),
            layer_count: p.layer_count,
            active: p.active,
            modified_at_secs: p
                .modified_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
        .collect();

    serde_json::to_value(rpc_profiles)
        .map_err(|e| RpcError::internal_error(format!("Serialization failed: {}", e)))
}

/// Create a new profile
pub async fn create_profile(
    profile_service: &ProfileService,
    params: Value,
) -> Result<Value, RpcError> {
    let params: CreateProfileParams = serde_json::from_value(params)
        .map_err(|e| RpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

    // Validate input parameters
    params
        .validate()
        .map_err(|e| RpcError::invalid_params(format!("Validation failed: {}", e)))?;

    log::info!("RPC: create_profile name={}", params.name);

    // Validate profile name
    validate_profile_name(&params.name)?;

    // Parse template
    let template = match params.template.as_str() {
        "blank" => ProfileTemplate::Blank,
        "simple_remap" => ProfileTemplate::SimpleRemap,
        "capslock_escape" => ProfileTemplate::CapslockEscape,
        "vim_navigation" => ProfileTemplate::VimNavigation,
        "gaming" => ProfileTemplate::Gaming,
        _ => {
            return Err(RpcError::invalid_params(format!(
                "Invalid template: {}. Valid templates: blank, simple_remap, capslock_escape, vim_navigation, gaming",
                params.template
            )))
        }
    };

    // Call profile service
    let profile_info = profile_service
        .create_profile(&params.name, template)
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, format!("Failed to create profile: {}", e)))?;

    // Convert to RPC format
    let rpc_info = ProfileRpcInfo {
        name: profile_info.name,
        layer_count: profile_info.layer_count,
        active: profile_info.active,
        modified_at_secs: profile_info
            .modified_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    serde_json::to_value(rpc_info)
        .map_err(|e| RpcError::internal_error(format!("Serialization failed: {}", e)))
}

/// Activate a profile
pub async fn activate_profile(state: &AppState, params: Value) -> Result<Value, RpcError> {
    let params: ActivateProfileParams = serde_json::from_value(params)
        .map_err(|e| RpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

    // Validate input parameters
    params
        .validate()
        .map_err(|e| RpcError::invalid_params(format!("Validation failed: {}", e)))?;

    log::info!("RPC: activate_profile name={}", params.name);

    // Validate profile name
    validate_profile_name(&params.name)?;

    // Call profile service
    let result = state
        .profile_service
        .activate_profile(&params.name)
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, format!("Failed to activate profile: {}", e)))?;

    // Update shared daemon state if available (Windows hot-reload)
    // This signals the daemon event loop to detect the profile change and reload
    if let Some(daemon_state) = &state.daemon_state {
        daemon_state.set_active_profile(Some(params.name.clone()));
        log::debug!(
            "Updated daemon shared state with active profile: {}",
            params.name
        );
    }

    // Broadcast event to WebSocket subscribers
    let event = ServerMessage::Event {
        channel: "profiles".to_string(),
        data: serde_json::json!({
            "action": "activated",
            "profile": params.name
        }),
    };
    if let Err(e) = state.event_broadcaster.send(event) {
        log::warn!("Failed to broadcast profile activated event: {}", e);
    }

    // Convert to RPC format
    let rpc_result = ActivationRpcResult {
        success: result.success,
        compile_time_ms: result.compile_time_ms,
        reload_time_ms: result.reload_time_ms,
        error: result.error,
    };

    serde_json::to_value(rpc_result)
        .map_err(|e| RpcError::internal_error(format!("Serialization failed: {}", e)))
}

/// Delete a profile
pub async fn delete_profile(
    profile_service: &ProfileService,
    params: Value,
) -> Result<Value, RpcError> {
    let params: DeleteProfileParams = serde_json::from_value(params)
        .map_err(|e| RpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

    // Validate input parameters
    params
        .validate()
        .map_err(|e| RpcError::invalid_params(format!("Validation failed: {}", e)))?;

    log::info!("RPC: delete_profile name={}", params.name);

    // Validate profile name
    validate_profile_name(&params.name)?;

    // Call profile service
    profile_service
        .delete_profile(&params.name)
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, format!("Failed to delete profile: {}", e)))?;

    // Return success
    Ok(serde_json::json!({
        "deleted": true,
        "name": params.name
    }))
}

/// Duplicate a profile
pub async fn duplicate_profile(
    profile_service: &ProfileService,
    params: Value,
) -> Result<Value, RpcError> {
    let params: DuplicateProfileParams = serde_json::from_value(params)
        .map_err(|e| RpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

    // Validate input parameters
    params
        .validate()
        .map_err(|e| RpcError::invalid_params(format!("Validation failed: {}", e)))?;

    log::info!(
        "RPC: duplicate_profile src={} dest={}",
        params.src_name,
        params.dest_name
    );

    // Validate profile names
    validate_profile_name(&params.src_name)?;
    validate_profile_name(&params.dest_name)?;

    // Call profile service
    let profile_info = profile_service
        .duplicate_profile(&params.src_name, &params.dest_name)
        .await
        .map_err(|e| {
            RpcError::new(
                INTERNAL_ERROR,
                format!("Failed to duplicate profile: {}", e),
            )
        })?;

    // Convert to RPC format
    let rpc_info = ProfileRpcInfo {
        name: profile_info.name,
        layer_count: profile_info.layer_count,
        active: profile_info.active,
        modified_at_secs: profile_info
            .modified_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    serde_json::to_value(rpc_info)
        .map_err(|e| RpcError::internal_error(format!("Serialization failed: {}", e)))
}

/// Rename a profile
pub async fn rename_profile(
    profile_service: &ProfileService,
    params: Value,
) -> Result<Value, RpcError> {
    let params: RenameProfileParams = serde_json::from_value(params)
        .map_err(|e| RpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

    // Validate input parameters
    params
        .validate()
        .map_err(|e| RpcError::invalid_params(format!("Validation failed: {}", e)))?;

    log::info!(
        "RPC: rename_profile old={} new={}",
        params.old_name,
        params.new_name
    );

    // Validate profile names
    validate_profile_name(&params.old_name)?;
    validate_profile_name(&params.new_name)?;

    // Call profile service
    let profile_info = profile_service
        .rename_profile(&params.old_name, &params.new_name)
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, format!("Failed to rename profile: {}", e)))?;

    // Convert to RPC format
    let rpc_info = ProfileRpcInfo {
        name: profile_info.name,
        layer_count: profile_info.layer_count,
        active: profile_info.active,
        modified_at_secs: profile_info
            .modified_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    serde_json::to_value(rpc_info)
        .map_err(|e| RpcError::internal_error(format!("Serialization failed: {}", e)))
}

/// Get profile configuration source code
pub async fn get_profile_config(
    profile_service: &ProfileService,
    params: Value,
) -> Result<Value, RpcError> {
    let params: GetProfileConfigParams = serde_json::from_value(params)
        .map_err(|e| RpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

    // Validate input parameters
    params
        .validate()
        .map_err(|e| RpcError::invalid_params(format!("Validation failed: {}", e)))?;

    log::debug!("RPC: get_profile_config name={}", params.name);

    // Validate profile name
    validate_profile_name(&params.name)?;

    // Call profile service
    let source = profile_service
        .get_profile_config(&params.name)
        .await
        .map_err(|e| {
            RpcError::new(
                INTERNAL_ERROR,
                format!("Failed to get profile config: {}", e),
            )
        })?;

    // Convert to RPC format
    let rpc_config = ProfileConfigRpc {
        name: params.name,
        source,
    };

    serde_json::to_value(rpc_config)
        .map_err(|e| RpcError::internal_error(format!("Serialization failed: {}", e)))
}

/// Set profile configuration source code
pub async fn set_profile_config(
    profile_service: &ProfileService,
    params: Value,
) -> Result<Value, RpcError> {
    let params: SetProfileConfigParams = serde_json::from_value(params)
        .map_err(|e| RpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

    // Validate input parameters
    params
        .validate()
        .map_err(|e| RpcError::invalid_params(format!("Validation failed: {}", e)))?;

    log::info!(
        "RPC: set_profile_config name={} source_length={}",
        params.name,
        params.source.len()
    );

    // Validate profile name
    validate_profile_name(&params.name)?;

    // Call profile service
    profile_service
        .set_profile_config(&params.name, &params.source)
        .await
        .map_err(|e| {
            RpcError::new(
                INTERNAL_ERROR,
                format!("Failed to set profile config: {}", e),
            )
        })?;

    // Return success
    Ok(serde_json::json!({
        "success": true,
        "name": params.name
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_profile_name_valid() {
        assert!(validate_profile_name("my-profile").is_ok());
        assert!(validate_profile_name("profile_123").is_ok());
        assert!(validate_profile_name("Test").is_ok());
    }

    #[test]
    fn test_validate_profile_name_empty() {
        assert!(validate_profile_name("").is_err());
    }

    #[test]
    fn test_validate_profile_name_path_traversal() {
        assert!(validate_profile_name("../etc/passwd").is_err());
        assert!(validate_profile_name("..").is_err());
        assert!(validate_profile_name("test/../profile").is_err());
    }

    #[test]
    fn test_validate_profile_name_path_separators() {
        assert!(validate_profile_name("test/profile").is_err());
        assert!(validate_profile_name("test\\profile").is_err());
        assert!(validate_profile_name("/root").is_err());
    }
}
