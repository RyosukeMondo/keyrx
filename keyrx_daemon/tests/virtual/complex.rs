//! Complex tests: modified output and advanced mappings.
//!
//! Tests modified output mappings, lock state transitions, and conditional layers.

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

