//! Virtual E2E Tests for keyrx daemon.
//!
//! These tests use virtual input devices (uinput) to test the complete
//! keyboard remapping pipeline without requiring physical hardware.
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
//! cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests
//! ```
//!
//! Tests automatically skip with a message if uinput/input access is not available.
//!
//! # Granting Access
//!
//! ```bash
//! sudo usermod -aG uinput,input $USER
//! # Log out and back in for changes to take effect
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

// ============================================================================
// Modifier State Tests - Requirement 5.2
// ============================================================================

/// Test modifier activation produces no output.
///
/// When a key is configured as a modifier (state change only), pressing it
/// should not produce any output events.
#[test]
fn test_modifier_no_output() {
    keyrx_daemon::skip_if_no_uinput!();
    // CapsLock activates modifier 0 (no output)
    let config = E2EConfig::modifier(KeyCode::CapsLock, 0);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press and release modifier key
    let input = TestEvents::tap(KeyCode::CapsLock);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(150))
        .expect("Failed to inject and capture");

    // Expect no output events - modifier only sets internal state
    assert!(
        captured.is_empty(),
        "Modifier key should produce no output events, but got: {:?}",
        captured
    );
}

/// Test modifier key hold and release.
///
/// Verifies that pressing a modifier sets internal state, and releasing it
/// clears the state, all without producing any output.
#[test]
fn test_modifier_hold_release_no_output() {
    keyrx_daemon::skip_if_no_uinput!();
    let config = E2EConfig::modifier(KeyCode::CapsLock, 0);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Just press (hold) the modifier
    let press_input = TestEvents::press(KeyCode::CapsLock);
    let captured_press = harness
        .inject_and_capture(&press_input, Duration::from_millis(100))
        .expect("Failed to inject press");

    assert!(
        captured_press.is_empty(),
        "Modifier press should produce no output, but got: {:?}",
        captured_press
    );

    // Now release the modifier
    let release_input = TestEvents::release(KeyCode::CapsLock);
    let captured_release = harness
        .inject_and_capture(&release_input, Duration::from_millis(100))
        .expect("Failed to inject release");

    assert!(
        captured_release.is_empty(),
        "Modifier release should produce no output, but got: {:?}",
        captured_release
    );
}

// ============================================================================
// Lock State Tests - Requirement 5.3
// ============================================================================

/// Test lock toggle on first press.
///
/// Lock keys toggle internal state on press. The first press activates the lock.
#[test]
fn test_lock_toggle_no_output() {
    keyrx_daemon::skip_if_no_uinput!();
    // ScrollLock toggles lock 0 (no output)
    let config = E2EConfig::lock(KeyCode::ScrollLock, 0);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press lock key (toggles on)
    let input = TestEvents::tap(KeyCode::ScrollLock);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(150))
        .expect("Failed to inject and capture");

    // Expect no output - lock only sets internal state
    assert!(
        captured.is_empty(),
        "Lock key should produce no output events, but got: {:?}",
        captured
    );
}

/// Test lock toggle on second press.
///
/// The second press of a lock key should toggle the lock off.
/// Neither press nor release should produce output.
#[test]
fn test_lock_double_toggle_no_output() {
    keyrx_daemon::skip_if_no_uinput!();
    let config = E2EConfig::lock(KeyCode::ScrollLock, 0);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // First tap toggles lock ON
    let first_tap = TestEvents::tap(KeyCode::ScrollLock);
    let captured_first = harness
        .inject_and_capture(&first_tap, Duration::from_millis(100))
        .expect("Failed to inject first tap");

    assert!(
        captured_first.is_empty(),
        "First lock tap should produce no output, but got: {:?}",
        captured_first
    );

    // Second tap toggles lock OFF
    let second_tap = TestEvents::tap(KeyCode::ScrollLock);
    let captured_second = harness
        .inject_and_capture(&second_tap, Duration::from_millis(100))
        .expect("Failed to inject second tap");

    assert!(
        captured_second.is_empty(),
        "Second lock tap should produce no output, but got: {:?}",
        captured_second
    );
}

/// Test lock release produces no output.
///
/// Unlike modifiers (which are momentary), locks only toggle on press.
/// Release should be ignored and produce no output.
#[test]
fn test_lock_release_ignored() {
    keyrx_daemon::skip_if_no_uinput!();
    let config = E2EConfig::lock(KeyCode::ScrollLock, 0);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press the lock key
    let press_input = TestEvents::press(KeyCode::ScrollLock);
    let captured_press = harness
        .inject_and_capture(&press_input, Duration::from_millis(100))
        .expect("Failed to inject press");

    assert!(
        captured_press.is_empty(),
        "Lock press should produce no output"
    );

    // Release should also produce no output (locks toggle on press only)
    let release_input = TestEvents::release(KeyCode::ScrollLock);
    let captured_release = harness
        .inject_and_capture(&release_input, Duration::from_millis(100))
        .expect("Failed to inject release");

    assert!(
        captured_release.is_empty(),
        "Lock release should produce no output (locks toggle on press only)"
    );
}

