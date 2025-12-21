//! FFI tracing event export for Dart DevTools Timeline integration.
//!
//! This module provides FFI functions to export trace events to Dart,
//! enabling visualization of FFI calls in Flutter DevTools Timeline.
//!
//! ## Architecture
//!
//! - **Event Collection**: FFI functions emit trace events
//! - **Event Buffer**: Thread-safe circular buffer stores recent events
//! - **Export API**: FFI functions expose events to Dart

#![allow(clippy::expect_used)] // Legacy code - expect() is acceptable for trace buffer locks
//! - **Timeline Integration**: Dart converts to Timeline events
//!
//! ## Usage
//!
//! ### Rust Side
//!
//! ```ignore
//! use crate::ffi::tracing_export::{emit_trace_event, TraceEvent};
//!
//! #[ffi_export(domain = "engine", version = "1.0")]
//! pub extern "C" fn keyrx_engine_start() -> i32 {
//!     let event_id = emit_trace_event(TraceEvent {
//!         name: "keyrx_engine_start".to_string(),
//!         domain: "engine".to_string(),
//!         start_time_us: start_time,
//!         duration_us: None, // Set on completion
//!         correlation_id: correlation_id(),
//!         args: sanitize_args(&[("foo", "bar")]),
//!         result_status: ResultStatus::Pending,
//!     });
//!
//!     // Do work...
//!
//!     complete_trace_event(event_id, ResultStatus::Success, duration);
//! }
//! ```
//!
//! ### Dart Side
//!
//! ```dart
//! // Poll for trace events
//! final events = keyrx_trace_get_events();
//! for (final event in events) {
//!   Timeline.startSync(event.name, arguments: event.args);
//!   // ... or Timeline.instantSync for completed events
//! }
//! ```

use std::collections::HashMap;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::tracing::correlation_id;

/// Maximum number of trace events to buffer
const MAX_TRACE_EVENTS: usize = 1000;

/// Result status for trace events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub enum ResultStatus {
    /// Event is pending (not yet completed)
    Pending = 0,
    /// Event completed successfully
    Success = 1,
    /// Event completed with error
    Error = 2,
    /// Event was skipped
    Skipped = 3,
}

/// A trace event representing an FFI call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Unique event ID
    pub id: u64,
    /// Function name (e.g., "keyrx_engine_start")
    pub name: String,
    /// Domain (e.g., "engine", "device", "profile")
    pub domain: String,
    /// Start time in microseconds since UNIX epoch
    pub start_time_us: u64,
    /// Duration in microseconds (None if not yet completed)
    pub duration_us: Option<u64>,
    /// Correlation ID for tracking request flows
    pub correlation_id: u64,
    /// Sanitized arguments (sensitive data removed)
    pub args: HashMap<String, String>,
    /// Result status
    pub result_status: ResultStatus,
}

/// Global trace event buffer
static TRACE_BUFFER: OnceLock<Arc<Mutex<TraceBuffer>>> = OnceLock::new();

/// Get or initialize the trace buffer
fn trace_buffer() -> &'static Arc<Mutex<TraceBuffer>> {
    TRACE_BUFFER.get_or_init(|| Arc::new(Mutex::new(TraceBuffer::new())))
}

/// Circular buffer for trace events
struct TraceBuffer {
    events: Vec<TraceEvent>,
    next_id: u64,
    write_index: usize,
    enabled: bool,
}

impl TraceBuffer {
    fn new() -> Self {
        Self {
            events: Vec::with_capacity(MAX_TRACE_EVENTS),
            next_id: 1,
            write_index: 0,
            enabled: false,
        }
    }

    /// Enable trace event collection
    fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable trace event collection
    fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if tracing is enabled
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Add a trace event to the buffer
    fn add_event(&mut self, mut event: TraceEvent) -> u64 {
        if !self.enabled {
            return 0;
        }

        let event_id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        event.id = event_id;

        if self.events.len() < MAX_TRACE_EVENTS {
            // Buffer not full yet, just append
            self.events.push(event);
        } else {
            // Buffer full, use circular overwrite
            self.events[self.write_index] = event;
            self.write_index = (self.write_index + 1) % MAX_TRACE_EVENTS;
        }

        event_id
    }

    /// Get all trace events
    fn get_events(&self) -> Vec<TraceEvent> {
        self.events.clone()
    }

