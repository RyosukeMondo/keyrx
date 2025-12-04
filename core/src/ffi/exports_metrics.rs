//! FFI exports for metrics snapshot and real-time updates.
//!
//! This module provides C-ABI functions for accessing performance metrics
//! from Flutter. All functions are thread-safe and implemented in the
//! observability domain (`crate::ffi::domains::observability`).
//!
//! # API Overview
//!
//! ## Snapshot API (Polling)
//! - `keyrx_metrics_snapshot_json()` - Get current metrics as JSON
//! - `keyrx_metrics_snapshot()` - Get metrics as FFI struct
//! - `keyrx_metrics_free_snapshot()` - Free FFI struct
//!
//! ## Callback API (Push)
//! - `keyrx_metrics_set_callback()` - Register callback for updates
//! - `keyrx_metrics_start_updates()` - Start periodic background updates
//! - `keyrx_metrics_stop_updates()` - Stop background updates
//! - `keyrx_metrics_trigger_callback()` - Trigger immediate update
//!
//! ## Threshold API (Alerts)
//! - `keyrx_metrics_set_threshold_callback()` - Register callback for threshold violations
//! - `keyrx_metrics_set_thresholds()` - Configure threshold values
//! - `keyrx_metrics_get_thresholds()` - Get current threshold values
//!
//! # Example Usage
//!
//! ## Polling Approach (Recommended)
//!
//! ```c
//! // Get metrics snapshot as JSON
//! char* json = keyrx_metrics_snapshot_json();
//! if (json) {
//!     process_metrics(json);
//!     keyrx_free_string(json);  // From core exports
//! }
//! ```
//!
//! ## Callback Approach
//!
//! ```c
//! void on_metrics_update(const MetricsSnapshotFfi* snapshot) {
//!     // Process snapshot fields directly
//!     printf("Events processed: %llu\n", snapshot->events_processed);
//!     printf("P95 latency: %llu us\n", snapshot->event_latency_p95);
//! }
//!
//! keyrx_metrics_set_callback(on_metrics_update);
//! keyrx_metrics_start_updates();
//! // ... later ...
//! keyrx_metrics_stop_updates();
//! keyrx_metrics_set_callback(NULL);  // Unregister
//! ```
//!
//! ## Struct-based Approach
//!
//! ```c
//! MetricsSnapshotFfi* snapshot = keyrx_metrics_snapshot();
//! if (snapshot) {
//!     printf("Memory used: %llu bytes\n", snapshot->memory_used);
//!     keyrx_metrics_free_snapshot(snapshot);
//! }
//! ```
//!
//! ## Threshold Callback Approach
//!
//! ```c
//! void on_threshold_violation(const ThresholdViolation* violation) {
//!     switch (violation->violation_type) {
//!         case 0: // LatencyWarning
//!             printf("Warning: Latency %llu us exceeds %llu us\n",
//!                    violation->actual_value, violation->threshold_value);
//!             break;
//!         case 1: // LatencyError
//!             printf("Error: Latency %llu us exceeds %llu us\n",
//!                    violation->actual_value, violation->threshold_value);
//!             break;
//!         case 2: // MemoryWarning
//!             printf("Warning: Memory %llu bytes exceeds %llu bytes\n",
//!                    violation->actual_value, violation->threshold_value);
//!             break;
//!         case 3: // MemoryError
//!             printf("Error: Memory %llu bytes exceeds %llu bytes\n",
//!                    violation->actual_value, violation->threshold_value);
//!             break;
//!     }
//! }
//!
//! // Configure thresholds (50us warn, 100us error, 100MB warn, 500MB error)
//! keyrx_metrics_set_thresholds(50, 100, 104857600, 524288000);
//!
//! // Register callback
//! keyrx_metrics_set_threshold_callback(on_threshold_violation);
//!
//! // Start updates - thresholds checked automatically
//! keyrx_metrics_start_updates();
//!
//! // ... later ...
//! keyrx_metrics_stop_updates();
//! keyrx_metrics_set_threshold_callback(NULL);  // Unregister
//! ```
//!
//! # Metrics Data
//!
//! The snapshot contains:
//! - `timestamp` - Unix timestamp in milliseconds
//! - `event_latency_p50` - 50th percentile latency (microseconds)
//! - `event_latency_p95` - 95th percentile latency (microseconds)
//! - `event_latency_p99` - 99th percentile latency (microseconds)
//! - `events_processed` - Total number of events processed
//! - `errors_count` - Total number of errors encountered
//! - `memory_used` - Current memory usage (bytes)
//!
//! # Threshold Violations
//!
//! Threshold violations contain:
//! - `timestamp` - Unix timestamp in milliseconds
//! - `violation_type` - Type of violation (0=LatencyWarning, 1=LatencyError, 2=MemoryWarning, 3=MemoryError)
//! - `actual_value` - The actual value that exceeded the threshold
//! - `threshold_value` - The threshold that was exceeded
//!
//! # Thread Safety
//!
//! All functions are thread-safe and can be called from any thread.
//!
//! # Memory Management
//!
//! - Strings returned by `keyrx_metrics_snapshot_json()` must be freed
//!   with `keyrx_free_string()` (from core exports)
//! - Structs returned by `keyrx_metrics_snapshot()` must be freed
//!   with `keyrx_metrics_free_snapshot()`
//! - Callback receives a pointer valid only during the callback invocation

