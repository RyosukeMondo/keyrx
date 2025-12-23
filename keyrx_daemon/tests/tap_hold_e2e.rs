//! Virtual E2E Tests for Tap-Hold functionality.
//!
//! These tests verify tap-hold behavior using virtual input devices (uinput)
//! to test the complete keyboard remapping pipeline.
//!
//! # Tap-Hold Behavior
//!
//! Tap-hold keys have dual functionality:
//! - **Tap**: Quick press and release outputs the tap key
//! - **Hold**: Holding past the threshold activates a modifier
//! - **Permissive Hold**: Another key pressed while pending confirms hold
//!
//! # Running These Tests
//!
//! These tests require:
//! - Linux with uinput module loaded (`sudo modprobe uinput`)
//! - Read/write access to `/dev/uinput` (add user to 'uinput' group)
//! - Read access to `/dev/input/event*` (add user to 'input' group)
//! - The keyrx_daemon binary built
//!
//! Run with:
//! ```bash
//! cargo test -p keyrx_daemon --features linux --test tap_hold_e2e
//! ```
//!
//! Tests automatically skip with a message if uinput/input access is not available.

#![cfg(all(target_os = "linux", feature = "linux"))]

mod e2e_harness;

use std::time::Duration;

use e2e_harness::{E2EConfig, E2EHarness, TestEvents};
use keyrx_core::config::KeyCode;

// ============================================================================
// Tap Path Tests - Quick press and release outputs tap key
// ============================================================================

/// Test tap-hold key quick tap outputs tap key.
///
/// When a tap-hold key is pressed and released quickly (under threshold),
/// it should output the tap key.
#[test]
fn test_tap_hold_quick_tap() {
    keyrx_daemon::skip_if_no_uinput!();

    // CapsLock: tap=Escape, hold=modifier 0, 200ms threshold
    let config = E2EConfig::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Quick tap (press + immediate release) should output Escape
    let input = TestEvents::tap(KeyCode::CapsLock);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(300))
        .expect("Failed to inject and capture");

    // Expect Escape tap (press + release)
    let expected = TestEvents::tap(KeyCode::Escape);
    harness
        .verify(&captured, &expected)
        .expect("Quick CapsLock tap should produce Escape tap");
}

/// Test multiple quick taps produce multiple tap keys.
///
/// Verifies that tap-hold state resets correctly after each tap.
#[test]
fn test_tap_hold_multiple_quick_taps() {
    keyrx_daemon::skip_if_no_uinput!();

    let config = E2EConfig::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // First tap
    let input1 = TestEvents::tap(KeyCode::CapsLock);
    let captured1 = harness
        .inject_and_capture(&input1, Duration::from_millis(300))
        .expect("Failed to inject first tap");
    harness
        .verify(&captured1, &TestEvents::tap(KeyCode::Escape))
        .expect("First tap should produce Escape");

    // Second tap
    let input2 = TestEvents::tap(KeyCode::CapsLock);
    let captured2 = harness
        .inject_and_capture(&input2, Duration::from_millis(300))
        .expect("Failed to inject second tap");
    harness
        .verify(&captured2, &TestEvents::tap(KeyCode::Escape))
        .expect("Second tap should also produce Escape");

    // Third tap
    let input3 = TestEvents::tap(KeyCode::CapsLock);
    let captured3 = harness
        .inject_and_capture(&input3, Duration::from_millis(300))
        .expect("Failed to inject third tap");
    harness
        .verify(&captured3, &TestEvents::tap(KeyCode::Escape))
        .expect("Third tap should also produce Escape");
}

/// Test quick tap with different keys.
///
/// Verifies tap behavior works for various key combinations.
#[test]
fn test_tap_hold_space_tap() {
    keyrx_daemon::skip_if_no_uinput!();

    // Space: tap=Space, hold=modifier 1, 150ms threshold
    let config = E2EConfig::tap_hold(KeyCode::Space, KeyCode::Space, 1, 150);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Quick tap should output Space
    let input = TestEvents::tap(KeyCode::Space);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(300))
        .expect("Failed to inject and capture");

    let expected = TestEvents::tap(KeyCode::Space);
    harness
        .verify(&captured, &expected)
        .expect("Quick Space tap should produce Space");
}

