//! FFI exports for panic telemetry and recovery events.
//!
//! This module provides C-ABI functions for accessing panic and recovery
//! telemetry from Flutter. All functions are thread-safe.
//!
//! # API Overview
//!
//! ## Snapshot API (Polling)
//! - `keyrx_telemetry_snapshot_json()` - Get current telemetry as JSON
//! - `keyrx_telemetry_snapshot()` - Get telemetry as FFI struct
//! - `keyrx_telemetry_free_snapshot()` - Free FFI struct
//!
//! ## Reset API
//! - `keyrx_telemetry_reset()` - Reset all counters (for testing)
//!
//! # Example Usage
//!
//! ## Polling Approach (Recommended)
//!
//! ```c
//! // Get telemetry snapshot as JSON
//! char* json = keyrx_telemetry_snapshot_json();
//! if (json) {
//!     process_telemetry(json);
//!     keyrx_free_string(json);  // From core exports
//! }
//! ```
//!
//! ## Struct-based Approach
//!
//! ```c
//! PanicTelemetryFfi* telemetry = keyrx_telemetry_snapshot();
//! if (telemetry) {
//!     printf("Total panics: %llu\n", telemetry->total_panics);
//!     printf("Recoveries: %llu\n", telemetry->total_recoveries);
//!
//!     // Check recent events
//!     for (size_t i = 0; i < telemetry->events_count; i++) {
//!         PanicEventFfi* event = &telemetry->events[i];
//!         printf("Panic at %llu: %s - %s\n",
//!                event->timestamp, event->context, event->message);
//!     }
//!
//!     keyrx_telemetry_free_snapshot(telemetry);
//! }
//! ```
//!
//! # Telemetry Data
//!
//! The snapshot contains:
//! - `total_panics` - Total number of panics caught
//! - `total_recoveries` - Total number of successful recoveries
//! - `circuit_breaker_opens` - Total circuit breaker opens
//! - `circuit_breaker_closes` - Total circuit breaker closes
//! - `events_count` - Number of recent panic events
//! - `events` - Array of recent panic events
//!
//! Each panic event contains:
//! - `timestamp` - Unix timestamp in milliseconds
//! - `context` - Where the panic occurred (C string)
//! - `message` - Panic message (C string)
//! - `backtrace` - Optional backtrace (C string, NULL if none)
//! - `recovered` - Whether recovery was successful (0 or 1)
//!
//! # Thread Safety
//!
//! All functions are thread-safe and can be called from any thread.
//!
//! # Memory Management
//!
//! - Strings returned by `keyrx_telemetry_snapshot_json()` must be freed
//!   with `keyrx_free_string()` (from core exports)
//! - Structs returned by `keyrx_telemetry_snapshot()` must be freed
//!   with `keyrx_telemetry_free_snapshot()`

#![allow(unsafe_code)]

use crate::safety::panic_telemetry::{get_telemetry, reset_telemetry};
use std::ffi::{c_char, CString};
use std::ptr;

/// FFI representation of a panic event.
#[repr(C)]
#[derive(Debug)]
pub struct PanicEventFfi {
    /// Unix timestamp in milliseconds.
    pub timestamp: u64,

    /// Context where panic occurred (e.g., "keyboard_callback").
    /// Owned C string, must not be freed separately.
    pub context: *const c_char,

    /// Panic message.
    /// Owned C string, must not be freed separately.
    pub message: *const c_char,

    /// Optional backtrace. NULL if not available.
    /// Owned C string, must not be freed separately.
    pub backtrace: *const c_char,

    /// Whether recovery was successful (0 = false, 1 = true).
    pub recovered: u8,
}

/// FFI representation of panic telemetry.
#[repr(C)]
#[derive(Debug)]
pub struct PanicTelemetryFfi {
    /// Total number of panics caught.
    pub total_panics: u64,

    /// Total number of successful recoveries.
    pub total_recoveries: u64,

    /// Total number of circuit breaker opens.
    pub circuit_breaker_opens: u64,

    /// Total number of circuit breaker closes.
    pub circuit_breaker_closes: u64,

    /// Number of recent panic events.
    pub events_count: usize,

