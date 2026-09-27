//! Configuration file loading module.
//!
//! Loads and validates `.krx` binary configuration files into an owned
//! [`ConfigRoot`]. The file bytes are dropped after deserialization, so loading
//! repeatedly (hot-reload on every profile activation) does not leak memory.

use std::path::Path;

use keyrx_core::config::ConfigRoot;

use crate::error::ConfigError;

/// Loads and validates a .krx configuration file.
///
/// This function:
/// 1. Reads the file from disk
/// 2. Validates the .krx file format (magic bytes, version, hash)
/// 3. Deserializes the configuration using rkyv
///
/// # Arguments
///
/// * `path` - Path to the .krx configuration file
///
/// # Returns
///
/// Returns the owned, validated ConfigRoot on success.
///
/// # Errors
///
/// Returns `ConfigError::FileNotFound` if:
/// - The file does not exist
///
/// Returns `ConfigError::Io` if:
/// - The process lacks read permissions
/// - An I/O error occurs while reading
///
/// Returns `ConfigError::ParseError` if:
/// - The file has invalid magic bytes (not a .krx file)
/// - The .krx format version is incompatible
/// - The hash does not match (data corruption)
/// - The rkyv archive structure is invalid
///
/// # Examples
///
/// ```ignore
/// use keyrx_daemon::config_loader::load_config;
///
/// // Load configuration from a file
/// let config = load_config("config.krx")?;
///
/// // Access devices (ArchivedVec requires .as_slice() for iteration)
/// for device in config.devices.as_slice() {
///     println!("Device pattern: {}", device.identifier.pattern);
/// }
/// # Ok::<(), keyrx_daemon::error::ConfigError>(())
/// ```
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ConfigRoot, ConfigError> {
    let path_ref = path.as_ref();

    // Check if file exists first for better error messages
    if !path_ref.exists() {
        return Err(ConfigError::FileNotFound {
            path: path_ref.to_path_buf(),
        });
    }

    // Read file bytes
    let bytes = std::fs::read(path_ref).map_err(ConfigError::Io)?;

    // Validate (magic, version, hash, rkyv structure), then copy out of the
    // archive so the bytes can be freed.
    let archived =
        keyrx_compiler::serialize::deserialize(&bytes).map_err(|e| ConfigError::ParseError {
            path: path_ref.to_path_buf(),
            reason: e.to_string(),
        })?;
    let config: ConfigRoot = match rkyv::Deserialize::deserialize(archived, &mut rkyv::Infallible) {
        Ok(config) => config,
        Err(infallible) => match infallible {},
    };

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_compiler::serialize::serialize;
    use keyrx_core::config::{
        DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
    };
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_config() -> ConfigRoot {
        ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: "Test Device".to_string(),
                },
                mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
            }],
            metadata: Metadata {
                compilation_timestamp: 1234567890,
                compiler_version: "1.0.0".to_string(),
                source_hash: "test_hash".to_string(),
            },
        }
    }

    #[test]
    fn test_load_valid_config() {
        // Create a valid .krx file
        let config = create_test_config();
        let bytes = serialize(&config).expect("Serialization failed");

        // Write to temporary file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(&bytes)
            .expect("Failed to write to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        // Load the configuration
        let result = load_config(temp_file.path());
        assert!(result.is_ok());

        let loaded = result.unwrap();
        assert_eq!(loaded.devices.len(), 1);
        assert_eq!(loaded.devices[0].mappings.len(), 1);
    }

    #[test]
    fn test_load_missing_file() {
        let result = load_config("/nonexistent/path/to/config.krx");
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::FileNotFound { .. })));
    }

    #[test]
    fn test_load_corrupted_magic() {
        // Create a file with invalid magic bytes
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(b"INVALID DATA")
            .expect("Failed to write to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let result = load_config(temp_file.path());
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::ParseError { .. })));
    }

    #[test]
    fn test_load_corrupted_hash() {
        // Create a valid .krx file
        let config = create_test_config();
        let mut bytes = serialize(&config).expect("Serialization failed");

        // Corrupt the hash (bytes 8-40 are the hash)
        bytes[8] = !bytes[8];

        // Write to temporary file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(&bytes)
            .expect("Failed to write to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        let result = load_config(temp_file.path());
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::ParseError { .. })));
    }
}
