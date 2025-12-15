//! Complex FFI Types Example
//!
//! This example demonstrates FFI functions with complex custom types,
//! enums, nested structures, and comprehensive error handling.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Custom error type for profile operations.
#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("Profile not found: {0}")]
    NotFound(String),

    #[error("Invalid profile data: {0}")]
    InvalidData(String),

    #[error("Profile already exists: {0}")]
    AlreadyExists(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Profile state enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileState {
    /// Profile is inactive.
    Inactive,
    /// Profile is active and processing events.
    Active,
    /// Profile is paused.
    Paused,
    /// Profile encountered an error.
    Error { message: String },
}

/// Device information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub connected: bool,
}

/// Profile configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub device: DeviceInfo,
    pub state: ProfileState,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Create a new profile with the given configuration.
///
/// This demonstrates:
/// - Complex custom types as parameters
/// - Custom error types
/// - Nested structures
/// - Optional fields
///
/// # Arguments
///
/// * `config` - The profile configuration
///
/// # Returns
///
/// The created profile configuration with generated timestamps.
///
/// # Errors
///
/// Returns `ProfileError::AlreadyExists` if a profile with the same ID exists.
/// Returns `ProfileError::InvalidData` if the configuration is invalid.
// #[ffi_export(domain = "profile", version = "1.0")]
pub fn create_profile(config: ProfileConfig) -> Result<ProfileConfig, ProfileError> {
    // Validate configuration
    if config.id.is_empty() {
        return Err(ProfileError::InvalidData("ID cannot be empty".to_string()));
    }

    if config.name.is_empty() {
        return Err(ProfileError::InvalidData("Name cannot be empty".to_string()));
    }

    // In a real implementation, check if profile already exists
    // For this example, we just return the config with current timestamps
    let mut result = config;
    result.created_at = current_timestamp();
    result.updated_at = result.created_at;
    result.state = ProfileState::Inactive;

    Ok(result)
}

/// Get a profile by ID.
///
/// # Arguments
///
/// * `profile_id` - The profile ID to look up
///
/// # Returns
///
/// The profile configuration if found.
///
/// # Errors
///
/// Returns `ProfileError::NotFound` if the profile doesn't exist.
// #[ffi_export(domain = "profile", version = "1.0")]
pub fn get_profile(profile_id: &str) -> Result<ProfileConfig, ProfileError> {
    if profile_id.is_empty() {
        return Err(ProfileError::InvalidData("ID cannot be empty".to_string()));
    }

    // In a real implementation, look up the profile
    // For this example, return a mock profile
    Ok(ProfileConfig {
        id: profile_id.to_string(),
        name: format!("Profile {}", profile_id),
        description: Some("Example profile".to_string()),
        device: DeviceInfo {
            id: "device-001".to_string(),
            name: "Example Device".to_string(),
            vendor_id: 0x1234,
            product_id: 0x5678,
            connected: true,
        },
        state: ProfileState::Active,
        created_at: current_timestamp() - 3600,
        updated_at: current_timestamp(),
    })
}

/// Update a profile's state.
///
/// # Arguments
///
/// * `profile_id` - The profile ID to update
/// * `new_state` - The new state to set
///
/// # Errors
///
/// Returns `ProfileError::NotFound` if the profile doesn't exist.
// #[ffi_export(domain = "profile", version = "1.0")]
pub fn update_profile_state(
    profile_id: &str,
    new_state: ProfileState,
) -> Result<ProfileConfig, ProfileError> {
    let mut profile = get_profile(profile_id)?;
    profile.state = new_state;
    profile.updated_at = current_timestamp();
    Ok(profile)
}

/// List all profiles with optional filtering.
///
/// # Arguments
///
/// * `state_filter` - Optional state to filter by
///
/// # Returns
///
/// A vector of all profiles matching the filter.
// #[ffi_export(domain = "profile", version = "1.0")]
pub fn list_profiles(state_filter: Option<ProfileState>) -> Result<Vec<ProfileConfig>, ProfileError> {
    // In a real implementation, query the profile store
    // For this example, return mock data
    let all_profiles = vec![
        create_mock_profile("profile-1", ProfileState::Active),
        create_mock_profile("profile-2", ProfileState::Inactive),
        create_mock_profile("profile-3", ProfileState::Paused),
    ];

    let filtered = match state_filter {
        Some(state) => all_profiles
            .into_iter()
            .filter(|p| p.state == state)
            .collect(),
        None => all_profiles,
    };

    Ok(filtered)
}

// Helper functions

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn create_mock_profile(id: &str, state: ProfileState) -> ProfileConfig {
    ProfileConfig {
        id: id.to_string(),
        name: format!("Profile {}", id),
        description: None,
        device: DeviceInfo {
            id: format!("device-{}", id),
            name: "Mock Device".to_string(),
            vendor_id: 0x1234,
            product_id: 0x5678,
            connected: true,
        },
        state,
        created_at: current_timestamp() - 7200,
        updated_at: current_timestamp(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_profile_success() {
        let config = ProfileConfig {
            id: "test-profile".to_string(),
            name: "Test Profile".to_string(),
            description: Some("A test profile".to_string()),
            device: DeviceInfo {
                id: "device-001".to_string(),
                name: "Test Device".to_string(),
                vendor_id: 0x1234,
                product_id: 0x5678,
                connected: true,
            },
            state: ProfileState::Inactive,
            created_at: 0,
            updated_at: 0,
        };

        let result = create_profile(config);
        assert!(result.is_ok());

        let profile = result.unwrap();
        assert_eq!(profile.id, "test-profile");
        assert!(profile.created_at > 0);
        assert_eq!(profile.state, ProfileState::Inactive);
    }

    #[test]
    fn test_create_profile_empty_id() {
        let config = ProfileConfig {
            id: "".to_string(),
            name: "Test".to_string(),
            description: None,
            device: DeviceInfo {
                id: "device-001".to_string(),
                name: "Test Device".to_string(),
                vendor_id: 0x1234,
                product_id: 0x5678,
                connected: true,
            },
            state: ProfileState::Inactive,
            created_at: 0,
            updated_at: 0,
        };

        let result = create_profile(config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProfileError::InvalidData(_)));
    }

    #[test]
    fn test_get_profile() {
        let result = get_profile("test-profile");
        assert!(result.is_ok());

        let profile = result.unwrap();
        assert_eq!(profile.id, "test-profile");
    }

    #[test]
    fn test_update_profile_state() {
        let result = update_profile_state("test-profile", ProfileState::Paused);
        assert!(result.is_ok());

        let profile = result.unwrap();
        assert_eq!(profile.state, ProfileState::Paused);
    }

    #[test]
    fn test_list_profiles_no_filter() {
        let result = list_profiles(None);
        assert!(result.is_ok());

        let profiles = result.unwrap();
        assert_eq!(profiles.len(), 3);
    }

    #[test]
    fn test_list_profiles_with_filter() {
        let result = list_profiles(Some(ProfileState::Active));
        assert!(result.is_ok());

        let profiles = result.unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].state, ProfileState::Active);
    }
}
