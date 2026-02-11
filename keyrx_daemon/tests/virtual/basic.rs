//! Basic remapping tests.
//!
//! Tests simple key remapping and multiple sequential remaps.

#![cfg(any(
    all(target_os = "linux", feature = "linux"),
    all(target_os = "windows", feature = "windows")
))]

mod e2e_harness;

use std::time::Duration;

use e2e_harness::{E2EConfig, E2EHarness, TestEvents};
use keyrx_core::config::{KeyCode, KeyMapping};
use keyrx_core::runtime::KeyEvent;

/// Test simple A → B remapping (press event).
///
/// Verifies that when A is pressed, B is output instead.
#[test]
fn test_simple_remap_press() {
    keyrx_daemon::skip_if_no_uinput!();
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
fn test_simple_remap_release() {
    keyrx_daemon::skip_if_no_uinput!();
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
fn test_simple_remap_tap() {
    keyrx_daemon::skip_if_no_uinput!();
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
fn test_multiple_remaps_different_keys() {
    keyrx_daemon::skip_if_no_uinput!();
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
fn test_repeated_remap_sequence() {
    keyrx_daemon::skip_if_no_uinput!();
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
fn test_alternating_remapped_keys() {
    keyrx_daemon::skip_if_no_uinput!();
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

