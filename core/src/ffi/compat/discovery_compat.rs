//! Backward-compatible shims for Discovery FFI callbacks.
//!
//! These functions maintain the existing API for Flutter code while
//! forwarding to the new unified EventRegistry system. All functions
//! are marked as deprecated to guide migration to the new API.
//!
//! # Migration Guide
//!
//! **Old API (deprecated):**
//! ```c
//! // Register separate callback functions
//! keyrx_on_discovery_progress(my_progress_callback);
//! keyrx_on_discovery_duplicate(my_duplicate_callback);
//! keyrx_on_discovery_summary(my_summary_callback);
//! ```
//!
//! **New API (recommended):**
//! ```c
//! // Register via unified EventRegistry
//! keyrx_register_event_callback(DiscoveryProgress, my_callback);
//! keyrx_register_event_callback(DiscoveryDuplicate, my_callback);
//! keyrx_register_event_callback(DiscoverySummary, my_callback);
//! ```
//!
//! The new API provides:
//! - Single callback registration function
//! - Consistent event type enum
//! - Better discoverability
//! - Future extensibility

#![allow(unsafe_code)]

use crate::ffi::domains::discovery::{global_event_registry, refresh_discovery_sink};
use crate::ffi::events::{EventCallback, EventType};

// ─── Discovery Callbacks (Deprecated) ──────────────────────────────────────

/// Register a callback for discovery progress updates.
///
/// # Deprecation Notice
///
/// This function is deprecated. Use the unified event registration API instead:
/// ```c
/// keyrx_register_event_callback(DiscoveryProgress, callback);
/// ```
///
/// The provided pointer/length pair references a JSON payload that is only
/// valid for the duration of the callback.
///
/// # Arguments
/// * `callback` - Optional callback function. Pass NULL to unregister.
#[deprecated(
    since = "0.2.0",
    note = "Use keyrx_register_event_callback(DiscoveryProgress, callback) instead"
)]
#[no_mangle]
pub extern "C" fn keyrx_on_discovery_progress(callback: Option<EventCallback>) {
    global_event_registry().register(EventType::DiscoveryProgress, callback);
    refresh_discovery_sink();
}

/// Register a callback for duplicate key warnings during discovery.
///
/// # Deprecation Notice
///
/// This function is deprecated. Use the unified event registration API instead:
/// ```c
/// keyrx_register_event_callback(DiscoveryDuplicate, callback);
/// ```
///
/// The provided pointer/length pair references a JSON payload that is only
/// valid for the duration of the callback.
///
/// # Arguments
/// * `callback` - Optional callback function. Pass NULL to unregister.
#[deprecated(
    since = "0.2.0",
    note = "Use keyrx_register_event_callback(DiscoveryDuplicate, callback) instead"
)]
#[no_mangle]
pub extern "C" fn keyrx_on_discovery_duplicate(callback: Option<EventCallback>) {
    global_event_registry().register(EventType::DiscoveryDuplicate, callback);
    refresh_discovery_sink();
}

/// Register a callback for discovery summaries (completed, cancelled, or bypassed).
///
/// # Deprecation Notice
///
/// This function is deprecated. Use the unified event registration API instead:
/// ```c
/// keyrx_register_event_callback(DiscoverySummary, callback);
/// ```
///
/// The provided pointer/length pair references a JSON payload that is only
/// valid for the duration of the callback.
///
/// # Arguments
/// * `callback` - Optional callback function. Pass NULL to unregister.
#[deprecated(
    since = "0.2.0",
    note = "Use keyrx_register_event_callback(DiscoverySummary, callback) instead"
)]
#[no_mangle]
pub extern "C" fn keyrx_on_discovery_summary(callback: Option<EventCallback>) {
    global_event_registry().register(EventType::DiscoverySummary, callback);
    refresh_discovery_sink();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::slice;
    use std::sync::{Mutex, OnceLock};

    fn test_store() -> &'static Mutex<Vec<Vec<u8>>> {
        static STORE: OnceLock<Mutex<Vec<Vec<u8>>>> = OnceLock::new();
        STORE.get_or_init(|| Mutex::new(Vec::new()))
    }

    unsafe extern "C" fn test_callback(ptr: *const u8, len: usize) {
        let slice = slice::from_raw_parts(ptr, len);
        test_store().lock().unwrap().push(slice.to_vec());
    }

    #[test]
    #[serial]
    fn deprecated_progress_callback_still_works() {
        test_store().lock().unwrap().clear();

        // Clear any previous registrations
        let registry = global_event_registry();
        registry.clear();

        // Even though deprecated, the function should still work
        #[allow(deprecated)]
        keyrx_on_discovery_progress(Some(test_callback));

        assert!(registry.is_registered(EventType::DiscoveryProgress));

        // Unregister
        #[allow(deprecated)]
        keyrx_on_discovery_progress(None);
        assert!(!registry.is_registered(EventType::DiscoveryProgress));
    }

    #[test]
    #[serial]
    fn deprecated_duplicate_callback_still_works() {
        test_store().lock().unwrap().clear();

        let registry = global_event_registry();
        registry.clear();

        #[allow(deprecated)]
        keyrx_on_discovery_duplicate(Some(test_callback));

        assert!(registry.is_registered(EventType::DiscoveryDuplicate));

        #[allow(deprecated)]
        keyrx_on_discovery_duplicate(None);
        assert!(!registry.is_registered(EventType::DiscoveryDuplicate));
    }

    #[test]
    #[serial]
    fn deprecated_summary_callback_still_works() {
        test_store().lock().unwrap().clear();

        let registry = global_event_registry();
        registry.clear();

        #[allow(deprecated)]
        keyrx_on_discovery_summary(Some(test_callback));

        assert!(registry.is_registered(EventType::DiscoverySummary));

        #[allow(deprecated)]
        keyrx_on_discovery_summary(None);
        assert!(!registry.is_registered(EventType::DiscoverySummary));
    }

    #[test]
    #[serial]
    fn deprecated_callbacks_refresh_sink() {
        let registry = global_event_registry();
        registry.clear();

        // Test that calling deprecated functions still refreshes the discovery sink
        #[allow(deprecated)]
        keyrx_on_discovery_progress(Some(test_callback));

        // The sink should be set because we registered a callback
        // (This is tested indirectly - the sink routing is tested in domains/discovery.rs)

        #[allow(deprecated)]
        keyrx_on_discovery_progress(None);
    }
}
