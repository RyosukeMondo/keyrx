#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::field_reassign_with_default,
    clippy::useless_conversion,
    clippy::assertions_on_constants,
    clippy::manual_div_ceil,
    clippy::manual_strip,
    clippy::len_zero,
    clippy::redundant_closure,
    clippy::manual_range_contains,
    clippy::default_constructed_unit_structs,
    clippy::clone_on_copy,
    clippy::io_other_error,
    clippy::bool_assert_comparison,
    clippy::approx_constant,
    clippy::let_unit_value,
    clippy::while_let_on_iterator,
    clippy::await_holding_lock,
    clippy::unnecessary_cast,
    clippy::drop_non_drop,
    clippy::needless_range_loop,
    unused_imports,
    unused_variables,
    dead_code,
    unsafe_code,
    clippy::collapsible_if,
    clippy::bool_comparison,
    unexpected_cfgs
)]
mod integration;

use keyrx_core::engine::{InputEvent, KeyCode, OutputAction};
use keyrx_core::mocks::MockInput;
use keyrx_core::traits::InputSource;

// ============================================================================
// MockInput Integration Tests
// ============================================================================

#[tokio::test]
async fn mock_input_start_stop_lifecycle() {
    let mut input = MockInput::new();

    // Start should succeed
    input.start().await.expect("start failed");

    // Stop should succeed
    input.stop().await.expect("stop failed");
}

#[tokio::test]
async fn mock_input_double_start_is_safe() {
    let mut input = MockInput::new();

    // First start
    input.start().await.expect("first start failed");

    // Second start should also succeed (idempotent)
    input.start().await.expect("second start failed");

    input.stop().await.expect("stop failed");
}

#[tokio::test]
async fn mock_input_double_stop_is_safe() {
    let mut input = MockInput::new();

    input.start().await.expect("start failed");

    // First stop
    input.stop().await.expect("first stop failed");

    // Second stop should also succeed (idempotent)
    input.stop().await.expect("second stop failed");
}

#[tokio::test]
async fn mock_input_event_channel_communication() {
    let mut input = MockInput::new();

    // Queue some events before starting
    input.queue_event(InputEvent::key_down(KeyCode::A, 100));
    input.queue_event(InputEvent::key_up(KeyCode::A, 200));
    input.queue_event(InputEvent::key_down(KeyCode::B, 300));

    input.start().await.expect("start failed");

    // Poll should return all queued events
    let events = input.poll_events().await.expect("poll_events failed");
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].key, KeyCode::A);
    assert!(events[0].pressed);
    assert_eq!(events[1].key, KeyCode::A);
    assert!(!events[1].pressed);
    assert_eq!(events[2].key, KeyCode::B);
    assert!(events[2].pressed);

    // Second poll should be empty
    let events = input.poll_events().await.expect("poll_events failed");
    assert!(events.is_empty());

    input.stop().await.expect("stop failed");
}

#[tokio::test]
async fn mock_input_output_action_logging() {
    let mut input = MockInput::new();

    input.start().await.expect("start failed");

    // Send various output actions
    input
        .send_output(OutputAction::KeyDown(KeyCode::Escape))
        .await
        .expect("send_output failed");
    input
        .send_output(OutputAction::KeyUp(KeyCode::Escape))
        .await
        .expect("send_output failed");
    input
        .send_output(OutputAction::Block)
        .await
        .expect("send_output failed");
    input
        .send_output(OutputAction::PassThrough)
        .await
        .expect("send_output failed");

    // Verify output log
    let log = input.output_log();
    assert_eq!(log.len(), 4);
    assert_eq!(log[0], OutputAction::KeyDown(KeyCode::Escape));
    assert_eq!(log[1], OutputAction::KeyUp(KeyCode::Escape));
    assert_eq!(log[2], OutputAction::Block);
    assert_eq!(log[3], OutputAction::PassThrough);

    input.stop().await.expect("stop failed");
}

#[tokio::test]
async fn mock_input_error_on_start() {
    let mut input = MockInput::new().with_error_on_start("Device busy");

    let result = input.start().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Device busy"));
}

#[tokio::test]
async fn mock_input_call_history_tracking() {
    let mut input = MockInput::new();

    input.start().await.expect("start failed");
    input.poll_events().await.expect("poll_events failed");
    input
        .send_output(OutputAction::Block)
        .await
        .expect("send_output failed");
    input.stop().await.expect("stop failed");

    // Verify call history
    use keyrx_core::mocks::MockCall;
    let history = input.call_history();
    assert!(matches!(history[0], MockCall::Start));
    assert!(matches!(history[1], MockCall::PollEvents));
    assert!(matches!(
        history[2],
        MockCall::SendOutput(OutputAction::Block)
    ));
    assert!(matches!(history[3], MockCall::Stop));
}

