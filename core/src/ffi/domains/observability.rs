//! FFI exports for observability (logs and metrics).
//!
//! This module provides FFI functions to access structured logs and metrics
//! from the Flutter UI layer. It supports both callback-based real-time
//! updates and polling-based access patterns.

#![allow(unsafe_code)]
#![allow(dead_code)]

use crate::ffi::context::FfiContext;
use crate::ffi::error::FfiError;
use crate::ffi::traits::FfiExportable;
use crate::observability::bridge::GLOBAL_LOG_BRIDGE;
use crate::observability::entry::CLogEntry;
#[cfg(test)]
use crate::observability::metrics_bridge::ThresholdViolation;
use crate::observability::metrics_bridge::{MetricsBridge, MetricsSnapshotFfi, ThresholdCallback};
use ffi_export::ffi_export;
use std::ffi::{c_char, CString};
use std::sync::{Arc, RwLock};

/// Global metrics bridge instance for FFI access.
static METRICS_BRIDGE: RwLock<Option<Arc<MetricsBridge>>> = RwLock::new(None);

/// FFI domain for observability functionality.
pub struct ObservabilityFfi;

impl FfiExportable for ObservabilityFfi {
    const DOMAIN: &'static str = "observability";

    fn init(_ctx: &mut FfiContext) -> Result<(), FfiError> {
        tracing::info!(
            service = "keyrx",
            event = "observability_ffi_init",
            component = "ffi_observability",
            "Observability FFI initialized"
        );
        Ok(())
    }

    fn cleanup(_ctx: &mut FfiContext) {
        tracing::info!(
            service = "keyrx",
            event = "observability_ffi_cleanup",
            component = "ffi_observability",
            "Observability FFI cleaned up"
        );

        // Clear callbacks and bridges
        // For GLOBAL_LOG_BRIDGE, we just clear the callback, but we don't drop the instance
        // since it's a lazy_static used by tracing.
        GLOBAL_LOG_BRIDGE.clear_callback();
        GLOBAL_LOG_BRIDGE.clear();

        if let Ok(mut bridge) = METRICS_BRIDGE.write() {
            if let Some(b) = bridge.as_ref() {
                b.clear_callback();
                b.stop_updates();
            }
            *bridge = None;
        }
    }
}

/// Initialize the log bridge for FFI access.
///
/// This must be called before using any log FFI functions.
/// If a bridge already exists, this function does nothing.
///
/// # Safety
/// This function is thread-safe.
#[no_mangle]
pub extern "C" fn keyrx_log_bridge_init() -> i32 {
    // No-op for global bridge, just return success
    tracing::debug!(
        service = "keyrx",
        event = "log_bridge_init",
        component = "ffi_observability",
        "Log bridge initialized (global)"
    );
    0
}

/// Register a callback for real-time log notifications.
///
/// The callback will be invoked for each log event that occurs.
/// Pass NULL to unregister the callback.
///
/// # Arguments
/// * `callback` - Function pointer to call on log events, or NULL to unregister
///
/// # Returns
/// - 0: Success
/// - -1: Failed to acquire bridge lock
///
/// # Safety
/// The callback function must be valid for the lifetime of the registration.
/// The callback receives a pointer to CLogEntry that is only valid during
/// the callback invocation - it must not be retained.
#[no_mangle]
pub extern "C" fn keyrx_log_set_callback(callback: Option<extern "C" fn(*const CLogEntry)>) -> i32 {
    if let Some(cb) = callback {
        GLOBAL_LOG_BRIDGE.set_callback(cb);
        tracing::debug!(
            service = "keyrx",
            event = "log_callback_set",
            component = "ffi_observability",
            "Log callback registered"
        );
    } else {
        GLOBAL_LOG_BRIDGE.clear_callback();
        tracing::debug!(
            service = "keyrx",
            event = "log_callback_cleared",
            component = "ffi_observability",
            "Log callback unregistered"
        );
    }
    0
}

/// Get the number of buffered log entries.
///
/// # Returns
/// Number of log entries in the buffer, or 0 if bridge is unavailable.
#[no_mangle]
pub extern "C" fn keyrx_log_count() -> usize {
    GLOBAL_LOG_BRIDGE.len()
}

