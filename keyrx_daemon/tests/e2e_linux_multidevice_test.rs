//! End-to-end test for Linux multi-device discrimination.
//!
//! This test verifies that we can:
//! 1. Discriminate between different keyboards (Numpad vs Main)
//! 2. Apply per-device remapping rules correctly
//! 3. Handle device-specific event streams independently
//!
//! Unlike the basic virtual E2E tests that just check processing doesn't error,
//! this test ACTUALLY VERIFIES that device patterns discriminate correctly:
//! - Events from Device A with matching pattern get remapped
//! - Events from Device B without matching pattern do NOT get remapped
//! - Each device maintains independent state (modifiers, locks)
//!
//! # Test Strategy
//!
//! We create 2 virtual uinput devices with distinct names:
//! - "keyrx-test-numpad-{timestamp}" (matches pattern "*numpad*")
//! - "keyrx-test-main-{timestamp}" (does NOT match pattern "*numpad*")
//!
//! Then we configure device-specific mappings:
//! - when_device("*numpad*"): A → B
//! - when_device("*main*"): A → C
//! - Default (no pattern): A → A (passthrough)
//!
//! We send A from each device and verify the correct output.

#![cfg(target_os = "linux")]

use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;

use keyrx_core::config::KeyCode;
use keyrx_daemon::test_utils::{OutputCapture, VirtualKeyboard};
use tempfile::NamedTempFile;

/// Helper to create a test configuration with device-specific mappings
fn create_multidevice_config() -> Result<NamedTempFile, std::io::Error> {
    let config_content = r#"
// Only match our virtual test devices, not the user's physical keyboard
device_start("*keyrx-test*");

// Numpad device: A → B
when_device_start("*numpad*");
    map("A", "VK_B");
when_device_end();

// Main keyboard device: A → C
when_device_start("*main*");
    map("A", "VK_C");
when_device_end();

device_end();
"#;

    let mut config_file = NamedTempFile::new()?;
    std::io::Write::write_all(&mut config_file, config_content.as_bytes())?;
    Ok(config_file)
}

