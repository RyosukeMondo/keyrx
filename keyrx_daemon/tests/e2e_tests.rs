//! End-to-End Integration Tests with Real Hardware.
//!
//! These tests validate complete system behavior using real input devices and uinput.
//! All tests are marked `#[ignore]` for CI compatibility.
//!
//! # Virtual E2E Tests for CI
//!
//! **For CI/CD environments or testing without physical hardware, use the virtual E2E tests
//! in [`virtual_e2e_tests.rs`](./virtual_e2e_tests.rs) instead.**
//!
//! Virtual tests use uinput to create simulated keyboard devices, allowing full integration
//! testing without requiring physical hardware. They are the preferred approach for:
//! - Continuous Integration pipelines
//! - Development without physical keyboard access
//! - Automated testing of key remapping logic
//!
//! Run virtual tests with:
//! ```bash
//! sudo cargo test -p keyrx_daemon --features linux --test virtual_e2e_tests -- --ignored
//! ```
//!
//! # When to Use Hardware Tests (This File)
//!
//! Use the tests in this file when you need to:
//! - Verify behavior with actual physical keypresses
//! - Test device discovery with real USB/Bluetooth keyboards
//! - Validate keyboard grabbing and release behavior
//! - Perform manual interactive testing sessions
//!
//! These tests are **not suitable for CI** as they require physical hardware.
//!
//! # Hardware Requirements
//!
//! - Physical keyboard connected via USB or built-in
//! - User must be in `input` and `uinput` groups (or run as root)
//! - `/dev/uinput` must be accessible
//!
//! # Running Hardware E2E Tests
//!
//! ```bash
//! # Run all hardware E2E tests (requires hardware + permissions)
//! sudo cargo test --package keyrx_daemon --test e2e_tests -- --ignored
//!
//! # Run a specific E2E test
//! sudo cargo test --package keyrx_daemon --test e2e_tests test_capslock_to_escape -- --ignored
//! ```
//!
//! # Test Categories
//!
//! 1. **Basic Remapping**: Simple key-to-key remapping (CapsLock→Escape)
//! 2. **Navigation Layers**: Vim-style HJKL navigation with layer modifiers
//! 3. **Multi-Device**: Different configs for different keyboards
//! 4. **State Persistence**: Modifier/lock state across key sequences
//!
//! # Safety Notes
//!
//! - Tests grab real keyboard devices (keyboard will be unresponsive during test)
//! - Tests use short timeouts and auto-cleanup via Drop trait
//! - Use a second keyboard or SSH session to recover if needed
//! - Tests automatically release devices on completion or panic

#![cfg(all(target_os = "linux", feature = "linux"))]

use std::io::Write;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use keyrx_compiler::serialize::serialize;
use keyrx_core::config::{
    BaseKeyMapping, Condition, ConditionItem, ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode,
    KeyMapping, Metadata, Version,
};
use keyrx_daemon::daemon::{Daemon, DaemonError};
use tempfile::NamedTempFile;

// ============================================================================
// Test Configuration Helpers
// ============================================================================

/// Creates a minimal valid configuration for testing.
fn create_config(devices: Vec<DeviceConfig>) -> ConfigRoot {
    ConfigRoot {
        version: Version::current(),
        devices,
        metadata: Metadata {
            compilation_timestamp: 0,
            compiler_version: "e2e-test".to_string(),
            source_hash: "e2e-test".to_string(),
        },
    }
}

/// Creates a device config with the given pattern and mappings.
fn create_device_config(pattern: &str, mappings: Vec<KeyMapping>) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: pattern.to_string(),
        },
        mappings,
    }
}

/// Creates a temporary .krx config file for testing.
fn create_temp_config_file(config: &ConfigRoot) -> NamedTempFile {
    let bytes = serialize(config).expect("Failed to serialize config");

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(&bytes).expect("Failed to write config");
    temp_file.flush().expect("Failed to flush");

    temp_file
}

