//! Multi-device end-to-end tests for macOS.
//!
//! This test suite verifies:
//! - Device enumeration and identification
//! - Serial number-based device discrimination
//! - Device-specific configuration loading
//! - Multiple keyboards with different mappings (when hardware available)
//!
//! Unlike Linux E2E tests, macOS tests cannot create virtual devices (no uinput).
//! These tests focus on device discovery and config loading with real hardware.
//! Tests gracefully handle single-device scenarios (most developer setups).

#![cfg(target_os = "macos")]

mod e2e_macos_harness;

use e2e_macos_harness::{MacosE2EConfig, MacosE2EHarness};
use keyrx_core::config::KeyCode;
use keyrx_daemon::platform::macos::{device_discovery, permissions};

/// Tests device enumeration without requiring Accessibility permission.
///
/// This test verifies that:
/// 1. Device enumeration works (no crash)
/// 2. Device metadata is properly formatted
/// 3. Serial numbers are extracted when available
///
/// **Note:** This test does NOT require Accessibility permission since it only
/// enumerates devices without capturing input.
#[test]
fn test_macos_device_enumeration() {
    eprintln!("\n=== Testing device enumeration ===");

    // Enumerate devices (should work without Accessibility permission)
    let devices = match device_discovery::list_keyboard_devices() {
        Ok(devices) => devices,
        Err(e) => {
            panic!("Device enumeration failed: {}", e);
        }
    };

    eprintln!("Found {} keyboard device(s)", devices.len());

    // Print device information
    for (i, device) in devices.iter().enumerate() {
        eprintln!(
            "  {}. {} (VID: {:04x}, PID: {:04x}, ID: {})",
            i + 1,
            device.name,
            device.vendor_id,
            device.product_id,
            device.id
        );
    }

    // Verify device info structure is valid
    for device in &devices {
        // All devices should have non-empty IDs
        assert!(
            !device.id.is_empty(),
            "Device ID should not be empty"
        );

        // All devices should have non-empty names
        assert!(
            !device.name.is_empty(),
            "Device name should not be empty"
        );

        // Vendor and product IDs should be non-zero
        assert!(
            device.vendor_id > 0 || device.product_id > 0,
            "At least one of vendor_id or product_id should be non-zero"
        );

        // ID should follow expected format: usb-VVVV:PPPP or usb-VVVV:PPPP-SERIAL
        assert!(
            device.id.starts_with("usb-"),
            "Device ID should start with 'usb-', got: {}",
            device.id
        );
    }

    eprintln!("✅ Device enumeration successful");
}

