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
//! Integration tests for driver error recovery mechanisms.
//!
//! Tests the automatic retry logic with exponential backoff and error handling paths.

use keyrx_core::drivers::common::error::DriverError;
use keyrx_core::drivers::common::recovery::{
    retry_with_backoff, retry_with_backoff_sync, RetryConfig,
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_retry_recovers_from_temporary_errors() {
    let config = RetryConfig {
        max_retries: 5,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        backoff_multiplier: 2.0,
    };

    let attempt_count = Arc::new(AtomicU32::new(0));
    let attempt_count_clone = attempt_count.clone();

    let result = retry_with_backoff(config, "test_operation", move || {
        let count_clone = attempt_count_clone.clone();
        async move {
            let count = count_clone.fetch_add(1, Ordering::SeqCst);
            if count < 3 {
                // Fail first 3 attempts with retryable error
                Err(DriverError::Temporary {
                    message: "Resource temporarily unavailable".to_string(),
                    retry_after: Duration::from_millis(5),
                })
            } else {
                // Succeed on 4th attempt
                Ok(42)
            }
        }
    })
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(attempt_count.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn test_retry_respects_max_retries() {
    let config = RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        backoff_multiplier: 2.0,
    };

    let attempt_count = Arc::new(AtomicU32::new(0));
    let attempt_count_clone = attempt_count.clone();

    let result = retry_with_backoff(config, "test_operation", move || {
        let count_clone = attempt_count_clone.clone();
        async move {
            count_clone.fetch_add(1, Ordering::SeqCst);
            // Always fail with retryable error
            Err::<(), _>(DriverError::DeviceDisconnected {
                device: "test_device".to_string(),
            })
        }
    })
    .await;

    assert!(result.is_err());
    // Should try initial + max_retries times
    assert_eq!(attempt_count.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn test_retry_fails_immediately_on_non_retryable_error() {
    let config = RetryConfig::default();
    let attempt_count = Arc::new(AtomicU32::new(0));
    let attempt_count_clone = attempt_count.clone();

    let result = retry_with_backoff(config, "test_operation", move || {
        let count_clone = attempt_count_clone.clone();
        async move {
            count_clone.fetch_add(1, Ordering::SeqCst);
            // Non-retryable error should fail immediately
            Err::<(), _>(DriverError::PermissionDenied {
                resource: "/dev/input/event0".to_string(),
                hint: "Add user to input group".to_string(),
            })
        }
    })
    .await;

    assert!(result.is_err());
    // Should only try once (no retries)
    assert_eq!(attempt_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_retry_uses_exponential_backoff() {
    let config = RetryConfig {
        max_retries: 5,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
    };

    let start = std::time::Instant::now();
    let attempt_count = Arc::new(AtomicU32::new(0));
    let attempt_count_clone = attempt_count.clone();

    let _ = retry_with_backoff(config, "test_operation", move || {
        let count_clone = attempt_count_clone.clone();
        async move {
            let count = count_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err::<(), _>(DriverError::Temporary {
                    message: "busy".to_string(),
                    retry_after: Duration::from_millis(10),
                })
            } else {
                Ok(())
            }
        }
    })
    .await;

    let elapsed = start.elapsed();
    // Should have some delay (at least initial_delay * 2 for 2 retries)
    assert!(elapsed >= Duration::from_millis(20));
}

#[test]
fn test_sync_retry_recovers_from_temporary_errors() {
    let config = RetryConfig {
        max_retries: 5,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        backoff_multiplier: 2.0,
    };

    let attempt_count = Arc::new(AtomicU32::new(0));
    let attempt_count_clone = attempt_count.clone();

    let result = retry_with_backoff_sync(config, "test_operation", move || {
        let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
        if count < 2 {
            Err(DriverError::GrabFailed {
                reason: "Device busy".to_string(),
            })
        } else {
            Ok(42)
        }
    });

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
}

#[test]
fn test_sync_retry_fails_immediately_on_non_retryable_error() {
    let config = RetryConfig::default();
    let attempt_count = Arc::new(AtomicU32::new(0));
    let attempt_count_clone = attempt_count.clone();

    let result = retry_with_backoff_sync(config, "test_operation", move || {
        attempt_count_clone.fetch_add(1, Ordering::SeqCst);
        Err::<(), _>(DriverError::DeviceNotFound {
            path: "/dev/input/event99".into(),
        })
    });

    assert!(result.is_err());
    assert_eq!(attempt_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_driver_error_is_retryable_classification() {
    // Retryable errors
    assert!(DriverError::Temporary {
        message: "test".to_string(),
        retry_after: Duration::from_millis(100),
    }
    .is_retryable());

    assert!(DriverError::DeviceDisconnected {
        device: "keyboard".to_string(),
    }
    .is_retryable());

    assert!(DriverError::Timeout {
        duration: Duration::from_secs(5),
    }
    .is_retryable());

    assert!(DriverError::GrabFailed {
        reason: "busy".to_string(),
    }
    .is_retryable());

    assert!(DriverError::InjectionFailed {
        reason: "busy".to_string(),
    }
    .is_retryable());

    // Non-retryable errors
    assert!(!DriverError::PermissionDenied {
        resource: "/dev/input/event0".to_string(),
        hint: "hint".to_string(),
    }
    .is_retryable());

    assert!(!DriverError::DeviceNotFound {
        path: "/dev/input/event0".into(),
    }
    .is_retryable());

    assert!(!DriverError::InvalidEvent {
        details: "test".to_string(),
    }
    .is_retryable());
}

#[test]
fn test_driver_error_suggested_actions() {
    let err = DriverError::PermissionDenied {
        resource: "/dev/input/event0".to_string(),
        hint: "Add to input group".to_string(),
    };
    let action = err.suggested_action();
    assert!(action.contains("permission") || action.contains("group"));

    let err = DriverError::DeviceNotFound {
        path: "/dev/input/event0".into(),
    };
    let action = err.suggested_action();
    assert!(action.contains("device") || action.contains("connected"));

    let err = DriverError::Temporary {
        message: "busy".to_string(),
        retry_after: Duration::from_millis(100),
    };
    let action = err.suggested_action();
    assert!(action.contains("retry") || action.contains("temporary"));
}

#[test]
fn test_driver_error_retry_delays() {
    let err = DriverError::Temporary {
        message: "test".to_string(),
        retry_after: Duration::from_millis(250),
    };
    assert_eq!(err.retry_delay(), Some(Duration::from_millis(250)));

    let err = DriverError::DeviceDisconnected {
        device: "keyboard".to_string(),
    };
    assert!(err.retry_delay().is_some());

    let err = DriverError::DeviceNotFound {
        path: "/dev/input/event0".into(),
    };
    assert_eq!(err.retry_delay(), None);
}

#[test]
fn test_retry_config_presets() {
    let default = RetryConfig::default();
    assert_eq!(default.max_retries, 5);
    assert_eq!(default.initial_delay, Duration::from_millis(100));

    let aggressive = RetryConfig::aggressive();
    assert!(aggressive.max_retries > default.max_retries);
    assert!(aggressive.initial_delay < default.initial_delay);

    let conservative = RetryConfig::conservative();
    assert!(conservative.max_retries < default.max_retries);
    assert!(conservative.initial_delay > default.initial_delay);
}

#[test]
fn test_retry_config_exponential_delay_calculation() {
    let config = RetryConfig {
        max_retries: 10,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        backoff_multiplier: 2.0,
    };

    let delay0 = config.delay_for_attempt(0);
    let delay1 = config.delay_for_attempt(1);
    let delay2 = config.delay_for_attempt(2);

    assert_eq!(delay0, Duration::from_millis(100));
    assert_eq!(delay1, Duration::from_millis(200));
    assert_eq!(delay2, Duration::from_millis(400));

    // Test that delay caps at max_delay
    let large_delay = config.delay_for_attempt(100);
    assert!(large_delay <= config.max_delay);
}

#[cfg(target_os = "linux")]
#[test]
fn test_linux_permission_denied_error_message() {
    let err = DriverError::linux_permission_denied("/dev/input/event0");
    match err {
        DriverError::PermissionDenied { resource, hint } => {
            assert_eq!(resource, "/dev/input/event0");
            assert!(hint.contains("input"));
            assert!(hint.contains("usermod"));
        }
        _ => panic!("Expected PermissionDenied variant"),
    }
}

#[cfg(target_os = "windows")]
#[test]
fn test_windows_permission_denied_error_message() {
    let err = DriverError::windows_permission_denied("keyboard hook");
    match err {
        DriverError::PermissionDenied { resource, hint } => {
            assert_eq!(resource, "keyboard hook");
            assert!(hint.contains("administrator"));
        }
        _ => panic!("Expected PermissionDenied variant"),
    }
}