#[tokio::test]
async fn mock_input_clear_operations() {
    let mut input = MockInput::new();

    input.start().await.expect("start failed");
    input
        .send_output(OutputAction::Block)
        .await
        .expect("send_output failed");

    assert!(!input.output_log().is_empty());
    assert!(!input.call_history().is_empty());

    input.clear_output_log();
    input.clear_call_history();

    assert!(input.output_log().is_empty());
    assert!(input.call_history().is_empty());

    input.stop().await.expect("stop failed");
}

// ============================================================================
// InputEvent Tests
// ============================================================================

#[test]
fn input_event_key_down_constructor() {
    let event = InputEvent::key_down(KeyCode::Space, 12345);
    assert_eq!(event.key, KeyCode::Space);
    assert!(event.pressed);
    assert_eq!(event.timestamp_us, 12345);
    // Verify default metadata fields
    assert_eq!(event.device_id, None);
    assert!(!event.is_repeat);
    assert!(!event.is_synthetic);
    assert_eq!(event.scan_code, 0);
}

#[test]
fn input_event_key_up_constructor() {
    let event = InputEvent::key_up(KeyCode::Enter, 67890);
    assert_eq!(event.key, KeyCode::Enter);
    assert!(!event.pressed);
    assert_eq!(event.timestamp_us, 67890);
    // Verify default metadata fields
    assert_eq!(event.device_id, None);
    assert!(!event.is_repeat);
    assert!(!event.is_synthetic);
    assert_eq!(event.scan_code, 0);
}

#[test]
fn input_event_with_metadata_constructor() {
    let event = InputEvent::with_metadata(
        KeyCode::A,
        true,
        99999,
        Some("test-device".to_string()),
        true,
        true,
        42,
        None,
    );
    assert_eq!(event.key, KeyCode::A);
    assert!(event.pressed);
    assert_eq!(event.timestamp_us, 99999);
    assert_eq!(event.device_id, Some("test-device".to_string()));
    assert!(event.is_repeat);
    assert!(event.is_synthetic);
    assert_eq!(event.scan_code, 42);
}

#[test]
fn input_event_default_impl() {
    let event = InputEvent::default();
    assert_eq!(event.key, KeyCode::Unknown(0));
    assert!(!event.pressed);
    assert_eq!(event.timestamp_us, 0);
    assert_eq!(event.device_id, None);
    assert!(!event.is_repeat);
    assert!(!event.is_synthetic);
    assert_eq!(event.scan_code, 0);
}

// ============================================================================
// Platform-Specific Hardware Tests
// ============================================================================

// Linux-specific tests
#[cfg(target_os = "linux")]
mod linux_tests {
    use super::*;
    use keyrx_core::drivers::LinuxInput;

    /// Test that list_keyboards can enumerate devices.
    /// This test is not ignored because it doesn't require special permissions
    /// to just list devices (though opening them might fail).
    #[test]
    fn list_keyboards_does_not_panic() {
        // This should not panic even without permissions
        let result = keyrx_core::drivers::list_keyboards();
        // We don't assert success because it might fail on systems without
        // /dev/input or without any keyboards attached
        match result {
            Ok(keyboards) => {
                println!("Found {} keyboard(s)", keyboards.len());
                for kb in &keyboards {
                    println!("  - {}", kb);
                }
            }
            Err(e) => {
                println!("list_keyboards failed (expected on some systems): {}", e);
            }
        }
    }

    /// Test LinuxInput creation with auto-detect (requires hardware).
    ///
    /// This test is ignored because it requires:
    /// - A real keyboard device
    /// - User in 'input' group or running as root
    /// - uinput module loaded
    #[ignore]
    #[tokio::test]
    async fn linux_input_create_auto_detect() {
        let result = LinuxInput::new(None);
        match result {
            Ok(input) => {
                assert!(!input.is_running());
                println!(
                    "Created LinuxInput for device: {}",
                    input.device_path().display()
                );
            }
            Err(e) => {
                // Expected on systems without proper permissions
                println!("LinuxInput::new failed (may be expected): {}", e);
            }
        }
    }

    /// Test LinuxInput start/stop lifecycle (requires hardware).
    ///
    /// This test is ignored because it requires real hardware and permissions.
    #[ignore]
    #[tokio::test]
    async fn linux_input_start_stop_lifecycle() {
        let mut input = match LinuxInput::new(None) {
            Ok(input) => input,
            Err(e) => {
                println!("Skipping test: {}", e);
                return;
            }
        };

        // Start should grab the keyboard
        if let Err(e) = input.start().await {
            println!("Start failed (may be expected): {}", e);
            return;
        }

        assert!(input.is_running());
        println!("LinuxInput started successfully");

        // Small delay to ensure thread is running
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Stop should release the keyboard
        input.stop().await.expect("stop failed");
        assert!(!input.is_running());
        println!("LinuxInput stopped successfully");
    }

