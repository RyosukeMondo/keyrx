//! Critical error type with fallback actions for KeyRx critical paths.
//!
//! This module provides `CriticalError` for use in critical execution paths
//! where panic is unacceptable. Each error variant includes a fallback action
//! and recoverability information to enable graceful degradation.
//!
//! # Design Philosophy
//!
//! - Every error must have a fallback action
//! - Errors are classified as recoverable or non-recoverable
//! - All errors are serializable for logging and FFI boundaries
//! - No error should ever cause a panic in critical paths
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::errors::critical::{CriticalError, FallbackAction};
//!
//! match do_something() {
//!     Err(err) if err.is_recoverable() => {
//!         let action = err.fallback_action();
//!         execute_fallback(action);
//!     }
//!     Err(err) => {
//!         // Non-recoverable, activate emergency mode
//!         activate_emergency_mode();
//!     }
//!     Ok(result) => handle_result(result),
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Fallback actions that can be taken when a critical error occurs.
///
/// These actions define how the system should respond to failures in
/// critical paths without panicking or crashing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FallbackAction {
    /// Switch to passthrough mode (no key remapping).
    ActivatePassthrough,

    /// Retry the operation after a delay.
    RetryAfter {
        /// Duration to wait before retrying.
        delay: Duration,
    },

    /// Use a default or last-known-good value.
    UseDefault,

    /// Skip the operation and continue.
    Skip,

    /// Stop processing and enter safe mode.
    EnterSafeMode,

    /// Disconnect the device and continue with others.
    DisconnectDevice {
        /// Name or path of the device to disconnect.
        device: String,
    },

    /// Reset to initial state and retry.
    ResetAndRetry,

    /// Continue with degraded functionality.
    ContinueDegraded {
        /// Description of what functionality is degraded.
        message: String,
    },
}

impl fmt::Display for FallbackAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FallbackAction::ActivatePassthrough => write!(f, "Activate passthrough mode"),
            FallbackAction::RetryAfter { delay } => {
                write!(f, "Retry after {:?}", delay)
            }
            FallbackAction::UseDefault => write!(f, "Use default value"),
            FallbackAction::Skip => write!(f, "Skip operation"),
            FallbackAction::EnterSafeMode => write!(f, "Enter safe mode"),
            FallbackAction::DisconnectDevice { device } => {
                write!(f, "Disconnect device: {}", device)
            }
            FallbackAction::ResetAndRetry => write!(f, "Reset and retry"),
            FallbackAction::ContinueDegraded { message } => {
                write!(f, "Continue with degraded functionality: {}", message)
            }
        }
    }
}

/// Critical errors that occur in KeyRx critical paths.
///
/// Each variant includes context and a defined fallback action.
/// All variants are serializable for logging and FFI boundaries.
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CriticalError {
    /// Driver failed to initialize.
    #[error("Driver initialization failed: {reason}")]
    DriverInitFailed {
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },

    /// Hook installation failed (Windows).
    #[error("Hook installation failed with code {code:#x}")]
    HookInstallFailed {
        code: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },

    /// Device grab failed (Linux).
    #[error("Failed to grab device: {device}")]
    DeviceGrabFailed {
        device: String,
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },

    /// Event injection failed.
    #[error("Event injection failed: {reason}")]
    InjectionFailed {
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },

    /// State machine error.
    #[error("State machine error: {details}")]
    StateMachineError {
        details: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },

    /// Engine processing failed.
    #[error("Engine processing failed: {reason}")]
    ProcessingFailed {
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },

    /// Device disconnected during operation.
    #[error("Device disconnected: {device}")]
    DeviceDisconnected {
        device: String,
    },

    /// Configuration load failed.
    #[error("Configuration load failed: {path}")]
    ConfigLoadFailed {
        path: PathBuf,
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },

    /// FFI boundary error.
    #[error("FFI error: {operation}")]
    FfiBoundaryError {
        operation: String,
        details: String,
    },

    /// Callback panic was caught.
    #[error("Callback panicked: {panic_message}")]
    CallbackPanic {
        panic_message: String,
        backtrace: Option<String>,
    },

    /// Thread communication failed.
    #[error("Channel error: {details}")]
    ChannelError {
        details: String,
    },

    /// Circuit breaker opened due to repeated failures.
    #[error("Circuit breaker opened after {failure_count} failures")]
    CircuitBreakerOpen {
        failure_count: usize,
        last_error: String,
    },

    /// Virtual device creation/access failed.
    #[error("Virtual device error: {message}")]
    VirtualDeviceError {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },

    /// Discovery failed to enumerate devices.
    #[error("Device discovery failed: {reason}")]
    DiscoveryFailed {
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },

    /// Invalid state transition attempted.
    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition {
        from: String,
        to: String,
    },

    /// Platform-specific I/O error.
    #[error("Platform I/O error: {message}")]
    PlatformIo {
        message: String,
        kind: String,
    },
}