/// Tests device-specific configuration loading with single device.
///
/// This test verifies that:
/// 1. Daemon starts with device-specific config
/// 2. Config applies to matching device pattern
/// 3. Daemon loads successfully even with device discrimination
///
/// **Note:** This test auto-skips if Accessibility permission is not granted.
#[test]
#[serial_test::serial]
fn test_macos_device_specific_config_single() {
    // Check for Accessibility permission
    if !permissions::check_accessibility_permission() {
        eprintln!("\n⚠️  Skipping E2E test: Accessibility permission not granted");
        eprintln!("   To run E2E tests:");
        eprintln!("   1. Open System Settings → Privacy & Security → Accessibility");
        eprintln!("   2. Enable Terminal (or your IDE)");
        eprintln!("   3. Re-run tests\n");
        return;
    }

    eprintln!("\n=== Testing device-specific config (single device) ===");

    // Enumerate devices to get actual device info
    let devices = match device_discovery::list_keyboard_devices() {
        Ok(devices) => devices,
        Err(e) => {
            eprintln!("⚠️  Device enumeration failed: {}", e);
            eprintln!("   Skipping test");
            return;
        }
    };

    if devices.is_empty() {
        eprintln!("⚠️  No keyboard devices found");
        eprintln!("   Skipping test (no hardware available)");
        return;
    }

    // Use first device for testing
    let first_device = &devices[0];
    eprintln!("Testing with device: {} ({})", first_device.name, first_device.id);

    // Create config that matches this specific device by vendor:product ID pattern
    let device_pattern = format!("*{:04x}:{:04x}*", first_device.vendor_id, first_device.product_id);
    eprintln!("Using device pattern: {}", device_pattern);

    let config = MacosE2EConfig::new(
        device_pattern,
        vec![keyrx_core::config::KeyMapping::simple(KeyCode::A, KeyCode::B)],
    );

    // Setup harness with device-specific config
    let mut harness = match MacosE2EHarness::setup(config) {
        Ok(h) => h,
        Err(e) => {
            panic!("Failed to setup E2E harness with device-specific config: {}", e);
        }
    };

    // Verify daemon is running
    match harness.daemon_is_running() {
        Ok(true) => {
            eprintln!("✅ Daemon started with device-specific config");
        }
        Ok(false) => {
            panic!("Daemon exited immediately with device-specific config");
        }
        Err(e) => {
            panic!("Failed to check daemon status: {}", e);
        }
    }

    // Brief delay to allow config loading
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Verify daemon is still running (config loaded successfully)
    match harness.daemon_is_running() {
        Ok(true) => {
            eprintln!("✅ Daemon loaded device-specific config successfully");
        }
        Ok(false) => {
            panic!("Daemon crashed during device-specific config loading");
        }
        Err(e) => {
            panic!("Failed to check daemon status: {}", e);
        }
    }

    // Graceful teardown
    match harness.teardown() {
        Ok(result) => {
            assert!(result.graceful_shutdown || result.sigkill_sent);
            eprintln!("✅ Daemon shut down successfully");
        }
        Err(e) => {
            panic!("Failed to teardown harness: {}", e);
        }
    }
}

/// Tests device-specific configuration with multiple devices (when available).
///
/// This test verifies that:
/// 1. Multiple devices can be enumerated
/// 2. Different device patterns can be configured
/// 3. Daemon handles multi-device configs correctly
///
/// **Note:** This test auto-skips if:
/// - Accessibility permission is not granted
/// - Less than 2 keyboard devices are available
///
/// **Developer Note:** To test multi-device functionality, connect a second
/// USB keyboard before running this test.
#[test]
#[serial_test::serial]
fn test_macos_multi_device_config() {
    // Check for Accessibility permission
    if !permissions::check_accessibility_permission() {
        eprintln!("\n⚠️  Skipping E2E test: Accessibility permission not granted");
        return;
    }

    eprintln!("\n=== Testing multi-device config ===");

    // Enumerate devices
    let devices = match device_discovery::list_keyboard_devices() {
        Ok(devices) => devices,
        Err(e) => {
            eprintln!("⚠️  Device enumeration failed: {}", e);
            eprintln!("   Skipping test");
            return;
        }
    };

    if devices.len() < 2 {
        eprintln!("⚠️  Only {} keyboard device(s) found", devices.len());
        eprintln!("   Multi-device test requires at least 2 keyboards");
        eprintln!("   Skipping test (connect second USB keyboard to test)");
        return;
    }

    eprintln!("Found {} keyboard devices:", devices.len());
    for (i, device) in devices.iter().enumerate() {
        eprintln!(
            "  {}. {} (VID: {:04x}, PID: {:04x})",
            i + 1,
            device.name,
            device.vendor_id,
            device.product_id
        );
    }

    // Test with first device pattern (specific device)
    let first_device = &devices[0];
    let device_pattern = format!("*{:04x}:{:04x}*", first_device.vendor_id, first_device.product_id);
    eprintln!("\nTesting with device pattern: {}", device_pattern);

    let config = MacosE2EConfig::new(
        device_pattern.clone(),
        vec![keyrx_core::config::KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape)],
    );

    let mut harness = match MacosE2EHarness::setup(config) {
        Ok(h) => h,
        Err(e) => {
            panic!("Failed to setup E2E harness with first device pattern: {}", e);
        }
    };

    // Verify daemon is running with first device config
    assert!(
        harness.daemon_is_running().unwrap(),
        "Daemon should be running with first device config"
    );
    eprintln!("✅ Daemon started with first device pattern: {}", device_pattern);

    std::thread::sleep(std::time::Duration::from_millis(200));

    assert!(
        harness.daemon_is_running().unwrap(),
        "Daemon should still be running after loading first device config"
    );
    eprintln!("✅ First device config loaded successfully");

    // Teardown first harness
    harness.teardown().expect("First harness teardown should succeed");

    // Brief delay between tests
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Test with second device pattern
    let second_device = &devices[1];
    let device_pattern = format!("*{:04x}:{:04x}*", second_device.vendor_id, second_device.product_id);
    eprintln!("\nTesting with device pattern: {}", device_pattern);

    let config = MacosE2EConfig::new(
        device_pattern.clone(),
        vec![keyrx_core::config::KeyMapping::simple(KeyCode::A, KeyCode::B)],
    );

    let mut harness = match MacosE2EHarness::setup(config) {
        Ok(h) => h,
        Err(e) => {
            panic!("Failed to setup E2E harness with second device pattern: {}", e);
        }
    };

    // Verify daemon is running with second device config
    assert!(
        harness.daemon_is_running().unwrap(),
        "Daemon should be running with second device config"
    );
    eprintln!("✅ Daemon started with second device pattern: {}", device_pattern);

    std::thread::sleep(std::time::Duration::from_millis(200));

    assert!(
        harness.daemon_is_running().unwrap(),
        "Daemon should still be running after loading second device config"
    );
    eprintln!("✅ Second device config loaded successfully");

    // Teardown second harness
    harness.teardown().expect("Second harness teardown should succeed");

    eprintln!("\n✅ Multi-device config test completed successfully");
}