    /// Array of recent panic events.
    /// Allocated array, will be freed by keyrx_telemetry_free_snapshot.
    pub events: *mut PanicEventFfi,
}

/// Gets panic telemetry as a JSON string.
///
/// Returns a null-terminated JSON string that must be freed with `keyrx_free_string()`.
/// Returns NULL on error.
///
/// # Safety
///
/// The returned pointer must be freed with `keyrx_free_string()` to avoid memory leaks.
///
/// # Example
///
/// ```c
/// char* json = keyrx_telemetry_snapshot_json();
/// if (json) {
///     printf("%s\n", json);
///     keyrx_free_string(json);
/// }
/// ```
#[no_mangle]
pub extern "C" fn keyrx_telemetry_snapshot_json() -> *mut c_char {
    let telemetry = get_telemetry();

    match serde_json::to_string(&telemetry) {
        Ok(json) => match CString::new(json) {
            Ok(c_str) => c_str.into_raw(),
            Err(e) => {
                tracing::error!(
                    service = "keyrx",
                    event = "telemetry_json_failed",
                    error = %e,
                    "Failed to create CString from telemetry JSON"
                );
                ptr::null_mut()
            }
        },
        Err(e) => {
            tracing::error!(
                service = "keyrx",
                event = "telemetry_serialize_failed",
                error = %e,
                "Failed to serialize telemetry to JSON"
            );
            ptr::null_mut()
        }
    }
}

/// Gets panic telemetry as an FFI struct.
///
/// Returns a pointer to `PanicTelemetryFfi` that must be freed with
/// `keyrx_telemetry_free_snapshot()`. Returns NULL on error.
///
/// # Safety
///
/// The returned pointer must be freed with `keyrx_telemetry_free_snapshot()`
/// to avoid memory leaks.
///
/// # Example
///
/// ```c
/// PanicTelemetryFfi* telemetry = keyrx_telemetry_snapshot();
/// if (telemetry) {
///     printf("Panics: %llu\n", telemetry->total_panics);
///     keyrx_telemetry_free_snapshot(telemetry);
/// }
/// ```
#[no_mangle]
pub extern "C" fn keyrx_telemetry_snapshot() -> *mut PanicTelemetryFfi {
    let telemetry = get_telemetry();

    // Convert events to FFI representation
    let mut ffi_events: Vec<PanicEventFfi> = Vec::with_capacity(telemetry.recent_events.len());

    for event in &telemetry.recent_events {
        let context = match CString::new(event.context.as_str()) {
            Ok(s) => s.into_raw(),
            Err(_) => continue,
        };

        let message = match CString::new(event.message.as_str()) {
            Ok(s) => s.into_raw(),
            Err(_) => {
                unsafe {
                    let _ = CString::from_raw(context);
                }
                continue;
            }
        };

        let backtrace = event
            .backtrace
            .as_ref()
            .and_then(|bt| CString::new(bt.as_str()).ok().map(|s| s.into_raw()))
            .unwrap_or(ptr::null_mut());

        ffi_events.push(PanicEventFfi {
            timestamp: event.timestamp,
            context,
            message,
            backtrace,
            recovered: if event.recovered { 1 } else { 0 },
        });
    }

    // Allocate events array
    let events_ptr = if ffi_events.is_empty() {
        ptr::null_mut()
    } else {
        let boxed = ffi_events.into_boxed_slice();
        Box::into_raw(boxed) as *mut PanicEventFfi
    };

    let events_count = telemetry.recent_events.len();

    // Create FFI struct
    let ffi_telemetry = Box::new(PanicTelemetryFfi {
        total_panics: telemetry.total_panics,
        total_recoveries: telemetry.total_recoveries,
        circuit_breaker_opens: telemetry.circuit_breaker_opens,
        circuit_breaker_closes: telemetry.circuit_breaker_closes,
        events_count,
        events: events_ptr,
    });

    Box::into_raw(ffi_telemetry)
}

