//! FFI exports for transition log access.
//!
//! This module provides C-ABI functions for accessing the transition log
//! from external language bindings (Python, JavaScript, etc.). The transition
//! log records all state transitions with complete before/after state snapshots.
//!
//! # API Overview
//!
//! ## JSON Export (Recommended)
//! - `keyrx_transition_log_export_json()` - Get all transitions as JSON
//! - `keyrx_transition_log_search_by_category_json()` - Filter by category
//! - `keyrx_transition_log_search_by_name_json()` - Filter by transition name
//!
//! ## Log Management
//! - `keyrx_transition_log_clear()` - Clear all log entries
//! - `keyrx_transition_log_len()` - Get number of entries
//! - `keyrx_transition_log_capacity()` - Get maximum capacity
//! - `keyrx_transition_log_statistics()` - Get statistics (total, unique, durations)
//!
//! # Feature Flag
//!
//! When the `transition-logging` feature is disabled, all functions return
//! empty results with zero overhead.
//!
//! # Example Usage
//!
//! ```c
//! // Export all transitions as JSON
//! char* json = keyrx_transition_log_export_json(engine_ptr);
//! if (json) {
//!     process_transitions(json);
//!     keyrx_free_string(json);
//! }
//!
//! // Get log statistics
//! uint64_t total, unique, total_dur, avg_dur;
//! keyrx_transition_log_statistics(engine_ptr, &total, &unique, &total_dur, &avg_dur);
//! printf("Total: %llu, Unique: %llu, Avg Duration: %llu ns\n", total, unique, avg_dur);
//!
//! // Clear the log
//! keyrx_transition_log_clear(engine_ptr);
//! ```
//!
//! # Thread Safety
//!
//! These functions require exclusive access to the engine instance.
//! The caller must ensure proper synchronization.
//!
//! # Memory Management
//!
//! Strings returned by export functions must be freed with `keyrx_free_string()`.

#![allow(unsafe_code)]

use crate::ffi::engine_instance::with_global_engine;
use std::ffi::{c_char, CString};

/// Export all transition log entries as JSON.
///
/// Returns a JSON array containing all transitions in chronological order.
/// Each entry includes the transition type, state before/after snapshots,
/// timing information, and metadata.
///
/// # Arguments
///
/// * `engine_ptr` - Opaque pointer to the AdvancedEngine instance
///
/// # Returns
///
/// Pointer to a null-terminated JSON string, or null on error.
/// The caller must free the returned string with `keyrx_free_string()`.
///
/// # Safety
///
/// `engine_ptr` must be a valid pointer to an AdvancedEngine instance.
#[no_mangle]
pub extern "C" fn keyrx_transition_log_export_json(
    _engine_ptr: *const std::ffi::c_void,
) -> *mut c_char {
    with_global_engine(|engine| {
        let log = engine.transition_log();
        if let Ok(json_str) = log.export_json() {
            if let Ok(c_str) = CString::new(json_str) {
                return c_str.into_raw();
            }
        }
        std::ptr::null_mut()
    })
    .unwrap_or(std::ptr::null_mut())
}

/// Get the number of transition log entries currently stored.
///
/// # Arguments
///
/// * `engine_ptr` - Opaque pointer to the AdvancedEngine instance
///
/// # Returns
///
/// Number of entries in the log.
///
/// # Safety
///
/// `engine_ptr` must be a valid pointer to an AdvancedEngine instance.
#[no_mangle]
pub extern "C" fn keyrx_transition_log_len(_engine_ptr: *const std::ffi::c_void) -> usize {
    with_global_engine(|engine| engine.transition_log().len()).unwrap_or(0)
}

/// Get the maximum capacity of the transition log.
///
/// # Arguments
///
/// * `engine_ptr` - Opaque pointer to the AdvancedEngine instance
///
/// # Returns
///
/// Maximum number of entries the log can hold.
///
/// # Safety
///
/// `engine_ptr` must be a valid pointer to an AdvancedEngine instance.
#[no_mangle]
pub extern "C" fn keyrx_transition_log_capacity(_engine_ptr: *const std::ffi::c_void) -> usize {
    with_global_engine(|engine| engine.transition_log().capacity()).unwrap_or(0)
}

