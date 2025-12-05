//! Platform-agnostic device hotplug watcher interface.
//!
//! This module defines a lightweight contract for platform-specific device
//! watchers (e.g., inotify on Linux, WM_DEVICECHANGE on Windows). Implementors
//! should emit `DeviceEvent` updates and maintain an internal view of current
//! devices plus their `DeviceState`.

use crate::drivers::common::error::DriverError;
use crate::drivers::DeviceInfo;
use crossbeam_channel::{Receiver, Sender};
use std::fmt;
use std::io;
use std::time::Instant;

/// Sender type for device events.
pub type DeviceEventSender = Sender<DeviceEvent>;
/// Receiver type for device events.
pub type DeviceEventReceiver = Receiver<DeviceEvent>;
/// Result alias for watcher operations.
pub type WatcherResult<T> = Result<T, DeviceWatchError>;

/// Recoverable error emitted by a device watcher.
#[derive(Debug, Clone)]
pub struct DeviceWatchError {
    message: String,
    retryable: bool,
}

impl DeviceWatchError {
    /// Create a new error with explicit retryability.
    pub fn new(message: impl Into<String>, retryable: bool) -> Self {
        Self {
            message: message.into(),
            retryable,
        }
    }

    /// Human-readable description of the failure.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Whether the caller should attempt to retry the failed operation.
    pub fn retryable(&self) -> bool {
        self.retryable
    }
}

impl fmt::Display for DeviceWatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DeviceWatchError {}

impl From<DriverError> for DeviceWatchError {
    fn from(err: DriverError) -> Self {
        let retryable = err.is_retryable();
        Self {
            message: err.to_string(),
            retryable,
        }
    }
}

impl From<io::Error> for DeviceWatchError {
    fn from(err: io::Error) -> Self {
        let retryable = matches!(
            err.kind(),
            io::ErrorKind::Interrupted
                | io::ErrorKind::WouldBlock
                | io::ErrorKind::TimedOut
                | io::ErrorKind::ConnectionAborted
                | io::ErrorKind::ConnectionReset
        );
        Self {
            message: err.to_string(),
            retryable,
        }
    }
}

/// State of a tracked device.
#[derive(Debug, Clone)]
pub enum DeviceState {
    /// Device is connected and actively monitored.
    Active,
    /// Device is temporarily paused (e.g., unplugged) since the given instant.
    Paused { since: Instant },
    /// Device failed and will require recovery.
    Failed { error: DeviceWatchError },
}

impl DeviceState {
    /// True when the device is in a failed state.
    pub fn is_failed(&self) -> bool {
        matches!(self, DeviceState::Failed { .. })
    }
}

/// Event emitted by a watcher when device topology changes.
#[derive(Debug, Clone)]
pub enum DeviceEvent {
    /// New device appeared and should be registered.
    Connected(DeviceInfo),
    /// Device disconnected; callers can tear down associated resources.
    Disconnected(DeviceInfo),
    /// Non-fatal watcher error (e.g., transient IO failure).
    Error {
        /// The affected device, if known.
        device: Option<DeviceInfo>,
        /// Contextual error information.
        error: DeviceWatchError,
    },
}

/// Platform-agnostic contract for device watchers.
pub trait DeviceWatcher: Send + Sync {
    /// Start monitoring for device connect/disconnect events.
    fn start(&self) -> WatcherResult<()>;

    /// Stop monitoring and release platform resources.
    fn stop(&self);

    /// Subscribe to real-time device events.
    ///
    /// Receivers are cheap to clone; implementations should return a handle
    /// tied to their internal broadcast channel.
    fn subscribe(&self) -> DeviceEventReceiver;

    /// Snapshot current devices and their states.
    fn snapshot(&self) -> Vec<(DeviceInfo, DeviceState)>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::common::error::DriverError;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn driver_error_conversion_preserves_retryable_flag() {
        let retryable = DriverError::Temporary {
            message: "temporary".to_string(),
            retry_after: Duration::from_millis(10),
        };
        let permanent = DriverError::DeviceNotFound {
            path: PathBuf::from("/dev/input/event0"),
        };

        let retryable_err: DeviceWatchError = retryable.into();
        let permanent_err: DeviceWatchError = permanent.into();

        assert!(retryable_err.retryable());
        assert!(!permanent_err.retryable());
    }

    #[test]
    fn device_state_clone_keeps_error_message() {
        let original = DeviceState::Failed {
            error: DeviceWatchError::new("channel closed", false),
        };
        let cloned = original.clone();

        match cloned {
            DeviceState::Failed { error } => {
                assert_eq!(error.message(), "channel closed");
                assert!(!error.retryable());
            }
            other => panic!("expected failed state, got {other:?}"),
        }
    }
}