    /// Update an existing event (e.g., to set duration and result status)
    fn update_event<F>(&mut self, event_id: u64, updater: F)
    where
        F: FnOnce(&mut TraceEvent),
    {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == event_id) {
            updater(event);
        }
    }

    /// Clear all trace events
    fn clear(&mut self) {
        self.events.clear();
        self.write_index = 0;
    }

    /// Get events matching a filter
    fn get_filtered_events<F>(&self, filter: F) -> Vec<TraceEvent>
    where
        F: Fn(&TraceEvent) -> bool,
    {
        self.events.iter().filter(|e| filter(e)).cloned().collect()
    }
}

/// Get current time in microseconds since UNIX epoch
pub fn current_time_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

/// Emit a trace event for an FFI call.
///
/// Returns the event ID which can be used to update the event later.
///
/// # Arguments
///
/// * `name` - Function name (e.g., "keyrx_engine_start")
/// * `domain` - Domain (e.g., "engine", "device")
/// * `args` - Sanitized arguments (no sensitive data!)
///
/// # Examples
///
/// ```ignore
/// let event_id = emit_trace_event_start(
///     "keyrx_engine_start",
///     "engine",
///     &[("mode", "normal")],
/// );
/// // ... do work ...
/// emit_trace_event_end(event_id, ResultStatus::Success);
/// ```
pub fn emit_trace_event_start(name: &str, domain: &str, args: &[(&str, &str)]) -> u64 {
    let mut buffer = trace_buffer().lock().expect("Failed to lock trace buffer");

    if !buffer.is_enabled() {
        return 0;
    }

    let event = TraceEvent {
        id: 0, // Will be set by add_event
        name: name.to_string(),
        domain: domain.to_string(),
        start_time_us: current_time_us(),
        duration_us: None,
        correlation_id: correlation_id(),
        args: args
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        result_status: ResultStatus::Pending,
    };

    buffer.add_event(event)
}

/// Complete a trace event with duration and result status.
///
/// # Arguments
///
/// * `event_id` - Event ID returned from `emit_trace_event_start`
/// * `result_status` - Final result status
pub fn emit_trace_event_end(event_id: u64, result_status: ResultStatus) {
    if event_id == 0 {
        return; // Tracing was disabled
    }

    let mut buffer = trace_buffer().lock().expect("Failed to lock trace buffer");
    let now = current_time_us();

    buffer.update_event(event_id, |event| {
        let duration = now.saturating_sub(event.start_time_us);
        event.duration_us = Some(duration);
        event.result_status = result_status;
    });
}

/// Enable FFI trace event collection.
///
/// Must be called before any trace events will be collected.
///
/// # Safety
///
/// This function is FFI-safe.
#[no_mangle]
pub extern "C" fn keyrx_trace_enable() {
    use std::time::Instant;
    let start = Instant::now();

    let mut buffer = trace_buffer().lock().expect("Failed to lock trace buffer");
    buffer.enable();
    tracing::info!("FFI trace event collection enabled");
    crate::ffi_record_metrics!("keyrx_trace_enable", "tracing", start, &Ok::<_, ()>(()));
}

/// Disable FFI trace event collection.
///
/// # Safety
///
/// This function is FFI-safe.
#[no_mangle]
pub extern "C" fn keyrx_trace_disable() {
    use std::time::Instant;
    let start = Instant::now();

    let mut buffer = trace_buffer().lock().expect("Failed to lock trace buffer");
    buffer.disable();
    tracing::info!("FFI trace event collection disabled");
    crate::ffi_record_metrics!("keyrx_trace_disable", "tracing", start, &Ok::<_, ()>(()));
}

/// Check if trace event collection is enabled.
///
/// # Safety
///
/// This function is FFI-safe.
#[no_mangle]
pub extern "C" fn keyrx_trace_is_enabled() -> bool {
    use std::time::Instant;
    let start = Instant::now();

    let buffer = trace_buffer().lock().expect("Failed to lock trace buffer");
    let result = buffer.is_enabled();
    crate::ffi_record_metrics!("keyrx_trace_is_enabled", "tracing", start, &Ok::<_, ()>(()));
    result
}

/// Get the number of buffered trace events.
///
/// # Safety
///
/// This function is FFI-safe.
#[no_mangle]
pub extern "C" fn keyrx_trace_get_count() -> usize {
    use std::time::Instant;
    let start = Instant::now();

    let buffer = trace_buffer().lock().expect("Failed to lock trace buffer");
    let result = buffer.events.len();
    crate::ffi_record_metrics!("keyrx_trace_get_count", "tracing", start, &Ok::<_, ()>(()));
    result
}