    /// Test LinuxInput graceful shutdown on drop (requires hardware).
    ///
    /// This test verifies that dropping LinuxInput properly releases resources.
    #[ignore]
    #[tokio::test]
    async fn linux_input_drop_cleanup() {
        {
            let mut input = match LinuxInput::new(None) {
                Ok(input) => input,
                Err(e) => {
                    println!("Skipping test: {}", e);
                    return;
                }
            };

            if let Err(e) = input.start().await {
                println!("Start failed: {}", e);
                return;
            }

            // Let it run briefly
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // Drop without calling stop() - should clean up automatically
            println!("Dropping LinuxInput without explicit stop()...");
        }

        println!("LinuxInput dropped - keyboard should be released");
    }

    /// Test that LinuxInput can poll events without blocking (requires hardware).
    #[ignore]
    #[tokio::test]
    async fn linux_input_poll_is_nonblocking() {
        let mut input = match LinuxInput::new(None) {
            Ok(input) => input,
            Err(e) => {
                println!("Skipping test: {}", e);
                return;
            }
        };

        if let Err(e) = input.start().await {
            println!("Start failed: {}", e);
            return;
        }

        // poll_events should return immediately even with no keys pressed
        let start_time = std::time::Instant::now();
        let events = input.poll_events().await.expect("poll_events failed");
        let elapsed = start_time.elapsed();

        // Should complete in well under 100ms (non-blocking)
        assert!(
            elapsed.as_millis() < 100,
            "poll_events took too long: {:?}",
            elapsed
        );
        println!(
            "poll_events returned {} events in {:?}",
            events.len(),
            elapsed
        );

        input.stop().await.expect("stop failed");
    }
}

// Windows-specific tests
#[cfg(target_os = "windows")]
mod windows_tests {
    use super::*;
    use keyrx_core::drivers::WindowsInput;

    /// Test that list_keyboards returns the system keyboard.
    #[test]
    fn list_keyboards_returns_system_keyboard() {
        let keyboards = keyrx_core::drivers::list_keyboards().expect("list_keyboards failed");
        assert!(!keyboards.is_empty());
        assert!(keyboards[0].is_keyboard());
        println!("Found keyboard: {}", keyboards[0]);
    }

    /// Test WindowsInput creation.
    #[test]
    fn windows_input_create() {
        let input = WindowsInput::new().expect("WindowsInput::new failed");
        assert!(!input.is_running());
        println!("Created WindowsInput");
    }

    /// Test WindowsInput start/stop lifecycle (requires running on Windows).
    ///
    /// This test is ignored because it requires a Windows desktop environment
    /// and may interfere with normal keyboard operation.
    #[ignore]
    #[tokio::test]
    async fn windows_input_start_stop_lifecycle() {
        let mut input = WindowsInput::new().expect("WindowsInput::new failed");

        // Start should install the keyboard hook
        input.start().await.expect("start failed");
        assert!(input.is_running());
        println!("WindowsInput started successfully");

        // Small delay to ensure thread is running
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Stop should uninstall the hook
        input.stop().await.expect("stop failed");
        assert!(!input.is_running());
        println!("WindowsInput stopped successfully");
    }

    /// Test WindowsInput graceful shutdown on drop.
    #[ignore]
    #[tokio::test]
    async fn windows_input_drop_cleanup() {
        {
            let mut input = WindowsInput::new().expect("WindowsInput::new failed");

            input.start().await.expect("start failed");

            // Let it run briefly
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // Drop without calling stop() - should clean up automatically
            println!("Dropping WindowsInput without explicit stop()...");
        }

        println!("WindowsInput dropped - hook should be uninstalled");
    }

    /// Test that WindowsInput can poll events without blocking.
    #[ignore]
    #[tokio::test]
    async fn windows_input_poll_is_nonblocking() {
        let mut input = WindowsInput::new().expect("WindowsInput::new failed");

        input.start().await.expect("start failed");

        // poll_events should return immediately even with no keys pressed
        let start_time = std::time::Instant::now();
        let events = input.poll_events().await.expect("poll_events failed");
        let elapsed = start_time.elapsed();

        // Should complete in well under 100ms (non-blocking)
        assert!(
            elapsed.as_millis() < 100,
            "poll_events took too long: {:?}",
            elapsed
        );
        println!(
            "poll_events returned {} events in {:?}",
            events.len(),
            elapsed
        );

        input.stop().await.expect("stop failed");
    }
}

// ============================================================================
// Driver Safety Integration Tests
// ============================================================================

// Include the driver safety tests from the integration module
#[cfg(test)]
mod driver_safety_tests {
    // Re-export all tests from the integration::drivers modules
    pub use crate::integration::drivers::error_recovery::*;

    #[cfg(target_os = "windows")]
    pub use crate::integration::drivers::windows_safety::*;

    #[cfg(target_os = "linux")]
    pub use crate::integration::drivers::linux_safety::*;
}