/// Drain buffered log entries into a JSON array string.
///
/// This removes all entries from the buffer and returns them as a
/// JSON-formatted string. The caller must free the returned string
/// using `keyrx_free_string`.
///
/// # Returns
/// Pointer to JSON array string, or NULL on error.
/// The JSON format is: `[{...}, {...}, ...]` where each object is a LogEntry.
///
/// # Safety
/// The returned pointer must be freed with `keyrx_free_string`.
#[no_mangle]
pub extern "C" fn keyrx_log_drain() -> *mut c_char {
    let entries = GLOBAL_LOG_BRIDGE.drain();

    match serde_json::to_string(&entries) {
        Ok(json) => match CString::new(json) {
            Ok(c_string) => {
                tracing::trace!(
                    service = "keyrx",
                    event = "log_drain",
                    component = "ffi_observability",
                    count = entries.len(),
                    "Drained log entries"
                );
                c_string.into_raw()
            }
            Err(e) => {
                tracing::error!(
                    service = "keyrx",
                    event = "log_drain_failed",
                    component = "ffi_observability",
                    error = %e,
                    "Failed to create C string from JSON"
                );
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            tracing::error!(
                service = "keyrx",
                event = "log_drain_failed",
                component = "ffi_observability",
                error = %e,
                "Failed to serialize log entries to JSON"
            );
            std::ptr::null_mut()
        }
    }
}

/// Clear all buffered log entries without returning them.
///
/// # Returns
/// - 0: Success
/// - -1: Bridge unavailable
#[no_mangle]
pub extern "C" fn keyrx_log_clear() -> i32 {
    GLOBAL_LOG_BRIDGE.clear();
    tracing::debug!(
        service = "keyrx",
        event = "log_clear",
        component = "ffi_observability",
        "Log buffer cleared"
    );
    0
}

/// Enable or disable log buffering.
///
/// When disabled, log events are not captured by the bridge.
///
/// # Arguments
/// * `enabled` - 1 to enable, 0 to disable
///
/// # Returns
/// - 0: Success
/// - -1: Bridge unavailable
#[no_mangle]
pub extern "C" fn keyrx_log_set_enabled(enabled: i32) -> i32 {
    let is_enabled = enabled != 0;
    GLOBAL_LOG_BRIDGE.set_enabled(is_enabled);
    tracing::debug!(
        service = "keyrx",
        event = "log_enabled_changed",
        component = "ffi_observability",
        enabled = is_enabled,
        "Log bridge enabled state changed"
    );
    0
}

/// Initialize the metrics bridge with the provided collector.
///
/// # Arguments
/// * `collector` - Pointer to a MetricsCollector implementation
///
/// # Returns
/// - 0: Success
/// - -1: Failed
///
/// # Safety
/// This function is internal and should not be called directly from FFI.
/// The collector pointer must remain valid for the lifetime of the bridge.
#[doc(hidden)]
pub fn init_metrics_bridge(
    collector: Arc<dyn crate::observability::metrics_bridge::MetricsCollector>,
) -> i32 {
    if let Ok(mut bridge) = METRICS_BRIDGE.write() {
        *bridge = Some(Arc::new(MetricsBridge::new(collector)));
        tracing::debug!(
            service = "keyrx",
            event = "metrics_bridge_init",
            component = "ffi_observability",
            "Metrics bridge initialized"
        );
        0
    } else {
        tracing::error!(
            service = "keyrx",
            event = "metrics_bridge_init_failed",
            component = "ffi_observability",
            error = "lock_poisoned",
            "Failed to acquire metrics bridge lock"
        );
        -1
    }
}

/// Get the metrics bridge instance.
fn get_metrics_bridge() -> Option<Arc<MetricsBridge>> {
    if let Ok(guard) = METRICS_BRIDGE.read() {
        guard.clone()
    } else {
        None
    }
}

/// Register a callback for real-time metrics updates.
///
/// The callback will be invoked periodically with updated metrics.
/// Pass NULL to unregister the callback.
///
/// # Arguments
/// * `callback` - Function pointer to call with metrics, or NULL to unregister
///
/// # Returns
/// - 0: Success
/// - -1: Failed to acquire bridge lock
///
/// # Safety
/// The callback function must be valid for the lifetime of the registration.
/// The callback receives a pointer to MetricsSnapshotFfi that is only valid
/// during the callback invocation - it must not be retained.
#[no_mangle]
pub extern "C" fn keyrx_metrics_set_callback(
    callback: Option<extern "C" fn(*const MetricsSnapshotFfi)>,
) -> i32 {
    if let Some(bridge) = get_metrics_bridge() {
        if let Some(cb) = callback {
            bridge.set_callback(cb);
            tracing::debug!(
                service = "keyrx",
                event = "metrics_callback_set",
                component = "ffi_observability",
                "Metrics callback registered"
            );
        } else {
            bridge.clear_callback();
            tracing::debug!(
                service = "keyrx",
                event = "metrics_callback_cleared",
                component = "ffi_observability",
                "Metrics callback unregistered"
            );
        }
        0
    } else {
        tracing::error!(
            service = "keyrx",
            event = "metrics_callback_failed",
            component = "ffi_observability",
            error = "bridge_unavailable",
            "Failed to access metrics bridge"
        );
        -1
    }
}

/// Get a snapshot of the current metrics.
///
/// This function returns a `MetricsSnapshot` object containing a point-in-time
/// view of all collected metrics, including latency, memory, and error counts.
///
/// The `ffi_export` macro automatically generates two versions of this function:
/// 1. `keyrx_metrics_snapshot_json()`: Returns the snapshot as a JSON string.
/// 2. `keyrx_metrics_snapshot()`: Returns a C-compatible struct (`FfiResult<MetricsSnapshot>`).
///    The struct must be freed with `keyrx_free_metrics_snapshot`.
///
/// # Returns
///
/// An `FfiResult` containing the `MetricsSnapshot` on success, or an `FfiError`
/// if the metrics bridge is not available.
#[ffi_export(domain = "observability")]
fn metrics_snapshot() -> crate::ffi::error::FfiResult<MetricsSnapshotFfi> {
    if let Some(bridge) = get_metrics_bridge() {
        let snapshot = bridge.snapshot();
        Ok(snapshot.into())
    } else {
        tracing::error!(
            service = "keyrx",
            event = "metrics_snapshot_failed",
            component = "ffi_observability",
            error = "bridge_unavailable",
            "Metrics bridge not available"
        );
        Err(FfiError::new(
            "METRICS_UNAVAILABLE",
            "Metrics bridge is not initialized",
        ))
    }
}

/// Start background metrics updates.
///
/// When enabled, the metrics callback (if registered) will be invoked
/// periodically with updated metrics.
///
/// # Returns
///
/// An `FfiResult<()>` indicating success or failure if the bridge is unavailable.
#[ffi_export(domain = "observability")]
fn metrics_start_updates() -> crate::ffi::error::FfiResult<()> {
    if let Some(bridge) = get_metrics_bridge() {
        bridge.start_updates();
        tracing::debug!(
            service = "keyrx",
            event = "metrics_updates_started",
            component = "ffi_observability",
            "Metrics background updates started"
        );
        Ok(())
    } else {
        Err(FfiError::new(
            "METRICS_UNAVAILABLE",
            "Metrics bridge is not initialized",
        ))
    }
}

/// Stop background metrics updates.
///
/// # Returns
///
/// An `FfiResult<()>` indicating success or failure if the bridge is unavailable.
#[ffi_export(domain = "observability")]
fn metrics_stop_updates() -> crate::ffi::error::FfiResult<()> {
    if let Some(bridge) = get_metrics_bridge() {
        bridge.stop_updates();
        tracing::debug!(
            service = "keyrx",
            event = "metrics_updates_stopped",
            component = "ffi_observability",
            "Metrics background updates stopped"
        );
        Ok(())
    } else {
        Err(FfiError::new(
            "METRICS_UNAVAILABLE",
            "Metrics bridge is not initialized",
        ))
    }
}

/// Trigger an immediate metrics callback (if registered).
///
/// This can be used for on-demand updates without waiting for the
/// background update interval.
///
/// # Returns
///
/// An `FfiResult<()>` indicating success or failure if the bridge is unavailable.
#[ffi_export(domain = "observability")]
fn metrics_trigger_callback() -> crate::ffi::error::FfiResult<()> {
    if let Some(bridge) = get_metrics_bridge() {
        bridge.trigger_callback();
        tracing::trace!(
            service = "keyrx",
            event = "metrics_callback_triggered",
            component = "ffi_observability",
            "Metrics callback triggered manually"
        );
        Ok(())
    } else {
        Err(FfiError::new(
            "METRICS_UNAVAILABLE",
            "Metrics bridge is not initialized",
        ))
    }
}

/// Register a callback for threshold violations.
///
/// The callback will be invoked when metrics exceed configured thresholds.
/// Pass NULL to unregister the callback.
///
/// # Arguments
/// * `callback` - Function pointer to call on threshold violations, or NULL to unregister
///
/// # Returns
/// - 0: Success
/// - -1: Failed to acquire bridge lock
///
/// # Safety
/// The callback function must be valid for the lifetime of the registration.
/// The callback receives a pointer to ThresholdViolation that is only valid
/// during the callback invocation - it must not be retained.
#[no_mangle]
pub extern "C" fn keyrx_metrics_set_threshold_callback(
    callback: Option<ThresholdCallback>,
) -> i32 {
    if let Some(bridge) = get_metrics_bridge() {
        if let Some(cb) = callback {
            bridge.set_threshold_callback(cb);
            tracing::debug!(
                service = "keyrx",
                event = "threshold_callback_set",
                component = "ffi_observability",
                "Threshold callback registered"
            );
        } else {
            bridge.clear_threshold_callback();
            tracing::debug!(
                service = "keyrx",
                event = "threshold_callback_cleared",
                component = "ffi_observability",
                "Threshold callback unregistered"
            );
        }
        0
    } else {
        tracing::error!(
            service = "keyrx",
            event = "threshold_callback_failed",
            component = "ffi_observability",
            error = "bridge_unavailable",
            "Failed to access metrics bridge"
        );
        -1
    }
}

/// Set threshold values for violation detection.
///
/// # Arguments
///
/// * `thresholds` - A `MetricsThresholds` object containing the new threshold values.
///
/// # Returns
///
/// An `FfiResult<()>` indicating success or failure.
#[ffi_export(domain = "observability")]
fn metrics_set_thresholds(
    thresholds: crate::observability::metrics_bridge::MetricsThresholds,
) -> crate::ffi::error::FfiResult<()> {
    if let Some(bridge) = get_metrics_bridge() {
        bridge.set_thresholds(thresholds.clone());
        tracing::debug!(
            service = "keyrx",
            event = "thresholds_set",
            component = "ffi_observability",
            ?thresholds,
            "Metrics thresholds updated"
        );
        Ok(())
    } else {
        Err(FfiError::new(
            "METRICS_UNAVAILABLE",
            "Metrics bridge is not initialized",
        ))
    }
}

/// Get current threshold values.
///
/// # Returns
///
/// An `FfiResult<MetricsThresholds>` containing the current thresholds on success.
#[ffi_export(domain = "observability")]
fn metrics_get_thresholds(
) -> crate::ffi::error::FfiResult<crate::observability::metrics_bridge::MetricsThresholds> {
    if let Some(bridge) = get_metrics_bridge() {
        let thresholds = bridge.get_thresholds();
        Ok(thresholds)
    } else {
        Err(FfiError::new(
            "METRICS_UNAVAILABLE",
            "Metrics bridge is not initialized",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::metrics_bridge::{MetricsThresholds, NoOpMetricsCollector};
    use serial_test::serial;

    fn cleanup_test_state() {
        // Clear global state without needing FfiContext
        GLOBAL_LOG_BRIDGE.clear_callback();
        GLOBAL_LOG_BRIDGE.clear();

        if let Ok(mut bridge) = METRICS_BRIDGE.write() {
            if let Some(b) = bridge.as_ref() {
                b.clear_callback();
                b.stop_updates();
            }
            *bridge = None;
        }
    }

    #[test]
    #[serial]
    fn test_log_bridge_init() {
        cleanup_test_state();

        assert_eq!(keyrx_log_bridge_init(), 0);
        assert_eq!(keyrx_log_bridge_init(), 0); // Idempotent
    }

    #[test]
    #[serial]
    fn test_log_count_and_clear() {
        cleanup_test_state();
        keyrx_log_bridge_init();

        // Initially empty
        assert_eq!(keyrx_log_count(), 0);

        // Clear should work even on empty buffer
        assert_eq!(keyrx_log_clear(), 0);
    }

    #[test]
    #[serial]
    fn test_log_set_enabled() {
        cleanup_test_state();
        keyrx_log_bridge_init();

        assert_eq!(keyrx_log_set_enabled(0), 0);
        assert_eq!(keyrx_log_set_enabled(1), 0);
    }

    #[test]
    #[serial]
    fn test_log_drain() {
        cleanup_test_state();
        keyrx_log_bridge_init();

        let ptr = keyrx_log_drain();
        assert!(!ptr.is_null());

        unsafe {
            let c_str = CString::from_raw(ptr);
            let s = c_str.to_str().unwrap();
            // Should be an empty JSON array
            assert_eq!(s, "[]");
        }
    }

    #[test]
    #[serial]
    fn test_log_callback() {
        cleanup_test_state();
        keyrx_log_bridge_init();

        extern "C" fn test_callback(_entry: *const CLogEntry) {
            // Test callback
        }

        assert_eq!(keyrx_log_set_callback(Some(test_callback)), 0);
        assert_eq!(keyrx_log_set_callback(None), 0);
    }

    #[test]
    #[serial]
    fn test_metrics_bridge_init() {
        cleanup_test_state();

        let collector = Arc::new(NoOpMetricsCollector);
        assert_eq!(init_metrics_bridge(collector), 0);
    }

    // Tests that rely on ffi_export generated functions are commented out until
    // we can ensure they are visible/compilable in this context.
    /*
    #[test]
    #[serial]
    fn test_metrics_snapshot_export() {
        cleanup_test_state();

        let collector = Arc::new(NoOpMetricsCollector);
        init_metrics_bridge(collector);

        // ...
    }
    */
}
