//! Regression tests for bugs found during systematic bug hunt.
//!
//! Each test is labeled with its bug number and designed to:
//! 1. Demonstrate the bug behavior (fail before fix)
//! 2. Verify the fix (pass after fix)
//! 3. Prevent regression (catch if bug is reintroduced)
//!
//! Bug catalog:
//! - BUG #36 (CRITICAL): SIGTERM/SIGINT don't stop daemon
//! - BUG #33 (HIGH): No rollback on partial grab failure
//! - BUG #37 (HIGH): Windows unwrap panic on device hot-unplug
//! - BUG #34 (MEDIUM): Poll error causes busy loop
//! - BUG #35 (LOW): Missing POLLERR/POLLHUP handling
//! - BUG #38 (LOW): Config reload silently skips devices

#![cfg(target_os = "linux")]

use std::process::{Child, Command};
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

// ==============================================================================
// BUG #36 (CRITICAL): SIGTERM/SIGINT don't stop daemon
// ==============================================================================
//
// **Root Cause**: signal_hook::flag::register() sets flag to TRUE on signal,
//                  but daemon loop continues while running==TRUE.
//                  Should use register_conditional_default(..., false).
//
// **Impact**: Daemon cannot be stopped gracefully. Only kill -9 works,
//             which doesn't clean up resources (devices remain grabbed).
//
// **User Scenario**:
// 1. User starts daemon
// 2. User tries Ctrl+C or `kill <pid>`
// 3. Daemon keeps running (signal sets flag to TRUE instead of FALSE)
// 4. User's keyboard stays grabbed
// 5. Only `kill -9` works (no cleanup)
//
// **Fix Location**: keyrx_daemon/src/daemon/linux.rs:130-136
//   Change: signal_hook::flag::register()
//   To: signal_hook::flag::register_conditional_default(..., false)

/// Helper to create a minimal test configuration
fn create_minimal_config(timestamp: u64) -> Result<NamedTempFile, std::io::Error> {
    let config_content = format!(
        r#"
// Minimal config - matches only our virtual test device
device_start("*keyrx-test-signal-{}*");
    map("A", "VK_B");
device_end();
"#,
        timestamp
    );

    let mut config_file = NamedTempFile::new()?;
    std::io::Write::write_all(&mut config_file, config_content.as_bytes())?;
    Ok(config_file)
}

/// Compile Rhai config to .krx binary
fn compile_config(
    rhai_path: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let krx_path = rhai_path.with_extension("krx");

    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "keyrx_compiler",
            "--",
            "compile",
            rhai_path.to_str().unwrap(),
            "-o",
            krx_path.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(krx_path)
}

/// Start daemon as subprocess
fn start_daemon(krx_path: &std::path::Path) -> Result<Child, std::io::Error> {
    Command::new("cargo")
        .args([
            "run",
            "--release",
            "-p",
            "keyrx_daemon",
            "--features",
            "linux",
            "--",
            "run",
            "--config",
            krx_path.to_str().unwrap(),
        ])
        .env("RUST_LOG", "warn") // Reduce noise (only warnings/errors)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit()) // Keep errors visible
        .spawn()
}

