//! Virtual E2E Tests for keyrx daemon.
//!
//! These tests use virtual input devices (uinput) to test the complete
//! keyboard remapping pipeline without requiring physical hardware.
//!
//! # Running These Tests
//!
//! These tests require:
//! - Linux with uinput module loaded (`sudo modprobe uinput`)
//! - Write access to `/dev/uinput` (usually requires root or uinput group)
//! - The keyrx_daemon binary built
//!
//! Run with:
//! ```bash
//! sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests -- --ignored
//! ```
//!
//! Or run all E2E tests:
//! ```bash
//! sudo cargo test -p keyrx_daemon --features linux -- --ignored
//! ```

#![cfg(all(target_os = "linux", feature = "linux"))]

mod e2e_harness;

use std::time::Duration;

use e2e_harness::{E2EConfig, E2EHarness, TestEvents};
use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEvent;

// ============================================================================
// Simple Remap Tests - Requirement 5.1
// ============================================================================

/// Test simple A → B remapping (press event).
///
/// Verifies that when A is pressed, B is output instead.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_simple_remap_press -- --ignored"]
fn test_simple_remap_press() {
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject Press(A)
    let input = TestEvents::press(KeyCode::A);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(100))
        .expect("Failed to inject and capture");

    // Expect Press(B)
    let expected = TestEvents::press(KeyCode::B);
    harness
        .verify(&captured, &expected)
        .expect("Press A should produce Press B");
}

/// Test simple A → B remapping (release event).
///
/// Verifies that when A is released, B release is output instead.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_simple_remap_release -- --ignored"]
fn test_simple_remap_release() {
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // First press A to establish state
    harness
        .inject(&TestEvents::press(KeyCode::A))
        .expect("Failed to inject press");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Now inject Release(A)
    let input = TestEvents::release(KeyCode::A);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(100))
        .expect("Failed to inject and capture");

    // Expect Release(B)
    let expected = TestEvents::release(KeyCode::B);
    harness
        .verify(&captured, &expected)
        .expect("Release A should produce Release B");
}

/// Test simple A → B remapping (complete key tap).
///
/// Verifies that a complete tap (press + release) of A produces
/// a complete tap of B.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_simple_remap_tap -- --ignored"]
fn test_simple_remap_tap() {
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject tap(A) = [Press(A), Release(A)]
    let input = TestEvents::tap(KeyCode::A);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(100))
        .expect("Failed to inject and capture");

    // Expect tap(B) = [Press(B), Release(B)]
    let expected = TestEvents::tap(KeyCode::B);
    harness
        .verify(&captured, &expected)
        .expect("Tap A should produce Tap B");
}

// ============================================================================
// Multiple Remaps in Sequence Tests
// ============================================================================

/// Test multiple different remaps in the same configuration.
///
/// Verifies that when multiple remaps are configured (A→B, C→D),
/// each key is correctly remapped.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_multiple_remaps_different_keys -- --ignored"]
fn test_multiple_remaps_different_keys() {
    // Configure A→B and C→D
    let config = E2EConfig::simple_remaps(vec![(KeyCode::A, KeyCode::B), (KeyCode::C, KeyCode::D)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test A→B
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture A");
    harness
        .verify(&captured, &TestEvents::tap(KeyCode::B))
        .expect("Tap A should produce Tap B");

    // Test C→D
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::C), Duration::from_millis(100))
        .expect("Failed to inject and capture C");
    harness
        .verify(&captured, &TestEvents::tap(KeyCode::D))
        .expect("Tap C should produce Tap D");
}

/// Test sequence of same remapped key.
///
/// Verifies that repeatedly pressing the same remapped key works correctly.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_repeated_remap_sequence -- --ignored"]
fn test_repeated_remap_sequence() {
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject A three times
    let input = TestEvents::taps(&[KeyCode::A, KeyCode::A, KeyCode::A]);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(200))
        .expect("Failed to inject and capture");

    // Expect B three times
    let expected = TestEvents::taps(&[KeyCode::B, KeyCode::B, KeyCode::B]);
    harness
        .verify(&captured, &expected)
        .expect("Three taps of A should produce three taps of B");
}

/// Test alternating between remapped keys.
///
/// Verifies that alternating between different remapped keys works correctly.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_alternating_remapped_keys -- --ignored"]
fn test_alternating_remapped_keys() {
    let config = E2EConfig::simple_remaps(vec![(KeyCode::A, KeyCode::B), (KeyCode::C, KeyCode::D)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject A, C, A pattern
    let input = TestEvents::taps(&[KeyCode::A, KeyCode::C, KeyCode::A]);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(200))
        .expect("Failed to inject and capture");

    // Expect B, D, B pattern
    let expected = TestEvents::taps(&[KeyCode::B, KeyCode::D, KeyCode::B]);
    harness
        .verify(&captured, &expected)
        .expect("A, C, A should produce B, D, B");
}

// ============================================================================
// Unmapped Key Passthrough Tests - Requirement 5.6
// ============================================================================

/// Test that unmapped keys pass through unchanged.
///
/// Verifies that when A→B is configured, pressing an unmapped key (C)
/// produces C without modification.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_unmapped_key_passthrough -- --ignored"]
fn test_unmapped_key_passthrough() {
    // Only A→B is configured
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject tap(C) which is not mapped
    let input = TestEvents::tap(KeyCode::C);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(100))
        .expect("Failed to inject and capture");

    // Expect tap(C) unchanged
    let expected = TestEvents::tap(KeyCode::C);
    harness
        .verify(&captured, &expected)
        .expect("Unmapped key C should pass through unchanged");
}

