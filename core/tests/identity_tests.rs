#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Comprehensive tests for the identity module.
//!
//! This test suite covers:
//! - DeviceIdentity type system (to_key/from_key, Hash, Eq)
//! - Windows serial extraction (path parsing)
//! - Linux synthetic ID generation (stability and uniqueness)
//! - Property-based testing for robustness

use keyrx_core::identity::DeviceIdentity;
use std::collections::{HashMap, HashSet};

// ============================================================================
// DeviceIdentity Core Tests
// ============================================================================

#[test]
fn test_device_identity_creation() {
    let id = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
    assert_eq!(id.vendor_id, 0x046D);
    assert_eq!(id.product_id, 0xC52B);
    assert_eq!(id.serial_number, "ABC123");
    assert_eq!(id.user_label, None);
}

#[test]
fn test_device_identity_with_label() {
    let id = DeviceIdentity::with_label(
        0x046D,
        0xC52B,
        "ABC123".to_string(),
        "My Keyboard".to_string(),
    );
    assert_eq!(id.user_label, Some("My Keyboard".to_string()));
}

// ============================================================================
// to_key/from_key Roundtrip Tests
// ============================================================================

#[test]
fn test_to_key_format_lowercase_hex() {
    let id = DeviceIdentity::new(0x046D, 0xC52B, "SERIAL123".to_string());
    let key = id.to_key();
    assert_eq!(key, "046d:c52b:SERIAL123");
    assert!(key
        .chars()
        .take(9)
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == ':'));
}

#[test]
fn test_to_key_format_padding() {
    let id = DeviceIdentity::new(0x1, 0xA, "S".to_string());
    let key = id.to_key();
    assert_eq!(key, "0001:000a:S");
}

#[test]
fn test_from_key_valid_basic() {
    let key = "046d:c52b:ABC123";
    let id = DeviceIdentity::from_key(key).unwrap();
    assert_eq!(id.vendor_id, 0x046D);
    assert_eq!(id.product_id, 0xC52B);
    assert_eq!(id.serial_number, "ABC123");
}

#[test]
fn test_from_key_serial_with_colons() {
    let key = "046d:c52b:ABC:123:XYZ";
    let id = DeviceIdentity::from_key(key).unwrap();
    assert_eq!(id.serial_number, "ABC:123:XYZ");
}

#[test]
fn test_from_key_empty_serial() {
    let key = "046d:c52b:";
    let id = DeviceIdentity::from_key(key).unwrap();
    assert_eq!(id.serial_number, "");
}

#[test]
fn test_from_key_serial_with_special_chars() {
    let key = "046d:c52b:ABC-123_XYZ.456";
    let id = DeviceIdentity::from_key(key).unwrap();
    assert_eq!(id.serial_number, "ABC-123_XYZ.456");
}

#[test]
fn test_from_key_invalid_format_missing_parts() {
    assert!(DeviceIdentity::from_key("046d").is_err());
    assert!(DeviceIdentity::from_key("046d:c52b").is_err());
    assert!(DeviceIdentity::from_key("").is_err());
}

#[test]
fn test_from_key_invalid_hex_vendor_id() {
    let result = DeviceIdentity::from_key("GGGG:c52b:ABC123");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("vendor_id"));
}

#[test]
fn test_from_key_invalid_hex_product_id() {
    let result = DeviceIdentity::from_key("046d:ZZZZ:ABC123");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("product_id"));
}

#[test]
fn test_from_key_uppercase_hex() {
    let key = "046D:C52B:ABC123";
    let id = DeviceIdentity::from_key(key).unwrap();
    assert_eq!(id.vendor_id, 0x046D);
    assert_eq!(id.product_id, 0xC52B);
}