// Re-export all metrics FFI functions from the observability domain.
// This provides a unified public API surface while keeping implementation
// in the domain modules.

pub use crate::ffi::domains::observability::{
    keyrx_metrics_get_thresholds, keyrx_metrics_set_callback, keyrx_metrics_set_threshold_callback,
    keyrx_metrics_set_thresholds, keyrx_metrics_snapshot, keyrx_metrics_snapshot_json,
    keyrx_metrics_start_updates, keyrx_metrics_stop_updates, keyrx_metrics_trigger_callback,
};

// Re-export types
pub use crate::observability::metrics_bridge::{
    MetricsSnapshotFfi, ThresholdViolation, ViolationType,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::domains::observability::init_metrics_bridge;
    use crate::observability::metrics_bridge::NoOpMetricsCollector;
    use serial_test::serial;
    use std::ffi::CString;
    use std::sync::Arc;

    fn init_test_metrics() {
        let collector = Arc::new(NoOpMetricsCollector);
        init_metrics_bridge(collector);
    }

    #[test]
    #[serial]
    fn test_snapshot_json_export() {
        init_test_metrics();

        let json_ptr = keyrx_metrics_snapshot_json();
        assert!(!json_ptr.is_null());

        unsafe {
            let c_str = CString::from_raw(json_ptr);
            let json_str = c_str.to_str().unwrap();
            assert!(json_str.contains("timestamp"));
        }
    }

    #[test]
    #[serial]
    fn test_snapshot_struct_export() {
        init_test_metrics();

        let snapshot_ptr = keyrx_metrics_snapshot();
        assert!(!snapshot_ptr.is_null());

        unsafe {
            let snapshot = &*snapshot_ptr;
            assert!(snapshot.timestamp > 0);

            // Free the snapshot
            crate::ffi::domains::observability::keyrx_metrics_free_snapshot(snapshot_ptr);
        }
    }

    #[test]
    #[serial]
    fn test_callback_api_exports() {
        init_test_metrics();

        extern "C" fn test_callback(_snapshot: *const MetricsSnapshotFfi) {}

        // Set callback
        assert_eq!(keyrx_metrics_set_callback(Some(test_callback)), 0);

        // Control updates
        assert_eq!(keyrx_metrics_start_updates(), 0);
        assert_eq!(keyrx_metrics_trigger_callback(), 0);
        assert_eq!(keyrx_metrics_stop_updates(), 0);

        // Unregister
        assert_eq!(keyrx_metrics_set_callback(None), 0);
    }

    #[test]
    #[serial]
    fn test_threshold_callback_exports() {
        init_test_metrics();

        extern "C" fn test_threshold_callback(_violation: *const ThresholdViolation) {}

        // Set threshold callback
        assert_eq!(
            keyrx_metrics_set_threshold_callback(Some(test_threshold_callback)),
            0
        );

        // Unregister
        assert_eq!(keyrx_metrics_set_threshold_callback(None), 0);
    }

    #[test]
    #[serial]
    fn test_threshold_configuration() {
        init_test_metrics();

        // Set thresholds
        assert_eq!(
            keyrx_metrics_set_thresholds(50, 100, 1024 * 1024, 5 * 1024 * 1024),
            0
        );

        // Get thresholds
        let mut latency_warn = 0u64;
        let mut latency_error = 0u64;
        let mut memory_warn = 0u64;
        let mut memory_error = 0u64;

        unsafe {
            assert_eq!(
                keyrx_metrics_get_thresholds(
                    &mut latency_warn,
                    &mut latency_error,
                    &mut memory_warn,
                    &mut memory_error
                ),
                0
            );
        }

        assert_eq!(latency_warn, 50);
        assert_eq!(latency_error, 100);
        assert_eq!(memory_warn, 1024 * 1024);
        assert_eq!(memory_error, 5 * 1024 * 1024);
    }
}