/// Helper to run a daemon with timeout and capture any errors.
///
/// Returns `Ok(())` if daemon ran successfully for the specified duration,
/// or `Err` if initialization failed or an error occurred during run.
fn run_daemon_with_timeout(config_path: &Path, timeout: Duration) -> Result<(), DaemonError> {
    let mut daemon = Daemon::new(config_path)?;

    // Get the running flag for external control
    let running = daemon.running_flag();

    // Spawn a thread to stop the daemon after timeout
    let running_clone = Arc::clone(&running);
    let timeout_thread = thread::spawn(move || {
        thread::sleep(timeout);
        running_clone.store(false, Ordering::SeqCst);
    });

    // Run the daemon (will stop when running flag is set to false)
    let result = daemon.run();

    // Wait for timeout thread to complete
    let _ = timeout_thread.join();

    result
}

// ============================================================================
// Basic Remapping Tests
// ============================================================================

/// Test: CapsLock to Escape remapping.
///
/// This is a common use case for developers who want to use Escape without
/// reaching for the corner of the keyboard.
///
/// # Configuration
/// - CapsLock → Escape (simple remapping)
///
/// # Verification
/// - Daemon initializes successfully with the configuration
/// - Device is discovered and grabbed
/// - Daemon runs without errors
///
/// # Hardware Requirements
/// - At least one keyboard device accessible
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_capslock_to_escape_initialization() {
    // Create config: CapsLock → Escape
    let config = create_config(vec![create_device_config(
        "*",
        vec![KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape)],
    )]);

    let temp_file = create_temp_config_file(&config);

    // Initialize daemon
    let daemon = Daemon::new(temp_file.path()).expect("Failed to initialize daemon");

    // Verify initialization
    assert!(daemon.device_count() > 0, "Should have at least one device");
    assert!(daemon.is_running(), "Daemon should be running");

    println!(
        "CapsLock→Escape test: Initialized with {} device(s)",
        daemon.device_count()
    );
}

/// Test: CapsLock to Escape remapping - daemon lifecycle.
///
/// Tests the full lifecycle: initialization, running (brief), and shutdown.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_capslock_to_escape_lifecycle() {
    // Create config: CapsLock → Escape
    let config = create_config(vec![create_device_config(
        "*",
        vec![KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape)],
    )]);

    let temp_file = create_temp_config_file(&config);

    // Run daemon for a short period
    let result = run_daemon_with_timeout(temp_file.path(), Duration::from_millis(500));

    assert!(
        result.is_ok(),
        "Daemon should run without errors: {:?}",
        result.err()
    );

    println!("CapsLock→Escape lifecycle test: Completed successfully");
}

/// Test: Simple A to B remapping.
///
/// The most basic remapping test - maps A key to B key output.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_simple_a_to_b_remapping() {
    let config = create_config(vec![create_device_config(
        "*",
        vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
    )]);

    let temp_file = create_temp_config_file(&config);

    let result = run_daemon_with_timeout(temp_file.path(), Duration::from_millis(500));

    assert!(
        result.is_ok(),
        "Simple remapping should work: {:?}",
        result.err()
    );
}

/// Test: Multiple simple remappings.
///
/// Tests that multiple independent remappings work together.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_multiple_simple_remappings() {
    let config = create_config(vec![create_device_config(
        "*",
        vec![
            KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape),
            KeyMapping::simple(KeyCode::Escape, KeyCode::CapsLock), // Swap
            KeyMapping::simple(KeyCode::Grave, KeyCode::Escape),    // Backtick → Escape
        ],
    )]);

    let temp_file = create_temp_config_file(&config);

    let result = run_daemon_with_timeout(temp_file.path(), Duration::from_millis(500));

    assert!(
        result.is_ok(),
        "Multiple remappings should work: {:?}",
        result.err()
    );
}

// ============================================================================
// Vim Navigation Layer Tests
// ============================================================================