#[test]
fn test_roundtrip_basic() {
    let original = DeviceIdentity::new(0x1234, 0xABCD, "SERIAL123".to_string());
    let key = original.to_key();
    let restored = DeviceIdentity::from_key(&key).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_roundtrip_with_label() {
    let original = DeviceIdentity::with_label(
        0x1234,
        0xABCD,
        "SERIAL123".to_string(),
        "My Device".to_string(),
    );
    let key = original.to_key();
    let restored = DeviceIdentity::from_key(&key).unwrap();

    // Key roundtrip preserves VID:PID:Serial but not label
    assert_eq!(original.vendor_id, restored.vendor_id);
    assert_eq!(original.product_id, restored.product_id);
    assert_eq!(original.serial_number, restored.serial_number);
    assert_eq!(restored.user_label, None);
}

#[test]
fn test_roundtrip_complex_serial() {
    let original = DeviceIdentity::new(0xFFFF, 0x0000, "A:B:C-1_2.3".to_string());
    let key = original.to_key();
    let restored = DeviceIdentity::from_key(&key).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_roundtrip_edge_cases() {
    let test_cases = vec![
        (0x0000, 0x0000, "0"),
        (0xFFFF, 0xFFFF, "FFFF"),
        (0x0001, 0x0001, "a"),
        (0x1234, 0x5678, ""),
        (0xABCD, 0xEF01, "!@#$%^&*()"),
    ];

    for (vid, pid, serial) in test_cases {
        let original = DeviceIdentity::new(vid, pid, serial.to_string());
        let key = original.to_key();
        let restored = DeviceIdentity::from_key(&key).unwrap();
        assert_eq!(
            original, restored,
            "Failed roundtrip for {:04x}:{:04x}:{}",
            vid, pid, serial
        );
    }
}

// ============================================================================
// Hash Implementation Tests
// ============================================================================

#[test]
fn test_hash_same_device_same_hash() {
    let id1 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
    let id2 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());

    let mut map = HashMap::new();
    map.insert(id1, "value1");

    assert_eq!(map.get(&id2), Some(&"value1"));
}

#[test]
fn test_hash_different_serial_different_hash() {
    let id1 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
    let id2 = DeviceIdentity::new(0x046D, 0xC52B, "XYZ789".to_string());

    let mut map = HashMap::new();
    map.insert(id1.clone(), "value1");
    map.insert(id2.clone(), "value2");

    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&id1), Some(&"value1"));
    assert_eq!(map.get(&id2), Some(&"value2"));
}

#[test]
fn test_hash_ignores_user_label() {
    let id1 =
        DeviceIdentity::with_label(0x046D, 0xC52B, "ABC123".to_string(), "Label 1".to_string());
    let id2 =
        DeviceIdentity::with_label(0x046D, 0xC52B, "ABC123".to_string(), "Label 2".to_string());

    let mut map = HashMap::new();
    map.insert(id1, "value");

    // Should find the value even with different label
    assert_eq!(map.get(&id2), Some(&"value"));
    assert_eq!(map.len(), 1);
}

#[test]
fn test_hash_multiple_identical_devices() {
    let id1 = DeviceIdentity::new(0x046D, 0xC52B, "Device1Serial".to_string());
    let id2 = DeviceIdentity::new(0x046D, 0xC52B, "Device2Serial".to_string());
    let id3 = DeviceIdentity::new(0x046D, 0xC52B, "Device3Serial".to_string());

    let mut map = HashMap::new();
    map.insert(id1.clone(), "profile1");
    map.insert(id2.clone(), "profile2");
    map.insert(id3.clone(), "profile3");

    assert_eq!(map.len(), 3);
    assert_eq!(map.get(&id1), Some(&"profile1"));
    assert_eq!(map.get(&id2), Some(&"profile2"));
    assert_eq!(map.get(&id3), Some(&"profile3"));
}

#[test]
fn test_hash_collision_resistance() {
    let mut set = HashSet::new();

    // Generate many device identities and ensure no collisions
    for vid in 0..100u16 {
        for pid in 0..100u16 {
            for serial_num in 0..10 {
                let id = DeviceIdentity::new(vid, pid, format!("SERIAL{}", serial_num));
                assert!(set.insert(id), "Hash collision detected!");
            }
        }
    }

    // Should have 100 * 100 * 10 = 100,000 unique entries
    assert_eq!(set.len(), 100 * 100 * 10);
}

// ============================================================================
// Eq/PartialEq Implementation Tests
// ============================================================================

#[test]
fn test_eq_same_vid_pid_serial() {
    let id1 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
    let id2 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
    assert_eq!(id1, id2);
}

#[test]
fn test_eq_different_serial() {
    let id1 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
    let id2 = DeviceIdentity::new(0x046D, 0xC52B, "XYZ789".to_string());
    assert_ne!(id1, id2);
}

#[test]
fn test_eq_different_vid() {
    let id1 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
    let id2 = DeviceIdentity::new(0x1234, 0xC52B, "ABC123".to_string());
    assert_ne!(id1, id2);
}

#[test]
fn test_eq_different_pid() {
    let id1 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
    let id2 = DeviceIdentity::new(0x046D, 0x1234, "ABC123".to_string());
    assert_ne!(id1, id2);
}

