//! Profile Registry domain FFI implementation.
//!
//! Implements FFI exports for the revolutionary mapping profile registry,
//! exposing profile management operations to Flutter.
#![allow(unsafe_code)]

use crate::ffi::error::{FfiError, FfiResult};
use crate::registry::profile::{Profile, ProfileRegistry};
use std::ffi::{c_char, CStr, CString};

/// List all profile IDs.
///
/// Returns JSON array of profile IDs.
///
/// # Returns
/// * `Ok(Vec<String>)` - List of profile IDs
pub async fn list_profiles(registry: &ProfileRegistry) -> FfiResult<Vec<String>> {
    let profiles = registry.list_profiles().await;
    Ok(profiles)
}

/// Get a profile by ID.
///
/// Returns JSON serialized profile.
///
/// # Arguments
/// * `registry` - Profile registry
/// * `profile_id` - Profile ID to retrieve
///
/// # Returns
/// * `Ok(Profile)` on success
/// * `Err(FfiError)` if profile not found
pub async fn get_profile(registry: &ProfileRegistry, profile_id: &str) -> FfiResult<Profile> {
    let profile = registry
        .get_profile(profile_id)
        .await
        .map_err(|e| FfiError::not_found(format!("Profile '{}': {}", profile_id, e)))?;

    // Return cloned profile (Arc is not FFI-safe, need to clone)
    Ok((*profile).clone())
}

/// Save a profile.
///
/// # Arguments
/// * `registry` - Profile registry
/// * `profile_json` - JSON string representing the profile
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(FfiError)` if validation fails or serialization error
pub async fn save_profile(registry: &ProfileRegistry, profile_json: &str) -> FfiResult<()> {
    // Parse JSON to Profile
    let profile: Profile = serde_json::from_str(profile_json)
        .map_err(|e| FfiError::deserialization_failed(format!("Invalid profile JSON: {}", e)))?;

    // Save profile
    registry
        .save_profile(&profile)
        .await
        .map_err(|e| FfiError::invalid_input(format!("Failed to save profile: {}", e)))?;

    Ok(())
}

/// Delete a profile by ID.
///
/// # Arguments
/// * `registry` - Profile registry
/// * `profile_id` - Profile ID to delete
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(FfiError)` if profile not found
pub async fn delete_profile(registry: &ProfileRegistry, profile_id: &str) -> FfiResult<()> {
    registry
        .delete_profile(profile_id)
        .await
        .map_err(|e| FfiError::not_found(format!("Profile '{}': {}", profile_id, e)))?;

    Ok(())
}

/// Find profiles compatible with a given layout type.
///
/// # Arguments
/// * `registry` - Profile registry
/// * `layout_type` - Layout type ("standard", "matrix", or "split")
///
/// # Returns
/// * `Ok(Vec<Profile>)` - List of compatible profiles
/// * `Err(FfiError)` if layout_type is invalid
pub async fn find_compatible_profiles(
    registry: &ProfileRegistry,
    layout_type: &str,
) -> FfiResult<Vec<Profile>> {
    use crate::registry::profile::LayoutType;

    // Parse layout type
    let layout = match layout_type {
        "standard" => LayoutType::Standard,
        "matrix" => LayoutType::Matrix,
        "split" => LayoutType::Split,
        _ => {
            return Err(FfiError::invalid_input(format!(
                "Invalid layout type: '{}'. Must be 'standard', 'matrix', or 'split'",
                layout_type
            )))
        }
    };

    let profiles = registry.find_compatible_profiles(&layout).await;

    // Clone profiles from Arc
    let cloned_profiles = profiles.iter().map(|p| (**p).clone()).collect();

    Ok(cloned_profiles)
}

// C-ABI exports with panic guards