/// Frees a panic telemetry snapshot.
///
/// # Safety
///
/// - `telemetry` must be a valid pointer returned by `keyrx_telemetry_snapshot()`
/// - `telemetry` must not be used after this call
/// - `telemetry` must not be NULL
///
/// # Example
///
/// ```c
/// PanicTelemetryFfi* telemetry = keyrx_telemetry_snapshot();
/// if (telemetry) {
///     // Use telemetry...
///     keyrx_telemetry_free_snapshot(telemetry);
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn keyrx_telemetry_free_snapshot(telemetry: *mut PanicTelemetryFfi) {
    if telemetry.is_null() {
        return;
    }

    let telemetry = Box::from_raw(telemetry);

    // Free events array
    if !telemetry.events.is_null() {
        let events = Vec::from_raw_parts(
            telemetry.events,
            telemetry.events_count,
            telemetry.events_count,
        );

        // Free strings in each event
        for event in events {
            if !event.context.is_null() {
                let _ = CString::from_raw(event.context as *mut c_char);
            }
            if !event.message.is_null() {
                let _ = CString::from_raw(event.message as *mut c_char);
            }
            if !event.backtrace.is_null() {
                let _ = CString::from_raw(event.backtrace as *mut c_char);
            }
        }
    }

    // telemetry is dropped here, freeing the struct itself
}

/// Resets all panic telemetry counters and clears events.
///
/// This is primarily for testing. In production, counters should accumulate.
///
/// Returns 0 on success, -1 on error.
///
/// # Example
///
/// ```c
/// keyrx_telemetry_reset();  // Start fresh
/// ```
#[no_mangle]
pub extern "C" fn keyrx_telemetry_reset() -> i32 {
    reset_telemetry();
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::safety::panic_telemetry::record_panic;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_snapshot_json_export() {
        reset_telemetry();
        record_panic("test_context", "test panic", None);

        let json_ptr = keyrx_telemetry_snapshot_json();
        assert!(!json_ptr.is_null());

        unsafe {
            let c_str = CString::from_raw(json_ptr);
            let json_str = c_str.to_str().unwrap();
            assert!(json_str.contains("total_panics"));
            assert!(json_str.contains("test_context"));
        }
    }

    #[test]
    #[serial]
    fn test_snapshot_struct_export() {
        reset_telemetry();
        record_panic("context1", "message1", Some("backtrace1".to_string()));

        let telemetry_ptr = keyrx_telemetry_snapshot();
        assert!(!telemetry_ptr.is_null());

        unsafe {
            let telemetry = &*telemetry_ptr;
            assert_eq!(telemetry.total_panics, 1);
            assert_eq!(telemetry.events_count, 1);
            assert!(!telemetry.events.is_null());

            let event = &*telemetry.events;
            assert!(!event.context.is_null());
            assert!(!event.message.is_null());
            assert!(!event.backtrace.is_null());

            let context = CString::from_raw(event.context as *mut c_char);
            assert_eq!(context.to_str().unwrap(), "context1");
            std::mem::forget(context); // Don't drop, will be freed by free_snapshot

            keyrx_telemetry_free_snapshot(telemetry_ptr);
        }
    }

    #[test]
    #[serial]
    fn test_empty_telemetry() {
        reset_telemetry();

        let telemetry_ptr = keyrx_telemetry_snapshot();
        assert!(!telemetry_ptr.is_null());

        unsafe {
            let telemetry = &*telemetry_ptr;
            assert_eq!(telemetry.total_panics, 0);
            assert_eq!(telemetry.events_count, 0);
            assert!(telemetry.events.is_null());

            keyrx_telemetry_free_snapshot(telemetry_ptr);
        }
    }

    #[test]
    #[serial]
    fn test_reset_export() {
        record_panic("test", "panic", None);
        assert_eq!(keyrx_telemetry_reset(), 0);

        let telemetry = get_telemetry();
        assert_eq!(telemetry.total_panics, 0);
    }

    #[test]
    #[serial]
    fn test_multiple_events() {
        reset_telemetry();
        record_panic("ctx1", "msg1", None);
        record_panic("ctx2", "msg2", Some("bt2".to_string()));

        let telemetry_ptr = keyrx_telemetry_snapshot();
        assert!(!telemetry_ptr.is_null());

        unsafe {
            let telemetry = &*telemetry_ptr;
            assert_eq!(telemetry.total_panics, 2);
            assert_eq!(telemetry.events_count, 2);

            keyrx_telemetry_free_snapshot(telemetry_ptr);
        }
    }
}