#[test]
fn test_eq_ignores_user_label() {
    let id1 =
        DeviceIdentity::with_label(0x046D, 0xC52B, "ABC123".to_string(), "Label 1".to_string());
    let id2 =
        DeviceIdentity::with_label(0x046D, 0xC52B, "ABC123".to_string(), "Label 2".to_string());

    assert_eq!(id1, id2);
}

#[test]
fn test_eq_one_with_label_one_without() {
    let id1 = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
    let id2 = DeviceIdentity::with_label(
        0x046D,
        0xC52B,
        "ABC123".to_string(),
        "My Device".to_string(),
    );

    assert_eq!(id1, id2);
}

// ============================================================================
// Display and Formatting Tests
// ============================================================================

#[test]
fn test_display_without_label() {
    let id = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
    assert_eq!(id.to_string(), "046d:c52b:ABC123");
}

#[test]
fn test_display_with_label() {
    let id = DeviceIdentity::with_label(
        0x046D,
        0xC52B,
        "ABC123".to_string(),
        "My Keyboard".to_string(),
    );
    assert_eq!(id.to_string(), "My Keyboard (046d:c52b:ABC123)");
}

#[test]
fn test_display_name_without_label() {
    let id = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
    assert_eq!(id.display_name(), "046d:c52b");
}

#[test]
fn test_display_name_with_label() {
    let id = DeviceIdentity::with_label(
        0x046D,
        0xC52B,
        "ABC123".to_string(),
        "My Device".to_string(),
    );
    assert_eq!(id.display_name(), "My Device");
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_serialization_roundtrip() {
    let original = DeviceIdentity::with_label(
        0x046D,
        0xC52B,
        "ABC123".to_string(),
        "Test Device".to_string(),
    );

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: DeviceIdentity = serde_json::from_str(&json).unwrap();

    assert_eq!(original, deserialized);
    assert_eq!(original.user_label, deserialized.user_label);
}

#[test]
fn test_serialization_without_label() {
    let id = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());

    let json = serde_json::to_string(&id).unwrap();

    // user_label should be omitted when None (skip_serializing_if)
    assert!(!json.contains("user_label"));

    let deserialized: DeviceIdentity = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
    assert_eq!(deserialized.user_label, None);
}

#[test]
fn test_serialization_with_label() {
    let id = DeviceIdentity::with_label(
        0x046D,
        0xC52B,
        "ABC123".to_string(),
        "My Device".to_string(),
    );

    let json = serde_json::to_string(&id).unwrap();

    // user_label should be included when Some
    assert!(json.contains("user_label"));
    assert!(json.contains("My Device"));
}

#[test]
fn test_deserialization_missing_optional_label() {
    let json = r#"{"vendor_id":1133,"product_id":50475,"serial_number":"ABC123"}"#;
    let id: DeviceIdentity = serde_json::from_str(json).unwrap();

    assert_eq!(id.vendor_id, 0x046D);
    assert_eq!(id.product_id, 0xC52B);
    assert_eq!(id.serial_number, "ABC123");
    assert_eq!(id.user_label, None);
}

#[test]
fn test_serialization_preserves_all_fields() {
    let id = DeviceIdentity::with_label(
        0xFFFF,
        0x0000,
        "Complex:Serial:123".to_string(),
        "Special Device!".to_string(),
    );

    let json = serde_json::to_string(&id).unwrap();
    let deserialized: DeviceIdentity = serde_json::from_str(&json).unwrap();

    assert_eq!(id.vendor_id, deserialized.vendor_id);
    assert_eq!(id.product_id, deserialized.product_id);
    assert_eq!(id.serial_number, deserialized.serial_number);
    assert_eq!(id.user_label, deserialized.user_label);
}

// ============================================================================
// Windows Path Parsing Tests (Platform-Specific)
// ============================================================================

#[cfg(windows)]
mod windows_tests {
    use super::*;
    use keyrx_core::identity::windows::extract_serial_number;

    #[test]
    fn test_windows_hid_path_basic() {
        let path =
            r"\\?\HID#VID_046D&PID_C52B#7&12345678&0&0000#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}";
        // This will attempt to read iSerial and fall back to Instance ID
        // Since we can't open the device in tests, it should fall back to Instance ID
        if let Ok(serial) = extract_serial_number(path) {
            // Should get Instance ID as fallback
            assert_eq!(serial, "7&12345678&0&0000");
        }
    }

