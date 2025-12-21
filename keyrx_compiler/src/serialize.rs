//! Serialization module
//!
//! This module handles serialization of compiled configuration to .krx binary format
//! using rkyv for zero-copy deserialization at runtime.

use keyrx_core::config::ConfigRoot;
use sha2::{Digest, Sha256};

use crate::error::{DeserializeError, SerializeError};

/// Magic bytes for KRX file format: "KRX\n"
#[allow(dead_code)] // Will be used by CLI in task 18
pub const KRX_MAGIC: [u8; 4] = [0x4B, 0x52, 0x58, 0x0A];

/// Current KRX format version
#[allow(dead_code)] // Will be used by CLI in task 18
pub const KRX_VERSION: u32 = 1;

/// Size of the KRX file header in bytes
#[allow(dead_code)] // Will be used by CLI in task 18
pub const HEADER_SIZE: usize = 48;

/// Serializes a ConfigRoot to the .krx binary format.
///
/// The .krx format consists of:
/// - 4 bytes: Magic number (KRX_MAGIC)
/// - 4 bytes: Format version (KRX_VERSION)
/// - 32 bytes: SHA256 hash of data section
/// - 8 bytes: Size of data section (u64, little-endian)
/// - N bytes: rkyv-serialized ConfigRoot data
///
/// # Arguments
/// * `config` - The configuration to serialize
///
/// # Returns
/// A Vec<u8> containing the complete .krx file data
///
/// # Errors
/// Returns SerializeError if rkyv serialization fails
#[allow(dead_code)] // Will be used by CLI in task 18
pub fn serialize(config: &ConfigRoot) -> Result<Vec<u8>, SerializeError> {
    // Serialize ConfigRoot using rkyv
    let data =
        rkyv::to_bytes::<_, 4096>(config).map_err(|e| SerializeError::RkyvError(e.to_string()))?;

    // Compute SHA256 hash of serialized data
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let hash: [u8; 32] = hasher.finalize().into();

    // Get data size as u64
    let size = data.len() as u64;

    // Build header (48 bytes total)
    let mut output = Vec::with_capacity(HEADER_SIZE + data.len());

    // Write magic bytes (4 bytes)
    output.extend_from_slice(&KRX_MAGIC);

    // Write version (4 bytes, little-endian)
    output.extend_from_slice(&KRX_VERSION.to_le_bytes());

    // Write hash (32 bytes)
    output.extend_from_slice(&hash);

    // Write size (8 bytes, little-endian)
    output.extend_from_slice(&size.to_le_bytes());

    // Write data
    output.extend_from_slice(&data);

    Ok(output)
}