// ============================================================================
// Hold Path Tests - Holding past threshold activates modifier
// ============================================================================

/// Test tap-hold key held past threshold activates modifier.
///
/// When a tap-hold key is held past the threshold and then released,
/// no tap key should be produced (modifier was activated).
#[test]
fn test_tap_hold_hold_past_threshold() {
    keyrx_daemon::skip_if_no_uinput!();

    // CapsLock: tap=Escape, hold=modifier 0, 100ms threshold
    // Using shorter threshold for faster tests
    let config = E2EConfig::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 100);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press CapsLock
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to inject press");

    // Wait past threshold (150ms > 100ms)
    std::thread::sleep(Duration::from_millis(150));

    // Release CapsLock
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to inject release");

    // Capture any events
    let captured = harness
        .capture(Duration::from_millis(200))
        .expect("Failed to capture");

    // Modifier activation produces no output events
    // (modifier is internal state, not a key event)
    assert!(
        captured.is_empty(),
        "Hold past threshold should produce no output events, but got: {:?}",
        captured
    );
}

/// Test hold path activates modifier for conditional mappings.
///
/// When held, the modifier should be active and conditional mappings should work.
#[test]
fn test_tap_hold_hold_activates_layer() {
    keyrx_daemon::skip_if_no_uinput!();

    // CapsLock: tap=Escape, hold=modifier 0 with H→Left
    let config = E2EConfig::tap_hold_with_layer(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        100, // 100ms threshold
        vec![(KeyCode::H, KeyCode::Left)],
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press and hold CapsLock past threshold
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to inject CapsLock press");
    std::thread::sleep(Duration::from_millis(150));
    let _ = harness.drain();

    // Now press H - should be remapped to Left (modifier is active)
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(200))
        .expect("Failed to inject H");

    let expected = TestEvents::tap(KeyCode::Left);
    harness
        .verify(&captured, &expected)
        .expect("H should be Left when CapsLock held past threshold");

    // Release CapsLock
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release CapsLock");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Now H should pass through (modifier released)
    let captured_after = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(200))
        .expect("Failed to inject H after release");
    harness
        .verify(&captured_after, &TestEvents::tap(KeyCode::H))
        .expect("H should pass through after CapsLock released");
}

// ============================================================================
// Permissive Hold Tests - Another key during pending confirms hold
// ============================================================================

/// Test permissive hold - another key while pending confirms hold.
///
/// When another key is pressed while a tap-hold key is in pending state
/// (before threshold), it immediately confirms the hold behavior.
#[test]
fn test_tap_hold_permissive_hold() {
    keyrx_daemon::skip_if_no_uinput!();

    // CapsLock: tap=Escape, hold=modifier 0 with H→Left
    // Long threshold to ensure we're testing permissive hold, not timeout
    let config = E2EConfig::tap_hold_with_layer(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        500, // 500ms threshold - we won't wait this long
        vec![(KeyCode::H, KeyCode::Left)],
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press CapsLock (enters pending state)
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to inject CapsLock press");

    // Small delay, but well under threshold
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Press H while CapsLock is pending - this should trigger permissive hold
    // H should be remapped to Left because modifier is now confirmed
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(200))
        .expect("Failed to inject H");

    let expected = TestEvents::tap(KeyCode::Left);
    harness
        .verify(&captured, &expected)
        .expect("H should be Left (permissive hold activated)");

    // Release CapsLock
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release CapsLock");
}