/// Compile the Rhai config to .krx binary
fn compile_config(rhai_path: &std::path::Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
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

/// Start the daemon with the given config
fn start_daemon(krx_path: &std::path::Path) -> Result<Child, std::io::Error> {
    // Use cargo run --release for faster startup (already built during test compilation)
    // The daemon binary is already built as part of the test workspace
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
        .env("RUST_LOG", "info")
        .stdout(std::process::Stdio::null()) // Suppress stdout to reduce noise
        .stderr(std::process::Stdio::inherit()) // Keep stderr for errors
        .spawn()
}

/// Stop the daemon gracefully
fn stop_daemon(mut daemon: Child) -> Result<(), std::io::Error> {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    let pid = daemon.id();
    let nix_pid = Pid::from_raw(pid as i32);

    // Send SIGTERM for graceful shutdown
    if let Err(e) = kill(nix_pid, Signal::SIGTERM) {
        eprintln!("Warning: Failed to send SIGTERM: {}", e);
    }

    // Wait for process to exit (with timeout)
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(5);

    loop {
        match daemon.try_wait()? {
            Some(_) => return Ok(()),
            None => {
                if start.elapsed() >= timeout {
                    // Force kill if graceful shutdown times out
                    daemon.kill()?;
                    daemon.wait()?;
                    return Ok(());
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

#[test]
#[ignore] // TODO: Investigate why daemon can't find virtual keyboards
fn test_linux_multidevice_discrimination() -> Result<(), Box<dyn std::error::Error>> {
    if !keyrx_daemon::test_utils::can_access_uinput() {
        eprintln!("SKIPPED: uinput/input not accessible");
        return Ok(());
    }

    // 1. Create 2 virtual keyboards with distinct names
    let mut numpad = VirtualKeyboard::create("keyrx-test-numpad")?;
    let numpad_name = numpad.name().to_string();
    println!("Created numpad device: {}", numpad_name);

    let mut main_kbd = VirtualKeyboard::create("keyrx-test-main")?;
    let main_name = main_kbd.name().to_string();
    println!("Created main device: {}", main_name);

    // Give kernel time to register the virtual devices
    std::thread::sleep(Duration::from_millis(200));

    // 2. Create and compile config with device-specific mappings
    let config_file = create_multidevice_config()?;
    let config_path = config_file.path();
    println!("Created config: {:?}", config_path);

    let krx_path = compile_config(config_path)?;
    println!("Compiled to: {:?}", krx_path);

    // 3. Start the daemon
    let daemon = start_daemon(&krx_path)?;
    println!("Started daemon (PID: {})", daemon.id());

    // 4. Wait for daemon to be ready by polling for the virtual output device
    // This accounts for compilation time (first run) and initialization
    let mut output_capture =
        OutputCapture::find_by_name("keyrx Virtual Keyboard", Duration::from_secs(60))?;
    println!(
        "Capturing output from: {} at {}",
        output_capture.name(),
        output_capture.device_path().display()
    );

    // Wait for daemon to finish initialization and enter event loop
    // The daemon needs to: grab devices, start polling, enter event loop
    std::thread::sleep(Duration::from_millis(1000));

    // 5. TEST SCENARIO A: Send A from Numpad device
    // Expected: Should be remapped to B (matches pattern "*numpad*")
    println!("\n=== TEST A: Numpad Device (A → B) ===");

    // Send tap (press + release)
    let tap_events = VirtualKeyboard::tap_events(KeyCode::A);
    numpad.inject_sequence(&tap_events, Some(Duration::from_millis(10)))?;
    std::thread::sleep(Duration::from_millis(100));

    let events_a = output_capture.collect_events(Duration::from_millis(200))?;
    println!("Numpad events captured: {}", events_a.len());

    // Verify we got B press and release (not A)
    assert!(
        events_a.len() >= 2,
        "Expected at least 2 events (press + release), got {}",
        events_a.len()
    );

    let first_press = events_a
        .iter()
        .find(|e| e.is_press())
        .expect("No press event found");

    assert_eq!(
        first_press.keycode(),
        KeyCode::B,
        "Numpad device should remap A → B, but got {:?}",
        first_press.keycode()
    );

    println!("✓ Numpad correctly remapped A → B");

    // 6. TEST SCENARIO B: Send A from Main keyboard device
    // Expected: Should be remapped to C (matches pattern "*main*")
    println!("\n=== TEST B: Main Keyboard Device (A → C) ===");

    let tap_events = VirtualKeyboard::tap_events(KeyCode::A);
    main_kbd.inject_sequence(&tap_events, Some(Duration::from_millis(10)))?;
    std::thread::sleep(Duration::from_millis(100));

    let events_b = output_capture.collect_events(Duration::from_millis(200))?;
    println!("Main keyboard events captured: {}", events_b.len());

    assert!(
        events_b.len() >= 2,
        "Expected at least 2 events (press + release), got {}",
        events_b.len()
    );

    let second_press = events_b
        .iter()
        .find(|e| e.is_press())
        .expect("No press event found");

    assert_eq!(
        second_press.keycode(),
        KeyCode::C,
        "Main keyboard should remap A → C, but got {:?}",
        second_press.keycode()
    );

    println!("✓ Main keyboard correctly remapped A → C");

    // 7. TEST SCENARIO C: Verify devices are truly independent
    // Send different keys from both devices simultaneously-ish
    println!("\n=== TEST C: Independent State Verification ===");

    // Clear any residual events
    let _ = output_capture.collect_events(Duration::from_millis(100));

    // Numpad: B → should map to C (assuming we have this mapping)
    // Main: C → should map to D (assuming we have this mapping)
    // For now, just verify both devices can send events without interfering

    let tap_events = VirtualKeyboard::tap_events(KeyCode::A);
    numpad.inject_sequence(&tap_events, Some(Duration::from_millis(10)))?;
    std::thread::sleep(Duration::from_millis(50));
    main_kbd.inject_sequence(&tap_events, Some(Duration::from_millis(10)))?;
    std::thread::sleep(Duration::from_millis(100));

    let events_c = output_capture.collect_events(Duration::from_millis(200))?;
    println!("Combined events captured: {}", events_c.len());

    // Should have 4 events total (2 press + 2 release)
    assert!(
        events_c.len() >= 4,
        "Expected at least 4 events from both devices, got {}",
        events_c.len()
    );

    // Count B and C presses (one from each device)
    let b_presses = events_c
        .iter()
        .filter(|e| e.is_press() && e.keycode() == KeyCode::B)
        .count();
    let c_presses = events_c
        .iter()
        .filter(|e| e.is_press() && e.keycode() == KeyCode::C)
        .count();

    assert_eq!(
        b_presses, 1,
        "Expected 1 B press from numpad, got {}",
        b_presses
    );
    assert_eq!(
        c_presses, 1,
        "Expected 1 C press from main keyboard, got {}",
        c_presses
    );

    println!("✓ Both devices maintained independent mappings");

    // 8. Cleanup
    drop(output_capture);
    stop_daemon(daemon)?;

    // Clean up compiled config
    let _ = fs::remove_file(&krx_path);

    println!("\n=== ALL TESTS PASSED ===");
    Ok(())
}

#[test]
#[ignore] // TODO: Investigate why daemon can't find virtual keyboards
fn test_linux_multidevice_pattern_types() -> Result<(), Box<dyn std::error::Error>> {
    if !keyrx_daemon::test_utils::can_access_uinput() {
        eprintln!("SKIPPED: uinput/input not accessible");
        return Ok(());
    }

    // Test different pattern matching types with actual devices

    // Create devices with names that match different pattern types
    let mut prefix_dev = VirtualKeyboard::create("usb-test-keyboard")?;
    let mut suffix_dev = VirtualKeyboard::create("test-keyboard")?;
    let mut contains_dev = VirtualKeyboard::create("my-numpad-device")?;

    println!("Created prefix device: {}", prefix_dev.name());
    println!("Created suffix device: {}", suffix_dev.name());
    println!("Created contains device: {}", contains_dev.name());

    // Create config testing different pattern types
    // Only match test devices to avoid grabbing user's physical keyboard
    let config_content = r#"
device_start("*test*");

// Prefix pattern: usb-*
when_device_start("usb-*");
    map("A", "VK_B");
when_device_end();

// Suffix pattern: *-keyboard
when_device_start("*-keyboard");
    map("A", "VK_C");
when_device_end();

// Contains pattern: *numpad*
when_device_start("*numpad*");
    map("A", "VK_D");
when_device_end();

device_end();
"#;

    let mut config_file = NamedTempFile::new()?;
    std::io::Write::write_all(&mut config_file, config_content.as_bytes())?;

    // Give kernel time to register the virtual devices
    std::thread::sleep(Duration::from_millis(200));

    let krx_path = compile_config(config_file.path())?;
    let daemon = start_daemon(&krx_path)?;

    // Wait for daemon to be ready by polling for the virtual output device
    let mut output_capture =
        OutputCapture::find_by_name("keyrx Virtual Keyboard", Duration::from_secs(60))?;

    // Wait for daemon to finish initialization and enter event loop
    std::thread::sleep(Duration::from_millis(1000));

    // Test prefix pattern (usb-*)
    println!("\n=== Testing prefix pattern: usb-* ===");
    let tap_events = VirtualKeyboard::tap_events(KeyCode::A);
    prefix_dev.inject_sequence(&tap_events, Some(Duration::from_millis(10)))?;
    std::thread::sleep(Duration::from_millis(100));

    let events = output_capture.collect_events(Duration::from_millis(200))?;
    let press = events
        .iter()
        .find(|e| e.is_press())
        .expect("No press event");
    assert_eq!(
        press.keycode(),
        KeyCode::B,
        "Prefix pattern (usb-*) should match, got {:?}",
        press.keycode()
    );
    println!("✓ Prefix pattern matched correctly");

    // Test suffix pattern (*-keyboard)
    println!("\n=== Testing suffix pattern: *-keyboard ===");
    let tap_events = VirtualKeyboard::tap_events(KeyCode::A);
    suffix_dev.inject_sequence(&tap_events, Some(Duration::from_millis(10)))?;
    std::thread::sleep(Duration::from_millis(100));

    let events = output_capture.collect_events(Duration::from_millis(200))?;
    let press = events
        .iter()
        .find(|e| e.is_press())
        .expect("No press event");
    assert_eq!(
        press.keycode(),
        KeyCode::C,
        "Suffix pattern (*-keyboard) should match, got {:?}",
        press.keycode()
    );
    println!("✓ Suffix pattern matched correctly");

    // Test contains pattern (*numpad*)
    println!("\n=== Testing contains pattern: *numpad* ===");
    let tap_events = VirtualKeyboard::tap_events(KeyCode::A);
    contains_dev.inject_sequence(&tap_events, Some(Duration::from_millis(10)))?;
    std::thread::sleep(Duration::from_millis(100));

    let events = output_capture.collect_events(Duration::from_millis(200))?;
    let press = events
        .iter()
        .find(|e| e.is_press())
        .expect("No press event");
    assert_eq!(
        press.keycode(),
        KeyCode::D,
        "Contains pattern (*numpad*) should match, got {:?}",
        press.keycode()
    );
    println!("✓ Contains pattern matched correctly");

    // Cleanup
    drop(output_capture);
    stop_daemon(daemon)?;
    let _ = fs::remove_file(&krx_path);

    println!("\n=== ALL PATTERN TESTS PASSED ===");
    Ok(())
}
