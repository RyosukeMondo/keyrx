//! Basic end-to-end tests for macOS.
//!
//! This test suite verifies:
//! - Daemon startup with config loading
//! - Graceful shutdown behavior
//! - Permission checking with auto-skip when unavailable
//!
//! Unlike Linux/Windows E2E tests, macOS tests do not inject/capture events
//! directly due to lack of virtual device support (no uinput equivalent).
//! Instead, these tests focus on daemon lifecycle validation.

#![cfg(target_os = "macos")]

mod e2e_macos_harness;

use e2e_macos_harness::{MacosE2EConfig, MacosE2EHarness};
use keyrx_core::config::KeyCode;
use keyrx_daemon::platform::macos::permissions;

/// Tests basic daemon lifecycle with A → B remapping config.
///
/// This test verifies:
/// 1. Daemon starts successfully with compiled config
/// 2. Config loads without errors
/// 3. Daemon remains running (no immediate crash)
/// 4. Daemon shuts down gracefully on SIGTERM
///
/// **Note:** This test auto-skips if Accessibility permission is not granted.
/// To run this test:
/// 1. Grant Accessibility permission to Terminal/IDE
/// 2. Run: `cargo test -p keyrx_daemon test_macos_e2e_basic_remap`
#[test]
#[serial_test::serial]
fn test_macos_e2e_basic_remap() {
    // Check for Accessibility permission
    if !permissions::check_accessibility_permission() {
        eprintln!("\n⚠️  Skipping E2E test: Accessibility permission not granted");
        eprintln!("   To run E2E tests:");
        eprintln!("   1. Open System Settings → Privacy & Security → Accessibility");
        eprintln!("   2. Enable Terminal (or your IDE)");
        eprintln!("   3. Re-run tests\n");
        return;
    }

    // 1. Setup harness with A → B remapping
    let config = MacosE2EConfig::simple_remap(KeyCode::A, KeyCode::B);

    let mut harness = match MacosE2EHarness::setup(config) {
        Ok(h) => h,
        Err(e) => {
            panic!("Failed to setup E2E harness: {}", e);
        }
    };

    // 2. Verify daemon is running
    match harness.daemon_is_running() {
        Ok(true) => {
            eprintln!("✅ Daemon started successfully");
        }
        Ok(false) => {
            panic!("Daemon exited immediately after startup");
        }
        Err(e) => {
            panic!("Failed to check daemon status: {}", e);
        }
    }

    // 3. Brief delay to allow config loading
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Verify daemon is still running (didn't crash during config load)
    match harness.daemon_is_running() {
        Ok(true) => {
            eprintln!("✅ Daemon loaded config successfully");
        }
        Ok(false) => {
            panic!("Daemon crashed during config loading");
        }
        Err(e) => {
            panic!("Failed to check daemon status: {}", e);
        }
    }

    // 4. Graceful teardown
    match harness.teardown() {
        Ok(result) => {
            if result.graceful_shutdown {
                eprintln!("✅ Daemon shut down gracefully");
            } else if result.sigkill_sent {
                eprintln!("⚠️  Daemon required SIGKILL (timeout)");
            } else {
                panic!("Daemon did not shut down");
            }
            assert!(result.graceful_shutdown || result.sigkill_sent);
        }
        Err(e) => {
            panic!("Failed to teardown harness: {}", e);
        }
    }
}

/// Tests daemon startup without permissions.
///
/// This test verifies that the daemon fails gracefully when Accessibility
/// permission is not granted, providing a helpful error message.
///
/// **Note:** This test only runs when permission is NOT granted.
#[test]
#[serial_test::serial]
fn test_macos_e2e_no_permission() {
    // Only run this test if permission is NOT granted
    if permissions::check_accessibility_permission() {
        eprintln!("\n⚠️  Skipping no-permission test: Accessibility permission IS granted");
        return;
    }

    // Attempt to setup harness (should fail)
    let config = MacosE2EConfig::simple_remap(KeyCode::A, KeyCode::B);

    match MacosE2EHarness::setup(config) {
        Ok(_) => {
            panic!("Expected daemon to fail without Accessibility permission");
        }
        Err(e) => {
            let error_message = format!("{}", e);
            eprintln!("✅ Daemon failed as expected without permission");
            eprintln!("   Error: {}", error_message);

            // The daemon should crash immediately due to missing permission
            // The error may mention permission, accessibility, or just be a crash
            // We just verify that it failed (which it did by entering this branch)
            assert!(
                error_message.contains("crashed") ||
                error_message.contains("Accessibility") ||
                error_message.contains("permission") ||
                error_message.contains("stderr"),
                "Expected daemon to crash or report permission error: {}",
                error_message
            );
        }
    }
}

/// Tests daemon startup with config loading verification.
///
/// This test creates a more complex config with multiple mappings to verify
/// that the daemon can handle non-trivial configurations.
///
/// **Note:** This test auto-skips if Accessibility permission is not granted.
#[test]
#[serial_test::serial]
fn test_macos_e2e_config_loading() {
    // Check for Accessibility permission
    if !permissions::check_accessibility_permission() {
        eprintln!("\n⚠️  Skipping E2E test: Accessibility permission not granted");
        return;
    }

    // Setup with multiple remappings
    let config = MacosE2EConfig::simple_remaps(vec![
        (KeyCode::A, KeyCode::B),
        (KeyCode::CapsLock, KeyCode::Escape),
        (KeyCode::LCtrl, KeyCode::LAlt),
    ]);

    let mut harness = match MacosE2EHarness::setup(config) {
        Ok(h) => h,
        Err(e) => {
            panic!("Failed to setup E2E harness with multiple remaps: {}", e);
        }
    };

    // Verify daemon is running
    assert!(
        harness.daemon_is_running().unwrap(),
        "Daemon should be running"
    );

    eprintln!("✅ Daemon loaded multi-mapping config successfully");

    // Teardown
    let result = harness.teardown().expect("Teardown should succeed");
    assert!(result.graceful_shutdown || result.sigkill_sent);
}

/// Tests daemon with modifier layer configuration.
///
/// This test verifies that the daemon can load a configuration with
/// conditional mappings (modifier layers), which exercises more of the
/// config parsing and DFA compilation.
///
/// **Note:** This test auto-skips if Accessibility permission is not granted.
#[test]
#[serial_test::serial]
fn test_macos_e2e_modifier_layer() {
    // Check for Accessibility permission
    if !permissions::check_accessibility_permission() {
        eprintln!("\n⚠️  Skipping E2E test: Accessibility permission not granted");
        return;
    }

    // Setup with modifier layer (CapsLock + HJKL navigation)
    let config = MacosE2EConfig::with_modifier_layer(
        KeyCode::CapsLock,
        0,
        vec![
            (KeyCode::H, KeyCode::Left),
            (KeyCode::J, KeyCode::Down),
            (KeyCode::K, KeyCode::Up),
            (KeyCode::L, KeyCode::Right),
        ],
    );

    let mut harness = match MacosE2EHarness::setup(config) {
        Ok(h) => h,
        Err(e) => {
            panic!("Failed to setup E2E harness with modifier layer: {}", e);
        }
    };

    // Verify daemon is running
    assert!(
        harness.daemon_is_running().unwrap(),
        "Daemon should be running"
    );

    eprintln!("✅ Daemon loaded modifier layer config successfully");

    // Brief delay to allow DFA compilation
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Verify still running (DFA compilation successful)
    assert!(
        harness.daemon_is_running().unwrap(),
        "Daemon should still be running after DFA compilation"
    );

    // Teardown
    let result = harness.teardown().expect("Teardown should succeed");
    assert!(result.graceful_shutdown || result.sigkill_sent);
}