impl CriticalError {
    /// Returns whether this error is recoverable.
    ///
    /// Recoverable errors can be handled with a fallback action without
    /// requiring system shutdown or user intervention.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if error.is_recoverable() {
    ///     let action = error.fallback_action();
    ///     execute_fallback(action);
    /// } else {
    ///     activate_emergency_mode();
    /// }
    /// ```
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors that can be handled with fallbacks
            CriticalError::InjectionFailed { .. } => true,
            CriticalError::DeviceDisconnected { .. } => true,
            CriticalError::ConfigLoadFailed { .. } => true,
            CriticalError::DeviceGrabFailed { .. } => true,
            CriticalError::ProcessingFailed { .. } => true,
            CriticalError::ChannelError { .. } => true,
            CriticalError::CircuitBreakerOpen { .. } => true,
            CriticalError::VirtualDeviceError { .. } => true,
            CriticalError::DiscoveryFailed { .. } => true,
            CriticalError::PlatformIo { .. } => true,

            // Non-recoverable errors requiring safe mode or shutdown
            CriticalError::DriverInitFailed { .. } => false,
            CriticalError::HookInstallFailed { .. } => false,
            CriticalError::StateMachineError { .. } => false,
            CriticalError::FfiBoundaryError { .. } => false,
            CriticalError::CallbackPanic { .. } => false,
            CriticalError::InvalidStateTransition { .. } => false,
        }
    }

    /// Returns the fallback action for this error.
    ///
    /// This defines how the system should respond to the error without
    /// panicking or crashing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// match error.fallback_action() {
    ///     FallbackAction::ActivatePassthrough => {
    ///         engine.switch_to_passthrough();
    ///     }
    ///     FallbackAction::RetryAfter { delay } => {
    ///         tokio::time::sleep(delay).await;
    ///         retry_operation();
    ///     }
    ///     _ => {}
    /// }
    /// ```
    pub fn fallback_action(&self) -> FallbackAction {
        match self {
            CriticalError::DriverInitFailed { .. } => FallbackAction::EnterSafeMode,

            CriticalError::HookInstallFailed { .. } => FallbackAction::EnterSafeMode,

            CriticalError::DeviceGrabFailed { device, .. } => {
                FallbackAction::DisconnectDevice {
                    device: device.clone(),
                }
            }

            CriticalError::InjectionFailed { .. } => FallbackAction::RetryAfter {
                delay: Duration::from_millis(50),
            },

            CriticalError::StateMachineError { .. } => FallbackAction::ResetAndRetry,

            CriticalError::ProcessingFailed { reason, .. } => FallbackAction::ContinueDegraded {
                message: format!("Processing degraded: {}", reason),
            },

            CriticalError::DeviceDisconnected { device } => FallbackAction::DisconnectDevice {
                device: device.clone(),
            },

            CriticalError::ConfigLoadFailed { .. } => FallbackAction::UseDefault,

            CriticalError::FfiBoundaryError { .. } => FallbackAction::EnterSafeMode,

            CriticalError::CallbackPanic { .. } => FallbackAction::ActivatePassthrough,

            CriticalError::ChannelError { .. } => FallbackAction::RetryAfter {
                delay: Duration::from_millis(100),
            },

            CriticalError::CircuitBreakerOpen { .. } => FallbackAction::ActivatePassthrough,

            CriticalError::VirtualDeviceError { .. } => FallbackAction::ActivatePassthrough,

            CriticalError::DiscoveryFailed { .. } => FallbackAction::ContinueDegraded {
                message: "Running with limited device support".to_string(),
            },

            CriticalError::InvalidStateTransition { .. } => FallbackAction::ResetAndRetry,

            CriticalError::PlatformIo { .. } => FallbackAction::RetryAfter {
                delay: Duration::from_millis(100),
            },
        }
    }

    /// Creates a critical error from a driver error.
    ///
    /// This is a convenience method for converting driver errors to critical errors
    /// in critical paths.
    pub fn from_driver_error(err: crate::drivers::common::error::DriverError) -> Self {
        use crate::drivers::common::error::DriverError;

        match err {
            DriverError::DeviceNotFound { path } => CriticalError::DiscoveryFailed {
                reason: format!("Device not found: {}", path.display()),
                cause: None,
            },

            DriverError::PermissionDenied { resource, hint } => {
                CriticalError::DriverInitFailed {
                    reason: format!("Permission denied: {} ({})", resource, hint),
                    cause: None,
                }
            }

            DriverError::DeviceDisconnected { device } => {
                CriticalError::DeviceDisconnected { device }
            }

            DriverError::HookFailed { code } => CriticalError::HookInstallFailed {
                code,
                cause: None,
            },

            DriverError::GrabFailed { reason } => CriticalError::DeviceGrabFailed {
                device: "unknown".to_string(),
                reason,
                cause: None,
            },

            DriverError::VirtualDeviceError { message } => CriticalError::VirtualDeviceError {
                message,
                cause: None,
            },

            DriverError::InjectionFailed { reason } => CriticalError::InjectionFailed {
                reason,
                cause: None,
            },

            DriverError::InvalidEvent { details } => CriticalError::ProcessingFailed {
                reason: format!("Invalid event: {}", details),
                cause: None,
            },

            DriverError::CallbackPanic { panic_message } => CriticalError::CallbackPanic {
                panic_message,
                backtrace: None,
            },

            DriverError::ChannelError { details } => CriticalError::ChannelError { details },

            DriverError::InitializationFailed { reason } => CriticalError::DriverInitFailed {
                reason,
                cause: None,
            },

            DriverError::Platform(io_err) => CriticalError::PlatformIo {
                message: io_err.to_string(),
                kind: format!("{:?}", io_err.kind()),
            },

            _ => CriticalError::ProcessingFailed {
                reason: err.to_string(),
                cause: None,
            },
        }
    }

    /// Creates a critical error from an I/O error.
    pub fn from_io_error(err: io::Error, context: impl Into<String>) -> Self {
        CriticalError::PlatformIo {
            message: format!("{}: {}", context.into(), err),
            kind: format!("{:?}", err.kind()),
        }
    }

    /// Adds a source error to this critical error.
    pub fn with_source(mut self, source: impl fmt::Display) -> Self {
        match &mut self {
            CriticalError::DriverInitFailed { cause: s, .. }
            | CriticalError::HookInstallFailed { cause: s, .. }
            | CriticalError::DeviceGrabFailed { cause: s, .. }
            | CriticalError::InjectionFailed { cause: s, .. }
            | CriticalError::StateMachineError { cause: s, .. }
            | CriticalError::ProcessingFailed { cause: s, .. }
            | CriticalError::ConfigLoadFailed { cause: s, .. }
            | CriticalError::VirtualDeviceError { cause: s, .. }
            | CriticalError::DiscoveryFailed { cause: s, .. } => {
                *s = Some(source.to_string());
            }
            _ => {}
        }
        self
    }
}