/// Test multiple unmapped keys in sequence.
///
/// Verifies that a sequence of unmapped keys all pass through correctly.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_multiple_unmapped_keys_passthrough -- --ignored"]
fn test_multiple_unmapped_keys_passthrough() {
    // Only A→B is configured
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject sequence of unmapped keys
    let input = TestEvents::taps(&[KeyCode::X, KeyCode::Y, KeyCode::Z]);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(200))
        .expect("Failed to inject and capture");

    // Expect same sequence unchanged
    let expected = TestEvents::taps(&[KeyCode::X, KeyCode::Y, KeyCode::Z]);
    harness
        .verify(&captured, &expected)
        .expect("Unmapped keys X, Y, Z should all pass through unchanged");
}

/// Test mixed remapped and unmapped keys.
///
/// Verifies that remapped keys are transformed while unmapped keys
/// pass through in the same sequence.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_mixed_mapped_unmapped_keys -- --ignored"]
fn test_mixed_mapped_unmapped_keys() {
    // A→B configured, but not C
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject: C (unmapped), A (mapped), C (unmapped)
    let input = TestEvents::taps(&[KeyCode::C, KeyCode::A, KeyCode::C]);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(200))
        .expect("Failed to inject and capture");

    // Expect: C (pass), B (remapped from A), C (pass)
    let expected = TestEvents::taps(&[KeyCode::C, KeyCode::B, KeyCode::C]);
    harness
        .verify(&captured, &expected)
        .expect("C should pass through, A should become B");
}

/// Test special keys passthrough (modifiers, function keys).
///
/// Verifies that special keys like Shift, Ctrl, F-keys pass through
/// when not explicitly mapped.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_special_keys_passthrough -- --ignored"]
fn test_special_keys_passthrough() {
    // Only A→B configured
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test Escape passthrough
    let captured = harness
        .inject_and_capture(
            &TestEvents::tap(KeyCode::Escape),
            Duration::from_millis(100),
        )
        .expect("Failed to inject Escape");
    harness
        .verify(&captured, &TestEvents::tap(KeyCode::Escape))
        .expect("Escape should pass through");

    // Test F1 passthrough
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::F1), Duration::from_millis(100))
        .expect("Failed to inject F1");
    harness
        .verify(&captured, &TestEvents::tap(KeyCode::F1))
        .expect("F1 should pass through");

    // Test Tab passthrough
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::Tab), Duration::from_millis(100))
        .expect("Failed to inject Tab");
    harness
        .verify(&captured, &TestEvents::tap(KeyCode::Tab))
        .expect("Tab should pass through");
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test CapsLock → Escape remapping (common use case).
///
/// This is a very common remapping that many users want.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_capslock_to_escape -- --ignored"]
fn test_capslock_to_escape() {
    let config = E2EConfig::simple_remap(KeyCode::CapsLock, KeyCode::Escape);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    let captured = harness
        .inject_and_capture(
            &TestEvents::tap(KeyCode::CapsLock),
            Duration::from_millis(100),
        )
        .expect("Failed to inject and capture");

    harness
        .verify(&captured, &TestEvents::tap(KeyCode::Escape))
        .expect("CapsLock should become Escape");
}

/// Test empty configuration (all keys passthrough).
///
/// Verifies that with no mappings configured, all keys pass through.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_empty_config_passthrough -- --ignored"]
fn test_empty_config_passthrough() {
    // No mappings
    let config = E2EConfig::default();
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture");

    harness
        .verify(&captured, &TestEvents::tap(KeyCode::A))
        .expect("A should pass through with empty config");
}

/// Test rapid key taps.
///
/// Verifies that rapid key presses are all captured and remapped correctly.
#[test]
#[ignore = "requires uinput access and daemon binary - run with: sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests test_rapid_key_taps -- --ignored"]
fn test_rapid_key_taps() {
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject 5 rapid taps
    let input: Vec<KeyEvent> = (0..5).flat_map(|_| TestEvents::tap(KeyCode::A)).collect();

    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(300))
        .expect("Failed to inject and capture");

    // Expect 5 taps of B
    let expected: Vec<KeyEvent> = (0..5).flat_map(|_| TestEvents::tap(KeyCode::B)).collect();
    harness
        .verify(&captured, &expected)
        .expect("All 5 rapid taps should be correctly remapped");
}
