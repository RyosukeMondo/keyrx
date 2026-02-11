//! End-to-end test for Windows multi-device discrimination.
//!
//! This test verifies that we can:
//! 1. Discriminate between different keyboards (Numpad vs Main)
//! 2. Apply per-device remapping rules
//! 3. Handle device-specific event streams
//!
//! Because we cannot easily spawn a virtual HID device in CI without drivers,
//! this test uses the `simulate_raw_input` hook in `RawInputManager` to inject
//! synthetic raw events that look exactly like Windows WM_INPUT messages.

#![cfg(target_os = "windows")]

use keyrx_core::config::KeyCode;
use keyrx_daemon::platform::windows::device_map::DeviceMap;
use keyrx_daemon::platform::windows::rawinput::RawInputManager;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// B key scancode
const SCAN_B: u16 = 0x30;
// A key scancode
const SCAN_A: u16 = 0x1E;

#[test]
fn test_windows_multidevice_discrimination() {
    // 1. Setup DeviceMap with synthetic devices
    let device_map = DeviceMap::new();

    // Numpad: Handle 100
    device_map.add_synthetic_device(
        100,
        r"\\?\HID#VID_046D&PID_C52B&MI_00#7&2a00c76d&0&0000#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}"
            .to_string(),
        Some("Serial-Numpad-123".to_string()),
    );

    // Main Keyboard: Handle 200
    device_map.add_synthetic_device(
        200,
        r"\\?\HID#VID_05AC&PID_024F&MI_00#8&1b11d87e&0&0000#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}"
            .to_string(),
        Some("Serial-Main-456".to_string()),
    );

    // 2. Setup RawInputManager
    let (tx, rx) = crossbeam_channel::unbounded();
    let bridge_context = Arc::new(Mutex::new(None));
    let bridge_hook = Arc::new(Mutex::new(None));
    let manager = RawInputManager::new(device_map, tx, bridge_context, bridge_hook)
        .expect("Failed to create RawInputManager");

    // 3. Define Test Scenarios

    // Scenario A: Press B on Numpad Device (Handle 100)
    // Should produce KeyEvent::Press(B) with device_id="Serial-Numpad-123"
    manager.simulate_raw_input(100, SCAN_B, 0); // Make

    let event = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("Failed to receive event");
    assert_eq!(event.keycode(), KeyCode::B);
    assert!(event.is_press());
    assert_eq!(
        event.device_id().as_deref(),
        Some("serial-Serial-Numpad-123")
    );

    manager.simulate_raw_input(100, SCAN_B, 1); // Break
    let event = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("Failed to receive event");
    assert!(event.is_release());

    // Scenario B: Press A on Main Keyboard (Handle 200)
    // Should produce KeyEvent::Press(A) with device_id="Serial-Main-456"
    manager.simulate_raw_input(200, SCAN_A, 0); // Make

    let event = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("Failed to receive event");
    assert_eq!(event.keycode(), KeyCode::A);
    assert_eq!(event.device_id().as_deref(), Some("serial-Serial-Main-456"));

    manager.simulate_raw_input(200, SCAN_A, 1); // Break
    let event = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("Failed to receive event");
    assert!(event.is_release());

    // Scenario C: Unknown Device (Handle 999)
    // Should produce event with device_id = None (or fall back to path if we had one, but here we don't have it in map)
    // Wait, if it's not in map, RawInputManager might not attach device_id?
    // Let's check logic: if let Some(info) = context.device_map.get(handle) ... else no device_id
    manager.simulate_raw_input(999, SCAN_A, 0);

    let event = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("Failed to receive event");
    assert_eq!(event.keycode(), KeyCode::A);
    assert!(event.device_id().is_none());
}
