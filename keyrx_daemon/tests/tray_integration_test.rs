//! Integration tests for system tray functionality
//!
//! These tests verify that:
//! - System tray can be initialized (or fails gracefully in headless environments)
//! - Menu events (Reload, Exit) are properly sent through the channel
//! - poll_event() correctly retrieves events from the queue
//!
//! Note: These tests require a graphical environment (X11/Wayland) on Linux.
//! They will be skipped in headless CI environments.

#[cfg(target_os = "linux")]
mod linux_tray_tests {
    use keyrx_daemon::platform::linux::tray::LinuxSystemTray;
    use keyrx_daemon::platform::{SystemTray, TrayControlEvent};
    use std::time::Duration;

    /// Helper: Check if we're in a graphical environment
    #[allow(dead_code)]
    fn has_display() -> bool {
        std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok()
    }

    #[test]
    #[ignore] // Requires graphical environment - GTK will segfault in headless CI
    fn test_tray_initialization() {
        // Test that tray can be created successfully in graphical environment
        let tray = LinuxSystemTray::new().expect("Failed to create tray in graphical environment");

        // Clean up
        drop(tray);
    }

    #[test]
    #[ignore] // Requires graphical environment
    fn test_tray_poll_event_no_interaction() {
        // Test that poll_event() returns None when no menu items are clicked
        let mut tray = LinuxSystemTray::new().expect("Failed to create tray");

        // Poll immediately - should return None since no user interaction
        let event = tray.poll_event();
        assert_eq!(
            event, None,
            "Should have no events without user interaction"
        );

        // Poll again - still None
        let event = tray.poll_event();
        assert_eq!(event, None, "Should still have no events");

        // Clean up
        tray.shutdown().expect("Failed to shutdown tray");
    }

    #[test]
    #[ignore] // Requires user interaction - cannot automate clicking menu items
    fn test_tray_reload_event_manual() {
        // This test documents expected behavior but cannot be automated
        // Manual test procedure:
        //
        // 1. Run this test: cargo test test_tray_reload_event_manual -- --ignored
        // 2. Click the system tray icon
        // 3. Click "Reload Config" menu item
        // 4. Test should pass
        //
        // Expected: poll_event() should return TrayControlEvent::Reload

        let mut tray = LinuxSystemTray::new().expect("Failed to create tray");

        println!("=== MANUAL TEST ===");
        println!("1. Click the KeyRx system tray icon");
        println!("2. Click 'Reload Config'");
        println!("Waiting 30 seconds for user interaction...");

        // Poll for 30 seconds
        let start = std::time::Instant::now();
        let mut received_event = false;

        while start.elapsed() < Duration::from_secs(30) {
            if let Some(event) = tray.poll_event() {
                assert_eq!(event, TrayControlEvent::Reload, "Expected Reload event");
                received_event = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        assert!(
            received_event,
            "Did not receive Reload event (did you click the menu?)"
        );

        tray.shutdown().expect("Failed to shutdown tray");
    }

    #[test]
    #[ignore] // Requires user interaction - cannot automate clicking menu items
    fn test_tray_exit_event_manual() {
        // This test documents expected behavior but cannot be automated
        // Manual test procedure:
        //
        // 1. Run this test: cargo test test_tray_exit_event_manual -- --ignored
        // 2. Click the system tray icon
        // 3. Click "Exit" menu item
        // 4. Test should pass
        //
        // Expected: poll_event() should return TrayControlEvent::Exit

        let mut tray = LinuxSystemTray::new().expect("Failed to create tray");

        println!("=== MANUAL TEST ===");
        println!("1. Click the KeyRx system tray icon");
        println!("2. Click 'Exit'");
        println!("Waiting 30 seconds for user interaction...");

        // Poll for 30 seconds
        let start = std::time::Instant::now();
        let mut received_event = false;

        while start.elapsed() < Duration::from_secs(30) {
            if let Some(event) = tray.poll_event() {
                assert_eq!(event, TrayControlEvent::Exit, "Expected Exit event");
                received_event = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        assert!(
            received_event,
            "Did not receive Exit event (did you click the menu?)"
        );

        tray.shutdown().expect("Failed to shutdown tray");
    }

    #[test]
    #[ignore] // Requires graphical environment - GTK will segfault in headless CI
    fn test_tray_shutdown_idempotent() {
        // Test that shutdown() can be called multiple times safely
        let mut tray = LinuxSystemTray::new().expect("Failed to create tray");

        // First shutdown
        tray.shutdown().expect("First shutdown should succeed");

        // Second shutdown (idempotent)
        tray.shutdown().expect("Second shutdown should succeed");
    }

    #[test]
    #[ignore] // Requires graphical environment - GTK will segfault in headless CI
    fn test_tray_rapid_polling() {
        // Test: Rapid polling should never panic or deadlock
        let mut tray = LinuxSystemTray::new().expect("Failed to create tray");

        // Rapidly poll 100 times
        for _ in 0..100 {
            let _ = tray.poll_event();
        }

        tray.shutdown().expect("Failed to shutdown");
    }
}

#[cfg(target_os = "windows")]
mod windows_tray_tests {
    // TODO: Add Windows tray integration tests if/when Windows tray is implemented
    // Currently Windows uses tray-icon crate - tests would verify:
    // - Tray icon appears
    // - Menu events are received
    // - Shutdown is clean
}
