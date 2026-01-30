use crate::platform::windows::keycode::{keycode_to_vk, vk_to_keycode};
use keyrx_core::config::KeyCode;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

#[allow(unused_imports)]
use rkyv::Deserialize;

#[test]
fn test_vk_mapping_completeness() {
    // Test a few common keys
    assert_eq!(vk_to_keycode(VK_A as u16), Some(KeyCode::A));
    assert_eq!(vk_to_keycode(VK_SPACE as u16), Some(KeyCode::Space));
    assert_eq!(vk_to_keycode(VK_LSHIFT as u16), Some(KeyCode::LShift));

    assert_eq!(keycode_to_vk(KeyCode::A), Some(VK_A as u16));
    assert_eq!(keycode_to_vk(KeyCode::Space), Some(VK_SPACE as u16));
    assert_eq!(keycode_to_vk(KeyCode::LShift), Some(VK_LSHIFT as u16));
}

#[test]
fn test_unmapped_vk() {
    assert_eq!(vk_to_keycode(0x07), None); // Undefined VK code
}

#[test]
fn test_extended_keys() {
    use crate::platform::windows::inject::is_extended_key;
    assert!(is_extended_key(VK_RMENU as u16));
    assert!(is_extended_key(VK_RCONTROL as u16));
    assert!(is_extended_key(VK_LEFT as u16));
    assert!(!is_extended_key(VK_A as u16));
}

#[test]
fn test_platform_trait_usage() {
    use crate::platform::{Platform, WindowsPlatform};

    // Verify that WindowsPlatform can be used as a trait object (Box<dyn Platform>)
    let platform: Box<dyn Platform> = Box::new(WindowsPlatform::new());

    // This compile-time check verifies that the trait implementation is correct
    let _ = platform;
}

#[test]
fn test_parse_vid_pid() {
    use super::parse_vid_pid;

    // Test with a typical Windows HID device path
    let path =
        r"\\?\HID#VID_046D&PID_C52B&MI_00#7&2a00c76d&0&0000#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}";
    let (vendor_id, product_id) = parse_vid_pid(path);
    assert_eq!(vendor_id, 0x046D);
    assert_eq!(product_id, 0xC52B);

    // Test with partial path
    let path2 = r"\\?\HID#VID_1234&PID_5678";
    let (vendor_id2, product_id2) = parse_vid_pid(path2);
    assert_eq!(vendor_id2, 0x1234);
    assert_eq!(product_id2, 0x5678);

    // Test with no VID/PID
    let path3 = r"\\?\HID#SOMEDEVICE";
    let (vendor_id3, product_id3) = parse_vid_pid(path3);
    assert_eq!(vendor_id3, 0);
    assert_eq!(product_id3, 0);
}

#[test]
fn test_convert_device_info() {
    use super::convert_device_info;
    use crate::platform::windows::device_map::DeviceInfo;

    let device = DeviceInfo {
            handle: 0x1234,
            path: r"\\?\HID#VID_046D&PID_C52B&MI_00#7&2a00c76d&0&0000#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}".to_string(),
            serial: Some("7&2a00c76d&0&0000".to_string()),
        };

    let common = convert_device_info(&device);
    assert_eq!(common.id, "serial-7&2a00c76d&0&0000");
    assert_eq!(common.vendor_id, 0x046D);
    assert_eq!(common.product_id, 0xC52B);
    assert_eq!(common.path, device.path);
}

// ============================================================================
// Key Blocking Integration Tests
// ============================================================================

use keyrx_compiler::serialize::{deserialize, serialize};
use keyrx_core::config::{BaseKeyMapping, ConfigRoot, KeyMapping};
use keyrx_core::config::{Condition, Metadata, Version};
use std::collections::HashSet;

/// Create a test config with multiple layers and mappings
fn create_comprehensive_test_config() -> ConfigRoot {
    // Base layer mappings
    let base_mappings = vec![
        KeyMapping::Base(BaseKeyMapping::Simple {
            from: KeyCode::W,
            to: KeyCode::A,
        }),
        KeyMapping::Base(BaseKeyMapping::Simple {
            from: KeyCode::O,
            to: KeyCode::T,
        }),
        KeyMapping::Base(BaseKeyMapping::TapHold {
            from: KeyCode::B,
            tap: KeyCode::Enter,
            hold_modifier: 0,
            threshold_ms: 200,
        }),
    ];

    // MD_00 layer (B held)
    let md00_mappings = vec![KeyMapping::Conditional {
        condition: Condition::ModifierActive(0),
        mappings: vec![
            BaseKeyMapping::Simple {
                from: KeyCode::W,
                to: KeyCode::Num1,
            },
            BaseKeyMapping::Simple {
                from: KeyCode::O,
                to: KeyCode::Num8,
            },
        ],
    }];

    let mut all_mappings = base_mappings;
    all_mappings.extend(md00_mappings);

    ConfigRoot {
        version: Version {
            major: 1,
            minor: 0,
            patch: 0,
        },
        devices: vec![keyrx_core::config::DeviceConfig {
            identifier: keyrx_core::config::DeviceIdentifier {
                pattern: String::from("*"),
            },
            mappings: all_mappings,
        }],
        metadata: Metadata {
            compilation_timestamp: 0,
            compiler_version: String::from("0.1.0"),
            source_hash: String::from("test"),
        },
    }
}