/// Test permissive hold with multiple layer keys.
///
/// Verifies that vim-style navigation works with permissive hold.
#[test]
fn test_tap_hold_permissive_hold_vim_layer() {
    keyrx_daemon::skip_if_no_uinput!();

    // CapsLock: tap=Escape, hold=vim navigation layer
    let config = E2EConfig::tap_hold_with_layer(
        KeyCode::CapsLock,
        KeyCode::Escape,
        0,
        500, // Long threshold for permissive hold testing
        vec![
            (KeyCode::H, KeyCode::Left),
            (KeyCode::J, KeyCode::Down),
            (KeyCode::K, KeyCode::Up),
            (KeyCode::L, KeyCode::Right),
        ],
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press CapsLock (pending)
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press CapsLock");
    std::thread::sleep(Duration::from_millis(30));
    let _ = harness.drain();

    // Type HJKL while CapsLock is held (permissive hold)
    // Each key should confirm hold and use the layer
    let test_cases = [
        (KeyCode::H, KeyCode::Left),
        (KeyCode::J, KeyCode::Down),
        (KeyCode::K, KeyCode::Up),
        (KeyCode::L, KeyCode::Right),
    ];

    for (input_key, expected_key) in test_cases {
        let captured = harness
            .inject_and_capture(&TestEvents::tap(input_key), Duration::from_millis(200))
            .expect(&format!("Failed to inject {:?}", input_key));
        harness
            .verify(&captured, &TestEvents::tap(expected_key))
            .expect(&format!(
                "{:?} should become {:?} with permissive hold",
                input_key, expected_key
            ));
    }

    // Release CapsLock
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release CapsLock");
}

/// Test permissive hold doesn't produce tap key.
///
/// When permissive hold is triggered, the tap key should NOT be emitted.
#[test]
fn test_tap_hold_permissive_hold_no_tap_output() {
    keyrx_daemon::skip_if_no_uinput!();

    // CapsLock: tap=Escape, hold=modifier 0
    let config = E2EConfig::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 500);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press CapsLock (pending)
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press CapsLock");
    std::thread::sleep(Duration::from_millis(30));
    let _ = harness.drain();

    // Press another key to trigger permissive hold
    // The A key should pass through (no conditional mapping for it)
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(200))
        .expect("Failed to inject A");

    // A should pass through unchanged (no mapping for A in this config)
    harness
        .verify(&captured, &TestEvents::tap(KeyCode::A))
        .expect("A should pass through");

    // Now release CapsLock - should NOT produce Escape (hold was confirmed)
    let release_captured = harness
        .inject_and_capture(
            &TestEvents::release(KeyCode::CapsLock),
            Duration::from_millis(200),
        )
        .expect("Failed to capture release");

    assert!(
        release_captured.is_empty(),
        "Release after permissive hold should produce no events, but got: {:?}",
        release_captured
    );
}

// ============================================================================
// Edge Cases and Timing Tests
// ============================================================================

/// Test tap-hold at exact threshold boundary.
///
/// Behavior at exactly the threshold time (implementation-defined).
#[test]
fn test_tap_hold_at_threshold_boundary() {
    keyrx_daemon::skip_if_no_uinput!();

    // 100ms threshold
    let config = E2EConfig::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 100);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press CapsLock
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press CapsLock");

    // Wait exactly at threshold (100ms) - implementation may go either way
    std::thread::sleep(Duration::from_millis(100));

    // Release
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release CapsLock");

    // Capture - either tap key or no output (hold) is acceptable at boundary
    let captured = harness
        .capture(Duration::from_millis(200))
        .expect("Failed to capture");

    // At exact threshold, implementation may produce tap or hold
    // We just verify it's one of the expected behaviors
    if !captured.is_empty() {
        // If we got output, it should be the tap key
        harness
            .verify(&captured, &TestEvents::tap(KeyCode::Escape))
            .expect("If output at threshold, should be tap key");
    }
    // Empty output is also valid (threshold crossed to hold)
}

/// Test tap-hold with unmapped key interaction.
///
/// Verifies that unmapped keys during pending don't interfere.
#[test]
fn test_tap_hold_unmapped_key_during_pending() {
    keyrx_daemon::skip_if_no_uinput!();

    // Only CapsLock is tap-hold configured, no layer mappings
    let config = E2EConfig::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press CapsLock (pending)
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press CapsLock");
    std::thread::sleep(Duration::from_millis(30));
    let _ = harness.drain();

    // Press an unmapped key - should pass through and trigger permissive hold
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::X), Duration::from_millis(200))
        .expect("Failed to inject X");

    // X should pass through unchanged
    harness
        .verify(&captured, &TestEvents::tap(KeyCode::X))
        .expect("Unmapped key X should pass through during tap-hold");

    // Release CapsLock - no tap output (permissive hold was triggered)
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release CapsLock");
}