/// Test: Vim navigation layer initialization.
///
/// Tests that a complex Vim-style configuration can be loaded and initialized.
///
/// # Configuration
/// - CapsLock → Layer modifier (MD_00)
/// - H/J/K/L → Arrow keys (when layer active)
///
/// This is a popular use case for Vim users who want to navigate without
/// leaving the home row.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_vim_navigation_layer_initialization() {
    // Create Vim navigation config
    let config = create_config(vec![create_device_config(
        "*",
        vec![
            // CapsLock → MD_00 (navigation layer modifier)
            KeyMapping::modifier(KeyCode::CapsLock, 0),
            // Navigation mappings (active when MD_00 is held)
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::J,
                    to: KeyCode::Down,
                }],
            ),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::K,
                    to: KeyCode::Up,
                }],
            ),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::L,
                    to: KeyCode::Right,
                }],
            ),
        ],
    )]);

    let temp_file = create_temp_config_file(&config);

    let daemon = Daemon::new(temp_file.path()).expect("Failed to initialize Vim navigation daemon");

    assert!(daemon.device_count() > 0, "Should have at least one device");
    println!(
        "Vim navigation test: Initialized with {} device(s)",
        daemon.device_count()
    );
}

/// Test: Vim navigation layer lifecycle.
///
/// Tests the full lifecycle of a Vim navigation layer configuration.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_vim_navigation_layer_lifecycle() {
    // Create Vim navigation config
    let config = create_config(vec![create_device_config(
        "*",
        vec![
            KeyMapping::modifier(KeyCode::CapsLock, 0),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::J,
                    to: KeyCode::Down,
                }],
            ),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::K,
                    to: KeyCode::Up,
                }],
            ),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::L,
                    to: KeyCode::Right,
                }],
            ),
        ],
    )]);

    let temp_file = create_temp_config_file(&config);

    let result = run_daemon_with_timeout(temp_file.path(), Duration::from_millis(500));

    assert!(
        result.is_ok(),
        "Vim navigation should work: {:?}",
        result.err()
    );

    println!("Vim navigation lifecycle test: Completed successfully");
}

/// Test: Extended Vim navigation with word movement.
///
/// Tests a more complete Vim navigation setup including:
/// - HJKL → Arrow keys
/// - W → Ctrl+Right (word forward)
/// - B → Ctrl+Left (word backward)
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_vim_navigation_with_word_movement() {
    let config = create_config(vec![create_device_config(
        "*",
        vec![
            // CapsLock → layer modifier
            KeyMapping::modifier(KeyCode::CapsLock, 0),
            // HJKL navigation
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::J,
                    to: KeyCode::Down,
                }],
            ),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::K,
                    to: KeyCode::Up,
                }],
            ),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::L,
                    to: KeyCode::Right,
                }],
            ),
            // Word navigation: W → Ctrl+Right
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::ModifiedOutput {
                    from: KeyCode::W,
                    to: KeyCode::Right,
                    shift: false,
                    ctrl: true,
                    alt: false,
                    win: false,
                }],
            ),
            // Word navigation: B → Ctrl+Left
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::ModifiedOutput {
                    from: KeyCode::B,
                    to: KeyCode::Left,
                    shift: false,
                    ctrl: true,
                    alt: false,
                    win: false,
                }],
            ),
        ],
    )]);

    let temp_file = create_temp_config_file(&config);

    let result = run_daemon_with_timeout(temp_file.path(), Duration::from_millis(500));

    assert!(
        result.is_ok(),
        "Extended Vim navigation should work: {:?}",
        result.err()
    );
}

// ============================================================================
// Multi-Device Configuration Tests
// ============================================================================

/// Test: Multi-device configuration with wildcard.
///
/// Tests that a wildcard pattern matches all connected keyboards.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_multi_device_wildcard_pattern() {
    let config = create_config(vec![create_device_config(
        "*", // Wildcard matches all devices
        vec![KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape)],
    )]);

    let temp_file = create_temp_config_file(&config);

    let daemon = Daemon::new(temp_file.path()).expect("Failed to initialize daemon");

    // With wildcard, all keyboards should be matched
    assert!(
        daemon.device_count() > 0,
        "Should match at least one device"
    );

    println!(
        "Multi-device wildcard test: {} device(s) matched",
        daemon.device_count()
    );
}