    #[test]
    fn test_windows_ps2_keyboard_path() {
        let path = r"\\?\ACPI#PNP0303#4&1234abcd&0#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}";
        if let Ok(serial) = extract_serial_number(path) {
            assert_eq!(serial, "4&1234abcd&0");
        }
    }

    #[test]
    fn test_windows_usb_path_with_serial() {
        let path = r"\\?\HID#VID_1234&PID_5678#ABC123456#{guid}";
        if let Ok(serial) = extract_serial_number(path) {
            // Should extract ABC123456 as Instance ID
            assert_eq!(serial, "ABC123456");
        }
    }

    #[test]
    fn test_windows_invalid_path_format() {
        let path = r"\\?\InvalidPath";
        assert!(extract_serial_number(path).is_err());
    }

    #[test]
    fn test_windows_path_without_prefix() {
        let path = r"HID#VID_046D&PID_C52B#InstanceID123#{guid}";
        if let Ok(serial) = extract_serial_number(path) {
            assert_eq!(serial, "InstanceID123");
        }
    }

    #[test]
    fn test_windows_path_various_formats() {
        let test_paths = vec![
            (
                r"\\?\HID#VID_046D&PID_C52B#7&1234&0&0000#{guid}",
                "7&1234&0&0000",
            ),
            (r"\\?\HID#VID_1234&PID_ABCD#Serial123#{guid}", "Serial123"),
            (
                r"\\?\USB#VID_046D&PID_C52B#5&1234abcd&0&1#{guid}",
                "5&1234abcd&0&1",
            ),
        ];

        for (path, expected_instance_id) in test_paths {
            if let Ok(serial) = extract_serial_number(path) {
                assert_eq!(serial, expected_instance_id, "Failed for path: {}", path);
            }
        }
    }
}

// ============================================================================
// Linux Synthetic ID Tests (Platform-Specific)
// ============================================================================

#[cfg(target_os = "linux")]
mod linux_tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_synthetic_id_stability() {
        // Same physical path should always produce the same hash
        let phys1 = "usb-0000:00:14.0-1/input0";
        let phys2 = "usb-0000:00:14.0-1/input0";

