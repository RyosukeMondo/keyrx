//! Migration domain FFI implementation.
//!
//! Implements FFI exports for the profile migration system,
//! exposing migration operations to Flutter.
#![allow(unsafe_code)]

use crate::ffi::error::{FfiError, FfiResult};
use crate::migration::{MigrationReport, MigrationV1ToV2};
use crate::registry::profile::ProfileRegistry;
use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;

/// Run migration from V1 to V2 profiles.
///
/// Returns JSON serialized MigrationReport.
///
/// # Arguments
/// * `old_profiles_dir` - Directory containing old V1 profiles
/// * `profile_registry` - Profile registry for saving new V2 profiles
/// * `create_backup` - Whether to create backups before migration
///
/// # Returns
/// * `Ok(MigrationReport)` with migration results
/// * `Err(FfiError)` if migration fails critically
pub async fn run_migration(
    old_profiles_dir: PathBuf,
    profile_registry: ProfileRegistry,
    create_backup: bool,
) -> FfiResult<MigrationReport> {
    let migrator = MigrationV1ToV2::new(old_profiles_dir, profile_registry, create_backup);

    let report = migrator
        .migrate()
        .await
        .map_err(|e| FfiError::invalid_input(format!("Migration failed: {}", e)))?;

    Ok(report)
}

/// Check if migration is needed.
///
/// Returns true if old profiles directory exists and contains profiles.
///
/// # Arguments
/// * `old_profiles_dir` - Directory to check for old V1 profiles
///
/// # Returns
/// * `Ok(bool)` - true if migration needed
pub fn check_migration_needed(old_profiles_dir: PathBuf) -> FfiResult<bool> {
    if !old_profiles_dir.exists() {
        return Ok(false);
    }

    // Check if directory contains any JSON files
    let entries = std::fs::read_dir(&old_profiles_dir)
        .map_err(|e| FfiError::internal(format!("Failed to read directory: {}", e)))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| FfiError::internal(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            return Ok(true);
        }
    }

    Ok(false)
}

// C-ABI exports with panic guards