/// Clear all entries from the transition log.
///
/// # Arguments
///
/// * `engine_ptr` - Opaque pointer to the AdvancedEngine instance
///
/// # Safety
///
/// `engine_ptr` must be a valid pointer to an AdvancedEngine instance.
#[no_mangle]
pub extern "C" fn keyrx_transition_log_clear(_engine_ptr: *mut std::ffi::c_void) {
    with_global_engine(|engine| engine.transition_log_mut().clear());
}

/// Get statistics about the transition log.
///
/// Returns a tuple of statistics via output parameters:
/// - `total`: Total entries currently stored
/// - `unique_names`: Number of unique transition names
/// - `total_duration`: Sum of all transition processing durations (nanoseconds)
/// - `avg_duration`: Average processing duration per entry (nanoseconds)
///
/// # Arguments
///
/// * `engine_ptr` - Opaque pointer to the AdvancedEngine instance
/// * `total` - Output: Total entries
/// * `unique_names` - Output: Number of unique transition names
/// * `total_duration` - Output: Total processing time (nanoseconds)
/// * `avg_duration` - Output: Average processing time (nanoseconds)
///
/// # Safety
///
/// All pointers must be valid. `engine_ptr` must point to an AdvancedEngine instance.
#[no_mangle]
pub unsafe extern "C" fn keyrx_transition_log_statistics(
    _engine_ptr: *const std::ffi::c_void,
    total: *mut usize,
    unique_names: *mut usize,
    total_duration: *mut u64,
    avg_duration: *mut u64,
) {
    let stats =
        with_global_engine(|engine| engine.transition_log().statistics()).unwrap_or((0, 0, 0, 0));

    if !total.is_null() {
        *total = stats.0;
    }
    if !unique_names.is_null() {
        *unique_names = stats.1;
    }
    if !total_duration.is_null() {
        *total_duration = stats.2;
    }
    if !avg_duration.is_null() {
        *avg_duration = stats.3;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{AdvancedEngine, TimingConfig};
    use crate::ffi::engine_instance::{clear_global_engine, set_global_engine};
    use crate::scripting::RhaiRuntime;
    use std::sync::{Arc, Mutex};

    fn setup_mock_engine() {
        let rhai_runtime = Arc::new(Mutex::new(
            RhaiRuntime::new().expect("Failed to create rhai runtime"),
        ));
        let engine = AdvancedEngine::new(rhai_runtime, TimingConfig::default());
        let shared_engine = Arc::new(Mutex::new(engine));
        set_global_engine(shared_engine);
    }

    fn teardown_mock_engine() {
        clear_global_engine();
    }

    #[test]
    fn test_connected_functions() {
        setup_mock_engine();
        let null_ptr = std::ptr::null();

        // Initial state
        assert_eq!(keyrx_transition_log_len(null_ptr), 0);

        #[cfg(feature = "transition-logging")]
        assert_eq!(keyrx_transition_log_capacity(null_ptr), 10_000);

        let mut total = 0usize;
        let mut unique = 0usize;
        let mut total_dur = 0u64;
        let mut avg_dur = 0u64;

        unsafe {
            keyrx_transition_log_statistics(
                null_ptr,
                &mut total,
                &mut unique,
                &mut total_dur,
                &mut avg_dur,
            );
        }

        assert_eq!(total, 0);

        // Test JSON export
        let json_ptr = keyrx_transition_log_export_json(null_ptr);
        assert!(!json_ptr.is_null());
        unsafe {
            let c_str = CString::from_raw(json_ptr);
            let json_str = c_str.to_str().unwrap();
            assert_eq!(json_str, "[]");
        }

        teardown_mock_engine();
    }
}