/// Extract all source keys from a config
fn extract_source_keys(config: &ConfigRoot) -> HashSet<KeyCode> {
    let mut keys = HashSet::new();
    for device in &config.devices {
        for mapping in &device.mappings {
            extract_from_mapping(mapping, &mut keys);
        }
    }
    keys
}

fn extract_from_mapping(mapping: &KeyMapping, keys: &mut HashSet<KeyCode>) {
    match mapping {
        KeyMapping::Base(base) => {
            let key = match base {
                BaseKeyMapping::Simple { from, .. } => *from,
                BaseKeyMapping::Modifier { from, .. } => *from,
                BaseKeyMapping::Lock { from, .. } => *from,
                BaseKeyMapping::TapHold { from, .. } => *from,
                BaseKeyMapping::ModifiedOutput { from, .. } => *from,
            };
            keys.insert(key);
        }
        KeyMapping::Conditional { mappings, .. } => {
            for base in mappings {
                let key = match base {
                    BaseKeyMapping::Simple { from, .. } => *from,
                    BaseKeyMapping::Modifier { from, .. } => *from,
                    BaseKeyMapping::Lock { from, .. } => *from,
                    BaseKeyMapping::TapHold { from, .. } => *from,
                    BaseKeyMapping::ModifiedOutput { from, .. } => *from,
                };
                keys.insert(key);
            }
        }
    }
}

#[test]
fn test_extract_unique_source_keys() {
    let config = create_comprehensive_test_config();
    let keys = extract_source_keys(&config);

    // W, O, B should be extracted (O appears in multiple layers but only counted once)
    assert_eq!(keys.len(), 3, "Should extract 3 unique source keys");
    assert!(keys.contains(&KeyCode::W));
    assert!(keys.contains(&KeyCode::O));
    assert!(keys.contains(&KeyCode::B));
}

#[test]
fn test_config_serialization_roundtrip() {
    let config = create_comprehensive_test_config();
    let bytes = serialize(&config).expect("Serialization should succeed");
    let archived = deserialize(&bytes).expect("Deserialization should succeed");
    assert_eq!(archived.devices.len(), 1);
}

#[test]
fn test_keycode_to_scancode_for_user_layout_keys() {
    use crate::platform::windows::keycode::keycode_to_scancode;

    let test_keys = vec![
        (KeyCode::W, Some(0x11)),
        (KeyCode::O, Some(0x18)),
        (KeyCode::B, Some(0x30)),
    ];

    for (keycode, expected) in test_keys {
        let scan = keycode_to_scancode(keycode);
        assert_eq!(scan, expected, "{:?} should convert correctly", keycode);
    }
}

#[test]
fn test_load_and_extract_from_user_layout_krx() {
    use std::path::PathBuf;

    // Try multiple possible paths (test runs from different directories)
    let possible_paths = vec![
        PathBuf::from("examples/user_layout.krx"),
        PathBuf::from("../examples/user_layout.krx"),
        PathBuf::from("../../examples/user_layout.krx"),
    ];

    let krx_path = possible_paths.iter().find(|p| p.exists());

    let Some(krx_path) = krx_path else {
        println!("Skipping: examples/user_layout.krx not found in any location");
        return;
    };

    let bytes = std::fs::read(&krx_path).expect("Should read user_layout.krx");
    let archived = deserialize(&bytes).expect("Should deserialize user_layout.krx");
    let owned: ConfigRoot = archived
        .deserialize(&mut rkyv::Infallible)
        .expect("Should convert to owned");

    let keys = extract_source_keys(&owned);
    println!("Extracted {} unique source keys", keys.len());

    assert!(keys.len() >= 30, "Should have at least 30 keys");
    assert!(keys.contains(&KeyCode::O), "Should extract O key");
    assert!(keys.contains(&KeyCode::W), "Should extract W key");
    assert!(keys.contains(&KeyCode::B), "Should extract B key");
}