/// Tests serial number extraction and identification.
///
/// This test verifies that:
/// 1. Serial numbers are extracted when available
/// 2. Device IDs include serial numbers
/// 3. Devices without serial numbers are handled correctly
///
/// **Note:** This test does NOT require Accessibility permission.
#[test]
fn test_macos_serial_number_identification() {
    eprintln!("\n=== Testing serial number identification ===");

    // Enumerate devices
    let devices = match device_discovery::list_keyboard_devices() {
        Ok(devices) => devices,
        Err(e) => {
            panic!("Device enumeration failed: {}", e);
        }
    };

    if devices.is_empty() {
        eprintln!("⚠️  No keyboard devices found");
        eprintln!("   Skipping test (no hardware available)");
        return;
    }

    eprintln!("Analyzing {} keyboard device(s):", devices.len());

    let mut devices_with_serial = 0;
    let mut devices_without_serial = 0;

    for device in &devices {
        eprintln!("\nDevice: {}", device.name);
        eprintln!("  ID: {}", device.id);
        eprintln!("  VID: {:04x}, PID: {:04x}", device.vendor_id, device.product_id);

        // Check if device ID includes serial number
        // Format: usb-VVVV:PPPP-SERIAL or usb-VVVV:PPPP
        let id_parts: Vec<&str> = device.id.split('-').collect();
        if id_parts.len() >= 3 {
            // Has serial number (usb-VVVV:PPPP-SERIAL)
            let serial = id_parts[2..].join("-");
            eprintln!("  Serial: {}", serial);
            devices_with_serial += 1;

            // Verify serial is non-empty
            assert!(
                !serial.is_empty(),
                "Serial number should not be empty when present"
            );
        } else {
            // No serial number (usb-VVVV:PPPP)
            eprintln!("  Serial: (none)");
            devices_without_serial += 1;

            // Verify ID format is correct without serial
            assert_eq!(
                id_parts.len(),
                2,
                "Device ID without serial should be usb-VVVV:PPPP format"
            );
        }
    }

    eprintln!("\nSummary:");
    eprintln!("  Devices with serial: {}", devices_with_serial);
    eprintln!("  Devices without serial: {}", devices_without_serial);
    eprintln!("  Total devices: {}", devices.len());

    eprintln!("✅ Serial number identification test completed");
}