/// Clear all buffered trace events.
///
/// # Safety
///
/// This function is FFI-safe.
#[no_mangle]
pub extern "C" fn keyrx_trace_clear() {
    use std::time::Instant;
    let start = Instant::now();

    let mut buffer = trace_buffer().lock().expect("Failed to lock trace buffer");
    buffer.clear();
    tracing::debug!("Trace event buffer cleared");
    crate::ffi_record_metrics!("keyrx_trace_clear", "tracing", start, &Ok::<_, ()>(()));
}

/// Get all trace events as a JSON string.
///
/// The caller is responsible for freeing the returned string using `keyrx_trace_free_json`.
///
/// Returns a null pointer if no events are available or serialization fails.
///
/// # Safety
///
/// The returned pointer must be freed using `keyrx_trace_free_json`.
#[no_mangle]
pub extern "C" fn keyrx_trace_get_events_json() -> *mut c_char {
    use std::time::Instant;
    let start = Instant::now();

    let buffer = trace_buffer().lock().expect("Failed to lock trace buffer");
    let events = buffer.get_events();

    let result = match serde_json::to_string(&events) {
        Ok(json) => {
            let c_str = std::ffi::CString::new(json).expect("Failed to create CString");
            c_str.into_raw()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to serialize trace events");
            std::ptr::null_mut()
        }
    };
    crate::ffi_record_metrics!(
        "keyrx_trace_get_events_json",
        "tracing",
        start,
        &Ok::<_, ()>(())
    );
    result
}

/// Get filtered trace events as a JSON string.
///
/// # Arguments
///
/// * `domain` - Filter by domain (null for all domains)
/// * `min_duration_us` - Minimum duration in microseconds (0 for no filter)
///
/// The caller is responsible for freeing the returned string using `keyrx_trace_free_json`.
///
/// Returns a null pointer if no events match or serialization fails.
///
/// # Safety
///
/// The `domain` pointer must be valid or null. The returned pointer must be freed using `keyrx_trace_free_json`.
#[no_mangle]
pub unsafe extern "C" fn keyrx_trace_get_filtered_events_json(
    domain: *const c_char,
    min_duration_us: u64,
) -> *mut c_char {
    use std::time::Instant;
    let start = Instant::now();

    let buffer = trace_buffer().lock().expect("Failed to lock trace buffer");

    // Parse domain filter if provided
    let domain_filter = if domain.is_null() {
        None
    } else {
        std::ffi::CStr::from_ptr(domain)
            .to_str()
            .ok()
            .map(|s| s.to_string())
    };

    let events = buffer.get_filtered_events(|event| {
        // Domain filter
        if let Some(ref domain) = domain_filter {
            if &event.domain != domain {
                return false;
            }
        }

        // Duration filter
        if min_duration_us > 0 {
            if let Some(duration) = event.duration_us {
                if duration < min_duration_us {
                    return false;
                }
            } else {
                return false; // Skip pending events
            }
        }

        true
    });

    let result = match serde_json::to_string(&events) {
        Ok(json) => {
            let c_str = std::ffi::CString::new(json).expect("Failed to create CString");
            c_str.into_raw()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to serialize filtered trace events");
            std::ptr::null_mut()
        }
    };
    crate::ffi_record_metrics!(
        "keyrx_trace_get_filtered_events_json",
        "tracing",
        start,
        &Ok::<_, ()>(())
    );
    result
}

/// Free a JSON string returned by trace functions.
///
/// # Safety
///
/// The pointer must have been returned by `keyrx_trace_get_events_json` or
/// `keyrx_trace_get_filtered_events_json` and must only be freed once.
#[no_mangle]
pub unsafe extern "C" fn keyrx_trace_free_json(json: *mut c_char) {
    use std::time::Instant;
    let start = Instant::now();

    if !json.is_null() {
        drop(std::ffi::CString::from_raw(json));
    }
    crate::ffi_record_metrics!("keyrx_trace_free_json", "tracing", start, &Ok::<_, ()>(()));
}