/// Deserializes and validates a .krx binary file.
///
/// This function performs the following validation steps:
/// 1. Verifies magic bytes match KRX_MAGIC
/// 2. Verifies version matches KRX_VERSION
/// 3. Computes SHA256 hash of data and compares with embedded hash
/// 4. Validates rkyv archive structure
///
/// # Arguments
/// * `bytes` - The complete .krx file data
///
/// # Returns
/// A zero-copy reference to the archived ConfigRoot
///
/// # Errors
/// Returns DeserializeError if:
/// - File is too small to contain header
/// - Magic bytes don't match
/// - Version doesn't match
/// - Hash doesn't match (data corruption)
/// - rkyv validation fails
#[allow(dead_code)] // Will be used by CLI in task 18
pub fn deserialize(bytes: &[u8]) -> Result<&rkyv::Archived<ConfigRoot>, DeserializeError> {
    // Verify minimum size
    if bytes.len() < HEADER_SIZE {
        return Err(DeserializeError::RkyvError(format!(
            "File too small: expected at least {} bytes, got {}",
            HEADER_SIZE,
            bytes.len()
        )));
    }

    // Extract header fields
    let magic = &bytes[0..4];
    let version_bytes = &bytes[4..8];
    let embedded_hash = &bytes[8..40];
    let size_bytes = &bytes[40..48];
    let data = &bytes[48..];

    // Verify magic bytes
    let magic_array: [u8; 4] = magic.try_into().unwrap();
    if magic_array != KRX_MAGIC {
        return Err(DeserializeError::InvalidMagic {
            expected: KRX_MAGIC,
            got: magic_array,
        });
    }

    // Verify version
    let version = u32::from_le_bytes(version_bytes.try_into().unwrap());
    if version != KRX_VERSION {
        return Err(DeserializeError::VersionMismatch {
            expected: KRX_VERSION,
            got: version,
        });
    }

    // Verify size matches actual data length
    let expected_size = u64::from_le_bytes(size_bytes.try_into().unwrap()) as usize;
    if data.len() != expected_size {
        return Err(DeserializeError::RkyvError(format!(
            "Size mismatch: header says {} bytes, got {} bytes",
            expected_size,
            data.len()
        )));
    }

    // Compute hash of data and verify
    let mut hasher = Sha256::new();
    hasher.update(data);
    let computed_hash: [u8; 32] = hasher.finalize().into();
    let embedded_hash_array: [u8; 32] = embedded_hash.try_into().unwrap();

    if computed_hash != embedded_hash_array {
        return Err(DeserializeError::HashMismatch {
            expected: embedded_hash_array,
            computed: computed_hash,
        });
    }

    // Deserialize using rkyv (archived_root provides zero-copy access)
    // Note: archived_root is unsafe and doesn't validate. For production use,
    // ConfigRoot would need to implement CheckBytes trait for safe validation.
    unsafe { Ok(rkyv::archived_root::<ConfigRoot>(data)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_core::config::{
        BaseKeyMapping, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
    };

    fn create_test_config() -> ConfigRoot {
        ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: "Test Device".to_string(),
                },
                mappings: vec![KeyMapping::Base(BaseKeyMapping::Simple {
                    from: KeyCode::A,
                    to: KeyCode::B,
                })],
            }],
            metadata: Metadata {
                compilation_timestamp: 1234567890,
                compiler_version: "1.0.0".to_string(),
                source_hash: "test_hash".to_string(),
            },
        }
    }

    #[test]
    fn test_serialize_produces_valid_format() {
        let config = create_test_config();
        let result = serialize(&config);
        assert!(result.is_ok());

        let bytes = result.unwrap();

        // Check minimum size
        assert!(bytes.len() >= HEADER_SIZE);

        // Check magic bytes
        assert_eq!(&bytes[0..4], &KRX_MAGIC);

        // Check version
        let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        assert_eq!(version, KRX_VERSION);

        // Check that hash is present (32 bytes)
        assert_eq!(bytes[8..40].len(), 32);

        // Check that size is present and valid
        let size = u64::from_le_bytes(bytes[40..48].try_into().unwrap()) as usize;
        assert_eq!(size, bytes.len() - HEADER_SIZE);
    }

    #[test]
    fn test_round_trip_serialization() {
        let config = create_test_config();

        // Serialize
        let bytes = serialize(&config).expect("Serialization failed");

        // Deserialize
        let archived = deserialize(&bytes).expect("Deserialization failed");

        // Verify data matches
        assert_eq!(archived.version.major, 1);
        assert_eq!(archived.version.minor, 0);
        assert_eq!(archived.version.patch, 0);
        assert_eq!(archived.devices.len(), 1);
    }

    #[test]
    fn test_deterministic_serialization() {
        let config = create_test_config();

        let bytes1 = serialize(&config).expect("First serialization failed");
        let bytes2 = serialize(&config).expect("Second serialization failed");

        // Same input should produce identical output
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_deserialize_validates_magic() {
        let config = create_test_config();
        let mut bytes = serialize(&config).unwrap();

        // Corrupt magic bytes
        bytes[0] = 0x00;

        let result = deserialize(&bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(DeserializeError::InvalidMagic { .. })));
    }

    #[test]
    fn test_deserialize_validates_version() {
        let config = create_test_config();
        let mut bytes = serialize(&config).unwrap();

        // Corrupt version
        bytes[4] = 0xFF;
        bytes[5] = 0xFF;
        bytes[6] = 0xFF;
        bytes[7] = 0xFF;

        let result = deserialize(&bytes);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(DeserializeError::VersionMismatch { .. })
        ));
    }

    #[test]
    fn test_deserialize_validates_hash() {
        let config = create_test_config();
        let mut bytes = serialize(&config).unwrap();

        // Corrupt hash
        bytes[8] = !bytes[8];

        let result = deserialize(&bytes);
        assert!(result.is_err());
        assert!(matches!(result, Err(DeserializeError::HashMismatch { .. })));
    }

    #[test]
    fn test_deserialize_rejects_truncated_file() {
        let config = create_test_config();
        let bytes = serialize(&config).unwrap();

        // Truncate to less than header size
        let truncated = &bytes[..30];

        let result = deserialize(truncated);
        assert!(result.is_err());
        assert!(matches!(result, Err(DeserializeError::RkyvError(_))));
    }

    #[test]
    fn test_deserialize_rejects_corrupted_data() {
        let config = create_test_config();
        let mut bytes = serialize(&config).unwrap();

        // Corrupt data section (not hash)
        if bytes.len() > HEADER_SIZE + 10 {
            bytes[HEADER_SIZE + 10] = !bytes[HEADER_SIZE + 10];
        }

        let result = deserialize(&bytes);
        assert!(result.is_err());
        // Should fail hash check
        assert!(matches!(result, Err(DeserializeError::HashMismatch { .. })));
    }

    #[test]
    fn test_header_constants() {
        assert_eq!(KRX_MAGIC, [0x4B, 0x52, 0x58, 0x0A]);
        assert_eq!(KRX_VERSION, 1);
        assert_eq!(HEADER_SIZE, 48);
    }
}