/// Convenience type alias for Results using CriticalError.
pub type CriticalResult<T> = Result<T, CriticalError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recoverable_errors_are_identified() {
        let err = CriticalError::InjectionFailed {
            reason: "test".to_string(),
            cause: None,
        };
        assert!(err.is_recoverable());

        let err = CriticalError::DeviceDisconnected {
            device: "keyboard".to_string(),
        };
        assert!(err.is_recoverable());
    }

    #[test]
    fn non_recoverable_errors_are_identified() {
        let err = CriticalError::DriverInitFailed {
            reason: "test".to_string(),
            cause: None,
        };
        assert!(!err.is_recoverable());

        let err = CriticalError::CallbackPanic {
            panic_message: "test".to_string(),
            backtrace: None,
        };
        assert!(!err.is_recoverable());
    }

    #[test]
    fn fallback_actions_are_appropriate() {
        let err = CriticalError::DeviceDisconnected {
            device: "keyboard".to_string(),
        };
        matches!(
            err.fallback_action(),
            FallbackAction::DisconnectDevice { .. }
        );

        let err = CriticalError::CallbackPanic {
            panic_message: "test".to_string(),
            backtrace: None,
        };
        assert_eq!(err.fallback_action(), FallbackAction::ActivatePassthrough);

        let err = CriticalError::ConfigLoadFailed {
            path: PathBuf::from("/etc/keyrx.toml"),
            cause: None,
        };
        assert_eq!(err.fallback_action(), FallbackAction::UseDefault);
    }

    #[test]
    fn fallback_action_display() {
        let action = FallbackAction::ActivatePassthrough;
        assert_eq!(action.to_string(), "Activate passthrough mode");

        let action = FallbackAction::RetryAfter {
            delay: Duration::from_millis(100),
        };
        assert!(action.to_string().contains("Retry after"));
    }

    #[test]
    fn error_serialization() {
        let err = CriticalError::InjectionFailed {
            reason: "test".to_string(),
            cause: None,
        };

        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("InjectionFailed"));
        assert!(json.contains("test"));

        let deserialized: CriticalError = serde_json::from_str(&json).unwrap();
        matches!(deserialized, CriticalError::InjectionFailed { .. });
    }

    #[test]
    fn circuit_breaker_error() {
        let err = CriticalError::CircuitBreakerOpen {
            failure_count: 5,
            last_error: "injection failed".to_string(),
        };

        assert!(err.is_recoverable());
        assert_eq!(err.fallback_action(), FallbackAction::ActivatePassthrough);
        assert!(err.to_string().contains("5 failures"));
    }

    #[test]
    fn with_source_adds_context() {
        let err = CriticalError::ProcessingFailed {
            reason: "test".to_string(),
            cause: None,
        };

        let err = err.with_source("original error");
        match err {
            CriticalError::ProcessingFailed { cause, .. } => {
                assert_eq!(cause, Some("original error".to_string()));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn from_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = CriticalError::from_io_error(io_err, "reading config");

        matches!(err, CriticalError::PlatformIo { .. });
        assert!(err.to_string().contains("reading config"));
    }
}