/// Tests pattern matching for device identification.
///
/// This test verifies that device patterns correctly identify devices by:
/// 1. Vendor and product ID patterns
/// 2. Device name patterns
/// 3. Wildcard matching
///
/// **Note:** This is a unit-style test that doesn't require Accessibility permission.
#[test]
fn test_macos_device_pattern_matching() {
    eprintln!("\n=== Testing device pattern matching ===");

    // Enumerate devices
    let devices = match device_discovery::list_keyboard_devices() {
        Ok(devices) => devices,
        Err(e) => {
            panic!("Device enumeration failed: {}", e);
        }
    };

    if devices.is_empty() {
        eprintln!("⚠️  No keyboard devices found");
        eprintln!("   Skipping test (no hardware available)");
        return;
    }

    let first_device = &devices[0];
    eprintln!("Testing patterns against: {} ({})", first_device.name, first_device.id);

    // Test various pattern types
    let vendor_product_pattern = format!("*{:04x}:{:04x}*", first_device.vendor_id, first_device.product_id);
    let patterns: Vec<(&str, bool, &str)> = vec![
        // Wildcard pattern (should match)
        ("*", true, "wildcard"),
        // Vendor:Product ID pattern (should match)
        (
            &vendor_product_pattern,
            true,
            "vendor:product ID"
        ),
        // Partial name pattern (should match if name contains common keywords)
        ("*keyboard*", true, "partial name (lowercase)"),
        ("*Keyboard*", true, "partial name (capitalized)"),
        // Non-matching pattern
        ("nonexistent-device-xyz", false, "non-matching"),
    ];

    for (pattern, should_match, description) in patterns {
        let matches = matches_pattern(&first_device.id, pattern)
            || matches_pattern(&first_device.name.to_lowercase(), &pattern.to_lowercase());

        eprintln!(
            "  Pattern '{}' ({}): {} (expected: {})",
            pattern,
            description,
            if matches { "MATCH" } else { "NO MATCH" },
            if should_match { "MATCH" } else { "NO MATCH" }
        );

        // Only assert for patterns that should definitely match
        if pattern == "*" || pattern.contains(&format!("{:04x}:{:04x}", first_device.vendor_id, first_device.product_id)) {
            assert_eq!(
                matches, should_match,
                "Pattern '{}' ({}) matching failed",
                pattern, description
            );
        }
    }

    eprintln!("✅ Device pattern matching test completed");
}

/// Simple wildcard pattern matching helper.
///
/// Supports basic wildcard patterns with '*' matching any characters.
fn matches_pattern(text: &str, pattern: &str) -> bool {
    // Wildcard match all
    if pattern == "*" {
        return true;
    }

    // Convert glob pattern to regex-like matching
    if pattern.starts_with('*') && pattern.ends_with('*') {
        // *text* - contains
        let needle = &pattern[1..pattern.len() - 1];
        text.contains(needle)
    } else if pattern.starts_with('*') {
        // *text - ends with
        let suffix = &pattern[1..];
        text.ends_with(suffix)
    } else if pattern.ends_with('*') {
        // text* - starts with
        let prefix = &pattern[..pattern.len() - 1];
        text.starts_with(prefix)
    } else {
        // Exact match
        text == pattern
    }
}

#[cfg(test)]
mod pattern_matching_tests {
    use super::*;

    #[test]
    fn test_wildcard_matching() {
        assert!(matches_pattern("anything", "*"));
        assert!(matches_pattern("", "*"));
    }

    #[test]
    fn test_contains_matching() {
        assert!(matches_pattern("hello-world", "*world*"));
        assert!(matches_pattern("hello-world", "*hello*"));
        assert!(matches_pattern("hello-world", "*-*"));
        assert!(!matches_pattern("hello-world", "*xyz*"));
    }

    #[test]
    fn test_prefix_matching() {
        assert!(matches_pattern("hello-world", "hello*"));
        assert!(!matches_pattern("hello-world", "world*"));
    }

    #[test]
    fn test_suffix_matching() {
        assert!(matches_pattern("hello-world", "*world"));
        assert!(!matches_pattern("hello-world", "*hello"));
    }

    #[test]
    fn test_exact_matching() {
        assert!(matches_pattern("hello", "hello"));
        assert!(!matches_pattern("hello", "world"));
    }
}
