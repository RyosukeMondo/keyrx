use crate::discovery::types::{
    default_schema_version, device_profiles_dir, DeviceId, DeviceProfile, ProfileSource,
    SCHEMA_VERSION,
};
use crate::validation::{
    SchemaError, SchemaRegistry, ValidationFailure, DEVICE_PROFILE_SCHEMA_NAME,
};
use chrono::Utc;
use serde_json::{Error as SerdeError, Value as JsonValue};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur while reading or writing device profiles.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to parse profile at {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: SerdeError,
    },
    #[error(
        "Profile schema mismatch at {path}: expected {expected}, found {found}. \
         Re-discovery required."
    )]
    SchemaMismatch {
        path: PathBuf,
        expected: u8,
        found: u8,
    },
    #[error("Profile validation failed at {path}: {source}")]
    Validation {
        path: PathBuf,
        #[source]
        source: ValidationFailure,
    },
}

/// Resolve the profile path for a given device.
pub fn profile_path(device_id: DeviceId) -> PathBuf {
    device_profiles_dir().join(device_id.to_filename())
}

/// Create a default profile for the given device.
pub fn default_profile_for(device_id: DeviceId) -> DeviceProfile {
    DeviceProfile {
        schema_version: default_schema_version(),
        vendor_id: device_id.vendor_id,
        product_id: device_id.product_id,
        name: None,
        discovered_at: Utc::now(),
        rows: 0,
        cols_per_row: Vec::new(),
        keymap: Default::default(),
        aliases: Default::default(),
        source: ProfileSource::Default,
    }
}

/// Read and validate a device profile from disk.
pub fn read_profile(device_id: DeviceId) -> Result<DeviceProfile, StorageError> {
    let path = profile_path(device_id);
    let data = fs::read_to_string(&path).map_err(|source| StorageError::Io {
        path: path.clone(),
        source,
    })?;

    let json_value: JsonValue =
        serde_json::from_str(&data).map_err(|source| StorageError::Parse {
            path: path.clone(),
            source,
        })?;

    if let Err(err) = validate_profile_schema(&path, &json_value) {
        log_profile_validation_error(&path, &err);

        if should_abort_profile_load(&err) {
            return Err(StorageError::Validation { path, source: err });
        }
    }

    let profile: DeviceProfile =
        serde_json::from_value(json_value).map_err(|source| StorageError::Parse {
            path: path.clone(),
            source,
        })?;

    validate_schema(profile, path)
}

/// Validate schema version and return a usable profile or error.
pub fn validate_schema(
    profile: DeviceProfile,
    path: PathBuf,
) -> Result<DeviceProfile, StorageError> {
    if profile.schema_version != SCHEMA_VERSION {
        return Err(StorageError::SchemaMismatch {
            path,
            expected: SCHEMA_VERSION,
            found: profile.schema_version,
        });
    }
    Ok(profile)
}

fn validate_profile_schema(path: &Path, json_value: &JsonValue) -> Result<(), ValidationFailure> {
    tracing::debug!(
        service = "keyrx",
        event = "profile_schema_validation_started",
        component = "discovery",
        path = %path.display(),
        "Validating device profile against schema"
    );

    SchemaRegistry::global().validate(DEVICE_PROFILE_SCHEMA_NAME, json_value)
}

fn log_profile_validation_error(path: &Path, error: &ValidationFailure) {
    match error {
        ValidationFailure::Invalid { issues, .. } => {
            tracing::error!(
                service = "keyrx",
                event = "profile_schema_invalid",
                component = "discovery",
                path = %path.display(),
                issue_count = issues.len(),
                "Profile schema validation failed"
            );

            for issue in issues {
                tracing::error!(
                    service = "keyrx",
                    event = "profile_schema_issue",
                    component = "discovery",
                    path = %path.display(),
                    instance_path = %issue.instance_path,
                    schema_path = %issue.schema_path,
                    message = %issue.message,
                    "Schema validation error"
                );
            }
        }
        ValidationFailure::Schema(SchemaError::Missing { .. }) => {
            tracing::warn!(
                service = "keyrx",
                event = "profile_schema_missing",
                component = "discovery",
                path = %path.display(),
                "Profile schema not found, skipping schema validation"
            );
        }
        ValidationFailure::Schema(SchemaError::Serialize { source, .. }) => {
            tracing::warn!(
                service = "keyrx",
                event = "profile_schema_serialize_error",
                component = "discovery",
                path = %path.display(),
                error = %source,
                "Failed to serialize profile schema for validation, continuing without schema check"
            );
        }
        ValidationFailure::Schema(SchemaError::Compile { message, .. }) => {
            tracing::error!(
                service = "keyrx",
                event = "profile_schema_compile_error",
                component = "discovery",
                path = %path.display(),
                error = %message,
                "Failed to compile profile schema, continuing without schema check"
            );
        }
    }
}

fn should_abort_profile_load(error: &ValidationFailure) -> bool {
    matches!(error, ValidationFailure::Invalid { .. })
}