#[test]
fn test_all_extracted_keys_convertible_to_scancodes() {
    use crate::platform::windows::keycode::keycode_to_scancode;
    use std::path::PathBuf;

    let possible_paths = vec![
        PathBuf::from("examples/user_layout.krx"),
        PathBuf::from("../examples/user_layout.krx"),
        PathBuf::from("../../examples/user_layout.krx"),
    ];

    let krx_path = possible_paths.iter().find(|p| p.exists());

    let Some(krx_path) = krx_path else {
        println!("Skipping: examples/user_layout.krx not found in any location");
        return;
    };

    let bytes = std::fs::read(&krx_path).expect("Should read");
    let archived = deserialize(&bytes).expect("Should deserialize");
    let owned: ConfigRoot = archived.deserialize(&mut rkyv::Infallible).expect("Should convert");

    let keys = extract_source_keys(&owned);
    let mut failed = Vec::new();

    for key in &keys {
        if keycode_to_scancode(*key).is_none() {
            failed.push(*key);
        }
    }

    // Filter out Japanese-specific keys that may not have Windows VK mappings
    let japanese_keys = vec![KeyCode::Ro, KeyCode::Hiragana, KeyCode::Katakana, KeyCode::Zenkaku];
    let non_japanese_failed: Vec<_> = failed
        .into_iter()
        .filter(|k| !japanese_keys.contains(k))
        .collect();

    assert!(
        non_japanese_failed.is_empty(),
        "These keys cannot be converted to scan codes: {:?}",
        non_japanese_failed
    );

    println!(
        "Successfully converted {} keys to scan codes (skipped {} Japanese-specific keys)",
        keys.len() - japanese_keys.len(),
        japanese_keys.len()
    );
}

#[test]
fn test_key_blocker_actually_blocks() {
    use crate::platform::windows::key_blocker::KeyBlocker;

    // This test verifies that KeyBlocker actually stores keys in its internal state
    // This was the CRITICAL BUG - tests verified pipeline but not actual blocking

    let blocker = KeyBlocker::new().expect("Should create blocker");

    // Initially no keys blocked
    assert_eq!(blocker.blocked_count(), 0, "Should start with no blocked keys");

    // Block some keys
    blocker.block_key(0x11); // W
    blocker.block_key(0x12); // E
    blocker.block_key(0x18); // O

    assert_eq!(blocker.blocked_count(), 3, "Should have 3 blocked keys");
    assert!(blocker.is_key_blocked(0x11), "W should be blocked");
    assert!(blocker.is_key_blocked(0x12), "E should be blocked");
    assert!(blocker.is_key_blocked(0x18), "O should be blocked");
    assert!(!blocker.is_key_blocked(0x30), "B should not be blocked yet");

    // Clear and re-add
    blocker.clear_all();
    assert_eq!(blocker.blocked_count(), 0, "Should have no blocked keys after clear");

    blocker.block_key(0x30); // B
    assert_eq!(blocker.blocked_count(), 1, "Should have 1 blocked key");
    assert!(blocker.is_key_blocked(0x30), "B should be blocked");
}

#[test]
fn test_configure_blocking_with_real_config() {
    use crate::platform::windows::key_blocker::KeyBlocker;
    use crate::platform::windows::keycode::keycode_to_scancode;
    use std::path::PathBuf;

    // This test simulates the actual configure_blocking flow

    let possible_paths = vec![
        PathBuf::from("examples/user_layout.krx"),
        PathBuf::from("../examples/user_layout.krx"),
        PathBuf::from("../../examples/user_layout.krx"),
    ];

    let krx_path = possible_paths.iter().find(|p| p.exists());

    let Some(krx_path) = krx_path else {
        println!("Skipping: examples/user_layout.krx not found");
        return;
    };

    // Load and deserialize config
    let bytes = std::fs::read(&krx_path).expect("Should read user_layout.krx");
    let archived = deserialize(&bytes).expect("Should deserialize user_layout.krx");
    let owned: ConfigRoot = archived
        .deserialize(&mut rkyv::Infallible)
        .expect("Should convert to owned");

    // Create a blocker
    let blocker = KeyBlocker::new().expect("Should create blocker");

    // Extract keys (mimics platform_state::extract_and_block_key)
    let keys = extract_source_keys(&owned);
    println!("Extracted {} unique source keys", keys.len());

    // Block each key
    let mut blocked = 0;
    for key in &keys {
        if let Some(scan_code) = keycode_to_scancode(*key) {
            blocker.block_key(scan_code);
            blocked += 1;
        }
    }

    println!("Blocked {} keys", blocked);

    // Verify blocker actually has the keys
    let actual_count = blocker.blocked_count();
    println!("Blocker reports {} keys blocked", actual_count);

    assert_eq!(
        actual_count, blocked,
        "Blocker should report same count as we blocked"
    );

    // Verify specific keys are blocked
    let w_scan = keycode_to_scancode(KeyCode::W).expect("W should convert");
    let e_scan = keycode_to_scancode(KeyCode::E).expect("E should convert");
    let o_scan = keycode_to_scancode(KeyCode::O).expect("O should convert");

    assert!(
        blocker.is_key_blocked(w_scan),
        "W key (0x{:04X}) should be blocked",
        w_scan
    );
    assert!(
        blocker.is_key_blocked(e_scan),
        "E key (0x{:04X}) should be blocked",
        e_scan
    );
    assert!(
        blocker.is_key_blocked(o_scan),
        "O key (0x{:04X}) should be blocked",
        o_scan
    );
}
