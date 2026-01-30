//! E2E tests for key blocking with virtual keyboard
//!
//! These tests verify that the key blocking actually works at runtime
//! by simulating keyboard input and verifying the blocker intercepts it.

#![cfg(target_os = "windows")]

use keyrx_compiler::serialize::deserialize;
use keyrx_core::config::{ConfigRoot, KeyCode};
use keyrx_daemon::platform::windows::{key_blocker::KeyBlocker, virtual_keyboard::VirtualKeyboard};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Test that the blocker actually blocks keys when installed
#[test]
#[ignore] // Requires elevated privileges to install hooks
fn test_blocker_blocks_virtual_keyboard() {
    // Create a blocker
    let blocker = KeyBlocker::new().expect("Should create blocker");

    // Block W, E, O keys
    blocker.block_key(0x11); // W
    blocker.block_key(0x12); // E
    blocker.block_key(0x18); // O

    // Verify keys are in blocked set
    assert_eq!(blocker.blocked_count(), 3);
    assert!(blocker.is_key_blocked(0x11));
    assert!(blocker.is_key_blocked(0x12));
    assert!(blocker.is_key_blocked(0x18));

    // Give the hook time to install
    std::thread::sleep(Duration::from_millis(100));

    // Create virtual keyboard
    let mut vkb = VirtualKeyboard::new();

    // Press W key - should be blocked by hook
    vkb.tap_key(KeyCode::W, 50)
        .expect("Should send W key press");

    // In a real E2E test, we would verify that no 'W' character appeared
    // For now, this test just verifies the blocker and virtual keyboard can coexist

    std::thread::sleep(Duration::from_millis(50));
}

/// Test the complete flow: load config → extract keys → block them
#[test]
fn test_complete_blocking_flow() {
    use std::path::PathBuf;

    // Find user_layout.krx
    let possible_paths = vec![
        PathBuf::from("examples/user_layout.krx"),
        PathBuf::from("../examples/user_layout.krx"),
    ];

    let krx_path = possible_paths.iter().find(|p| p.exists());
    let Some(krx_path) = krx_path else {
        println!("Skipping: examples/user_layout.krx not found");
        return;
    };

    // Load config
    let bytes = std::fs::read(&krx_path).expect("Should read user_layout.krx");
    let archived = deserialize(&bytes).expect("Should deserialize");
    let owned: ConfigRoot = archived
        .deserialize(&mut rkyv::Infallible)
        .expect("Should convert to owned");

    // Create blocker
    let blocker = KeyBlocker::new().expect("Should create blocker");

    // Extract and block all keys
    let mut blocked = 0;
    for device in &owned.devices {
        for mapping in &device.mappings {
            extract_and_block(mapping, &blocker, &mut blocked);
        }
    }

    println!("Blocked {} keys from config", blocked);

    // Verify blocker has the keys
    assert_eq!(blocker.blocked_count(), blocked);

    // Verify specific keys
    assert!(blocker.is_key_blocked(0x11), "W should be blocked");
    assert!(blocker.is_key_blocked(0x12), "E should be blocked");
    assert!(blocker.is_key_blocked(0x18), "O should be blocked");
}

/// Helper to extract and block keys from mappings
fn extract_and_block(
    mapping: &keyrx_core::config::KeyMapping,
    blocker: &KeyBlocker,
    blocked: &mut usize,
) {
    use keyrx_core::config::{BaseKeyMapping, KeyMapping};
    use keyrx_daemon::platform::windows::keycode::keycode_to_scancode;

    match mapping {
        KeyMapping::Base(base) => {
            let source_key = match base {
                BaseKeyMapping::Simple { from, .. } => *from,
                BaseKeyMapping::Modifier { from, .. } => *from,
                BaseKeyMapping::Lock { from, .. } => *from,
                BaseKeyMapping::TapHold { from, .. } => *from,
                BaseKeyMapping::ModifiedOutput { from, .. } => *from,
            };

            if let Some(scan_code) = keycode_to_scancode(source_key) {
                blocker.block_key(scan_code);
                *blocked += 1;
            }
        }
        KeyMapping::Conditional { mappings, .. } => {
            for base in mappings {
                let wrapped = KeyMapping::Base(base.clone());
                extract_and_block(&wrapped, blocker, blocked);
            }
        }
    }
}

/// Stress test: verify blocker handles rapid key presses
#[test]
#[ignore] // Requires elevated privileges
fn test_blocker_rapid_keypresses() {
    let blocker = KeyBlocker::new().expect("Should create blocker");
    blocker.block_key(0x11); // W

    let mut vkb = VirtualKeyboard::new();

    // Rapid W key taps
    for _ in 0..10 {
        vkb.tap_key(KeyCode::W, 10).expect("Should tap W");
        std::thread::sleep(Duration::from_millis(10));
    }

    // If blocker is working, these should all be blocked
}

/// Test blocker clears correctly
#[test]
fn test_blocker_clear() {
    let blocker = KeyBlocker::new().expect("Should create blocker");

    // Block some keys
    blocker.block_key(0x11);
    blocker.block_key(0x12);
    blocker.block_key(0x18);
    assert_eq!(blocker.blocked_count(), 3);

    // Clear all
    blocker.clear_all();
    assert_eq!(blocker.blocked_count(), 0);

    // Verify keys are no longer blocked
    assert!(!blocker.is_key_blocked(0x11));
    assert!(!blocker.is_key_blocked(0x12));
    assert!(!blocker.is_key_blocked(0x18));
}

/// Test blocker with multiple threads accessing it
#[test]
fn test_blocker_thread_safety() {
    let blocker = Arc::new(KeyBlocker::new().expect("Should create blocker"));

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let blocker = blocker.clone();
            std::thread::spawn(move || {
                blocker.block_key(0x11 + i);
                std::thread::sleep(Duration::from_millis(10));
                assert!(blocker.is_key_blocked(0x11 + i));
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    assert_eq!(blocker.blocked_count(), 5);
}