// ============================================================================
// Conditional Mapping Tests - Requirement 5.4
// ============================================================================

/// Test conditional mapping with modifier active.
///
/// When modifier is held, the conditional mapping should be applied.
#[test]
fn test_conditional_with_modifier_active() {
    keyrx_daemon::skip_if_no_uinput!();
    // CapsLock activates modifier 0, H→Left when modifier 0 is active
    let config =
        E2EConfig::with_modifier_layer(KeyCode::CapsLock, 0, vec![(KeyCode::H, KeyCode::Left)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press and hold modifier (CapsLock)
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to inject modifier press");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Now press H while modifier is held - should produce Left
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(100))
        .expect("Failed to inject and capture H");

    // Expect Left key (H is remapped when modifier 0 is active)
    let expected = TestEvents::tap(KeyCode::Left);
    harness
        .verify(&captured, &expected)
        .expect("H should become Left when modifier is active");

    // Release modifier
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to inject modifier release");
}

/// Test conditional mapping without modifier (passthrough).
///
/// When modifier is not active, the conditional mapping should not apply,
/// and the key should pass through unchanged.
#[test]
fn test_conditional_without_modifier_passthrough() {
    keyrx_daemon::skip_if_no_uinput!();
    // CapsLock activates modifier 0, H→Left when modifier 0 is active
    let config =
        E2EConfig::with_modifier_layer(KeyCode::CapsLock, 0, vec![(KeyCode::H, KeyCode::Left)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press H without modifier active - should pass through as H
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(100))
        .expect("Failed to inject and capture H");

    // Expect H unchanged (conditional not active)
    let expected = TestEvents::tap(KeyCode::H);
    harness
        .verify(&captured, &expected)
        .expect("H should pass through unchanged when modifier is not active");
}

/// Test conditional mapping with lock active.
///
/// When lock is toggled on, the conditional mapping should be applied.
#[test]
fn test_conditional_with_lock_active() {
    keyrx_daemon::skip_if_no_uinput!();
    // ScrollLock toggles lock 0, 1→F1 when lock 0 is active
    let config =
        E2EConfig::with_lock_layer(KeyCode::ScrollLock, 0, vec![(KeyCode::Num1, KeyCode::F1)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Toggle lock ON
    harness
        .inject(&TestEvents::tap(KeyCode::ScrollLock))
        .expect("Failed to toggle lock on");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Press 1 while lock is on - should produce F1
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::Num1), Duration::from_millis(100))
        .expect("Failed to inject and capture Num1");

    // Expect F1 key (Num1 is remapped when lock 0 is active)
    let expected = TestEvents::tap(KeyCode::F1);
    harness
        .verify(&captured, &expected)
        .expect("Num1 should become F1 when lock is active");
}

/// Test conditional mapping with lock inactive (passthrough).
///
/// When lock is toggled off, the conditional mapping should not apply.
#[test]
fn test_conditional_without_lock_passthrough() {
    keyrx_daemon::skip_if_no_uinput!();
    // ScrollLock toggles lock 0, 1→F1 when lock 0 is active
    let config =
        E2EConfig::with_lock_layer(KeyCode::ScrollLock, 0, vec![(KeyCode::Num1, KeyCode::F1)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press 1 without lock active - should pass through as 1
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::Num1), Duration::from_millis(100))
        .expect("Failed to inject and capture Num1");

    // Expect Num1 unchanged (lock not active)
    let expected = TestEvents::tap(KeyCode::Num1);
    harness
        .verify(&captured, &expected)
        .expect("Num1 should pass through unchanged when lock is not active");
}

/// Test conditional mapping after lock toggle off.
///
/// After toggling lock off, the conditional mapping should no longer apply.
#[test]
fn test_conditional_after_lock_toggle_off() {
    keyrx_daemon::skip_if_no_uinput!();
    let config =
        E2EConfig::with_lock_layer(KeyCode::ScrollLock, 0, vec![(KeyCode::Num1, KeyCode::F1)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Toggle lock ON
    harness
        .inject(&TestEvents::tap(KeyCode::ScrollLock))
        .expect("Failed to toggle lock on");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Verify Num1 → F1 while lock is on
    let captured_on = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::Num1), Duration::from_millis(100))
        .expect("Failed to capture with lock on");
    harness
        .verify(&captured_on, &TestEvents::tap(KeyCode::F1))
        .expect("Num1 should become F1 when lock is on");

    // Toggle lock OFF
    harness
        .inject(&TestEvents::tap(KeyCode::ScrollLock))
        .expect("Failed to toggle lock off");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Verify Num1 passes through now (lock is off)
    let captured_off = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::Num1), Duration::from_millis(100))
        .expect("Failed to capture with lock off");
    harness
        .verify(&captured_off, &TestEvents::tap(KeyCode::Num1))
        .expect("Num1 should pass through when lock is off");
}