/// Run migration from V1 to V2 profiles.
///
/// Returns JSON string: `ok:<migration_report_json>` or `error:<message>`.
///
/// # Arguments
/// * `old_profiles_dir` - Path to directory containing old V1 profiles
/// * `new_profiles_dir` - Path to directory for new V2 profiles
/// * `create_backup` - Whether to create backup (0 = no, 1 = yes)
///
/// # Safety
/// * `old_profiles_dir` and `new_profiles_dir` must be valid null-terminated C strings
/// * This function catches panics and returns an error string if a panic occurs
#[no_mangle]
pub unsafe extern "C" fn keyrx_migration_run(
    old_profiles_dir: *const c_char,
    new_profiles_dir: *const c_char,
    create_backup: i32,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        if old_profiles_dir.is_null() {
            return CString::new("error:old_profiles_dir is null")
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut());
        }

        if new_profiles_dir.is_null() {
            return CString::new("error:new_profiles_dir is null")
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut());
        }

        let old_dir_str = match CStr::from_ptr(old_profiles_dir).to_str() {
            Ok(s) => s,
            Err(_) => {
                return CString::new("error:old_profiles_dir is not valid UTF-8")
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        };

        let new_dir_str = match CStr::from_ptr(new_profiles_dir).to_str() {
            Ok(s) => s,
            Err(_) => {
                return CString::new("error:new_profiles_dir is not valid UTF-8")
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        };

        let old_dir = PathBuf::from(old_dir_str);
        let new_dir = PathBuf::from(new_dir_str);
        let backup = create_backup != 0;

        // Run migration in blocking context
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                let error_msg = format!("error:Failed to create runtime: {}", e);
                return CString::new(error_msg)
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut());
            }
        };

        let profile_registry = ProfileRegistry::with_directory(new_dir);
        let result = runtime.block_on(run_migration(old_dir, profile_registry, backup));

        match result {
            Ok(report) => {
                // Serialize report to JSON
                match serde_json::to_string(&report) {
                    Ok(json) => {
                        let response = format!("ok:{}", json);
                        CString::new(response)
                            .map(CString::into_raw)
                            .unwrap_or(std::ptr::null_mut())
                    }
                    Err(e) => {
                        let error_msg = format!("error:Failed to serialize report: {}", e);
                        CString::new(error_msg)
                            .map(CString::into_raw)
                            .unwrap_or(std::ptr::null_mut())
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("error:{}", e);
                CString::new(error_msg)
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        }
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_migration_run")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
}

/// Check if migration is needed.
///
/// Returns `ok:true` or `ok:false` or `error:<message>`.
///
/// # Arguments
/// * `old_profiles_dir` - Path to directory to check for old V1 profiles
///
/// # Safety
/// * `old_profiles_dir` must be a valid null-terminated C string
/// * This function catches panics and returns an error string if a panic occurs
#[no_mangle]
pub unsafe extern "C" fn keyrx_migration_check_needed(
    old_profiles_dir: *const c_char,
) -> *mut c_char {
    std::panic::catch_unwind(|| {
        if old_profiles_dir.is_null() {
            return CString::new("error:old_profiles_dir is null")
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut());
        }

        let dir_str = match CStr::from_ptr(old_profiles_dir).to_str() {
            Ok(s) => s,
            Err(_) => {
                return CString::new("error:old_profiles_dir is not valid UTF-8")
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        };

        let dir_path = PathBuf::from(dir_str);

        match check_migration_needed(dir_path) {
            Ok(needed) => {
                let response = if needed { "ok:true" } else { "ok:false" };
                CString::new(response)
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
            Err(e) => {
                let error_msg = format!("error:{}", e);
                CString::new(error_msg)
                    .map(CString::into_raw)
                    .unwrap_or(std::ptr::null_mut())
            }
        }
    })
    .unwrap_or_else(|_| {
        CString::new("error:panic in keyrx_migration_check_needed")
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::types::{DeviceProfile, PhysicalKey, ProfileSource};
    use serial_test::serial;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn create_test_device_profile() -> DeviceProfile {
        let mut keymap = HashMap::new();
        keymap.insert(
            30,
            PhysicalKey {
                scan_code: 30,
                row: 2,
                col: 0,
                alias: Some("A".to_string()),
            },
        );

        DeviceProfile {
            schema_version: 1,
            vendor_id: 0x1234,
            product_id: 0x5678,
            name: Some("Test Keyboard".to_string()),
            discovered_at: chrono::Utc::now(),
            rows: 6,
            cols_per_row: vec![15, 15, 15, 13, 12, 10],
            keymap,
            aliases: HashMap::new(),
            source: ProfileSource::Default,
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_run_migration_success() {
        let temp = tempdir().unwrap();
        let old_dir = temp.path().join("old_devices");
        let new_dir = temp.path().join("new_profiles");

        std::fs::create_dir_all(&old_dir).unwrap();
        std::fs::create_dir_all(&new_dir).unwrap();

        // Create test old profile
        let old_profile = create_test_device_profile();
        let profile_json = serde_json::to_string_pretty(&old_profile).unwrap();
        std::fs::write(old_dir.join("1234_5678.json"), profile_json).unwrap();

        let registry = ProfileRegistry::with_directory(new_dir);
        let result = run_migration(old_dir, registry, true).await;

        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.total_count, 1);
        assert_eq!(report.migrated_count, 1);
        assert_eq!(report.failed_count, 0);
    }

    #[test]
    fn test_check_migration_needed_with_profiles() {
        let temp = tempdir().unwrap();
        let old_dir = temp.path().join("old_devices");
        std::fs::create_dir_all(&old_dir).unwrap();

        // Create a test JSON file
        std::fs::write(old_dir.join("test.json"), "{}").unwrap();

        let result = check_migration_needed(old_dir);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_check_migration_needed_no_profiles() {
        let temp = tempdir().unwrap();
        let old_dir = temp.path().join("old_devices");
        std::fs::create_dir_all(&old_dir).unwrap();

        let result = check_migration_needed(old_dir);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_check_migration_needed_dir_not_exists() {
        let temp = tempdir().unwrap();
        let old_dir = temp.path().join("nonexistent");

        let result = check_migration_needed(old_dir);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_c_api_check_needed_null() {
        unsafe {
            let result = keyrx_migration_check_needed(std::ptr::null());
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            assert!(msg.starts_with("error:"));
            drop(CString::from_raw(result));
        }
    }

    #[test]
    fn test_c_api_check_needed_valid() {
        let temp = tempdir().unwrap();
        let old_dir = temp.path().join("old_devices");
        std::fs::create_dir_all(&old_dir).unwrap();

        unsafe {
            let dir_path = CString::new(old_dir.to_str().unwrap()).unwrap();
            let result = keyrx_migration_check_needed(dir_path.as_ptr());
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            assert_eq!(msg, "ok:false");
            drop(CString::from_raw(result));
        }
    }

    #[test]
    fn test_c_api_run_migration_null_old_dir() {
        let temp = tempdir().unwrap();
        let new_dir = temp.path().join("new_profiles");

        unsafe {
            let new_dir_path = CString::new(new_dir.to_str().unwrap()).unwrap();
            let result = keyrx_migration_run(std::ptr::null(), new_dir_path.as_ptr(), 1);
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            assert!(msg.starts_with("error:"));
            drop(CString::from_raw(result));
        }
    }

    #[test]
    fn test_c_api_run_migration_null_new_dir() {
        let temp = tempdir().unwrap();
        let old_dir = temp.path().join("old_devices");

        unsafe {
            let old_dir_path = CString::new(old_dir.to_str().unwrap()).unwrap();
            let result = keyrx_migration_run(old_dir_path.as_ptr(), std::ptr::null(), 1);
            assert!(!result.is_null());
            let c_str = CStr::from_ptr(result);
            let msg = c_str.to_str().unwrap();
            assert!(msg.starts_with("error:"));
            drop(CString::from_raw(result));
        }
    }
}
