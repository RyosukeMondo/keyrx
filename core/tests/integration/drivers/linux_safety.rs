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
//! Integration tests for Linux driver safety wrappers.
//!
//! Tests device lifecycle management, permission checking, and error handling.
//!
//! Note: Some tests require actual Linux input devices and are only run when
//! appropriate devices are available. Tests focus on error handling paths and
//! safety guarantees that can be verified without hardware.

use keyrx_core::drivers::common::error::DriverError;
use keyrx_core::drivers::linux::safety::permissions;
use std::path::Path;

#[test]
fn test_permission_check_nonexistent_device() {
    let result = permissions::check_device_access(Path::new("/dev/input/event9999"));
    assert!(result.is_err());

    if let Err(e) = result {
        match e {
            DriverError::DeviceNotFound { path } => {
                assert!(path.to_string_lossy().contains("event9999"));
            }
            _ => panic!("Expected DeviceNotFound, got: {:?}", e),
        }
    }
}

#[test]
fn test_uinput_permission_check() {
    // Test that we can check uinput access
    // This may succeed or fail depending on system setup
    let result = permissions::check_uinput_access();

    match result {
        Ok(()) => {
            // System has uinput access configured
        }
        Err(DriverError::PermissionDenied { resource, hint }) => {
            // Expected if not in input group or no udev rules
            assert!(resource.contains("uinput"));
            assert!(hint.contains("input") || hint.contains("udev"));
        }
        Err(DriverError::DeviceNotFound { path }) => {
            // /dev/uinput doesn't exist on this system
            assert!(path.to_string_lossy().contains("uinput"));
        }
        Err(DriverError::VirtualDeviceError { message }) => {
            // This happens when uinput module is not loaded or device node is missing
            assert!(message.contains("uinput") || message.contains("not found"));
        }
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }
}

#[test]
fn test_permission_error_contains_helpful_hints() {
    // Create a permission denied error and verify it has helpful content
    let error = DriverError::linux_permission_denied("/dev/input/event0");

    match error {
        DriverError::PermissionDenied { resource, hint } => {
            assert_eq!(resource, "/dev/input/event0");
            assert!(hint.contains("input"), "Hint should mention input group");
            assert!(
                hint.contains("usermod") || hint.contains("udev"),
                "Hint should mention usermod or udev"
            );
        }
        _ => panic!("Expected PermissionDenied variant"),
    }
}

#[test]
fn test_device_not_found_error_format() {
    let error = DriverError::DeviceNotFound {
        path: "/dev/input/event99".into(),
    };

    let error_string = format!("{}", error);
    assert!(error_string.contains("/dev/input/event99"));
    assert!(error_string.contains("not found"));
}

#[test]
fn test_grab_failed_error_is_retryable() {
    let error = DriverError::GrabFailed {
        reason: "Device busy".to_string(),
    };

    assert!(error.is_retryable(), "GrabFailed should be retryable");

    let delay = error.retry_delay();
    assert!(delay.is_some(), "GrabFailed should suggest a retry delay");
}

#[test]
fn test_device_disconnected_error_is_retryable() {
    let error = DriverError::DeviceDisconnected {
        device: "/dev/input/event3".to_string(),
    };

    assert!(
        error.is_retryable(),
        "DeviceDisconnected should be retryable"
    );

    let delay = error.retry_delay();
    assert!(
        delay.is_some(),
        "DeviceDisconnected should suggest a retry delay"
    );
}

#[test]
fn test_injection_failed_error_is_retryable() {
    let error = DriverError::InjectionFailed {
        reason: "Buffer full".to_string(),
    };

    assert!(error.is_retryable(), "InjectionFailed should be retryable");

    let delay = error.retry_delay();
    assert!(
        delay.is_some(),
        "InjectionFailed should suggest a retry delay"
    );
}

#[test]
fn test_virtual_device_error_format() {
    let error = DriverError::VirtualDeviceError {
        message: "Failed to create uinput device".to_string(),
    };

    let error_string = format!("{}", error);
    assert!(error_string.contains("uinput"));

    let action = error.suggested_action();
    assert!(action.contains("uinput"));
}

#[test]
fn test_ungrab_failed_error_suggested_action() {
    let error = DriverError::UngrabFailed {
        reason: "Device already ungrabbed".to_string(),
    };

    let action = error.suggested_action();
    assert!(action.contains("ungrab") || action.contains("ignore"));
}

#[test]
fn test_platform_io_error_retryability() {
    use std::io;

    // Interrupted IO should be retryable
    let io_err = io::Error::from(io::ErrorKind::Interrupted);
    let error = DriverError::Platform(io_err);
    assert!(error.is_retryable());

    // WouldBlock should be retryable
    let io_err = io::Error::from(io::ErrorKind::WouldBlock);
    let error = DriverError::Platform(io_err);
    assert!(error.is_retryable());

    // TimedOut should be retryable
    let io_err = io::Error::from(io::ErrorKind::TimedOut);
    let error = DriverError::Platform(io_err);
    assert!(error.is_retryable());

    // PermissionDenied should NOT be retryable
    let io_err = io::Error::from(io::ErrorKind::PermissionDenied);
    let error = DriverError::Platform(io_err);
    assert!(!error.is_retryable());

    // NotFound should NOT be retryable
    let io_err = io::Error::from(io::ErrorKind::NotFound);
    let error = DriverError::Platform(io_err);
    assert!(!error.is_retryable());
}

