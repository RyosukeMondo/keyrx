//! Discovery session FFI exports.
//!
//! Functions for device key discovery session management.
//!
//! # Migration Notice
//!
//! Discovery callback registration functions have been moved to the compat module.
//! They are still available for backward compatibility but are deprecated.
//! See `crate::ffi::compat::discovery_compat` for details.
#![allow(unsafe_code)]

// Note: Discovery callback registration functions (keyrx_on_discovery_progress, etc.)
// have been moved to crate::ffi::compat::discovery_compat for backward compatibility.
// They are re-exported from the main ffi module.

// ─── Discovery FFI Exports ─────────────────────────────────────────────────

// NOTE: The following functions have been migrated to domains/discovery.rs with #[ffi_export]
// They are temporarily commented out to avoid symbol conflicts during migration.
// These will be removed completely in a later task.

/* MIGRATED TO domains/discovery.rs - DO NOT USE
/// Start a discovery session for a device.
///
/// # Arguments
/// * `device_id` - Device identifier as "vendorId:productId" (e.g., "1234:5678")
/// * `rows` - Number of rows in the keyboard layout
/// * `cols_per_row_json` - JSON array of column counts per row (e.g., "[14, 14, 13, 12, 8]")
///
/// Returns JSON: `ok:{success: bool, error?: string, totalKeys?: number}`
///
/// Progress updates are delivered via the callbacks registered with:
/// - `keyrx_on_discovery_progress`
/// - `keyrx_on_discovery_duplicate`
/// - `keyrx_on_discovery_summary`
///
/// Call `keyrx_process_discovery_event` to process input events.
/// Call `keyrx_cancel_discovery` to cancel the session.
///
/// Caller must free with `keyrx_free_string`.
///
/// # Safety
/// `device_id` and `cols_per_row_json` must be valid null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn keyrx_start_discovery(
    device_id: *const c_char,
    rows: u8,
    cols_per_row_json: *const c_char,
) -> *mut c_char {
    if device_id.is_null() || cols_per_row_json.is_null() {
        let result = DiscoveryStartResult {
            success: false,
            error: Some("null pointer".to_string()),
            total_keys: None,
        };
        let payload = serde_json::to_string(&result)
            .map(|json| format!("ok:{json}"))
            .unwrap_or_else(|err| format!("error:{err}"));
        return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
    }

    let device_id_str = match CStr::from_ptr(device_id).to_str() {
        Ok(s) => s,
        Err(_) => {
            let result = DiscoveryStartResult {
                success: false,
                error: Some("invalid utf8 in device_id".to_string()),
                total_keys: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let cols_str = match CStr::from_ptr(cols_per_row_json).to_str() {
        Ok(s) => s,
        Err(_) => {
            let result = DiscoveryStartResult {
                success: false,
                error: Some("invalid utf8 in cols_per_row_json".to_string()),
                total_keys: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    // Parse device ID (format: "vendorId:productId")
    let parts: Vec<&str> = device_id_str.split(':').collect();
    let (vendor_id, product_id) = if parts.len() == 2 {
        let v = u16::from_str_radix(parts[0], 16).unwrap_or(0);
        let p = u16::from_str_radix(parts[1], 16).unwrap_or(0);
        (v, p)
    } else {
        let result = DiscoveryStartResult {
            success: false,
            error: Some("device_id must be 'vendorId:productId' (hex)".to_string()),
            total_keys: None,
        };
        let payload = serde_json::to_string(&result)
            .map(|json| format!("ok:{json}"))
            .unwrap_or_else(|err| format!("error:{err}"));
        return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
    };

    // Parse cols_per_row JSON array
    let cols_per_row: Vec<u8> = match serde_json::from_str(cols_str) {
        Ok(cols) => cols,
        Err(err) => {
            let result = DiscoveryStartResult {
                success: false,
                error: Some(format!("invalid cols_per_row JSON: {err}")),
                total_keys: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    // Find device by vendor/product ID
    let devices = match crate::drivers::list_keyboards() {
        Ok(d) => d,
        Err(err) => {
            let result = DiscoveryStartResult {
                success: false,
                error: Some(format!("failed to list devices: {err}")),
                total_keys: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let device = devices
        .iter()
        .find(|d| d.vendor_id == vendor_id && d.product_id == product_id);
    let device_path = match device {
        Some(d) => d.path.clone(),
        None => {
            let result = DiscoveryStartResult {
                success: false,
                error: Some(format!(
                    "device {:04x}:{:04x} not found",
                    vendor_id, product_id
                )),
                total_keys: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    // Create discovery session
    let dev_id = crate::discovery::DeviceId::new(vendor_id, product_id);
    let session = match crate::discovery::DiscoverySession::new(dev_id, rows, cols_per_row) {
        Ok(s) => s,
        Err(err) => {
            let result = DiscoveryStartResult {
                success: false,
                error: Some(format!("invalid layout: {err}")),
                total_keys: None,
            };
            let payload = serde_json::to_string(&result)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            return CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    let total_keys = session.progress().total;

    // Store session state
    DISCOVERY_CANCEL_FLAG.store(false, Ordering::SeqCst);
    if let Ok(mut guard) = discovery_session_slot().lock() {
        *guard = Some(DiscoverySessionState {
            session,
            device_path,
        });
    }

    let result = DiscoveryStartResult {
        success: true,
        error: None,
        total_keys: Some(total_keys),
    };
    let payload = serde_json::to_string(&result)
        .map(|json| format!("ok:{json}"))
        .unwrap_or_else(|err| format!("error:{err}"));

    CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
}

/// Process an input event during discovery.
///
/// # Arguments
/// * `scan_code` - The scan code of the key event
/// * `pressed` - Whether the key was pressed (true) or released (false)
/// * `timestamp_us` - Timestamp in microseconds
///
/// Returns:
/// - 0: Event processed successfully
/// - 1: Discovery session completed
/// - -1: No active discovery session
/// - -2: Discovery was cancelled
///
/// Progress/duplicate/summary callbacks will be invoked as appropriate.
#[no_mangle]
pub extern "C" fn keyrx_process_discovery_event(
    scan_code: u16,
    pressed: bool,
    timestamp_us: u64,
) -> i32 {
    if DISCOVERY_CANCEL_FLAG.load(Ordering::SeqCst) {
        return -2;
    }

    let mut guard = match discovery_session_slot().lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };

    let state = match guard.as_mut() {
        Some(s) => s,
        None => return -1,
    };

    let device_path_str = state.device_path.to_str().map(String::from);
    let event = crate::engine::InputEvent::with_metadata(
        crate::engine::KeyCode::Unknown(scan_code),
        pressed,
        timestamp_us,
        device_path_str,
        false,
        false,
        scan_code,
    );

    match state.session.handle_event(&event) {
        crate::discovery::SessionUpdate::Finished(_) => {
            // Clear session after completion
            *guard = None;
            1
        }
        _ => 0,
    }
}

/// Cancel an ongoing discovery session.
///
/// Returns:
/// - 0: Discovery cancelled successfully
/// - -1: No active discovery session
#[no_mangle]
pub extern "C" fn keyrx_cancel_discovery() -> i32 {
    DISCOVERY_CANCEL_FLAG.store(true, Ordering::SeqCst);

    let mut guard = match discovery_session_slot().lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };

    match guard.take() {
        Some(mut state) => {
            let summary = state.session.cancel("cancelled by user");
            // Publish the cancellation through callbacks
            crate::discovery::session::publish_session_update(
                &crate::discovery::SessionUpdate::Finished(summary),
            );
            0
        }
        None => -1,
    }
}

/// Get the current discovery progress.
///
/// Returns JSON: `ok:{captured, total, next?: {row, col}}` or `error:...`
///
/// Caller must free with `keyrx_free_string`.
#[no_mangle]
pub extern "C" fn keyrx_get_discovery_progress() -> *mut c_char {
    let guard = match discovery_session_slot().lock() {
        Ok(g) => g,
        Err(_) => {
            return CString::new("error:lock poisoned")
                .map_or_else(|_| ptr::null_mut(), CString::into_raw);
        }
    };

    match guard.as_ref() {
        Some(state) => {
            let progress = state.session.progress();
            let payload = serde_json::to_string(&progress)
                .map(|json| format!("ok:{json}"))
                .unwrap_or_else(|err| format!("error:{err}"));
            CString::new(payload).map_or_else(|_| ptr::null_mut(), CString::into_raw)
        }
        None => CString::new("error:no active discovery session")
            .map_or_else(|_| ptr::null_mut(), CString::into_raw),
    }
}
*/
// END MIGRATED TO domains/discovery.rs

