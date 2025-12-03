//! Core C-ABI exports for FFI.
//!
//! This module provides core init/common C-compatible functions for FFI integration.
//! Unsafe code is required for FFI interoperability.
#![allow(unsafe_code)]

use std::ffi::{c_char, CString};

use crate::ffi::domains::discovery::global_event_registry;
use crate::ffi::events::{EventCallback, EventType};

/// Initialize the KeyRx engine.
///
/// # Safety
/// This function is safe to call from any thread.
#[no_mangle]
pub extern "C" fn keyrx_init() -> i32 {
    let _ = tracing_subscriber::fmt::try_init();
    tracing::info!(
        service = "keyrx",
        event = "ffi_init",
        component = "ffi_exports",
        status = "ok",
        "KeyRx Core initialized"
    );
    0 // Success
}

/// Get the version string.
///
/// # Safety
/// The returned pointer is valid until the next call to this function.
#[no_mangle]
pub extern "C" fn keyrx_version() -> *const c_char {
    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}

/// Free a string allocated by KeyRx.
///
/// # Safety
/// `ptr` must be a pointer returned by a KeyRx function, or null.
#[no_mangle]
pub unsafe extern "C" fn keyrx_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Register a unified event callback.
///
/// This is the new unified API for registering callbacks across all domains.
/// It replaces domain-specific callback registration functions.
///
/// # Event Types (by integer code)
/// - 0: DiscoveryProgress
/// - 1: DiscoveryDuplicate
/// - 2: DiscoverySummary
/// - 3: EngineState
/// - 4: ValidationProgress
/// - 5: ValidationResult
/// - 6: DeviceConnected
/// - 7: DeviceDisconnected
/// - 8: TestProgress
/// - 9: TestResult
/// - 10: AnalysisProgress
/// - 11: AnalysisResult
/// - 12: DiagnosticsLog
/// - 13: DiagnosticsMetric
/// - 14: RecordingStarted
/// - 15: RecordingStopped
///
/// # Arguments
/// * `event_type_code` - Integer code for the event type (see list above)
/// * `callback` - Optional callback function. Pass NULL to unregister.
///
/// # Returns
/// - 0: Success
/// - -1: Invalid event type code
///
/// # Safety
/// The callback function must be valid for the lifetime of the registration.
#[no_mangle]
pub extern "C" fn keyrx_register_event_callback(
    event_type_code: i32,
    callback: Option<EventCallback>,
) -> i32 {
    let event_type = match event_type_code {
        0 => EventType::DiscoveryProgress,
        1 => EventType::DiscoveryDuplicate,
        2 => EventType::DiscoverySummary,
        3 => EventType::EngineState,
        4 => EventType::ValidationProgress,
        5 => EventType::ValidationResult,
        6 => EventType::DeviceConnected,
        7 => EventType::DeviceDisconnected,
        8 => EventType::TestProgress,
        9 => EventType::TestResult,
        10 => EventType::AnalysisProgress,
        11 => EventType::AnalysisResult,
        12 => EventType::DiagnosticsLog,
        13 => EventType::DiagnosticsMetric,
        14 => EventType::RecordingStarted,
        15 => EventType::RecordingStopped,
        _ => {
            tracing::warn!(
                service = "keyrx",
                component = "ffi_exports",
                event = "invalid_event_type",
                code = event_type_code,
                "Invalid event type code provided to keyrx_register_event_callback"
            );
            return -1;
        }
    };

    global_event_registry().register(event_type, callback);

    // Refresh discovery sink if registering discovery events
    if matches!(
        event_type,
        EventType::DiscoveryProgress | EventType::DiscoveryDuplicate | EventType::DiscoverySummary
    ) {
        crate::ffi::domains::discovery::refresh_discovery_sink();
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::ptr;

    #[test]
    fn init_is_idempotent() {
        assert_eq!(keyrx_init(), 0);
        assert_eq!(keyrx_init(), 0);
    }

    #[test]
    fn version_matches_package_version() {
        let version = unsafe { CStr::from_ptr(keyrx_version()) }
            .to_str()
            .expect("version string should be valid UTF-8");

        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn free_string_handles_null_pointer() {
        unsafe {
            keyrx_free_string(ptr::null_mut());
        }
    }

    #[test]
    fn register_event_callback_accepts_valid_codes() {
        // Test valid event type codes
        for code in 0..=15 {
            assert_eq!(keyrx_register_event_callback(code, None), 0);
        }
    }

    #[test]
    fn register_event_callback_rejects_invalid_codes() {
        // Test invalid event type codes
        assert_eq!(keyrx_register_event_callback(-1, None), -1);
        assert_eq!(keyrx_register_event_callback(16, None), -1);
        assert_eq!(keyrx_register_event_callback(100, None), -1);
    }

    #[test]
    fn register_event_callback_registers_callback() {
        unsafe extern "C" fn test_cb(_ptr: *const u8, _len: usize) {}

        // Clear registry first
        global_event_registry().clear();

        // Register callback
        assert_eq!(keyrx_register_event_callback(0, Some(test_cb)), 0);
        assert!(global_event_registry().is_registered(EventType::DiscoveryProgress));

        // Unregister
        assert_eq!(keyrx_register_event_callback(0, None), 0);
        assert!(!global_event_registry().is_registered(EventType::DiscoveryProgress));
    }
}