#[test]
fn test_suggested_actions_are_actionable() {
    // Verify that all error types return non-empty, helpful suggestions
    let test_cases = vec![
        (
            DriverError::DeviceNotFound {
                path: "/dev/input/event0".into(),
            },
            vec!["device", "connected", "path"],
        ),
        (
            DriverError::PermissionDenied {
                resource: "/dev/input/event0".to_string(),
                hint: "hint".to_string(),
            },
            vec!["permission", "group"],
        ),
        (
            DriverError::GrabFailed {
                reason: "busy".to_string(),
            },
            vec!["application", "device", "permission"],
        ),
        (
            DriverError::VirtualDeviceError {
                message: "test".to_string(),
            },
            vec!["uinput"],
        ),
        (
            DriverError::InjectionFailed {
                reason: "test".to_string(),
            },
            vec!["retry", "device"],
        ),
    ];

    for (error, keywords) in test_cases {
        let action = error.suggested_action();
        assert!(!action.is_empty(), "Suggested action should not be empty");

        let action_lower = action.to_lowercase();
        let has_keyword = keywords.iter().any(|kw| action_lower.contains(kw));
        assert!(
            has_keyword,
            "Action '{}' should contain one of {:?}",
            action, keywords
        );
    }
}

#[test]
fn test_error_debug_formatting() {
    // Ensure all error types can be formatted for debugging
    let errors = vec![
        DriverError::DeviceNotFound {
            path: "/dev/input/event0".into(),
        },
        DriverError::PermissionDenied {
            resource: "/dev/input/event0".to_string(),
            hint: "Add to input group".to_string(),
        },
        DriverError::DeviceDisconnected {
            device: "keyboard".to_string(),
        },
        DriverError::GrabFailed {
            reason: "busy".to_string(),
        },
        DriverError::UngrabFailed {
            reason: "already ungrabbed".to_string(),
        },
        DriverError::VirtualDeviceError {
            message: "test".to_string(),
        },
        DriverError::InjectionFailed {
            reason: "test".to_string(),
        },
    ];

    for error in errors {
        let debug = format!("{:?}", error);
        assert!(!debug.is_empty());

        let display = format!("{}", error);
        assert!(!display.is_empty());
    }
}

// Conditional tests that run only when we have access to real devices
#[cfg(all(target_os = "linux", feature = "integration_with_devices"))]
mod with_devices {
    use super::*;
    use keyrx_core::drivers::linux::safety::device::SafeDevice;

    #[test]
    fn test_safe_device_grab_ungrab_lifecycle() {
        // Try to find any available input device
        if let Ok(mut device) = find_test_device() {
            // Device should not be grabbed initially
            assert!(!device.is_grabbed());

            // Grab the device
            device.grab().expect("Should be able to grab device");
            assert!(device.is_grabbed());

            // Ungrab the device
            device.ungrab().expect("Should be able to ungrab device");
            assert!(!device.is_grabbed());
        }
    }

    #[test]
    fn test_safe_device_double_grab_is_safe() {
        if let Ok(mut device) = find_test_device() {
            device.grab().expect("First grab should succeed");

            // Second grab should be a no-op
            device.grab().expect("Second grab should be safe");
            assert!(device.is_grabbed());

            device.ungrab().ok();
        }
    }

    #[test]
    fn test_safe_device_auto_ungrab_on_drop() {
        // This test verifies RAII cleanup
        // Create and grab device in a scope
        {
            if let Ok(mut device) = find_test_device() {
                device.grab().expect("Should grab");
                // Device will be automatically ungrabbed on drop
            }
        }
        // If we reach here without hanging, the ungrab happened
    }

    fn find_test_device() -> Result<SafeDevice, DriverError> {
        // Try common device paths
        for i in 0..10 {
            let path = format!("/dev/input/event{}", i);
            if Path::new(&path).exists() {
                if let Ok(device) = SafeDevice::open(&path) {
                    return Ok(device);
                }
            }
        }
        Err(DriverError::DeviceNotFound {
            path: "/dev/input/event*".into(),
        })
    }
}

#[test]
fn test_permission_check_is_consistent() {
    // Multiple checks on the same nonexistent path should give same result
    let path = Path::new("/dev/input/event9999");

    let result1 = permissions::check_device_access(path);
    let result2 = permissions::check_device_access(path);

    assert_eq!(result1.is_ok(), result2.is_ok());
}

#[test]
fn test_error_chain_preserves_information() {
    use std::io;

    // Test that converting from io::Error preserves the error kind
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "test");
    let driver_error = DriverError::Platform(io_error);

    match driver_error {
        DriverError::Platform(e) => {
            assert_eq!(e.kind(), io::ErrorKind::PermissionDenied);
        }
        _ => panic!("Expected Platform variant"),
    }
}
