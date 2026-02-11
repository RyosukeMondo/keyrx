//! Passthrough and edge case tests.
//!
//! Tests unmapped key passthrough and edge cases.

#![cfg(any(
    all(target_os = "linux", feature = "linux"),
    all(target_os = "windows", feature = "windows")
))]

mod e2e_harness;

use std::time::Duration;

use e2e_harness::{E2EConfig, E2EHarness, TestEvents};
use keyrx_core::config::{KeyCode, KeyMapping};
use keyrx_core::runtime::KeyEvent;

// ============================================================================
// Unmapped Key Passthrough Tests - Requirement 5.6
// ============================================================================

/// Test that unmapped keys pass through unchanged.
///
/// Verifies that when A→B is configured, pressing an unmapped key (C)
/// produces C without modification.
#[test]
fn test_unmapped_key_passthrough() {
    keyrx_daemon::skip_if_no_uinput!();
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
fn test_multiple_unmapped_keys_passthrough() {
    keyrx_daemon::skip_if_no_uinput!();
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
fn test_mixed_mapped_unmapped_keys() {
    keyrx_daemon::skip_if_no_uinput!();
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
fn test_special_keys_passthrough() {
    keyrx_daemon::skip_if_no_uinput!();
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
fn test_capslock_to_escape() {
    keyrx_daemon::skip_if_no_uinput!();
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
fn test_empty_config_passthrough() {
    keyrx_daemon::skip_if_no_uinput!();
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
fn test_rapid_key_taps() {
    keyrx_daemon::skip_if_no_uinput!();
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