// ─── Test Helpers ──────────────────────────────────────────────────────────

#[cfg(test)]
pub(crate) fn clear_discovery_session() {
    // Clear the new global context state
    use crate::ffi::domains::discovery::{global_discovery_context, DiscoveryFfi};
    use crate::ffi::traits::FfiExportable;
    if let Ok(mut ctx_guard) = global_discovery_context().lock() {
        if let Some(ctx) = ctx_guard.as_mut() {
            if ctx.has_domain(DiscoveryFfi::DOMAIN) {
                if let Some(mut state_guard) = ctx.get_domain_mut::<Option<
                    crate::ffi::domains::discovery::DiscoverySessionState,
                >>(DiscoveryFfi::DOMAIN)
                {
                    if let Some(state_opt) = state_guard
                        .downcast_mut::<Option<crate::ffi::domains::discovery::DiscoverySessionState>>()
                    {
                        *state_opt = None;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::clear_discovery_session;
    use crate::discovery::{
        session::publish_session_update, DeviceId, DiscoveryProgress, DiscoverySummary,
        ExpectedPosition, PhysicalKey, SessionStatus, SessionUpdate,
    };
    use crate::ffi::keyrx_free_string;
    // Import the new FFI functions from domains/discovery
    use crate::ffi::domains::discovery::{
        keyrx_cancel_discovery, keyrx_get_discovery_progress, keyrx_process_discovery_event,
        keyrx_start_discovery,
    };
    // Import deprecated compat functions for testing
    use crate::ffi::compat::{keyrx_on_discovery_progress, keyrx_on_discovery_summary};
    use serial_test::serial;
    use std::collections::HashMap;
    use std::ffi::{CStr, CString};
    use std::ptr;
    use std::slice;
    use std::sync::{Mutex, OnceLock};

    fn progress_store() -> &'static Mutex<Vec<Vec<u8>>> {
        static STORE: OnceLock<Mutex<Vec<Vec<u8>>>> = OnceLock::new();
        STORE.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn summary_store() -> &'static Mutex<Vec<Vec<u8>>> {
        static STORE: OnceLock<Mutex<Vec<Vec<u8>>>> = OnceLock::new();
        STORE.get_or_init(|| Mutex::new(Vec::new()))
    }

    unsafe extern "C" fn record_progress(ptr: *const u8, len: usize) {
        let slice = slice::from_raw_parts(ptr, len);
        progress_store().lock().unwrap().push(slice.to_vec());
    }

    unsafe extern "C" fn record_summary(ptr: *const u8, len: usize) {
        let slice = slice::from_raw_parts(ptr, len);
        summary_store().lock().unwrap().push(slice.to_vec());
    }

    #[test]
    #[allow(deprecated)]
    fn discovery_callbacks_emit_json_payloads() {
        progress_store().lock().unwrap().clear();
        summary_store().lock().unwrap().clear();

        keyrx_on_discovery_progress(Some(record_progress));
        keyrx_on_discovery_summary(Some(record_summary));

        let progress = SessionUpdate::Progress(DiscoveryProgress {
            captured: 1,
            total: 3,
            next: Some(ExpectedPosition { row: 0, col: 1 }),
        });
        publish_session_update(&progress);

        let mut keymap = HashMap::new();
        keymap.insert(10, PhysicalKey::new(10, 0, 0));

        let mut aliases = HashMap::new();
        aliases.insert("r0_c0".to_string(), 10);

        let summary = SessionUpdate::Finished(DiscoverySummary {
            device_id: DeviceId::new(0x1, 0x2),
            status: SessionStatus::Completed,
            message: None,
            rows: 1,
            cols_per_row: vec![1],
            captured: 1,
            total: 1,
            next: None,
            unmapped: vec![],
            duplicates: vec![],
            keymap,
            aliases,
        });
        publish_session_update(&summary);

        let progress_payloads = progress_store().lock().unwrap();
        assert_eq!(progress_payloads.len(), 1);
        let progress_json: DiscoveryProgress =
            serde_json::from_slice(&progress_payloads[0]).expect("valid progress json");
        assert_eq!(progress_json.captured, 1);
        assert_eq!(progress_json.total, 3);

        let summary_payloads = summary_store().lock().unwrap();
        assert_eq!(summary_payloads.len(), 1);
        let summary_json: DiscoverySummary =
            serde_json::from_slice(&summary_payloads[0]).expect("valid summary json");
        assert_eq!(summary_json.status, SessionStatus::Completed);
        assert_eq!(summary_json.device_id.vendor_id, 0x1);

        keyrx_on_discovery_progress(None);
        keyrx_on_discovery_summary(None);
    }

    #[test]
    #[serial]
    fn start_discovery_null_pointers() {
        clear_discovery_session();

        let ptr = unsafe { keyrx_start_discovery(ptr::null(), 1, ptr::null()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        // New implementation returns error: for null pointers (caught by macro)
        if raw.starts_with("error:") {
            let json_str = &raw["error:".len()..];
            let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert!(
                result["message"].as_str().unwrap().contains("null pointer")
                    || result["message"].as_str().unwrap().contains("device_id")
            );
        } else {
            assert!(raw.starts_with("ok:"));
            let json_str = &raw["ok:".len()..];
            let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
            assert_eq!(result["success"], false);
            assert!(result["error"].as_str().unwrap().contains("null pointer"));
        }
    }

    #[test]
    #[serial]
    fn start_discovery_invalid_device_id_format() {
        clear_discovery_session();

        let device_id = CString::new("invalid").unwrap();
        let cols = CString::new("[3, 3]").unwrap();
        let ptr = unsafe { keyrx_start_discovery(device_id.as_ptr(), 2, cols.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["success"], false);
        assert!(result["error"]
            .as_str()
            .unwrap()
            .contains("vendorId:productId"));
    }

    #[test]
    #[serial]
    fn start_discovery_invalid_cols_json() {
        clear_discovery_session();

        let device_id = CString::new("0001:0002").unwrap();
        let cols = CString::new("not valid json").unwrap();
        let ptr = unsafe { keyrx_start_discovery(device_id.as_ptr(), 2, cols.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["success"], false);
        assert!(result["error"]
            .as_str()
            .unwrap()
            .contains("invalid cols_per_row JSON"));
    }

    #[test]
    #[serial]
    fn start_discovery_invalid_layout() {
        clear_discovery_session();

        // rows=2 but cols_per_row has 3 elements - mismatch
        let device_id = CString::new("0001:0002").unwrap();
        let cols = CString::new("[3, 3, 3]").unwrap();
        let ptr = unsafe { keyrx_start_discovery(device_id.as_ptr(), 2, cols.as_ptr()) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("ok:"));
        let json_str = &raw["ok:".len()..];
        let result: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(result["success"], false);
        // Either "device not found" or "invalid layout" depending on device availability
        assert!(result["error"].is_string());
    }

    #[test]
    #[serial]
    fn cancel_discovery_no_active_session() {
        clear_discovery_session();
        let ptr = unsafe { keyrx_cancel_discovery() };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        // New implementation returns JSON: ok:-1
        assert!(raw.starts_with("ok:"));
        let json_str = &raw[3..];
        let result: i32 = serde_json::from_str(json_str).unwrap();
        assert_eq!(result, -1);
    }

    #[test]
    #[serial]
    fn get_discovery_progress_no_active_session() {
        clear_discovery_session();
        let ptr = unsafe { keyrx_get_discovery_progress() };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        assert!(raw.starts_with("error:"));
        assert!(raw.contains("no active discovery session"));
    }

    #[test]
    #[serial]
    fn process_discovery_event_no_active_session() {
        clear_discovery_session();
        let ptr = unsafe { keyrx_process_discovery_event(30, true, 1000) };
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { keyrx_free_string(ptr) };

        // New implementation returns JSON: ok:-1
        assert!(raw.starts_with("ok:"));
        let json_str = &raw[3..];
        let result: i32 = serde_json::from_str(json_str).unwrap();
        assert_eq!(result, -1);
    }
}
