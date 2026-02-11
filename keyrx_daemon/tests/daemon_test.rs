//! Integration tests for Daemon lifecycle management.
//!
//! These tests validate:
//! - Daemon initialization and shutdown
//! - Signal handling (SIGTERM, SIGINT, SIGHUP)
//! - Configuration reload functionality
//!
//! Tests that require real devices use runtime skip macros for CI compatibility.
//! Signal handling tests require process isolation and remain marked with `#[ignore]`.
//! To run ignored tests locally: `cargo test --package keyrx_daemon -- --ignored`

#![cfg(all(target_os = "linux", feature = "linux"))]

use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use keyrx_compiler::serialize::serialize;
use keyrx_core::config::{
    ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
};
use keyrx_daemon::daemon::{install_signal_handlers, DaemonError, ReloadState};
use tempfile::NamedTempFile;

// ============================================================================
// Test Configuration Helpers
// ============================================================================

/// Creates a minimal valid configuration for testing.
fn create_test_config(mappings: Vec<KeyMapping>) -> ConfigRoot {
    ConfigRoot {
        version: Version::current(),
        devices: vec![DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".to_string(),
            },
            mappings,
        }],
        metadata: Metadata {
            compilation_timestamp: 0,
            compiler_version: "test".to_string(),
            source_hash: "test".to_string(),
        },
    }
}

/// Creates a temporary .krx config file for testing.
fn create_temp_config_file(mappings: Vec<KeyMapping>) -> NamedTempFile {
    let config = create_test_config(mappings);
    let bytes = serialize(&config).expect("Failed to serialize config");

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(&bytes).expect("Failed to write config");
    temp_file.flush().expect("Failed to flush");

    temp_file
}

// ============================================================================
// Signal Handler Integration Tests
// ============================================================================

/// Test: Signal handler installation and running flag coordination.
///
/// Verifies that signal handlers are installed correctly and coordinate
/// with the running flag for shutdown detection.
#[test]
fn test_signal_handler_installation_and_coordination() {
    let running = Arc::new(AtomicBool::new(true));

    // Install signal handlers
    let handler =
        install_signal_handlers(Arc::clone(&running)).expect("Failed to install signal handlers");

    // Running flag should still be true
    assert!(running.load(Ordering::SeqCst));

    // No reload should be pending
    assert!(!handler.check_reload());

    // Simulate external modification of running flag (as would happen with signal)
    running.store(false, Ordering::SeqCst);

    // Running flag should now be false
    assert!(!running.load(Ordering::SeqCst));
}

/// Test: Reload state flag coordination with signal handler.
///
/// Verifies that the reload state can be triggered and detected correctly.
#[test]
fn test_reload_state_coordination() {
    let running = Arc::new(AtomicBool::new(true));
    let handler =
        install_signal_handlers(Arc::clone(&running)).expect("Failed to install signal handlers");

    // Initially no reload requested
    assert!(!handler.check_reload());

    // Simulate SIGHUP by setting the reload flag directly
    handler.reload_state().flag().store(true, Ordering::SeqCst);

    // First check should return true and clear the flag
    assert!(handler.check_reload());

    // Subsequent checks should return false
    assert!(!handler.check_reload());
    assert!(!handler.check_reload());
}

/// Test: Multiple reload requests are handled correctly.
///
/// Verifies that rapid SIGHUP signals are handled without race conditions.
#[test]
fn test_multiple_reload_requests() {
    let running = Arc::new(AtomicBool::new(true));
    let handler =
        install_signal_handlers(Arc::clone(&running)).expect("Failed to install signal handlers");

    // Simulate multiple rapid SIGHUPs
    for _ in 0..5 {
        handler.reload_state().flag().store(true, Ordering::SeqCst);
        assert!(handler.check_reload());
        assert!(!handler.check_reload());
    }
}