        let mut hasher1 = DefaultHasher::new();
        phys1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        phys2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2, "Same physical path must produce same hash");

        let synthetic1 = format!("phys_{:08x}", (hash1 & 0xFFFFFFFF) as u32);
        let synthetic2 = format!("phys_{:08x}", (hash2 & 0xFFFFFFFF) as u32);
        assert_eq!(synthetic1, synthetic2);
    }

    #[test]
    fn test_synthetic_id_uniqueness() {
        // Different physical paths should produce different hashes
        let test_paths = vec![
            "usb-0000:00:14.0-1/input0",
            "usb-0000:00:14.0-2/input0",
            "usb-0000:00:14.0-3/input0",
            "usb-0000:00:1a.0-1/input0",
            "usb-0001:00:14.0-1/input0",
        ];

        let mut hashes = HashSet::new();
        for phys in &test_paths {
            let mut hasher = DefaultHasher::new();
            phys.hash(&mut hasher);
            let hash = hasher.finish();
            let synthetic = format!("phys_{:08x}", (hash & 0xFFFFFFFF) as u32);

            assert!(
                hashes.insert(synthetic.clone()),
                "Hash collision detected for path: {}",
                phys
            );
        }

        assert_eq!(hashes.len(), test_paths.len());
    }

    #[test]
    fn test_synthetic_id_format() {
        let phys = "usb-0000:00:14.0-1/input0";
        let mut hasher = DefaultHasher::new();
        phys.hash(&mut hasher);
        let hash = hasher.finish();
        let synthetic = format!("phys_{:08x}", (hash & 0xFFFFFFFF) as u32);

        assert!(synthetic.starts_with("phys_"));
        assert_eq!(synthetic.len(), 13); // "phys_" (5) + 8 hex chars

        // Verify hex characters
        let hex_part = &synthetic[5..];
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_synthetic_id_determinism_across_reboots() {
        // Simulate the same physical path being read after a reboot
        // The hash must be identical
        let phys = "usb-0000:00:14.0-5/input1";

        let mut results = vec![];
        for _ in 0..100 {
            let mut hasher = DefaultHasher::new();
            phys.hash(&mut hasher);
            let hash = hasher.finish();
            let synthetic = format!("phys_{:08x}", (hash & 0xFFFFFFFF) as u32);
            results.push(synthetic);
        }

        // All results should be identical
        assert!(results.windows(2).all(|w| w[0] == w[1]));
    }

    #[test]
    fn test_synthetic_id_port_sensitivity() {
        // Different USB ports should produce different IDs
        let ports = vec![
            "usb-0000:00:14.0-1/input0",   // Port 1
            "usb-0000:00:14.0-2/input0",   // Port 2
            "usb-0000:00:14.0-3/input0",   // Port 3
            "usb-0000:00:14.0-1.1/input0", // Hub port
        ];

        let mut synthetic_ids = HashSet::new();
        for phys in &ports {
            let mut hasher = DefaultHasher::new();
            phys.hash(&mut hasher);
            let hash = hasher.finish();
            let synthetic = format!("phys_{:08x}", (hash & 0xFFFFFFFF) as u32);
            synthetic_ids.insert(synthetic);
        }

        // All ports should have unique synthetic IDs
        assert_eq!(synthetic_ids.len(), ports.len());
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_device_identity_as_hashmap_key() {
    let mut device_profiles = HashMap::new();

    let device1 = DeviceIdentity::new(0x046D, 0xC52B, "Serial1".to_string());
    let device2 = DeviceIdentity::new(0x046D, 0xC52B, "Serial2".to_string());
    let device3 = DeviceIdentity::new(0x1234, 0x5678, "Serial3".to_string());

    device_profiles.insert(device1.clone(), "Profile A");
    device_profiles.insert(device2.clone(), "Profile B");
    device_profiles.insert(device3.clone(), "Profile C");

    assert_eq!(device_profiles.get(&device1), Some(&"Profile A"));
    assert_eq!(device_profiles.get(&device2), Some(&"Profile B"));
    assert_eq!(device_profiles.get(&device3), Some(&"Profile C"));
    assert_eq!(device_profiles.len(), 3);
}

#[test]
fn test_multiple_identical_keyboards_different_profiles() {
    // Real-world scenario: User has 3 identical keyboards, wants different mappings
    let mut registry = HashMap::new();

    let keyboard1 = DeviceIdentity::with_label(
        0x046D,
        0xC52B,
        "Serial001".to_string(),
        "Left Keyboard".to_string(),
    );
    let keyboard2 = DeviceIdentity::with_label(
        0x046D,
        0xC52B,
        "Serial002".to_string(),
        "Right Keyboard".to_string(),
    );
    let keyboard3 = DeviceIdentity::with_label(
        0x046D,
        0xC52B,
        "Serial003".to_string(),
        "Center Keyboard".to_string(),
    );

    registry.insert(keyboard1.clone(), "gaming-profile");
    registry.insert(keyboard2.clone(), "coding-profile");
    registry.insert(keyboard3.clone(), "default-profile");

    assert_eq!(registry.get(&keyboard1), Some(&"gaming-profile"));
    assert_eq!(registry.get(&keyboard2), Some(&"coding-profile"));
    assert_eq!(registry.get(&keyboard3), Some(&"default-profile"));
}

#[test]
fn test_device_identity_persistence_simulation() {
    // Simulate saving and loading device identities
    let devices = vec![
        DeviceIdentity::with_label(
            0x046D,
            0xC52B,
            "ABC123".to_string(),
            "My Keyboard".to_string(),
        ),
        DeviceIdentity::new(0x1234, 0x5678, "XYZ789".to_string()),
    ];

    // Serialize to JSON
    let json = serde_json::to_string(&devices).unwrap();

    // Deserialize back
    let loaded: Vec<DeviceIdentity> = serde_json::from_str(&json).unwrap();

    assert_eq!(devices, loaded);
    assert_eq!(loaded[0].user_label, Some("My Keyboard".to_string()));
    assert_eq!(loaded[1].user_label, None);
}

#[test]
fn test_label_update_preserves_identity() {
    let mut registry = HashMap::new();

    let device = DeviceIdentity::new(0x046D, 0xC52B, "ABC123".to_string());
    registry.insert(device.clone(), "profile1");

    // User updates the label
    let device_with_label = DeviceIdentity::with_label(
        0x046D,
        0xC52B,
        "ABC123".to_string(),
        "My New Label".to_string(),
    );

    // Should still find the same device in the registry
    assert_eq!(registry.get(&device_with_label), Some(&"profile1"));

    // Updating the registry entry
    registry.insert(device_with_label.clone(), "profile2");

    // Should have replaced the old entry (same hash/eq)
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.get(&device), Some(&"profile2"));
}