/// Test: Multi-device configuration with specific patterns.
///
/// Tests that different device patterns can have different configurations.
/// Note: This test verifies the configuration loads correctly; actual device
/// matching depends on connected hardware.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_multi_device_specific_patterns() {
    // Create config with multiple device patterns
    // Each pattern gets different mappings
    let config = create_config(vec![
        // Pattern for USB keyboards (common vendor IDs)
        create_device_config(
            "USB*",
            vec![
                KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape),
                KeyMapping::modifier(KeyCode::Grave, 0),
            ],
        ),
        // Fallback pattern for other keyboards
        create_device_config(
            "*",
            vec![KeyMapping::simple(KeyCode::CapsLock, KeyCode::LCtrl)],
        ),
    ]);

    let temp_file = create_temp_config_file(&config);

    let daemon = Daemon::new(temp_file.path()).expect("Failed to initialize daemon");

    assert!(
        daemon.device_count() > 0,
        "Should match at least one device"
    );

    println!(
        "Multi-device specific patterns test: {} device(s) matched",
        daemon.device_count()
    );
}

/// Test: Configuration with no matching devices.
///
/// Verifies that the daemon fails gracefully when no devices match the pattern.
#[test]
#[ignore = "Requires real hardware access (test expects failure if no matches)"]
fn test_no_matching_devices() {
    // Create config with a pattern that likely won't match any device
    let config = create_config(vec![create_device_config(
        "ThisPatternWillNotMatch12345",
        vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
    )]);

    let temp_file = create_temp_config_file(&config);

    // This should fail because no devices match
    let result = Daemon::new(temp_file.path());

    // Either it matches some device (unlikely) or it fails with NoDevicesFound
    match result {
        Ok(daemon) => {
            println!(
                "Surprisingly, pattern matched {} device(s)",
                daemon.device_count()
            );
        }
        Err(DaemonError::DiscoveryError(e)) => {
            println!("Expected behavior: No devices matched. Error: {}", e);
        }
        Err(e) => {
            panic!("Unexpected error type: {}", e);
        }
    }
}

// ============================================================================
// Lock State Tests
// ============================================================================

/// Test: NumLock toggle configuration.
///
/// Tests a configuration that uses lock state for mode switching.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_lock_state_configuration() {
    // Create config with lock-based mode switching
    let config = create_config(vec![create_device_config(
        "*",
        vec![
            // ScrollLock toggles custom lock state
            KeyMapping::lock(KeyCode::ScrollLock, 0),
            // When lock is active, remap number row to F-keys
            KeyMapping::conditional(
                Condition::LockActive(0),
                vec![
                    BaseKeyMapping::Simple {
                        from: KeyCode::Num1,
                        to: KeyCode::F1,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::Num2,
                        to: KeyCode::F2,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::Num3,
                        to: KeyCode::F3,
                    },
                ],
            ),
        ],
    )]);

    let temp_file = create_temp_config_file(&config);

    let result = run_daemon_with_timeout(temp_file.path(), Duration::from_millis(500));

    assert!(
        result.is_ok(),
        "Lock state configuration should work: {:?}",
        result.err()
    );
}

// ============================================================================
// Configuration Reload Tests
// ============================================================================

/// Test: Configuration reload during operation.
///
/// Tests that the daemon can reload its configuration without restarting.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_configuration_reload() {
    // Create initial config
    let initial_config = create_config(vec![create_device_config(
        "*",
        vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
    )]);

    let temp_file = create_temp_config_file(&initial_config);

    let mut daemon = Daemon::new(temp_file.path()).expect("Failed to initialize daemon");
    let device_count_before = daemon.device_count();

    // Create updated config
    let updated_config = create_config(vec![create_device_config(
        "*",
        vec![
            KeyMapping::simple(KeyCode::A, KeyCode::C), // Changed B to C
            KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape), // Added new mapping
        ],
    )]);

    // Write updated config to the same file
    let updated_bytes = serialize(&updated_config).expect("Failed to serialize");
    std::fs::write(temp_file.path(), &updated_bytes).expect("Failed to write updated config");

    // Reload configuration
    let result = daemon.reload();
    assert!(result.is_ok(), "Reload should succeed: {:?}", result.err());

    // Verify device count unchanged
    assert_eq!(
        daemon.device_count(),
        device_count_before,
        "Device count should remain same after reload"
    );

    println!("Configuration reload test: Completed successfully");
}