/// Atomically write a device profile to disk (temp file + rename).
pub fn write_profile(profile: &DeviceProfile) -> Result<PathBuf, StorageError> {
    let path = profile_path(DeviceId::new(profile.vendor_id, profile.product_id));

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|source| StorageError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
    }

    let serialized = serde_json::to_vec_pretty(profile).map_err(|source| StorageError::Parse {
        path: path.clone(),
        source,
    })?;

    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, serialized).map_err(|source| StorageError::Io {
        path: tmp_path.clone(),
        source,
    })?;

    fs::rename(&tmp_path, &path).map_err(|source| {
        // Best-effort cleanup of the temp file; ignore failure.
        let _ = fs::remove_file(&tmp_path);
        StorageError::Io {
            path: path.clone(),
            source,
        }
    })?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::test_utils::config_env_lock;
    use crate::validation::ValidationFailure;
    use serial_test::serial;
    use std::fs;
    use std::time::Duration;
    use tempfile::tempdir;

    fn with_temp_config<F: FnOnce()>(f: F) {
        let _guard = config_env_lock().lock().unwrap();
        let temp = tempdir().unwrap();
        let prev_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        let prev_home = std::env::var("HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", temp.path());
        std::env::remove_var("HOME");
        f();
        if let Some(xdg) = prev_xdg {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        if let Some(home) = prev_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    #[serial]
    fn roundtrip_write_and_read_profile() {
        with_temp_config(|| {
            let id = DeviceId::new(0x1234, 0x5678);
            let mut profile = default_profile_for(id);
            profile.rows = 1;
            profile.cols_per_row = vec![1];

            let path = write_profile(&profile).expect("write should succeed");
            assert!(path.exists());

            let loaded = read_profile(id).expect("read should succeed");
            assert_eq!(loaded.schema_version, SCHEMA_VERSION);
            assert_eq!(loaded.vendor_id, profile.vendor_id);
            assert_eq!(loaded.product_id, profile.product_id);
            assert_eq!(loaded.rows, 1);
            assert_eq!(loaded.cols_per_row, vec![1]);
        });
    }

    #[test]
    #[serial]
    fn read_profile_schema_mismatch() {
        with_temp_config(|| {
            let id = DeviceId::new(0x1111, 0x2222);
            let path = profile_path(id);
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).unwrap();
            }
            let bad_profile = format!(
                r#"{{
                    "schema_version": {},
                    "vendor_id": 4369,
                    "product_id": 8738,
                    "discovered_at": "2025-01-01T00:00:00Z",
                    "rows": 0,
                    "cols_per_row": [],
                    "keymap": {{}},
                    "aliases": {{}},
                    "source": "Default"
                }}"#,
                SCHEMA_VERSION.saturating_sub(1)
            );
            fs::write(&path, bad_profile).unwrap();

            let err = read_profile(id).expect_err("should fail on schema mismatch");
            assert!(matches!(err, StorageError::SchemaMismatch { .. }));
        });
    }

    #[test]
    #[serial]
    fn parse_error_for_corrupt_file() {
        with_temp_config(|| {
            let id = DeviceId::new(0xAAAA, 0xBBBB);
            let path = profile_path(id);
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).unwrap();
            }
            fs::write(&path, "not-json").unwrap();
            let err = read_profile(id).expect_err("should fail to parse");
            assert!(matches!(err, StorageError::Parse { .. }));
        });
    }

    #[test]
    #[serial]
    fn validation_error_for_invalid_profile() {
        with_temp_config(|| {
            let id = DeviceId::new(0xCAFE, 0xBABE);
            let path = profile_path(id);
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).unwrap();
            }
            let invalid_profile = r#"
                {
                    "schema_version": 1,
                    "vendor_id": 51966,
                    "product_id": 47806,
                    "discovered_at": "2025-01-01T00:00:00Z",
                    "rows": "two",
                    "cols_per_row": [1],
                    "keymap": {},
                    "aliases": {},
                    "source": "Default"
                }
            "#;
            fs::write(&path, invalid_profile).unwrap();

            let err = read_profile(id).expect_err("should fail validation");
            match err {
                StorageError::Validation { source, .. } => {
                    assert!(matches!(source, ValidationFailure::Invalid { .. }));
                }
                other => panic!("unexpected error: {:?}", other),
            }
        });
    }

    #[test]
    #[serial]
    fn write_is_atomic_with_temp_file() {
        with_temp_config(|| {
            let id = DeviceId::new(0x0A0A, 0x0B0B);
            let profile = default_profile_for(id);
            let path = profile_path(id);
            let tmp = path.with_extension("json.tmp");

            // Ensure temp file is removed after successful rename.
            let written = write_profile(&profile).expect("write should succeed");
            assert_eq!(written, path);
            std::thread::sleep(Duration::from_millis(10));
            assert!(!tmp.exists(), "temp file should be cleaned up");
        });
    }
}