/// List all profile IDs.
///
/// Returns JSON string: `ok:<json>` or `error:<message>`.
///
/// # Safety
/// This function catches panics and returns an error string if a panic occurs.
#[no_mangle]
pub extern "C" fn keyrx_profile_registry_list_profiles() -> *mut c_char {
    std::panic::catch_unwind(|| {
        // TODO: This needs access to the ProfileRegistry instance from the engine
        // For now, return an error indicating this needs integration
        let error_msg = "error:ProfileRegistry not yet integrated with FFI context";
        CString::new(error_msg)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_profile_registry_list_profiles")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
}

/// Get a profile by ID.
///
/// Returns JSON string: `ok:<profile_json>` or `error:<message>`.
///
/// # Arguments
/// * `profile_id` - Profile ID to retrieve
///
/// # Safety
/// * `profile_id` must be a valid null-terminated C string
/// * This function catches panics and returns an error string if a panic occurs
#[no_mangle]
pub unsafe extern "C" fn keyrx_profile_registry_get_profile(
    profile_id: *const c_char,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        if profile_id.is_null() {
            return CString::new("error:profile_id is null")
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut());
        }

        let id_str = match CStr::from_ptr(profile_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                return CString::new("error:profile_id is not valid UTF-8")
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        };

        // TODO: This needs access to the ProfileRegistry instance from the engine
        // For now, return an error indicating this needs integration
        let error_msg = format!(
            "error:ProfileRegistry not yet integrated (would get profile {})",
            id_str
        );
        CString::new(error_msg)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_profile_registry_get_profile")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
}

/// Save a profile.
///
/// Returns `ok:` or `error:<message>`.
///
/// # Arguments
/// * `profile_json` - JSON string representing the profile
///
/// # Safety
/// * `profile_json` must be a valid null-terminated C string
/// * This function catches panics and returns an error string if a panic occurs
#[no_mangle]
pub unsafe extern "C" fn keyrx_profile_registry_save_profile(
    profile_json: *const c_char,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        if profile_json.is_null() {
            return CString::new("error:profile_json is null")
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut());
        }

        let json_str = match CStr::from_ptr(profile_json).to_str() {
            Ok(s) => s,
            Err(_) => {
                return CString::new("error:profile_json is not valid UTF-8")
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        };

        // Validate JSON can be parsed
        match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(_) => {
                // TODO: This needs access to the ProfileRegistry instance from the engine
                // For now, return an error indicating this needs integration
                let error_msg = "error:ProfileRegistry not yet integrated (would save profile)";
                CString::new(error_msg)
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
            Err(e) => {
                let error_msg = format!("error:Invalid JSON: {}", e);
                CString::new(error_msg)
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        }
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_profile_registry_save_profile")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
}

/// Delete a profile by ID.
///
/// Returns `ok:` or `error:<message>`.
///
/// # Arguments
/// * `profile_id` - Profile ID to delete
///
/// # Safety
/// * `profile_id` must be a valid null-terminated C string
/// * This function catches panics and returns an error string if a panic occurs
#[no_mangle]
pub unsafe extern "C" fn keyrx_profile_registry_delete_profile(
    profile_id: *const c_char,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        if profile_id.is_null() {
            return CString::new("error:profile_id is null")
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut());
        }

        let id_str = match CStr::from_ptr(profile_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                return CString::new("error:profile_id is not valid UTF-8")
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        };

        // TODO: This needs access to the ProfileRegistry instance from the engine
        // For now, return an error indicating this needs integration
        let error_msg = format!(
            "error:ProfileRegistry not yet integrated (would delete profile {})",
            id_str
        );
        CString::new(error_msg)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_profile_registry_delete_profile")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
}

/// Find profiles compatible with a given layout type.
///
/// Returns JSON string: `ok:<json>` or `error:<message>`.
///
/// # Arguments
/// * `layout_type` - Layout type ("standard", "matrix", or "split")
///
/// # Safety
/// * `layout_type` must be a valid null-terminated C string
/// * This function catches panics and returns an error string if a panic occurs
#[no_mangle]
pub unsafe extern "C" fn keyrx_profile_registry_find_compatible_profiles(
    layout_type: *const c_char,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        if layout_type.is_null() {
            return CString::new("error:layout_type is null")
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut());
        }

        let type_str = match CStr::from_ptr(layout_type).to_str() {
            Ok(s) => s,
            Err(_) => {
                return CString::new("error:layout_type is not valid UTF-8")
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        };

        // Validate layout type
        if !["standard", "matrix", "split"].contains(&type_str) {
            let error_msg = format!(
                "error:Invalid layout type '{}'. Must be 'standard', 'matrix', or 'split'",
                type_str
            );
            return CString::new(error_msg)
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut());
        }

        // TODO: This needs access to the ProfileRegistry instance from the engine
        // For now, return an error indicating this needs integration
        let error_msg = format!(
            "error:ProfileRegistry not yet integrated (would find compatible profiles for {})",
            type_str
        );
        CString::new(error_msg)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_profile_registry_find_compatible_profiles")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::profile::{LayoutType, Profile};
    use tempfile::tempdir;

    fn test_profile(name: &str, layout: LayoutType) -> Profile {
        Profile::new(name, layout)
    }

    #[tokio::test]
    async fn test_list_profiles_empty() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let result = list_profiles(&registry).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_list_profiles_with_profiles() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let profile1 = test_profile("Profile 1", LayoutType::Matrix);
        let profile2 = test_profile("Profile 2", LayoutType::Standard);

        registry.save_profile(&profile1).await.unwrap();
        registry.save_profile(&profile2).await.unwrap();

        let result = list_profiles(&registry).await;
        assert!(result.is_ok());
        let profiles = result.unwrap();
        assert_eq!(profiles.len(), 2);
        assert!(profiles.contains(&profile1.id));
        assert!(profiles.contains(&profile2.id));
    }

    #[tokio::test]
    async fn test_get_profile_valid() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let profile = test_profile("Test Profile", LayoutType::Matrix);
        registry.save_profile(&profile).await.unwrap();

        let result = get_profile(&registry, &profile.id).await;
        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.id, profile.id);
        assert_eq!(loaded.name, profile.name);
    }

    #[tokio::test]
    async fn test_get_profile_not_found() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let result = get_profile(&registry, "nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_save_profile_valid() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let profile = test_profile("Test Profile", LayoutType::Standard);
        let json = serde_json::to_string(&profile).unwrap();

        let result = save_profile(&registry, &json).await;
        assert!(result.is_ok());

        // Verify it was saved
        let loaded = registry.get_profile(&profile.id).await.unwrap();
        assert_eq!(loaded.name, profile.name);
    }

    #[tokio::test]
    async fn test_save_profile_invalid_json() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let result = save_profile(&registry, "invalid json").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_profile_valid() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let profile = test_profile("Test Profile", LayoutType::Matrix);
        registry.save_profile(&profile).await.unwrap();

        let result = delete_profile(&registry, &profile.id).await;
        assert!(result.is_ok());

        // Verify it was deleted
        assert!(registry.get_profile(&profile.id).await.is_err());
    }

    #[tokio::test]
    async fn test_delete_profile_not_found() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let result = delete_profile(&registry, "nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_compatible_profiles_valid() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let matrix_profile = test_profile("Matrix Profile", LayoutType::Matrix);
        let standard_profile = test_profile("Standard Profile", LayoutType::Standard);

        registry.save_profile(&matrix_profile).await.unwrap();
        registry.save_profile(&standard_profile).await.unwrap();

        let result = find_compatible_profiles(&registry, "matrix").await;
        assert!(result.is_ok());
        let compatible = result.unwrap();
        assert_eq!(compatible.len(), 1);
        assert_eq!(compatible[0].id, matrix_profile.id);
    }

    #[tokio::test]
    async fn test_find_compatible_profiles_invalid_layout() {
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let result = find_compatible_profiles(&registry, "invalid").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_c_api_null_profile_id() {
        unsafe {
            let result = keyrx_profile_registry_get_profile(std::ptr::null());
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            assert!(msg.starts_with("error:"));
            drop(CString::from_raw(result));
        }
    }

    #[test]
    fn test_c_api_null_profile_json() {
        unsafe {
            let result = keyrx_profile_registry_save_profile(std::ptr::null());
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            assert!(msg.starts_with("error:"));
            drop(CString::from_raw(result));
        }
    }

    #[test]
    fn test_c_api_invalid_json() {
        unsafe {
            let invalid_json = CString::new("not valid json").unwrap();
            let result = keyrx_profile_registry_save_profile(invalid_json.as_ptr());
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            assert!(msg.contains("Invalid JSON"));
            drop(CString::from_raw(result));
        }
    }

    #[test]
    fn test_c_api_null_layout_type() {
        unsafe {
            let result = keyrx_profile_registry_find_compatible_profiles(std::ptr::null());
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            assert!(msg.starts_with("error:"));
            drop(CString::from_raw(result));
        }
    }

    #[test]
    fn test_c_api_invalid_layout_type() {
        unsafe {
            let invalid_layout = CString::new("invalid").unwrap();
            let result = keyrx_profile_registry_find_compatible_profiles(invalid_layout.as_ptr());
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            assert!(msg.contains("Invalid layout type"));
            drop(CString::from_raw(result));
        }
    }

    #[test]
    fn test_c_api_valid_layout_type() {
        unsafe {
            let layout = CString::new("matrix").unwrap();
            let result = keyrx_profile_registry_find_compatible_profiles(layout.as_ptr());
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            // Should indicate integration needed, not invalid layout
            assert!(msg.contains("not yet integrated"));
            drop(CString::from_raw(result));
        }
    }
}