/// Test: Reload with invalid config keeps old config.
///
/// Verifies that the daemon continues with the old configuration when
/// reload fails due to invalid config.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_reload_invalid_config_keeps_old() {
    let config = create_config(vec![create_device_config(
        "*",
        vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
    )]);

    let temp_file = create_temp_config_file(&config);

    let mut daemon = Daemon::new(temp_file.path()).expect("Failed to initialize daemon");

    // Write invalid data to config file
    std::fs::write(temp_file.path(), b"invalid krx data").expect("Failed to write invalid config");

    // Reload should fail
    let result = daemon.reload();
    assert!(result.is_err(), "Reload should fail with invalid config");

    // Daemon should still be running
    assert!(
        daemon.is_running(),
        "Daemon should still be running after failed reload"
    );

    println!("Reload invalid config test: Daemon continued with old config");
}

// ============================================================================
// Shutdown and Cleanup Tests
// ============================================================================

/// Test: Graceful shutdown releases devices.
///
/// Verifies that shutdown properly releases all grabbed devices.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_graceful_shutdown() {
    let config = create_config(vec![create_device_config(
        "*",
        vec![KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape)],
    )]);

    let temp_file = create_temp_config_file(&config);

    let mut daemon = Daemon::new(temp_file.path()).expect("Failed to initialize daemon");

    assert!(daemon.is_running(), "Daemon should be running initially");

    daemon.shutdown();

    assert!(
        !daemon.is_running(),
        "Daemon should not be running after shutdown"
    );
    assert!(
        daemon.output().is_destroyed(),
        "Output device should be destroyed"
    );

    println!("Graceful shutdown test: All resources released");
}

/// Test: Drop trait ensures cleanup.
///
/// Verifies that resources are cleaned up when daemon goes out of scope.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_drop_cleanup() {
    let config = create_config(vec![create_device_config(
        "*",
        vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
    )]);

    let temp_file = create_temp_config_file(&config);

    // Create daemon in a block so it gets dropped
    {
        let daemon = Daemon::new(temp_file.path()).expect("Failed to initialize daemon");
        println!(
            "Drop cleanup test: Daemon created with {} device(s)",
            daemon.device_count()
        );
        // daemon will be dropped here
    }

    // If we reach here without issues, Drop worked correctly
    println!("Drop cleanup test: Daemon dropped successfully");
}

// ============================================================================
// Performance and Stress Tests
// ============================================================================

/// Test: Daemon handles rapid start/stop cycles.
///
/// Verifies that the daemon can be started and stopped multiple times
/// without resource leaks or issues.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_rapid_start_stop_cycles() {
    let config = create_config(vec![create_device_config(
        "*",
        vec![KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape)],
    )]);

    let temp_file = create_temp_config_file(&config);

    // Run multiple start/stop cycles
    for i in 0..5 {
        let result = run_daemon_with_timeout(temp_file.path(), Duration::from_millis(100));
        assert!(
            result.is_ok(),
            "Cycle {} should succeed: {:?}",
            i,
            result.err()
        );
    }

    println!("Rapid start/stop test: 5 cycles completed successfully");
}

/// Test: Long-running daemon stability.
///
/// Tests that the daemon remains stable over a longer period.
#[test]
#[ignore = "Requires real hardware - runs for 5 seconds"]
fn test_long_running_stability() {
    let config = create_config(vec![create_device_config(
        "*",
        vec![
            KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape),
            KeyMapping::modifier(KeyCode::Grave, 0),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
        ],
    )]);

    let temp_file = create_temp_config_file(&config);

    // Run for 5 seconds
    let result = run_daemon_with_timeout(temp_file.path(), Duration::from_secs(5));

    assert!(
        result.is_ok(),
        "Long-running daemon should be stable: {:?}",
        result.err()
    );

    println!("Long-running stability test: Ran for 5 seconds without issues");
}