/// Test conditional mapping after modifier released.
///
/// After releasing modifier, the conditional mapping should no longer apply.
#[test]
fn test_conditional_after_modifier_released() {
    keyrx_daemon::skip_if_no_uinput!();
    let config =
        E2EConfig::with_modifier_layer(KeyCode::CapsLock, 0, vec![(KeyCode::H, KeyCode::Left)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press modifier
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press modifier");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Verify H → Left while modifier is held
    let captured_held = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(100))
        .expect("Failed to capture with modifier held");
    harness
        .verify(&captured_held, &TestEvents::tap(KeyCode::Left))
        .expect("H should become Left when modifier is held");

    // Release modifier
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release modifier");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Verify H passes through now (modifier released)
    let captured_released = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(100))
        .expect("Failed to capture with modifier released");
    harness
        .verify(&captured_released, &TestEvents::tap(KeyCode::H))
        .expect("H should pass through when modifier is released");
}

/// Test multiple conditional mappings in same layer.
///
/// Verifies that multiple keys can be remapped within the same modifier layer.
#[test]
fn test_multiple_conditionals_same_layer() {
    keyrx_daemon::skip_if_no_uinput!();
    // Vim-style navigation: CapsLock + HJKL → arrows
    let config = E2EConfig::with_modifier_layer(
        KeyCode::CapsLock,
        0,
        vec![
            (KeyCode::H, KeyCode::Left),
            (KeyCode::J, KeyCode::Down),
            (KeyCode::K, KeyCode::Up),
            (KeyCode::L, KeyCode::Right),
        ],
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press modifier
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press modifier");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Test each navigation key
    let test_cases = [
        (KeyCode::H, KeyCode::Left),
        (KeyCode::J, KeyCode::Down),
        (KeyCode::K, KeyCode::Up),
        (KeyCode::L, KeyCode::Right),
    ];

    for (input_key, expected_key) in test_cases {
        let captured = harness
            .inject_and_capture(&TestEvents::tap(input_key), Duration::from_millis(100))
            .expect(&format!("Failed to capture {:?}", input_key));
        harness
            .verify(&captured, &TestEvents::tap(expected_key))
            .expect(&format!(
                "{:?} should become {:?} when modifier is held",
                input_key, expected_key
            ));
    }

    // Release modifier
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release modifier");
}

// ============================================================================
// Modified Output Tests - Requirement 5.5
// ============================================================================

/// Test Shift+Key output sequence.
///
/// Verifies that a modified output mapping produces the correct event sequence:
/// Press: Press(LShift) → Press(key)
/// Release: Release(key) → Release(LShift)
#[test]
fn test_modified_output_shift() {
    keyrx_daemon::skip_if_no_uinput!();
    // A → Shift+1 (outputs '!' on most layouts)
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Num1,
        true,  // shift
        false, // ctrl
        false, // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test press event produces: Press(LShift), Press(Num1)
    let press_captured = harness
        .inject_and_capture(&TestEvents::press(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture press");

    let expected_press = vec![
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::Num1),
    ];
    harness
        .verify(&press_captured, &expected_press)
        .expect("Press A should produce Press(LShift), Press(Num1)");

    // Test release event produces: Release(Num1), Release(LShift)
    let release_captured = harness
        .inject_and_capture(&TestEvents::release(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture release");

    let expected_release = vec![
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Release(KeyCode::LShift),
    ];
    harness
        .verify(&release_captured, &expected_release)
        .expect("Release A should produce Release(Num1), Release(LShift)");
}

/// Test Ctrl+Key combination.
///
/// Verifies that Ctrl modifier is correctly applied to the output.
#[test]
fn test_modified_output_ctrl() {
    keyrx_daemon::skip_if_no_uinput!();
    // A → Ctrl+C (copy shortcut)
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::C,
        false, // shift
        true,  // ctrl
        false, // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test press event produces: Press(LCtrl), Press(C)
    let press_captured = harness
        .inject_and_capture(&TestEvents::press(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture press");

    let expected_press = vec![KeyEvent::Press(KeyCode::LCtrl), KeyEvent::Press(KeyCode::C)];
    harness
        .verify(&press_captured, &expected_press)
        .expect("Press A should produce Press(LCtrl), Press(C)");

    // Test release event produces: Release(C), Release(LCtrl)
    let release_captured = harness
        .inject_and_capture(&TestEvents::release(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture release");

    let expected_release = vec![
        KeyEvent::Release(KeyCode::C),
        KeyEvent::Release(KeyCode::LCtrl),
    ];
    harness
        .verify(&release_captured, &expected_release)
        .expect("Release A should produce Release(C), Release(LCtrl)");
}

/// Test Alt+Key combination.
///
/// Verifies that Alt modifier is correctly applied to the output.
#[test]
fn test_modified_output_alt() {
    keyrx_daemon::skip_if_no_uinput!();
    // A → Alt+Tab (window switcher)
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Tab,
        false, // shift
        false, // ctrl
        true,  // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test press event produces: Press(LAlt), Press(Tab)
    let press_captured = harness
        .inject_and_capture(&TestEvents::press(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture press");

    let expected_press = vec![
        KeyEvent::Press(KeyCode::LAlt),
        KeyEvent::Press(KeyCode::Tab),
    ];
    harness
        .verify(&press_captured, &expected_press)
        .expect("Press A should produce Press(LAlt), Press(Tab)");

    // Test release event produces: Release(Tab), Release(LAlt)
    let release_captured = harness
        .inject_and_capture(&TestEvents::release(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture release");

    let expected_release = vec![
        KeyEvent::Release(KeyCode::Tab),
        KeyEvent::Release(KeyCode::LAlt),
    ];
    harness
        .verify(&release_captured, &expected_release)
        .expect("Release A should produce Release(Tab), Release(LAlt)");
}

/// Test Ctrl+Shift+Key multiple modifier combination.
///
/// Verifies correct ordering when multiple modifiers are used:
/// Press order: LShift → LCtrl → key
/// Release order: key → LCtrl → LShift
#[test]
fn test_modified_output_ctrl_shift() {
    keyrx_daemon::skip_if_no_uinput!();
    // A → Ctrl+Shift+S (save as shortcut)
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::S,
        true,  // shift
        true,  // ctrl
        false, // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test press event produces: Press(LShift), Press(LCtrl), Press(S)
    // Note: Order is shift, ctrl, alt, win, key
    let press_captured = harness
        .inject_and_capture(&TestEvents::press(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture press");

    let expected_press = vec![
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::LCtrl),
        KeyEvent::Press(KeyCode::S),
    ];
    harness
        .verify(&press_captured, &expected_press)
        .expect("Press A should produce Press(LShift), Press(LCtrl), Press(S)");

    // Test release event produces: Release(S), Release(LCtrl), Release(LShift)
    // Reverse order of modifiers
    let release_captured = harness
        .inject_and_capture(&TestEvents::release(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture release");

    let expected_release = vec![
        KeyEvent::Release(KeyCode::S),
        KeyEvent::Release(KeyCode::LCtrl),
        KeyEvent::Release(KeyCode::LShift),
    ];
    harness
        .verify(&release_captured, &expected_release)
        .expect("Release A should produce Release(S), Release(LCtrl), Release(LShift)");
}

/// Test Ctrl+Alt+Key combination (common for system shortcuts).
///
/// Verifies correct ordering for Ctrl+Alt combinations.
#[test]
fn test_modified_output_ctrl_alt() {
    keyrx_daemon::skip_if_no_uinput!();
    // A → Ctrl+Alt+Delete style shortcut
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Delete,
        false, // shift
        true,  // ctrl
        true,  // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test press event produces: Press(LCtrl), Press(LAlt), Press(Delete)
    let press_captured = harness
        .inject_and_capture(&TestEvents::press(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture press");

    let expected_press = vec![
        KeyEvent::Press(KeyCode::LCtrl),
        KeyEvent::Press(KeyCode::LAlt),
        KeyEvent::Press(KeyCode::Delete),
    ];
    harness
        .verify(&press_captured, &expected_press)
        .expect("Press A should produce Press(LCtrl), Press(LAlt), Press(Delete)");

    // Test release event produces: Release(Delete), Release(LAlt), Release(LCtrl)
    let release_captured = harness
        .inject_and_capture(&TestEvents::release(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture release");

    let expected_release = vec![
        KeyEvent::Release(KeyCode::Delete),
        KeyEvent::Release(KeyCode::LAlt),
        KeyEvent::Release(KeyCode::LCtrl),
    ];
    harness
        .verify(&release_captured, &expected_release)
        .expect("Release A should produce Release(Delete), Release(LAlt), Release(LCtrl)");
}

/// Test all modifiers (Shift+Ctrl+Alt+Win).
///
/// Verifies correct ordering when all four modifiers are used.
#[test]
fn test_modified_output_all_modifiers() {
    keyrx_daemon::skip_if_no_uinput!();
    // A → Shift+Ctrl+Alt+Win+Z (hypothetical super shortcut)
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Z,
        true, // shift
        true, // ctrl
        true, // alt
        true, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Test press event produces: Press(LShift), Press(LCtrl), Press(LAlt), Press(LMeta), Press(Z)
    let press_captured = harness
        .inject_and_capture(&TestEvents::press(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture press");

    let expected_press = vec![
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::LCtrl),
        KeyEvent::Press(KeyCode::LAlt),
        KeyEvent::Press(KeyCode::LMeta),
        KeyEvent::Press(KeyCode::Z),
    ];
    harness
        .verify(&press_captured, &expected_press)
        .expect("Press A should produce all modifiers then Z");

    // Test release event produces: Release(Z), Release(LMeta), Release(LAlt), Release(LCtrl), Release(LShift)
    let release_captured = harness
        .inject_and_capture(&TestEvents::release(KeyCode::A), Duration::from_millis(100))
        .expect("Failed to inject and capture release");

    let expected_release = vec![
        KeyEvent::Release(KeyCode::Z),
        KeyEvent::Release(KeyCode::LMeta),
        KeyEvent::Release(KeyCode::LAlt),
        KeyEvent::Release(KeyCode::LCtrl),
        KeyEvent::Release(KeyCode::LShift),
    ];
    harness
        .verify(&release_captured, &expected_release)
        .expect("Release A should release Z then all modifiers in reverse");
}

/// Test complete modified output tap sequence.
///
/// Verifies that a full tap (press+release) produces the complete correct sequence.
#[test]
fn test_modified_output_complete_tap() {
    keyrx_daemon::skip_if_no_uinput!();
    // A → Shift+1 complete tap
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Num1,
        true,  // shift
        false, // ctrl
        false, // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject a complete tap (press + release)
    let captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(150))
        .expect("Failed to inject and capture tap");

    // Expected full sequence:
    // Press(LShift), Press(Num1), Release(Num1), Release(LShift)
    let expected = vec![
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::Num1),
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Release(KeyCode::LShift),
    ];
    harness
        .verify(&captured, &expected)
        .expect("Tap A should produce complete Shift+1 sequence");
}

/// Test multiple modified output taps in sequence.
///
/// Verifies that multiple modified output mappings work correctly in sequence.
#[test]
fn test_modified_output_multiple_taps() {
    keyrx_daemon::skip_if_no_uinput!();
    // A → Shift+1
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Num1,
        true,  // shift
        false, // ctrl
        false, // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // First tap
    let captured1 = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(150))
        .expect("Failed to inject first tap");

    let expected_tap = vec![
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::Num1),
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Release(KeyCode::LShift),
    ];
    harness
        .verify(&captured1, &expected_tap)
        .expect("First tap should produce complete Shift+1 sequence");

    // Second tap - verify no state leakage
    let captured2 = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(150))
        .expect("Failed to inject second tap");

    harness
        .verify(&captured2, &expected_tap)
        .expect("Second tap should produce same complete Shift+1 sequence");

    // Third tap
    let captured3 = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(150))
        .expect("Failed to inject third tap");

    harness
        .verify(&captured3, &expected_tap)
        .expect("Third tap should produce same complete Shift+1 sequence");
}

/// Test modified output with unmapped key interleaving.
///
/// Verifies that modified output mappings don't affect unmapped keys.
#[test]
fn test_modified_output_with_passthrough() {
    keyrx_daemon::skip_if_no_uinput!();
    // A → Shift+1, but B is unmapped
    let config = E2EConfig::modified_output(
        KeyCode::A,
        KeyCode::Num1,
        true,  // shift
        false, // ctrl
        false, // alt
        false, // win
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // First test unmapped key passes through
    let b_captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::B), Duration::from_millis(100))
        .expect("Failed to inject B");
    harness
        .verify(&b_captured, &TestEvents::tap(KeyCode::B))
        .expect("B should pass through unchanged");

    // Then test modified output still works
    let a_captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::A), Duration::from_millis(150))
        .expect("Failed to inject A");
    let expected = vec![
        KeyEvent::Press(KeyCode::LShift),
        KeyEvent::Press(KeyCode::Num1),
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Release(KeyCode::LShift),
    ];
    harness
        .verify(&a_captured, &expected)
        .expect("A should produce Shift+1");

    // And unmapped key still passes through after
    let c_captured = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::C), Duration::from_millis(100))
        .expect("Failed to inject C");
    harness
        .verify(&c_captured, &TestEvents::tap(KeyCode::C))
        .expect("C should pass through unchanged");
}

// ============================================================================
// Multi-Event Sequence Tests - Requirement 5.7
// ============================================================================

/// Test typing pattern with multiple taps in sequence.
///
/// Verifies that complex typing patterns work correctly - multiple keys
/// tapped in sequence without any event loss or reordering.
#[test]
fn test_typing_pattern_sequence() {
    keyrx_daemon::skip_if_no_uinput!();
    // Multiple remaps: A→1, B→2, C→3
    let config = E2EConfig::simple_remaps(vec![
        (KeyCode::A, KeyCode::Num1),
        (KeyCode::B, KeyCode::Num2),
        (KeyCode::C, KeyCode::Num3),
    ]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Type "ABC" which should produce "123"
    let input = TestEvents::taps(&[KeyCode::A, KeyCode::B, KeyCode::C]);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(300))
        .expect("Failed to inject and capture typing sequence");

    // Expect "123" - all three key taps in order
    let expected = TestEvents::taps(&[KeyCode::Num1, KeyCode::Num2, KeyCode::Num3]);
    harness
        .verify(&captured, &expected)
        .expect("Typing ABC should produce 123 in correct order");
}

/// Test typing pattern with mixed mapped and unmapped keys.
///
/// Verifies that typing with interleaved mapped/unmapped keys works correctly.
#[test]
fn test_typing_mixed_mapped_unmapped() {
    keyrx_daemon::skip_if_no_uinput!();
    // Only A→1, B→2 mapped; X, Y unmapped
    let config = E2EConfig::simple_remaps(vec![
        (KeyCode::A, KeyCode::Num1),
        (KeyCode::B, KeyCode::Num2),
    ]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Type "AXBY" - mixed mapped and unmapped
    let input = TestEvents::taps(&[KeyCode::A, KeyCode::X, KeyCode::B, KeyCode::Y]);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(400))
        .expect("Failed to inject and capture mixed sequence");

    // Expect "1X2Y" - mapped keys transformed, unmapped passed through
    let expected = TestEvents::taps(&[KeyCode::Num1, KeyCode::X, KeyCode::Num2, KeyCode::Y]);
    harness
        .verify(&captured, &expected)
        .expect("Typing AXBY should produce 1X2Y in correct order");
}

/// Test modifier hold during typing (shift layer).
///
/// Verifies that holding a modifier while typing multiple keys applies
/// the modifier layer to all subsequent keys.
#[test]
fn test_modifier_hold_during_typing() {
    keyrx_daemon::skip_if_no_uinput!();
    // CapsLock activates modifier 0
    // H→Left, J→Down, K→Up, L→Right when modifier 0 is active
    let config = E2EConfig::with_modifier_layer(
        KeyCode::CapsLock,
        0,
        vec![
            (KeyCode::H, KeyCode::Left),
            (KeyCode::J, KeyCode::Down),
            (KeyCode::K, KeyCode::Up),
            (KeyCode::L, KeyCode::Right),
        ],
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press and hold modifier
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press modifier");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Type HJKL while modifier is held - should produce arrow keys
    let input = TestEvents::taps(&[KeyCode::H, KeyCode::J, KeyCode::K, KeyCode::L]);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(400))
        .expect("Failed to inject and capture HJKL sequence");

    // Expect Left, Down, Up, Right
    let expected = TestEvents::taps(&[KeyCode::Left, KeyCode::Down, KeyCode::Up, KeyCode::Right]);
    harness
        .verify(&captured, &expected)
        .expect("HJKL with modifier held should produce arrow keys");

    // Release modifier
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release modifier");
}

/// Test state accumulation across events.
///
/// Verifies that lock state persists correctly across many key events
/// and that state changes are properly maintained.
#[test]
fn test_state_accumulation_lock() {
    keyrx_daemon::skip_if_no_uinput!();
    // ScrollLock toggles lock 0, Num1→F1 when lock 0 is active
    let config =
        E2EConfig::with_lock_layer(KeyCode::ScrollLock, 0, vec![(KeyCode::Num1, KeyCode::F1)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Without lock: Num1 passes through
    let captured1 = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::Num1), Duration::from_millis(100))
        .expect("Failed to capture without lock");
    harness
        .verify(&captured1, &TestEvents::tap(KeyCode::Num1))
        .expect("Num1 should pass through without lock");

    // Toggle lock ON
    harness
        .inject(&TestEvents::tap(KeyCode::ScrollLock))
        .expect("Failed to toggle lock on");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // With lock: Num1 → F1 (test multiple times to verify state persists)
    for i in 0..3 {
        let captured = harness
            .inject_and_capture(&TestEvents::tap(KeyCode::Num1), Duration::from_millis(100))
            .expect(&format!("Failed to capture with lock (iteration {})", i));
        harness
            .verify(&captured, &TestEvents::tap(KeyCode::F1))
            .expect(&format!(
                "Num1 should become F1 with lock active (iteration {})",
                i
            ));
    }

    // Toggle lock OFF
    harness
        .inject(&TestEvents::tap(KeyCode::ScrollLock))
        .expect("Failed to toggle lock off");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // After lock off: Num1 passes through again
    let captured_final = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::Num1), Duration::from_millis(100))
        .expect("Failed to capture after lock off");
    harness
        .verify(&captured_final, &TestEvents::tap(KeyCode::Num1))
        .expect("Num1 should pass through after lock toggled off");
}

/// Test state transitions during rapid typing.
///
/// Verifies that state changes during rapid typing don't cause issues.
#[test]
fn test_state_transition_rapid_typing() {
    keyrx_daemon::skip_if_no_uinput!();
    // CapsLock activates modifier 0, H→Left when modifier 0 is active
    let config =
        E2EConfig::with_modifier_layer(KeyCode::CapsLock, 0, vec![(KeyCode::H, KeyCode::Left)]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Type H without modifier - should pass through
    let captured1 = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(100))
        .expect("Failed to capture H without modifier");
    harness
        .verify(&captured1, &TestEvents::tap(KeyCode::H))
        .expect("H should pass through without modifier");

    // Press modifier, type H, release modifier - all rapidly
    let rapid_sequence = vec![
        KeyEvent::Press(KeyCode::CapsLock),
        KeyEvent::Press(KeyCode::H),
        KeyEvent::Release(KeyCode::H),
        KeyEvent::Release(KeyCode::CapsLock),
    ];
    let captured2 = harness
        .inject_and_capture(&rapid_sequence, Duration::from_millis(200))
        .expect("Failed to capture rapid modifier+key sequence");

    // Expect Left key (H remapped while modifier was held)
    let expected = TestEvents::tap(KeyCode::Left);
    harness
        .verify(&captured2, &expected)
        .expect("H should become Left during rapid modifier sequence");

    // Verify modifier is released - H should pass through again
    let captured3 = harness
        .inject_and_capture(&TestEvents::tap(KeyCode::H), Duration::from_millis(100))
        .expect("Failed to capture H after modifier released");
    harness
        .verify(&captured3, &TestEvents::tap(KeyCode::H))
        .expect("H should pass through after modifier released");
}

/// Test complex vim-style navigation layer.
///
/// Verifies that a full vim navigation layer works correctly with
/// multiple navigation keys used in sequence.
#[test]
fn test_vim_navigation_layer_complex() {
    keyrx_daemon::skip_if_no_uinput!();
    // CapsLock activates modifier 0
    // Vim-style: HJKL → arrows, W/B → Ctrl+Right/Left (word navigation)
    let config = E2EConfig::with_modifier_layer(
        KeyCode::CapsLock,
        0,
        vec![
            (KeyCode::H, KeyCode::Left),
            (KeyCode::J, KeyCode::Down),
            (KeyCode::K, KeyCode::Up),
            (KeyCode::L, KeyCode::Right),
            (KeyCode::Num0, KeyCode::Home), // 0 → Home
            (KeyCode::Num4, KeyCode::End),  // $ (Shift+4) → End, here just 4→End for simplicity
        ],
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press and hold modifier
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press modifier");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Simulate vim navigation: go to start of line (0), then move right twice (ll)
    let vim_sequence = TestEvents::taps(&[KeyCode::Num0, KeyCode::L, KeyCode::L]);
    let captured = harness
        .inject_and_capture(&vim_sequence, Duration::from_millis(300))
        .expect("Failed to inject vim navigation sequence");

    // Expect: Home, Right, Right
    let expected = TestEvents::taps(&[KeyCode::Home, KeyCode::Right, KeyCode::Right]);
    harness
        .verify(&captured, &expected)
        .expect("Vim navigation 0ll should produce Home, Right, Right");

    // Continue with down and right: jl
    let more_nav = TestEvents::taps(&[KeyCode::J, KeyCode::L]);
    let captured2 = harness
        .inject_and_capture(&more_nav, Duration::from_millis(200))
        .expect("Failed to inject jl sequence");

    let expected2 = TestEvents::taps(&[KeyCode::Down, KeyCode::Right]);
    harness
        .verify(&captured2, &expected2)
        .expect("jl should produce Down, Right");

    // Release modifier
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release modifier");
}

/// Test no event loss during rapid key sequences.
///
/// Verifies that rapid key sequences don't lose any events.
#[test]
fn test_no_event_loss_rapid_sequence() {
    keyrx_daemon::skip_if_no_uinput!();
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Inject 10 rapid taps of A
    let input: Vec<KeyEvent> = (0..10).flat_map(|_| TestEvents::tap(KeyCode::A)).collect();

    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(500))
        .expect("Failed to inject rapid sequence");

    // Expect exactly 10 taps of B (20 events total: 10 press + 10 release)
    assert_eq!(
        captured.len(),
        20,
        "Expected 20 events (10 taps), got {}",
        captured.len()
    );

    let expected: Vec<KeyEvent> = (0..10).flat_map(|_| TestEvents::tap(KeyCode::B)).collect();
    harness
        .verify(&captured, &expected)
        .expect("All 10 rapid taps should be captured without loss");
}

/// Test event ordering is preserved.
///
/// Verifies that events are not reordered during processing.
#[test]
fn test_event_ordering_preserved() {
    keyrx_daemon::skip_if_no_uinput!();
    // Map A→1, B→2, C→3 to easily track ordering
    let config = E2EConfig::simple_remaps(vec![
        (KeyCode::A, KeyCode::Num1),
        (KeyCode::B, KeyCode::Num2),
        (KeyCode::C, KeyCode::Num3),
    ]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Create interleaved press/release sequence: Press(A), Press(B), Release(A), Press(C), Release(B), Release(C)
    let input = vec![
        KeyEvent::Press(KeyCode::A),
        KeyEvent::Press(KeyCode::B),
        KeyEvent::Release(KeyCode::A),
        KeyEvent::Press(KeyCode::C),
        KeyEvent::Release(KeyCode::B),
        KeyEvent::Release(KeyCode::C),
    ];

    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(300))
        .expect("Failed to inject interleaved sequence");

    // Expected order must be preserved: Press(1), Press(2), Release(1), Press(3), Release(2), Release(3)
    let expected = vec![
        KeyEvent::Press(KeyCode::Num1),
        KeyEvent::Press(KeyCode::Num2),
        KeyEvent::Release(KeyCode::Num1),
        KeyEvent::Press(KeyCode::Num3),
        KeyEvent::Release(KeyCode::Num2),
        KeyEvent::Release(KeyCode::Num3),
    ];
    harness
        .verify(&captured, &expected)
        .expect("Event ordering must be preserved through daemon");
}

/// Test overlapping key presses (like gaming scenarios).
///
/// Verifies that overlapping key presses work correctly.
#[test]
fn test_overlapping_key_presses() {
    keyrx_daemon::skip_if_no_uinput!();
    let config = E2EConfig::simple_remaps(vec![
        (KeyCode::W, KeyCode::Up),
        (KeyCode::A, KeyCode::Left),
        (KeyCode::S, KeyCode::Down),
        (KeyCode::D, KeyCode::Right),
    ]);
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Simulate WASD with overlapping presses (diagonal movement):
    // Press W, Press D (both held), Release W, Release D
    let input = vec![
        KeyEvent::Press(KeyCode::W),
        KeyEvent::Press(KeyCode::D),
        KeyEvent::Release(KeyCode::W),
        KeyEvent::Release(KeyCode::D),
    ];

    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(200))
        .expect("Failed to inject WASD sequence");

    let expected = vec![
        KeyEvent::Press(KeyCode::Up),
        KeyEvent::Press(KeyCode::Right),
        KeyEvent::Release(KeyCode::Up),
        KeyEvent::Release(KeyCode::Right),
    ];
    harness
        .verify(&captured, &expected)
        .expect("Overlapping WASD presses should map correctly");
}

/// Test lock state with extended typing session.
///
/// Simulates an extended typing session with lock layer active,
/// verifying state persists through many key events.
#[test]
fn test_extended_lock_session() {
    keyrx_daemon::skip_if_no_uinput!();
    // Lock layer: numbers become function keys
    let config = E2EConfig::with_lock_layer(
        KeyCode::ScrollLock,
        0,
        vec![
            (KeyCode::Num1, KeyCode::F1),
            (KeyCode::Num2, KeyCode::F2),
            (KeyCode::Num3, KeyCode::F3),
            (KeyCode::Num4, KeyCode::F4),
            (KeyCode::Num5, KeyCode::F5),
        ],
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Toggle lock ON
    harness
        .inject(&TestEvents::tap(KeyCode::ScrollLock))
        .expect("Failed to toggle lock on");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Type 1-2-3-4-5 sequence multiple times
    for round in 0..3 {
        let input = TestEvents::taps(&[
            KeyCode::Num1,
            KeyCode::Num2,
            KeyCode::Num3,
            KeyCode::Num4,
            KeyCode::Num5,
        ]);
        let captured = harness
            .inject_and_capture(&input, Duration::from_millis(400))
            .expect(&format!("Failed to capture round {}", round));

        let expected = TestEvents::taps(&[
            KeyCode::F1,
            KeyCode::F2,
            KeyCode::F3,
            KeyCode::F4,
            KeyCode::F5,
        ]);
        harness.verify(&captured, &expected).expect(&format!(
            "Round {}: Numbers should map to F-keys with lock active",
            round
        ));
    }

    // Toggle lock OFF
    harness
        .inject(&TestEvents::tap(KeyCode::ScrollLock))
        .expect("Failed to toggle lock off");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Verify numbers pass through now
    let final_input = TestEvents::taps(&[KeyCode::Num1, KeyCode::Num2]);
    let final_captured = harness
        .inject_and_capture(&final_input, Duration::from_millis(200))
        .expect("Failed to capture after lock off");
    harness
        .verify(
            &final_captured,
            &TestEvents::taps(&[KeyCode::Num1, KeyCode::Num2]),
        )
        .expect("Numbers should pass through after lock toggled off");
}

/// Test modifier layer with unmapped key passthrough.
///
/// Verifies that keys not in the modifier layer pass through unchanged
/// even when the modifier is active.
#[test]
fn test_modifier_layer_passthrough() {
    keyrx_daemon::skip_if_no_uinput!();
    // Only HJKL mapped in the layer, other keys should pass through
    let config = E2EConfig::with_modifier_layer(
        KeyCode::CapsLock,
        0,
        vec![
            (KeyCode::H, KeyCode::Left),
            (KeyCode::J, KeyCode::Down),
            (KeyCode::K, KeyCode::Up),
            (KeyCode::L, KeyCode::Right),
        ],
    );
    let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

    // Press modifier
    harness
        .inject(&TestEvents::press(KeyCode::CapsLock))
        .expect("Failed to press modifier");
    std::thread::sleep(Duration::from_millis(50));
    let _ = harness.drain();

    // Type mixed: H (mapped), X (unmapped), J (mapped), Y (unmapped)
    let input = TestEvents::taps(&[KeyCode::H, KeyCode::X, KeyCode::J, KeyCode::Y]);
    let captured = harness
        .inject_and_capture(&input, Duration::from_millis(400))
        .expect("Failed to inject mixed sequence");

    // H→Left, X→X (passthrough), J→Down, Y→Y (passthrough)
    let expected = TestEvents::taps(&[KeyCode::Left, KeyCode::X, KeyCode::Down, KeyCode::Y]);
    harness
        .verify(&captured, &expected)
        .expect("Mapped keys should transform, unmapped should pass through");

    // Release modifier
    harness
        .inject(&TestEvents::release(KeyCode::CapsLock))
        .expect("Failed to release modifier");
}