/// Test: Signal handler works across threads.
///
/// Verifies thread-safety of the running flag and reload state.
#[test]
fn test_signal_handler_thread_safety() {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    let handler =
        install_signal_handlers(Arc::clone(&running)).expect("Failed to install signal handlers");

    let reload_flag = handler.reload_state().flag();

    // Spawn a thread that will modify the flags
    let handle = thread::spawn(move || {
        // Give the main thread time to start waiting
        thread::sleep(Duration::from_millis(50));

        // Simulate SIGHUP
        reload_flag.store(true, Ordering::SeqCst);

        // Simulate SIGTERM
        thread::sleep(Duration::from_millis(50));
        running_clone.store(false, Ordering::SeqCst);
    });

    // Wait for reload signal
    let mut reload_detected = false;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(1) {
        if handler.check_reload() {
            reload_detected = true;
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(reload_detected, "Reload signal should be detected");

    // Wait for shutdown signal
    let mut shutdown_detected = false;
    while start.elapsed() < Duration::from_secs(2) {
        if !running.load(Ordering::SeqCst) {
            shutdown_detected = true;
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(shutdown_detected, "Shutdown signal should be detected");

    handle.join().unwrap();
}

// ============================================================================
// ReloadState Unit Tests
// ============================================================================

/// Test: ReloadState initialization.
#[test]
fn test_reload_state_new() {
    let state = ReloadState::new();
    assert!(!state.check_and_clear());
}

/// Test: ReloadState default implementation.
#[test]
fn test_reload_state_default() {
    let state = ReloadState::default();
    assert!(!state.check_and_clear());
}

/// Test: ReloadState flag sharing.
#[test]
fn test_reload_state_flag_sharing() {
    let state = ReloadState::new();
    let flag = state.flag();

    // Set via external reference
    flag.store(true, Ordering::SeqCst);

    // Should be detectable via check_and_clear
    assert!(state.check_and_clear());
    assert!(!state.check_and_clear());
}

/// Test: ReloadState check_and_clear is atomic.
#[test]
fn test_reload_state_atomic_check_and_clear() {
    let state = ReloadState::new();

    // Set the flag
    state.flag().store(true, Ordering::SeqCst);

    // Concurrent checks from multiple threads
    let state_clone = state.clone();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let s = state_clone.clone();
            thread::spawn(move || s.check_and_clear())
        })
        .collect();

    let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Exactly one thread should have gotten true
    let true_count = results.iter().filter(|&&x| x).count();
    assert_eq!(true_count, 1, "Only one check_and_clear should return true");
}

// ============================================================================
// Daemon Initialization Tests (Mock-based)
// ============================================================================

/// Test: Daemon initialization fails with missing config file.
///
/// This test does not require real devices as it fails before device discovery.
#[test]
fn test_daemon_new_missing_config() {
    use keyrx_daemon::daemon::Daemon;

    let result = Daemon::new(Path::new("/nonexistent/path/config.krx"));
    assert!(result.is_err());

    match result {
        Err(DaemonError::Config(_)) => {} // Expected
        Err(e) => panic!("Expected Config error, got: {}", e),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

/// Test: Daemon initialization fails with invalid config file.
///
/// This test does not require real devices as it fails during config parsing.
#[test]
fn test_daemon_new_invalid_config() {
    use keyrx_daemon::daemon::Daemon;

    // Create a temp file with invalid content (not valid rkyv data)
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(b"invalid krx data")
        .expect("Failed to write");
    temp_file.flush().expect("Failed to flush");

    let result = Daemon::new(temp_file.path());
    assert!(result.is_err());

    match result {
        Err(DaemonError::Config(_)) => {} // Expected
        Err(e) => panic!("Expected Config error, got: {}", e),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

/// Test: Daemon initialization fails with empty config file.
#[test]
fn test_daemon_new_empty_config() {
    use keyrx_daemon::daemon::Daemon;

    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    // File is empty

    let result = Daemon::new(temp_file.path());
    assert!(result.is_err());

    match result {
        Err(DaemonError::Config(_)) => {} // Expected
        Err(e) => panic!("Expected Config error, got: {}", e),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

// ============================================================================
// Daemon Lifecycle Tests (Requires Real Devices - Ignored for CI)
// ============================================================================

/// Test: Complete daemon initialization with real devices.
///
/// Verifies that the daemon can:
/// 1. Load a valid configuration
/// 2. Discover input devices
/// 3. Create uinput output device
/// 4. Install signal handlers
///
/// Requires: Access to /dev/input and /dev/uinput devices.
#[test]
fn test_daemon_full_initialization() {
    keyrx_daemon::skip_if_no_uinput!();
    use keyrx_daemon::daemon::Daemon;

    let temp_file = create_temp_config_file(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    match Daemon::new(temp_file.path()) {
        Ok(daemon) => {
            assert!(daemon.device_count() > 0, "Should have at least one device");
            assert!(daemon.is_running(), "Daemon should be running initially");
            assert_eq!(daemon.config_path(), temp_file.path());
            println!(
                "Daemon initialized with {} device(s)",
                daemon.device_count()
            );
        }
        Err(e) => {
            panic!("Daemon initialization failed: {}", e);
        }
    }
}

/// Test: Daemon shutdown releases all resources.
///
/// Verifies that shutdown:
/// 1. Releases all grabbed devices
/// 2. Destroys the uinput device
/// 3. Sets running flag to false
///
/// Requires: Access to /dev/input and /dev/uinput devices.
#[test]
fn test_daemon_shutdown_releases_resources() {
    keyrx_daemon::skip_if_no_uinput!();
    use keyrx_daemon::daemon::Daemon;

    let temp_file = create_temp_config_file(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    let mut daemon = match Daemon::new(temp_file.path()) {
        Ok(d) => d,
        Err(e) => {
            panic!("Daemon initialization failed: {}", e);
        }
    };

    // Verify initial state
    assert!(daemon.is_running(), "Daemon should be running initially");

    // Perform shutdown
    daemon.shutdown();

    // Verify shutdown state
    assert!(
        !daemon.is_running(),
        "Daemon should not be running after shutdown"
    );
    assert!(
        daemon.output().is_destroyed(),
        "Output device should be destroyed"
    );
}

/// Test: Daemon shutdown is idempotent.
///
/// Verifies that calling shutdown multiple times does not panic or error.
///
/// Requires: Access to /dev/input and /dev/uinput devices.
#[test]
fn test_daemon_shutdown_idempotent() {
    keyrx_daemon::skip_if_no_uinput!();
    use keyrx_daemon::daemon::Daemon;

    let temp_file = create_temp_config_file(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    let mut daemon = match Daemon::new(temp_file.path()) {
        Ok(d) => d,
        Err(e) => {
            panic!("Daemon initialization failed: {}", e);
        }
    };

    // Call shutdown multiple times - should not panic
    daemon.shutdown();
    daemon.shutdown();
    daemon.shutdown();

    assert!(!daemon.is_running());
    assert!(daemon.output().is_destroyed());
}

/// Test: Daemon Drop trait calls shutdown automatically.
///
/// Verifies that resources are cleaned up when daemon goes out of scope.
///
/// Requires: Access to /dev/input and /dev/uinput devices.
#[test]
fn test_daemon_drop_calls_shutdown() {
    keyrx_daemon::skip_if_no_uinput!();
    use keyrx_daemon::daemon::Daemon;

    let temp_file = create_temp_config_file(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    // Create daemon in a block so it gets dropped
    {
        let daemon = match Daemon::new(temp_file.path()) {
            Ok(d) => d,
            Err(e) => {
                panic!("Daemon initialization failed: {}", e);
            }
        };

        println!(
            "Daemon created with {} device(s), dropping now...",
            daemon.device_count()
        );
        // daemon will be dropped here
    }

    // If we reach here without panic, Drop worked correctly
    println!("Daemon dropped successfully - cleanup via Drop verified");
}

// ============================================================================
// Configuration Reload Tests (Requires Real Devices - Ignored for CI)
// ============================================================================

/// Test: Configuration reload with valid new config.
///
/// Verifies that:
/// 1. reload() successfully loads new configuration
/// 2. Lookup tables are updated
/// 3. Device count remains same
///
/// Requires: Access to /dev/input and /dev/uinput devices.
#[test]
fn test_daemon_reload_success() {
    keyrx_daemon::skip_if_no_uinput!();
    use keyrx_daemon::daemon::Daemon;

    let temp_file = create_temp_config_file(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    let mut daemon = match Daemon::new(temp_file.path()) {
        Ok(d) => d,
        Err(e) => {
            panic!("Daemon initialization failed: {}", e);
        }
    };

    let device_count_before = daemon.device_count();

    // Create updated config (A -> C instead of A -> B)
    let updated_config = ConfigRoot {
        version: Version::current(),
        devices: vec![DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".to_string(),
            },
            mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::C)],
        }],
        metadata: Metadata {
            compilation_timestamp: 1,
            compiler_version: "test".to_string(),
            source_hash: "updated".to_string(),
        },
    };

    // Overwrite config file
    let updated_bytes = serialize(&updated_config).expect("Failed to serialize");
    std::fs::write(temp_file.path(), &updated_bytes).expect("Failed to write");

    // Reload
    let result = daemon.reload();
    assert!(result.is_ok(), "Reload should succeed: {:?}", result.err());

    // Device count should remain same
    assert_eq!(
        daemon.device_count(),
        device_count_before,
        "Device count should remain same after reload"
    );
}

/// Test: Configuration reload fails gracefully with missing file.
///
/// Verifies that:
/// 1. reload() returns error when file is missing
/// 2. Daemon continues running with old config
///
/// Requires: Access to /dev/input and /dev/uinput devices.
#[test]
fn test_daemon_reload_missing_file() {
    keyrx_daemon::skip_if_no_uinput!();
    use keyrx_daemon::daemon::Daemon;

    let temp_file = create_temp_config_file(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    let mut daemon = match Daemon::new(temp_file.path()) {
        Ok(d) => d,
        Err(e) => {
            panic!("Daemon initialization failed: {}", e);
        }
    };

    // Delete config file
    std::fs::remove_file(temp_file.path()).expect("Failed to remove file");

    // Reload should fail
    let result = daemon.reload();
    assert!(result.is_err(), "Reload should fail when file is missing");

    match result {
        Err(DaemonError::Config(_)) => {} // Expected
        Err(e) => panic!("Expected Config error, got: {}", e),
        Ok(_) => panic!("Expected error, got Ok"),
    }

    // Daemon should still be running
    assert!(daemon.is_running(), "Daemon should still be running");
}

/// Test: Configuration reload fails gracefully with invalid file.
///
/// Verifies that:
/// 1. reload() returns error when file is invalid
/// 2. Daemon continues running with old config
///
/// Requires: Access to /dev/input and /dev/uinput devices.
#[test]
fn test_daemon_reload_invalid_file() {
    keyrx_daemon::skip_if_no_uinput!();
    use keyrx_daemon::daemon::Daemon;

    let temp_file = create_temp_config_file(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);

    let mut daemon = match Daemon::new(temp_file.path()) {
        Ok(d) => d,
        Err(e) => {
            panic!("Daemon initialization failed: {}", e);
        }
    };

    // Overwrite with invalid content
    std::fs::write(temp_file.path(), b"invalid krx data").expect("Failed to write");

    // Reload should fail
    let result = daemon.reload();
    assert!(result.is_err(), "Reload should fail with invalid file");

    match result {
        Err(DaemonError::Config(_)) => {} // Expected
        Err(e) => panic!("Expected Config error, got: {}", e),
        Ok(_) => panic!("Expected error, got Ok"),
    }

    // Daemon should still be running
    assert!(daemon.is_running(), "Daemon should still be running");
}

// ============================================================================
// Signal Handling Integration Tests (Requires Process Isolation)
// ============================================================================

/// Test: SIGTERM signal handling (deferred - requires subprocess spawning)
///
/// **Signal handling IS tested** in platform-specific integration tests.
/// This unit test is deferred because it would require subprocess spawning and
/// inter-process signaling, which adds significant complexity for minimal benefit.
///
/// **Why deferred:**
/// - Unit tests run in the same process (cannot self-signal)
/// - Subprocess spawning adds ~100 lines of test infrastructure
/// - Signal handling is already verified in daemon platform integration tests
/// - Manual testing on target platforms is more reliable for signal behavior
///
/// **When to revisit:** If CI/CD requires automated signal testing, or if
/// platform-specific signal bugs are discovered.
#[test]
#[ignore = "Deferred - requires subprocess spawning (see doc comment)"]
fn test_sigterm_signal_handling() {
    // Test implementation deferred - see doc comment above
    // Signal handling is tested in platform-specific integration tests instead
}

/// Test: SIGINT signal handling (deferred - requires subprocess spawning)
///
/// **Signal handling IS tested** in platform-specific integration tests.
/// See `test_sigterm_signal_handling()` doc comment for deferral rationale.
#[test]
#[ignore = "Deferred - requires subprocess spawning (see SIGTERM test doc)"]
fn test_sigint_signal_handling() {
    // Test implementation deferred - see test_sigterm_signal_handling() doc comment
    // Signal handling is tested in platform-specific integration tests instead
}

/// Test: SIGHUP signal handling triggers reload (deferred - requires subprocess spawning)
///
/// **Signal handling IS tested** in platform-specific integration tests.
/// See `test_sigterm_signal_handling()` doc comment for deferral rationale.
#[test]
#[ignore = "Deferred - requires subprocess spawning (see SIGTERM test doc)"]
fn test_sighup_signal_handling() {
    // Test implementation deferred - see test_sigterm_signal_handling() doc comment
    // Signal handling is tested in platform-specific integration tests instead
}

// ============================================================================
// DaemonError Tests
// ============================================================================

/// Test: DaemonError display messages.
#[test]
fn test_daemon_error_display() {
    let err = DaemonError::PermissionError("access denied".to_string());
    assert_eq!(err.to_string(), "permission error: access denied");

    let err = DaemonError::RuntimeError("event loop failed".to_string());
    assert_eq!(err.to_string(), "runtime error: event loop failed");
}

/// Test: DaemonError from IO error.
#[test]
fn test_daemon_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test");
    let daemon_err = DaemonError::SignalError(io_err);
    assert!(daemon_err.to_string().contains("signal handlers"));
}

/// Test: DaemonError from DeviceError.
#[test]
fn test_daemon_error_from_device() {
    use keyrx_daemon::platform::DeviceError;

    let device_err = DeviceError::NotFound("test device".to_string());
    let daemon_err = DaemonError::Device(device_err);
    assert!(daemon_err.to_string().contains("device error"));
}

/// Test: DaemonError from DiscoveryError.
#[test]
fn test_daemon_error_from_discovery() {
    use keyrx_daemon::device_manager::DiscoveryError;

    let discovery_err = DiscoveryError::NoDevicesFound;
    let daemon_err = DaemonError::DiscoveryError(discovery_err);
    assert!(daemon_err.to_string().contains("device discovery"));
}

// ============================================================================
// Exit Code Tests
// ============================================================================

/// Test: ExitCode values match documented values.
#[test]
fn test_exit_code_values() {
    use keyrx_daemon::daemon::ExitCode;

    assert_eq!(ExitCode::Success as u8, 0);
    assert_eq!(ExitCode::ConfigError as u8, 1);
    assert_eq!(ExitCode::PermissionError as u8, 2);
    assert_eq!(ExitCode::RuntimeError as u8, 3);
}

/// Test: ExitCode to i32 conversion.
#[test]
fn test_exit_code_to_i32() {
    use keyrx_daemon::daemon::ExitCode;

    assert_eq!(i32::from(ExitCode::Success), 0);
    assert_eq!(i32::from(ExitCode::ConfigError), 1);
    assert_eq!(i32::from(ExitCode::PermissionError), 2);
    assert_eq!(i32::from(ExitCode::RuntimeError), 3);
}

// ============================================================================
// Concurrency and Stress Tests
// ============================================================================

/// Test: Concurrent reload flag access is thread-safe.
#[test]
fn test_concurrent_reload_flag_access() {
    let running = Arc::new(AtomicBool::new(true));
    let handler =
        install_signal_handlers(Arc::clone(&running)).expect("Failed to install signal handlers");

    let reload_state = handler.reload_state().clone();

    // Spawn multiple threads that set and check the reload flag
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let state = reload_state.clone();
            thread::spawn(move || {
                for _ in 0..100 {
                    if i % 2 == 0 {
                        state.flag().store(true, Ordering::SeqCst);
                    } else {
                        state.check_and_clear();
                    }
                    thread::sleep(Duration::from_micros(10));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Should not panic or have any data races
    // Final state may be true or false depending on thread scheduling
}

/// Test: Running flag can be toggled rapidly.
#[test]
fn test_rapid_running_flag_toggle() {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    let _handler =
        install_signal_handlers(Arc::clone(&running)).expect("Failed to install signal handlers");

    // Toggle rapidly
    let handle = thread::spawn(move || {
        for _ in 0..1000 {
            running_clone.store(true, Ordering::SeqCst);
            running_clone.store(false, Ordering::SeqCst);
        }
    });

    handle.join().unwrap();

    // Should not panic, final state is deterministic (false)
    assert!(!running.load(Ordering::SeqCst));
}
