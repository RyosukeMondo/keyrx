//! Bug-hunting end-to-end tests for Windows.
//!
//! This test suite targets edge cases, race conditions, and platform-specific
//! quirks that were identified during the modernization of the Windows platform.

#![cfg(target_os = "windows")]

use crate::e2e_harness::{E2EConfig, E2EHarness};
use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEvent;
use std::time::Duration;

mod e2e_harness;

/// Test remapping of extended keys (arrows, home, etc.)
/// This verifies that the E0 scancode prefix is correctly preserved and mapped.
#[test]
fn test_extended_key_remapping() -> Result<(), crate::e2e_harness::E2EError> {
    // Setup: UpArrow -> W
    let config = E2EConfig::simple_remap(KeyCode::Up, KeyCode::W);
    let mut harness = E2EHarness::setup(config)?;

    // Injection: Press/Release UpArrow
    let input = vec![KeyEvent::Press(KeyCode::Up), KeyEvent::Release(KeyCode::Up)];

    // Expected: Press/Release W
    let expected = vec![KeyEvent::Press(KeyCode::W), KeyEvent::Release(KeyCode::W)];

    harness.test_mapping(&input, &expected, Duration::from_secs(1))?;
    harness.teardown()?;
    Ok(())
}

/// Test remapping regular keys TO extended keys
#[test]
fn test_remap_to_extended_key() -> Result<(), crate::e2e_harness::E2EError> {
    // Setup: A -> UpArrow
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::Up);
    let mut harness = E2EHarness::setup(config)?;

    let input = vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)];

    let expected = vec![KeyEvent::Press(KeyCode::Up), KeyEvent::Release(KeyCode::Up)];

    harness.test_mapping(&input, &expected, Duration::from_secs(1))?;
    harness.teardown()?;
    Ok(())
}

/// Test rapid input chording / spamming.
/// Verifies that the message queue and remapping engine can handle high throughput.
#[test]
fn test_rapid_input_burst() -> Result<(), crate::e2e_harness::E2EError> {
    // Setup: A -> B
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config)?;

    let mut input = Vec::new();
    let mut expected = Vec::new();

    for _ in 0..20 {
        input.push(KeyEvent::Press(KeyCode::A));
        input.push(KeyEvent::Release(KeyCode::A));
        expected.push(KeyEvent::Press(KeyCode::B));
        expected.push(KeyEvent::Release(KeyCode::B));
    }

    // Use a slightly longer timeout for the burst
    harness.test_mapping(&input, &expected, Duration::from_secs(2))?;
    harness.teardown()?;
    Ok(())
}