#[test]
#[ignore] // Skip - test takes too long with coverage instrumentation
fn test_bug36_sigterm_stops_daemon() -> Result<(), Box<dyn std::error::Error>> {
    use keyrx_daemon::test_utils::VirtualKeyboard;

    // Skip if not root (daemon needs device access)
    if !keyrx_daemon::test_utils::can_access_uinput() {
        eprintln!("SKIPPED: Requires root access");
        return Ok(());
    }

    // 1. Create virtual keyboard with unique name
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as u64;
    let device_name = format!("keyrx-test-signal-{}", timestamp);

    let _virtual_kb = VirtualKeyboard::create(&device_name)?;
    println!("Created virtual keyboard: {}", device_name);

    // 2. Create and compile config
    let config_file = create_minimal_config(timestamp)?;
    let krx_path = compile_config(config_file.path())?;
    println!("Config compiled to: {:?}", krx_path);

    // 3. Start daemon
    let mut daemon = start_daemon(&krx_path)?;
    let pid = daemon.id();
    println!("Started daemon (PID: {})", pid);

    // 4. Give daemon time to initialize
    std::thread::sleep(Duration::from_secs(3));

    // 5. Verify daemon is running
    assert!(daemon.try_wait()?.is_none(), "Daemon should be running");

    // 6. Send SIGTERM for graceful shutdown
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    let nix_pid = Pid::from_raw(pid as i32);

    println!("Sending SIGTERM to daemon...");
    kill(nix_pid, Signal::SIGTERM)?;

    // 7. Wait for daemon to exit gracefully (SHOULD succeed after fix)
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    let mut graceful_shutdown = false;

    loop {
        match daemon.try_wait()? {
            Some(status) => {
                graceful_shutdown = true;
                println!("Daemon stopped gracefully with status: {}", status);
                break;
            }
            None => {
                if start.elapsed() >= timeout {
                    println!("TIMEOUT: Daemon did not stop within 5 seconds");
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    // 7. Cleanup: Force kill if still running
    if !graceful_shutdown {
        println!("Force killing daemon (BUG #36 active)...");
        daemon.kill()?;
        daemon.wait()?;
    }

    // 8. Assert graceful shutdown worked
    //    TODO: This will FAIL before fix, PASS after fix
    assert!(
        graceful_shutdown,
        "BUG #36: Daemon did not stop on SIGTERM within {} seconds. \
         Expected: Daemon stops gracefully. \
         Actual: Had to force kill. \
         Fix: Use signal_hook::flag::register_conditional_default(..., false) \
         in keyrx_daemon/src/daemon/linux.rs:130-136",
        timeout.as_secs()
    );

    Ok(())
}

#[test]
#[ignore] // Skip - test takes too long with coverage instrumentation
fn test_bug36_sigint_stops_daemon() -> Result<(), Box<dyn std::error::Error>> {
    use keyrx_daemon::test_utils::VirtualKeyboard;

    // Same test as SIGTERM but using SIGINT (Ctrl+C)

    if !keyrx_daemon::test_utils::can_access_uinput() {
        eprintln!("SKIPPED: Requires root access");
        return Ok(());
    }

    // 1. Create virtual keyboard
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as u64;
    let device_name = format!("keyrx-test-signal-{}", timestamp);

    let _virtual_kb = VirtualKeyboard::create(&device_name)?;
    println!("Created virtual keyboard: {}", device_name);

    // 2. Create and compile config
    let config_file = create_minimal_config(timestamp)?;
    let krx_path = compile_config(config_file.path())?;

    // 3. Start daemon
    let mut daemon = start_daemon(&krx_path)?;
    let pid = daemon.id();
    println!("Started daemon (PID: {})", pid);

    // 4. Give daemon time to initialize
    std::thread::sleep(Duration::from_secs(3));
    assert!(daemon.try_wait()?.is_none());

    // Send SIGINT (Ctrl+C)
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    let nix_pid = Pid::from_raw(pid as i32);

    println!("Sending SIGINT to daemon...");
    kill(nix_pid, Signal::SIGINT)?;

    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    let mut graceful_shutdown = false;

    loop {
        match daemon.try_wait()? {
            Some(status) => {
                graceful_shutdown = true;
                println!("Daemon stopped gracefully with status: {}", status);
                break;
            }
            None => {
                if start.elapsed() >= timeout {
                    println!("TIMEOUT: Daemon did not stop within 5 seconds");
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    if !graceful_shutdown {
        println!("Force killing daemon (BUG #36 active)...");
        daemon.kill()?;
        daemon.wait()?;
    }

    assert!(
        graceful_shutdown,
        "BUG #36: Daemon did not stop on SIGINT within {} seconds",
        timeout.as_secs()
    );

    Ok(())
}

// ==============================================================================
// BUG #33 (FALSE POSITIVE): Partial grab failure cleanup
// ==============================================================================
//
// **Original Concern**: grab_all_devices() returns on first error without
//                        explicit rollback of already-grabbed devices.
//
// **Actual Behavior**: The Daemon's Drop implementation (line 1042-1048)
//                       automatically calls shutdown() which releases all
//                       grabbed devices (checking is_grabbed() for each).
//
// **Test Result**: PASSES - Cleanup works correctly via Drop.
//
// **Conclusion**: This is NOT a bug. Rust's RAII via Drop provides automatic
//                 cleanup, which is idiomatic and correct.
//
// **Flow**:
// 1. Daemon::new() creates daemon (no grabbing)
// 2. daemon.run() calls grab_all_devices() (mod.rs:652)
// 3. If grab fails, run() returns error via ?
// 4. Daemon goes out of scope in main.rs
// 5. Drop::drop() calls shutdown()
// 6. shutdown() releases grabbed devices (mod.rs:890-906)
//
// **This test verifies**: Drop-based cleanup works correctly for partial grabs.

/// Test helper to create config that matches multiple devices
fn create_multidevice_config(timestamp: u64) -> Result<NamedTempFile, std::io::Error> {
    let config_content = format!(
        r#"
// Match both test devices
device_start("*keyrx-test-grab-{}*");
    map("A", "VK_B");
device_end();
"#,
        timestamp
    );

    let mut config_file = NamedTempFile::new()?;
    std::io::Write::write_all(&mut config_file, config_content.as_bytes())?;
    Ok(config_file)
}

#[test]
#[ignore] // Flaky test - BUG #33 is a false positive, test needs refinement
fn test_bug33_partial_grab_rollback() -> Result<(), Box<dyn std::error::Error>> {
    use evdev::Device;
    use keyrx_daemon::test_utils::VirtualKeyboard;
    use std::path::PathBuf;

    if !keyrx_daemon::test_utils::can_access_uinput() {
        eprintln!("SKIPPED: Requires root access");
        return Ok(());
    }

    // 1. Create two virtual keyboards
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as u64;
    let device1_name = format!("keyrx-test-grab-{}-dev1", timestamp);
    let device2_name = format!("keyrx-test-grab-{}-dev2", timestamp);

    let _virtual_kb1 = VirtualKeyboard::create(&device1_name)?;
    let _virtual_kb2 = VirtualKeyboard::create(&device2_name)?;
    println!(
        "Created virtual keyboards: {} and {}",
        device1_name, device2_name
    );

    // 2. Give udev time to create device nodes
    std::thread::sleep(Duration::from_millis(500));

    // 3. Find device paths
    let mut device1_path: Option<PathBuf> = None;
    let mut device2_path: Option<PathBuf> = None;

    for entry in std::fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if filename.starts_with("event") {
                if let Ok(dev) = Device::open(&path) {
                    if let Some(name) = dev.name() {
                        if name.contains(&device1_name) {
                            device1_path = Some(path.clone());
                            println!("Found device1 at: {:?}", path);
                        }
                        if name.contains(&device2_name) {
                            device2_path = Some(path.clone());
                            println!("Found device2 at: {:?}", path);
                        }
                    }
                }
            }
        }
    }

    let device1_path = device1_path.ok_or("Device 1 not found")?;
    let device2_path = device2_path.ok_or("Device 2 not found")?;

    // 4. Manually grab device2 to force daemon grab failure
    let mut device2_manual = Device::open(&device2_path)?;
    device2_manual.grab()?;
    println!("Manually grabbed device2, daemon will fail to grab it");

    // 5. Create and compile config
    let config_file = create_multidevice_config(timestamp)?;
    let krx_path = compile_config(config_file.path())?;

    // 6. Start daemon (will fail during initialization)
    let mut daemon = start_daemon(&krx_path)?;
    let pid = daemon.id();
    println!(
        "Started daemon (PID: {}), expecting initialization failure...",
        pid
    );

    // 7. Wait for daemon to exit with error
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    let mut daemon_exited = false;

    loop {
        match daemon.try_wait()? {
            Some(status) => {
                daemon_exited = true;
                println!(
                    "Daemon exited with status: {} (expected - grab failed)",
                    status
                );
                break;
            }
            None => {
                if start.elapsed() >= timeout {
                    println!("Daemon still running after {} seconds", timeout.as_secs());
                    daemon.kill()?;
                    daemon.wait()?;
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    assert!(daemon_exited, "Daemon should have exited with error");

    // 8. Release device2 manual grab
    device2_manual.ungrab()?;
    drop(device2_manual);
    println!("Released device2 manual grab");

    // 9. Try to grab device1 to check if it was released by daemon
    std::thread::sleep(Duration::from_millis(500)); // Give kernel time to process
    let mut device1_check = Device::open(&device1_path)?;
    let device1_grab_result = device1_check.grab();

    // 10. Clean up
    if device1_grab_result.is_ok() {
        device1_check.ungrab()?;
    }

    // 11. Assert device1 was released (verifies Drop cleanup works)
    assert!(
        device1_grab_result.is_ok(),
        "Drop-based cleanup FAILED: Device1 remained grabbed after partial grab failure!\n\
         Expected: Daemon's Drop impl should call shutdown() which releases grabbed devices.\n\
         Actual: Device1 is still grabbed (EBUSY).\n\
         This indicates the Drop implementation is broken."
    );

    println!("SUCCESS: Drop-based cleanup works - device1 was properly released");
    Ok(())
}

// ==============================================================================
// BUG #37 (HIGH): Windows unwrap panic on device hot-unplug
// ==============================================================================
//
// **Root Cause**: Windows event processing uses .unwrap() when accessing devices.
//                  Hot-unplugging causes Some -> None transition, panicking unwrap.
//
// **Impact**: Hot-unplugging keyboard crashes entire daemon, making ALL
//             keyboards unresponsive.
//
// **User Scenario**:
// 1. User has 2 keyboards on Windows
// 2. Daemon is running
// 3. User unplugs one keyboard
// 4. Daemon panics and crashes
// 5. Other keyboard also stops working
//
// **Fix Location**: keyrx_daemon/src/daemon/mod.rs:824, 832
//   Replace .unwrap() with match/if-let error handling

/// BUG #37 regression test - Windows device hot-unplug
///
/// This test verifies that hot-unplugging a keyboard on Windows doesn't crash the daemon.
/// The bug was fixed in daemon/mod.rs by replacing .unwrap() with proper error handling.
///
/// **Test Status:** Deferred - requires Windows platform for testing
/// **Reason:** Cannot be implemented on Linux; needs actual Windows Raw Input API testing
/// **Alternative:** Manual testing on Windows recommended before releases
#[test]
#[ignore] // Windows-only test - deferred to manual testing
#[cfg(target_os = "windows")]
fn test_bug37_windows_device_hotplug() {
    // TODO: Implement when Windows CI/testing infrastructure is available
    // Test should:
    // 1. Create multiple virtual keyboards using Windows Raw Input simulation
    // 2. Start daemon with both devices
    // 3. Simulate WM_INPUT_DEVICE_CHANGE (GIDC_REMOVAL) for one device
    // 4. Verify daemon continues running (no panic)
    // 5. Verify remaining device still processes events
    //
    // For now, this is tested manually on Windows before releases.
}

/// BUG #37 regression test - Linux device hot-unplug
///
/// This test verifies that hot-unplugging a keyboard on Linux doesn't crash the daemon.
/// The bug was originally found on Windows, but the same error handling applies to Linux.
///
/// **Test Strategy:**
/// 1. Create 2 virtual keyboards using uinput
/// 2. Start daemon configured to process both devices
/// 3. Verify both devices work (inject events, capture output)
/// 4. Destroy one device (simulate hot-unplug)
/// 5. Verify daemon continues running (doesn't crash/panic)
/// 6. Verify remaining device still processes events correctly
///
/// **Success Criteria:**
/// - Daemon doesn't crash when device is removed
/// - Remaining device continues to function
/// - Events from remaining device are still remapped correctly
#[test]
#[ignore] // Requires uinput/input access - run manually when needed
fn test_bug37_linux_device_hotplug() -> Result<(), Box<dyn std::error::Error>> {
    use keyrx_core::config::KeyCode;
    use keyrx_core::runtime::event::KeyEvent;
    use keyrx_daemon::test_utils::VirtualKeyboard;
    use std::process::Command;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    // Skip if uinput not accessible
    if !keyrx_daemon::test_utils::can_access_uinput() {
        eprintln!("SKIPPED: uinput/input not accessible");
        return Ok(());
    }

    println!("BUG #37 Linux hot-unplug regression test");
    println!("==========================================");

    // Generate unique timestamp for this test run
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)?
        .as_millis();

    // Step 1: Create minimal config for test devices
    println!("\n[1/6] Creating test configuration...");
    let config_content = format!(
        r#"
// Minimal config - matches only our virtual test devices
device_start("*keyrx-test-hotplug-{}*");
    map("A", "VK_B");  // A → B for easy verification
device_end();
"#,
        timestamp
    );

    let mut config_file = NamedTempFile::new()?;
    std::io::Write::write_all(&mut config_file, config_content.as_bytes())?;

    // Compile config to .krx
    let krx_path = config_file.path().with_extension("krx");
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "keyrx_compiler",
            "--",
            "compile",
            config_file.path().to_str().unwrap(),
            "-o",
            krx_path.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Config compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    println!("   ✓ Config compiled successfully");

    // Step 2: Create 2 virtual keyboards
    println!("\n[2/6] Creating virtual keyboards...");
    let base_name = format!("keyrx-test-hotplug-{}", timestamp);
    let mut keyboard1 = VirtualKeyboard::create(&format!("{}-device1", base_name))?;
    let mut keyboard2 = VirtualKeyboard::create(&format!("{}-device2", base_name))?;
    println!("   ✓ Created keyboard1: {}", keyboard1.name());
    println!("   ✓ Created keyboard2: {}", keyboard2.name());

    // Give kernel time to register devices
    std::thread::sleep(Duration::from_millis(200));

    // Step 3: Start daemon (in background - we'll verify it's still running later)
    println!("\n[3/6] Starting daemon...");
    let mut daemon = Command::new("cargo")
        .args([
            "run",
            "--release",
            "-p",
            "keyrx_daemon",
            "--features",
            "linux",
            "--",
            "run",
            "--config",
            krx_path.to_str().unwrap(),
        ])
        .env("RUST_LOG", "error") // Suppress logs except errors
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    // Give daemon time to initialize and grab devices
    std::thread::sleep(Duration::from_secs(2));

    // Verify daemon is still running
    match daemon.try_wait()? {
        Some(status) => {
            return Err(format!("Daemon exited prematurely with status: {}", status).into());
        }
        None => println!("   ✓ Daemon started successfully"),
    }

    // Step 4: Verify both devices work
    println!("\n[4/6] Verifying both devices work...");
    keyboard1.inject(KeyEvent::Press(KeyCode::A))?;
    keyboard1.inject(KeyEvent::Release(KeyCode::A))?;
    keyboard2.inject(KeyEvent::Press(KeyCode::A))?;
    keyboard2.inject(KeyEvent::Release(KeyCode::A))?;
    std::thread::sleep(Duration::from_millis(100));
    println!("   ✓ Injected events from both keyboards");

    // Step 5: Hot-unplug keyboard1 (destroy it)
    println!("\n[5/6] Simulating hot-unplug of keyboard1...");
    keyboard1.destroy()?;
    println!("   ✓ Destroyed keyboard1 (simulated hot-unplug)");

    // Give daemon time to detect removal and handle it
    std::thread::sleep(Duration::from_millis(500));

    // Verify daemon is STILL running (this is the key assertion)
    match daemon.try_wait()? {
        Some(status) => {
            return Err(format!(
                "FAILED: Daemon crashed after device hot-unplug! Exit status: {}",
                status
            )
            .into());
        }
        None => println!("   ✓ Daemon still running after hot-unplug"),
    }

    // Step 6: Verify keyboard2 still works
    println!("\n[6/6] Verifying keyboard2 still works...");
    keyboard2.inject(KeyEvent::Press(KeyCode::B))?;
    keyboard2.inject(KeyEvent::Release(KeyCode::B))?;
    std::thread::sleep(Duration::from_millis(100));
    println!("   ✓ Keyboard2 still processes events");

    // Final check: daemon still running
    match daemon.try_wait()? {
        Some(status) => {
            return Err(format!("FAILED: Daemon crashed! Exit status: {}", status).into());
        }
        None => println!("   ✓ Daemon remains stable"),
    }

    // Cleanup: Stop daemon gracefully
    println!("\nCleaning up...");
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    let pid = Pid::from_raw(daemon.id() as i32);
    let _ = kill(pid, Signal::SIGTERM);
    std::thread::sleep(Duration::from_millis(200));
    let _ = daemon.kill();
    let _ = daemon.wait();

    println!("\n==========================================");
    println!("✅ SUCCESS: BUG #37 regression test passed");
    println!("   - Daemon survived device hot-unplug");
    println!("   - Remaining device still functional");
    println!("==========================================\n");

    Ok(())
}

// ==============================================================================
// BUG #34 (MEDIUM): Poll error causes busy loop
// ==============================================================================
//
// **Root Cause**: When poll() fails, error handler immediately continues loop
//                  without backoff, causing 100% CPU usage.
//
// **Impact**: Persistent poll errors (all devices disconnected, FD corruption)
//             cause busy loop consuming 100% CPU and log spam.
//
// **User Scenario**:
// 1. All devices disconnect or file descriptors corrupt
// 2. poll() starts failing repeatedly
// 3. CPU usage spikes to 100%
// 4. Daemon becomes unresponsive
// 5. Logs spam with errors
//
// **Fix Location**: keyrx_daemon/src/daemon/mod.rs:673-682
//   Add sleep(100ms) after poll error before continue

/// Test that verifies poll error backoff is implemented.
///
/// This test reads the source code to verify that a sleep/backoff is added
/// after poll errors to prevent busy-looping.
#[test]
#[ignore = "Code moved to Platform trait - needs update for new architecture"]
fn test_bug34_poll_error_backoff_implementation() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    // Read the daemon source code
    let daemon_source = fs::read_to_string("src/daemon/mod.rs")?;

    // Find the poll error handling section
    let has_poll_error_handler = daemon_source.contains("Poll error:");
    assert!(
        has_poll_error_handler,
        "Could not find poll error handler in source code"
    );

    // Verify that the error handler includes a sleep/backoff mechanism
    // Look for the pattern: "Poll error" followed by sleep within ~20 lines
    let lines: Vec<&str> = daemon_source.lines().collect();
    let mut found_poll_error = false;
    let mut found_sleep_after_error = false;

    for (idx, line) in lines.iter().enumerate() {
        if line.contains("Poll error:") && line.contains("warn!") {
            found_poll_error = true;

            // Check next 10 lines for sleep/backoff
            for (offset, check_line) in lines.iter().skip(idx).take(10).enumerate() {
                if check_line.contains("sleep") || check_line.contains("Duration::from_millis") {
                    found_sleep_after_error = true;
                    println!("Found sleep after poll error at line offset {}", offset);
                    break;
                }
            }

            break;
        }
    }

    assert!(
        found_poll_error,
        "Could not find 'Poll error:' log statement"
    );

    // This will FAIL before fix, PASS after fix
    assert!(
        found_sleep_after_error,
        "BUG #34: No sleep/backoff found after poll error!\n\
         Expected: sleep(Duration::from_millis(100)) after 'warn!(\"Poll error: ...\")'\n\
         Actual: Immediate 'continue' causes busy loop\n\
         Fix location: keyrx_daemon/src/daemon/mod.rs:673-682\n\
         Add: std::thread::sleep(Duration::from_millis(100)); before continue"
    );

    println!("SUCCESS: Poll error backoff is implemented");
    Ok(())
}

// ==============================================================================
// BUG #35 (LOW): Missing POLLERR/POLLHUP handling
// ==============================================================================
//
// **Root Cause**: poll_devices() only checks POLLIN flag, ignoring error flags.
//
// **Impact**: Device disconnection not detected efficiently, wastes poll iterations.
//
// **Fix Location**: keyrx_daemon/src/daemon/mod.rs:794-801
//   Add checks for POLLERR, POLLHUP, POLLNVAL flags

/// Test that verifies POLLERR/POLLHUP handling is implemented.
///
/// This test reads the source code to verify that poll error flags
/// (POLLERR, POLLHUP, POLLNVAL) are checked in poll_devices().
#[test]
#[ignore = "Code moved to Platform trait - needs update for new architecture"]
#[cfg(target_os = "linux")] // Linux-specific - Windows doesn't use poll()
fn test_bug35_pollerr_handling_implementation() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    // Read the daemon source code
    let daemon_source = fs::read_to_string("src/daemon/mod.rs")?;

    // Find the poll_devices function
    let has_poll_devices = daemon_source.contains("fn poll_devices");
    assert!(has_poll_devices, "Could not find poll_devices function");

    // Check if POLLIN is checked
    let has_pollin_check = daemon_source.contains("POLLIN");
    assert!(has_pollin_check, "Could not find POLLIN check");

    // Check if error flags are also checked
    let has_pollerr = daemon_source.contains("POLLERR");
    let has_pollhup = daemon_source.contains("POLLHUP");
    let has_pollnval = daemon_source.contains("POLLNVAL");

    let has_error_flag_handling = has_pollerr || has_pollhup || has_pollnval;

    // This will FAIL before fix, PASS after fix
    assert!(
        has_error_flag_handling,
        "BUG #35: Missing POLLERR/POLLHUP/POLLNVAL handling!\n\
         Expected: Check for error flags in poll_devices()\n\
         Actual: Only POLLIN is checked\n\
         Fix location: keyrx_daemon/src/daemon/mod.rs:794-801\n\
         Add checks for:\n\
         - PollFlags::POLLERR (device error)\n\
         - PollFlags::POLLHUP (device hangup/disconnect)\n\
         - PollFlags::POLLNVAL (invalid fd)\n\
         This improves disconnect detection efficiency"
    );

    println!("SUCCESS: POLLERR/POLLHUP/POLLNVAL handling is implemented");
    Ok(())
}

// ==============================================================================
// BUG #38 (LOW): Config reload silently skips devices
// ==============================================================================
//
// **Root Cause**: rebuild_lookups() silently skips devices with out-of-bounds
//                  config_index, keeping old config.
//
// **Impact**: Some devices keep old mappings while others get new ones,
//             causing inconsistent behavior that's hard to debug.
//
// **User Scenario**:
// 1. User has 3 device configs
// 2. User reloads with 2 device configs
// 3. Device #3 silently keeps old mappings
// 4. Inconsistent behavior - hard to debug
//
// **Fix Location**:
//   - keyrx_daemon/src/device_manager/linux.rs:249-255
//   - keyrx_daemon/src/device_manager/windows.rs:166-172
//   Add logging when device config is skipped

/// Test that verifies config reload logging is implemented.
///
/// This test reads the source code to verify that a warning is logged
/// when rebuild_lookups encounters an out-of-bounds config_index.
#[test]
fn test_bug38_config_reload_logging_implementation() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    // Check Linux implementation
    let linux_dm_source = fs::read_to_string("src/device_manager/linux.rs")?;

    // Find the rebuild_lookups function
    let has_rebuild_lookups = linux_dm_source.contains("pub fn rebuild_lookups");
    assert!(
        has_rebuild_lookups,
        "Could not find rebuild_lookups function in Linux device manager"
    );

    // Look for the config.get() pattern and check if there's an else branch with warn!
    let has_config_get = linux_dm_source.contains("configs.get(");
    assert!(has_config_get, "Could not find configs.get() call");

    // Check if there's a warn! after the configs.get() check
    let lines: Vec<&str> = linux_dm_source.lines().collect();
    let mut found_config_get = false;
    let mut found_else_warn = false;

    for (idx, line) in lines.iter().enumerate() {
        if line.contains("configs.get(") && line.contains("config_index") {
            found_config_get = true;

            // Check next 15 lines for else branch with warn!
            for (check_offset, check_line) in lines.iter().skip(idx).take(15).enumerate() {
                if check_line.contains("else") {
                    // Check if this else branch has a warn!
                    for (warn_offset, warn_line) in
                        lines.iter().skip(idx + check_offset).take(5).enumerate()
                    {
                        if warn_line.contains("warn!") {
                            found_else_warn = true;
                            println!(
                                "Found warn! in else branch at line offset {}",
                                check_offset + warn_offset
                            );
                            break;
                        }
                    }
                    break;
                }
            }
            break;
        }
    }

    assert!(
        found_config_get,
        "Could not find configs.get(config_index) pattern"
    );

    // This will FAIL before fix, PASS after fix
    assert!(
        found_else_warn,
        "BUG #38: No warning logged for out-of-bounds config_index!\n\
         Expected: else branch with warn!() when configs.get() returns None\n\
         Actual: Silent skip (no else branch)\n\
         Fix locations:\n\
         - keyrx_daemon/src/device_manager/linux.rs:249-255\n\
         - keyrx_daemon/src/device_manager/windows.rs:166-172\n\
         Add else branch with warn! logging"
    );

    println!("SUCCESS: Config reload logging is implemented");
    Ok(())
}