/// Helper macro to trace an FFI function with automatic timing.
///
/// # Examples
///
/// ```ignore
/// #[ffi_export(domain = "engine", version = "1.0")]
/// pub extern "C" fn keyrx_engine_start() -> i32 {
///     trace_ffi_call!("keyrx_engine_start", "engine", {
///         // Function body
///         start_engine()
///     })
/// }
/// ```
#[macro_export]
macro_rules! trace_ffi_call {
    ($name:expr, $domain:expr, $body:expr) => {{
        let event_id = $crate::ffi::tracing_export::emit_trace_event_start($name, $domain, &[]);
        let result = $body;
        let status = if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| result.is_ok()))
            .unwrap_or(false)
        {
            $crate::ffi::tracing_export::ResultStatus::Success
        } else {
            $crate::ffi::tracing_export::ResultStatus::Error
        };
        $crate::ffi::tracing_export::emit_trace_event_end(event_id, status);
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_buffer_circular() {
        let mut buffer = TraceBuffer::new();
        buffer.enable();

        // Fill buffer beyond capacity
        for i in 0..MAX_TRACE_EVENTS + 100 {
            let event = TraceEvent {
                id: 0,
                name: format!("event_{}", i),
                domain: "test".to_string(),
                start_time_us: current_time_us(),
                duration_us: Some(100),
                correlation_id: i as u64,
                args: HashMap::new(),
                result_status: ResultStatus::Success,
            };
            buffer.add_event(event);
        }

        // Should only have MAX_TRACE_EVENTS
        assert_eq!(buffer.events.len(), MAX_TRACE_EVENTS);
    }

    #[test]
    fn test_trace_enable_disable() {
        let mut buffer = TraceBuffer::new();
        assert!(!buffer.is_enabled());

        buffer.enable();
        assert!(buffer.is_enabled());

        buffer.disable();
        assert!(!buffer.is_enabled());
    }

    #[test]
    fn test_trace_event_lifecycle() {
        let mut buffer = TraceBuffer::new();
        buffer.enable();

        let event = TraceEvent {
            id: 0,
            name: "test_function".to_string(),
            domain: "test".to_string(),
            start_time_us: current_time_us(),
            duration_us: None,
            correlation_id: 123,
            args: HashMap::new(),
            result_status: ResultStatus::Pending,
        };

        let event_id = buffer.add_event(event);
        assert!(event_id > 0);

        // Update event
        buffer.update_event(event_id, |e| {
            e.duration_us = Some(500);
            e.result_status = ResultStatus::Success;
        });

        let events = buffer.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].duration_us, Some(500));
        assert_eq!(events[0].result_status, ResultStatus::Success);
    }

    #[test]
    fn test_filtered_events() {
        let mut buffer = TraceBuffer::new();
        buffer.enable();

        // Add events from different domains
        for i in 0..10 {
            let domain = if i % 2 == 0 { "engine" } else { "device" };
            let event = TraceEvent {
                id: 0,
                name: format!("event_{}", i),
                domain: domain.to_string(),
                start_time_us: current_time_us(),
                duration_us: Some(i * 100),
                correlation_id: i,
                args: HashMap::new(),
                result_status: ResultStatus::Success,
            };
            buffer.add_event(event);
        }

        // Filter by domain
        let engine_events = buffer.get_filtered_events(|e| e.domain == "engine");
        assert_eq!(engine_events.len(), 5);

        // Filter by duration
        let slow_events = buffer.get_filtered_events(|e| e.duration_us.is_some_and(|d| d >= 500));
        assert_eq!(slow_events.len(), 5);
    }

    #[test]
    fn test_emit_trace_event_integration() {
        // Enable tracing
        keyrx_trace_enable();
        assert!(keyrx_trace_is_enabled());

        // Clear any existing events
        keyrx_trace_clear();
        assert_eq!(keyrx_trace_get_count(), 0);

        // Emit a trace event
        let event_id = emit_trace_event_start(
            "test_function",
            "test",
            &[("arg1", "value1"), ("arg2", "value2")],
        );
        assert!(event_id > 0);

        // Should have 1 pending event
        assert_eq!(keyrx_trace_get_count(), 1);

        // Complete the event
        emit_trace_event_end(event_id, ResultStatus::Success);

        // Verify event was updated
        let json = keyrx_trace_get_events_json();
        assert!(!json.is_null());

        let json_str = unsafe { std::ffi::CStr::from_ptr(json).to_str().unwrap() };
        let events: Vec<TraceEvent> = serde_json::from_str(json_str).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name, "test_function");
        assert_eq!(events[0].result_status, ResultStatus::Success);
        assert!(events[0].duration_us.is_some());

        unsafe { keyrx_trace_free_json(json) };

        // Disable tracing
        keyrx_trace_disable();
        assert!(!keyrx_trace_is_enabled());
    }
}
