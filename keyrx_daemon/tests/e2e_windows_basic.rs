//! Basic end-to-end test for Windows.
//!
//! This test verifies that the daemon can be started as a subprocess,
//! apply a simple remapping, and capture the remapped events using a hook.

#![cfg(target_os = "windows")]

use crate::e2e_harness::{E2EConfig, E2EHarness};
use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEvent;
use std::time::Duration;

mod e2e_harness;

#[test]
fn test_windows_basic_remap() -> Result<(), crate::e2e_harness::E2EError> {
    // 1. Setup harness with A -> B remapping
    let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
    let mut harness = E2EHarness::setup(config)?;

    // 2. Inject 'A' and expect 'B'
    let input = vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)];

    let expected = vec![KeyEvent::Press(KeyCode::B), KeyEvent::Release(KeyCode::B)];

    // Capture events with 500ms timeout
    harness.test_mapping(&input, &expected, Duration::from_millis(500))?;

    // 3. Teardown
    let result = harness.teardown()?;
    assert!(result.graceful_shutdown || result.sigkill_sent);

    Ok(())
}