/// Test multiple concurrent tap-hold keys.
///
/// Verifies that multiple tap-hold keys can be active simultaneously.
#[test]
fn test_tap_hold_multiple_concurrent() {
    keyrx_daemon::skip_if_no_uinput!();

    // Configure two tap-hold keys with different modifiers
    let config =
        E2EConfig::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200).with_mappings(vec![
            keyrx_core::config::KeyMapping::tap_hold(
                KeyCode::Space,
                KeyCode::Space,
                1, // Different modifier
                200,
            ),
        ]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press both tap-hold keys
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press CapsLock");
    harness
        .inject(&TestEvents::press(KeyCode::Space))
        .expect("Failed to press Space");

    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Release both in reverse order
    harness
        .inject(&TestEvents::release(KeyCode::Space))
        .expect("Failed to release Space");
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release CapsLock");

    // Both were interrupted by each other (permissive hold), no tap output
    let captured = harness
        .capture(Duration::from_millis(200))
        .expect("Failed to capture");

    // Permissive hold should have been triggered, so no tap outputs
    assert!(
        captured.is_empty(),
        "Concurrent tap-holds with permissive hold should produce no output, got: {:?}",
        captured
    );
}

/// Test rapid tap-hold sequences.
///
/// Verifies state resets properly between rapid taps.
#[test]
fn test_tap_hold_rapid_sequence() {
    keyrx_daemon::skip_if_no_uinput!();

    let config = E2EConfig::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Rapid sequence of 5 quick taps
    for i in 0..5 {
        let captured = harness
            .inject_and_capture(
                &TestEvents::tap(KeyCode::CapsLock),
                Duration::from_millis(300),
            )
            .expect(&format!("Failed on tap {}", i));

        harness
            .verify(&captured, &TestEvents::tap(KeyCode::Escape))
            .expect(&format!("Tap {} should produce Escape", i));
    }
}

// ============================================================================
// Integration with Other Features
// ============================================================================

/// Test tap-hold combined with simple remaps.
///
/// Verifies tap-hold works alongside simple key remapping.
#[test]
fn test_tap_hold_with_simple_remap() {
    keyrx_daemon::skip_if_no_uinput!();

    // Tap-hold on CapsLock + simple A→B remap
    let config =
        E2EConfig::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200).with_mappings(vec![
            keyrx_core::config::KeyMapping::simple(KeyCode::A, KeyCode::B),
        ]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test simple remap still works
    let captured_a = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(200))
        .expect("Failed to inject A");
    harness
        .verify(&captured_a, &TestEvents::tap(KeyCode::B))
        .expect("A should be remapped to B");

    // Test tap-hold still works
    let captured_caps = harness
        .inject_and_capture(
            &TestEvents::tap(KeyCode::CapsLock),
            Duration::from_millis(300),
        )
        .expect("Failed to inject CapsLock");
    harness
        .verify(&captured_caps, &TestEvents::tap(KeyCode::Escape))
        .expect("CapsLock tap should produce Escape");
}

/// Test tap-hold passthrough for unmapped keys.
///
/// Verifies unmapped keys pass through unchanged when tap-hold is configured.
#[test]
fn test_tap_hold_passthrough() {
    keyrx_daemon::skip_if_no_uinput!();

    let config = E2EConfig::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Unmapped keys should pass through
    let test_keys = [KeyCode::A, KeyCode::B, KeyCode::Enter, KeyCode::Tab];

    for key in test_keys {
        let captured = harness
            .inject_and_capture(&TestEvents::tap(key), Duration::from_millis(200))
            .expect(&format!("Failed to inject {:?}", key));
        harness
            .verify(&captured, &TestEvents::tap(key))
            .expect(&format!("{:?} should pass through unchanged", key));
    }
}

/// Test hold-then-tap sequence.
///
/// After releasing a held key, the next quick tap should work correctly.
#[test]
fn test_tap_hold_hold_then_tap() {
    keyrx_daemon::skip_if_no_uinput!();

    let config = E2EConfig::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 100);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // First, hold past threshold
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press CapsLock");
    std::thread::sleep(Duration::from_millis(150));
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release CapsLock");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Now do a quick tap - should produce tap key
    let captured = harness
        .inject_and_capture(
            &TestEvents::tap(KeyCode::CapsLock),
            Duration::from_millis(300),
        )
        .expect("Failed to inject tap after hold");

    harness
        .verify(&captured, &TestEvents::tap(KeyCode::Escape))
        .expect("Tap after hold should produce Escape");
}