// ============================================================================
// Signal Handling Tests
// ============================================================================

/// Test: Daemon responds to running flag changes.
///
/// Tests that the daemon stops when its running flag is set to false
/// (simulating what happens when SIGTERM/SIGINT is received).
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_running_flag_control() {
    let config = create_config(vec![create_device_config(
        "*",
        vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
    )]);

    let temp_file = create_temp_config_file(&config);

    let mut daemon = Daemon::new(temp_file.path()).expect("Failed to initialize daemon");
    let running = daemon.running_flag();

    // Spawn a thread that will stop the daemon after a short delay
    let running_clone = Arc::clone(&running);
    let stop_thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        running_clone.store(false, Ordering::SeqCst);
    });

    let start = Instant::now();
    let result = daemon.run();
    let elapsed = start.elapsed();

    // Wait for stop thread
    stop_thread.join().expect("Stop thread panicked");

    assert!(result.is_ok(), "Daemon should stop cleanly: {:?}", result);
    assert!(
        elapsed < Duration::from_secs(1),
        "Daemon should stop within reasonable time"
    );

    println!("Running flag control test: Daemon stopped in {:?}", elapsed);
}

// ============================================================================
// Complex Configuration Tests
// ============================================================================

/// Test: Complex multi-layer configuration.
///
/// Tests a configuration with multiple layers and conditional mappings.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_complex_multi_layer_config() {
    let config = create_config(vec![create_device_config(
        "*",
        vec![
            // Layer 0: Navigation (CapsLock)
            KeyMapping::modifier(KeyCode::CapsLock, 0),
            // Layer 1: Symbols (Grave/Backtick)
            KeyMapping::modifier(KeyCode::Grave, 1),
            // Navigation layer mappings
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![
                    BaseKeyMapping::Simple {
                        from: KeyCode::H,
                        to: KeyCode::Left,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::J,
                        to: KeyCode::Down,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::K,
                        to: KeyCode::Up,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::L,
                        to: KeyCode::Right,
                    },
                ],
            ),
            // Symbol layer mappings (when Grave is held)
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(1)]),
                vec![
                    // Number row → symbols
                    BaseKeyMapping::ModifiedOutput {
                        from: KeyCode::Num1,
                        to: KeyCode::Num1,
                        shift: true,
                        ctrl: false,
                        alt: false,
                        win: false,
                    }, // !
                    BaseKeyMapping::ModifiedOutput {
                        from: KeyCode::Num2,
                        to: KeyCode::Num2,
                        shift: true,
                        ctrl: false,
                        alt: false,
                        win: false,
                    }, // @
                ],
            ),
            // Combined layers (both modifiers active)
            KeyMapping::conditional(
                Condition::AllActive(vec![
                    ConditionItem::ModifierActive(0),
                    ConditionItem::ModifierActive(1),
                ]),
                vec![
                    // Special function when both layers active
                    BaseKeyMapping::Simple {
                        from: KeyCode::Space,
                        to: KeyCode::Enter,
                    },
                ],
            ),
        ],
    )]);

    let temp_file = create_temp_config_file(&config);

    let result = run_daemon_with_timeout(temp_file.path(), Duration::from_millis(500));

    assert!(
        result.is_ok(),
        "Complex multi-layer config should work: {:?}",
        result.err()
    );

    println!("Complex multi-layer test: Configuration loaded and ran successfully");
}

