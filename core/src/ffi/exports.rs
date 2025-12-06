//! Core C-ABI exports for FFI.
//!
//! This module provides core init/common C-compatible functions for FFI integration.
//! Unsafe code is required for FFI interoperability.
#![allow(unsafe_code)]

use std::ffi::{c_char, CString};
use std::path::PathBuf;
use std::sync::Arc;

use crate::config;
use crate::definitions::DeviceDefinitionLibrary;
use crate::ffi::domains::discovery::global_event_registry;
use crate::ffi::domains::engine::global_event_registry as engine_event_registry;
use crate::ffi::events::{EventCallback, EventType};
use crate::ffi::runtime::{
    clear_revolutionary_runtime, set_revolutionary_runtime, RevolutionaryRuntime,
};
use crate::registry::ProfileRegistry;

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

// ---------------------------------------------------------------------------
// Revolutionary runtime lifecycle (for FFI consumers like Flutter)
// ---------------------------------------------------------------------------

fn default_device_definitions() -> Arc<DeviceDefinitionLibrary> {
    let mut library = DeviceDefinitionLibrary::new();
    let mut loaded = 0usize;

    // Preferred search paths (in order):
    // 1) cwd/device_definitions
    // 2) config dir: ~/.config/keyrx/device_definitions
    let mut paths: Vec<PathBuf> = vec![];
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("device_definitions"));
    }
    paths.push(config::config_dir().join("device_definitions"));

    for path in paths {
        if path.exists() {
            match library.load_from_directory(&path) {
                Ok(count) => {
                    loaded += count;
                    tracing::info!(
                        service = "keyrx",
                        component = "ffi_exports",
                        event = "device_definitions_loaded",
                        path = %path.display(),
                        count,
                        "Loaded device definitions for FFI runtime"
                    );
                }
                Err(err) => {
                    tracing::warn!(
                        service = "keyrx",
                        component = "ffi_exports",
                        event = "device_definitions_load_failed",
                        path = %path.display(),
                        error = %err,
                        "Failed to load device definitions for FFI runtime"
                    );
                }
            }
        }
    }

    if loaded == 0 {
        tracing::warn!(
            service = "keyrx",
            component = "ffi_exports",
            event = "device_definitions_empty",
            "No device definitions loaded for FFI runtime; definition calls may return NOT_FOUND"
        );
    }

    Arc::new(library)
}

fn init_revolutionary_runtime() -> i32 {
    // Create registries using default locations.
    let (device_registry, _rx) = crate::registry::DeviceRegistry::new();
    let profile_registry = Arc::new(ProfileRegistry::new());
    let device_definitions = default_device_definitions();

    match set_revolutionary_runtime(RevolutionaryRuntime::new(
        device_registry,
        profile_registry,
        device_definitions,
    )) {
        Ok(_) => {
            tracing::info!(
                service = "keyrx",
                component = "ffi_exports",
                event = "revolutionary_runtime_initialized",
                "Revolutionary runtime initialized for FFI consumers"
            );
            0
        }
        Err(err) => {
            tracing::error!(
                service = "keyrx",
                component = "ffi_exports",
                event = "revolutionary_runtime_init_failed",
                error = %err,
                "Failed to initialize revolutionary runtime for FFI consumers"
            );
            -1
        }
    }
}

fn shutdown_revolutionary_runtime() -> i32 {
    match clear_revolutionary_runtime() {
        Ok(_) => {
            tracing::info!(
                service = "keyrx",
                component = "ffi_exports",
                event = "revolutionary_runtime_cleared",
                "Revolutionary runtime cleared for FFI consumers"
            );
            0
        }
        Err(err) => {
            tracing::error!(
                service = "keyrx",
                component = "ffi_exports",
                event = "revolutionary_runtime_clear_failed",
                error = %err,
                "Failed to clear revolutionary runtime for FFI consumers"
            );
            -1
        }
    }
}

/// Initialize the revolutionary runtime for FFI consumers (e.g., Flutter).
///
/// Returns 0 on success, negative on failure.
#[no_mangle]
pub extern "C" fn keyrx_revolutionary_runtime_init() -> i32 {
    std::panic::catch_unwind(init_revolutionary_runtime).unwrap_or(-2)
}

/// Shutdown/clear the revolutionary runtime.
///
/// Returns 0 on success, negative on failure.
#[no_mangle]
pub extern "C" fn keyrx_revolutionary_runtime_shutdown() -> i32 {
    std::panic::catch_unwind(shutdown_revolutionary_runtime).unwrap_or(-2)
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

    let registry = match event_type {
        EventType::EngineState => engine_event_registry(),
        _ => global_event_registry(),
    };

    registry.register(event_type, callback);

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
