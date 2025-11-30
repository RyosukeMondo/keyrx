//! Driver integration tests.
//!
//! These tests verify the integration between the driver components and the engine.
//! Tests that require real hardware access are marked with `#[ignore]` so they
//! are skipped on CI but can be run locally with `cargo test -- --ignored`.
//!
//! # Test Categories
//!
//! - **Unit tests**: Run without hardware, verify internal logic
//! - **Integration tests**: Verify driver lifecycle and channel communication
//! - **Hardware tests**: Require real keyboard devices, marked `#[ignore]`
//!
//! # Running Tests
//!
//! ```bash
//! # Run all non-hardware tests
//! cargo test --package keyrx-core --test driver_integration_test
//!
//! # Run hardware tests locally (requires proper permissions)
//! cargo test --package keyrx-core --test driver_integration_test -- --ignored
//! ```

use keyrx_core::drivers::DeviceInfo;
use keyrx_core::engine::{InputEvent, KeyCode, OutputAction};
use keyrx_core::mocks::MockInput;
use keyrx_core::traits::InputSource;
use std::path::PathBuf;

// ============================================================================
// DeviceInfo Tests
// ============================================================================

#[test]
fn device_info_creation() {
    let info = DeviceInfo::new(
        PathBuf::from("/dev/input/event0"),
        "Test Keyboard".to_string(),
        0x1234,
        0x5678,
        true,
    );

    assert_eq!(info.name(), "Test Keyboard");
    assert_eq!(info.path(), &PathBuf::from("/dev/input/event0"));
    assert_eq!(info.vendor_id(), 0x1234);
    assert_eq!(info.product_id(), 0x5678);
    assert!(info.is_keyboard());
}

#[test]
fn device_info_display_format() {
    let info = DeviceInfo::new(
        PathBuf::from("/dev/input/event5"),
        "My Keyboard".to_string(),
        0xABCD,
        0x1234,
        true,
    );

    let display = format!("{}", info);
    assert!(display.contains("My Keyboard"));
    assert!(display.contains("abcd:1234")); // hex format, lowercase
    assert!(display.contains("/dev/input/event5"));
}

#[test]
fn device_info_json_serialization() {
    let info = DeviceInfo::new(
        PathBuf::from("/dev/input/event0"),
        "USB Keyboard".to_string(),
        0x046D,
        0xC52B,
        true,
    );

    let json = serde_json::to_string(&info).expect("JSON serialization failed");
    assert!(json.contains("\"name\":\"USB Keyboard\""));
    assert!(json.contains("\"is_keyboard\":true"));
}

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
    assert_eq!(event.timestamp, 12345);
}

#[test]
fn input_event_key_up_constructor() {
    let event = InputEvent::key_up(KeyCode::Enter, 67890);
    assert_eq!(event.key, KeyCode::Enter);
    assert!(!event.pressed);
    assert_eq!(event.timestamp, 67890);
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
// Cross-Platform Channel Communication Tests
// ============================================================================

#[tokio::test]
async fn channel_communication_basic_flow() {
    // This test verifies the basic event flow pattern used by drivers
    use crossbeam_channel::{unbounded, TryRecvError};

    let (tx, rx) = unbounded::<InputEvent>();

    // Simulate driver sending events
    tx.send(InputEvent::key_down(KeyCode::CapsLock, 0)).unwrap();
    tx.send(InputEvent::key_up(KeyCode::CapsLock, 100)).unwrap();

    // Simulate engine receiving events
    let mut events = Vec::new();
    loop {
        match rx.try_recv() {
            Ok(event) => events.push(event),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
    }

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].key, KeyCode::CapsLock);
    assert!(events[0].pressed);
    assert_eq!(events[1].key, KeyCode::CapsLock);
    assert!(!events[1].pressed);
}

#[tokio::test]
async fn channel_communication_bounded_backpressure() {
    // Verify bounded channel behavior for potential backpressure scenarios
    use crossbeam_channel::bounded;

    let (tx, rx) = bounded::<InputEvent>(2);

    // Fill the channel
    tx.send(InputEvent::key_down(KeyCode::A, 0)).unwrap();
    tx.send(InputEvent::key_down(KeyCode::B, 0)).unwrap();

    // Channel is now full, try_send should fail
    let result = tx.try_send(InputEvent::key_down(KeyCode::C, 0));
    assert!(result.is_err());

    // Drain one event
    let _ = rx.recv().unwrap();

    // Now we should be able to send again
    tx.send(InputEvent::key_down(KeyCode::C, 0)).unwrap();
}

#[test]
fn channel_disconnect_detection() {
    use crossbeam_channel::{unbounded, TryRecvError};

    let (tx, rx) = unbounded::<InputEvent>();

    // Drop the sender
    drop(tx);

    // Receiver should detect disconnect
    match rx.try_recv() {
        Err(TryRecvError::Disconnected) => {
            // Expected
        }
        other => panic!("Expected Disconnected, got {:?}", other),
    }
}

// ============================================================================
// Graceful Shutdown Tests
// ============================================================================

#[tokio::test]
async fn graceful_shutdown_with_pending_events() {
    let mut input = MockInput::new();

    input.queue_event(InputEvent::key_down(KeyCode::A, 0));
    input.queue_event(InputEvent::key_down(KeyCode::B, 0));

    input.start().await.expect("start failed");

    // Stop without polling events - should not hang or panic
    input.stop().await.expect("stop failed");

    // Call history should show start and stop
    use keyrx_core::mocks::MockCall;
    let history = input.call_history();
    assert!(matches!(history[0], MockCall::Start));
    assert!(matches!(history[1], MockCall::Stop));
}

#[tokio::test]
async fn graceful_shutdown_concurrent_operations() {
    let mut input = MockInput::new();

    input.start().await.expect("start failed");

    // Queue event while running
    input.queue_event(InputEvent::key_down(KeyCode::Space, 0));

    // Poll and stop in sequence
    let events = input.poll_events().await.expect("poll_events failed");
    assert_eq!(events.len(), 1);

    input.stop().await.expect("stop failed");
}