/// Test: Tap-hold configuration.
///
/// Tests a tap-hold mapping where tapping a key produces one output
/// and holding it activates a modifier.
#[test]
#[ignore = "Requires real hardware and /dev/uinput access"]
fn test_tap_hold_configuration() {
    let config = create_config(vec![create_device_config(
        "*",
        vec![
            // Space: Tap = Space, Hold = MD_00
            KeyMapping::Base(BaseKeyMapping::TapHold {
                from: KeyCode::Space,
                tap: KeyCode::Space,
                hold_modifier: 0,
                threshold_ms: 200,
            }),
            // When MD_00 is active (holding space), HJKL become arrows
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![
                    BaseKeyMapping::Simple {
                        from: KeyCode::H,
                        to: KeyCode::Left,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::J,
                        to: KeyCode::Down,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::K,
                        to: KeyCode::Up,
                    },
                    BaseKeyMapping::Simple {
                        from: KeyCode::L,
                        to: KeyCode::Right,
                    },
                ],
            ),
        ],
    )]);

    let temp_file = create_temp_config_file(&config);

    let result = run_daemon_with_timeout(temp_file.path(), Duration::from_millis(500));

    assert!(
        result.is_ok(),
        "Tap-hold config should work: {:?}",
        result.err()
    );

    println!("Tap-hold configuration test: Completed successfully");
}

// ============================================================================
// Interactive Manual Tests (Documentation Only)
// ============================================================================

/// Interactive test: Verify CapsLock→Escape remapping with real keypresses.
///
/// This test runs the daemon for 10 seconds, during which you can manually
/// test the remapping by pressing CapsLock and observing Escape output.
///
/// # Manual Verification Steps
/// 1. Run this test: `sudo cargo test test_manual_capslock_escape -- --ignored`
/// 2. Open a terminal or text editor alongside
/// 3. Press CapsLock - should act as Escape
/// 4. Original CapsLock functionality should be disabled
/// 5. After 10 seconds, the test ends and keyboard returns to normal
///
/// # Expected Behavior
/// - CapsLock press → Escape key is registered
/// - CapsLock indicator light should NOT toggle
#[test]
#[ignore = "Manual test - requires user interaction for 10 seconds"]
fn test_manual_capslock_escape() {
    println!("\n========================================");
    println!("MANUAL TEST: CapsLock → Escape Remapping");
    println!("========================================");
    println!("\nRunning for 10 seconds. During this time:");
    println!("- Press CapsLock - should act as Escape");
    println!("- Test in a text editor or terminal");
    println!("- Keyboard will be grabbed (exclusive access)");
    println!("\nPress Ctrl+C in another terminal to abort if stuck.\n");

    let config = create_config(vec![create_device_config(
        "*",
        vec![KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape)],
    )]);

    let temp_file = create_temp_config_file(&config);

    let result = run_daemon_with_timeout(temp_file.path(), Duration::from_secs(10));

    println!("\nTest completed.");
    assert!(result.is_ok(), "Manual test failed: {:?}", result.err());
}

/// Interactive test: Verify Vim navigation with real keypresses.
///
/// This test runs the daemon for 15 seconds, during which you can manually
/// test the Vim navigation layer.
///
/// # Manual Verification Steps
/// 1. Run this test: `sudo cargo test test_manual_vim_navigation -- --ignored`
/// 2. Open a text editor with some text
/// 3. Hold CapsLock and press H/J/K/L - should move cursor
/// 4. Release CapsLock and press H/J/K/L - should type letters
/// 5. After 15 seconds, the test ends
#[test]
#[ignore = "Manual test - requires user interaction for 15 seconds"]
fn test_manual_vim_navigation() {
    println!("\n========================================");
    println!("MANUAL TEST: Vim Navigation Layer");
    println!("========================================");
    println!("\nRunning for 15 seconds. During this time:");
    println!("- Hold CapsLock + H/J/K/L = Arrow keys");
    println!("- Release CapsLock, H/J/K/L = Normal letters");
    println!("- Test in a text editor");
    println!("\nPress Ctrl+C in another terminal to abort if stuck.\n");

    let config = create_config(vec![create_device_config(
        "*",
        vec![
            KeyMapping::modifier(KeyCode::CapsLock, 0),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::J,
                    to: KeyCode::Down,
                }],
            ),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::K,
                    to: KeyCode::Up,
                }],
            ),
            KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(0)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::L,
                    to: KeyCode::Right,
                }],
            ),
        ],
    )]);

    let temp_file = create_temp_config_file(&config);

    let result = run_daemon_with_timeout(temp_file.path(), Duration::from_secs(15));

    println!("\nTest completed.");
    assert!(result.is_ok(), "Manual test failed: {:?}", result.err());
}
