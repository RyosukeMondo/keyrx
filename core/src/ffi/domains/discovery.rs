//! Discovery domain FFI implementation.
//!
//! Implements the FfiExportable trait for discovery session management.
//! This module replaces the global static pattern from exports_discovery.rs
//! with instance-scoped state management through FfiContext.
#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]

use crate::discovery::session::{set_session_update_sink, SessionUpdate};
use crate::discovery::{DeviceId, DiscoverySession, DiscoverySummary};
use crate::engine::InputEvent;
use crate::ffi::context::FfiContext;
use crate::ffi::domains::engine::global_event_registry;
use crate::ffi::error::{serialize_ffi_result, FfiError, FfiResult};
use crate::ffi::events::EventType;
use crate::ffi::traits::FfiExportable;
use std::ffi::{c_char, CStr, CString};
use std::sync::{Mutex, OnceLock};

/// Global discovery session for the running engine loop to feed into
static DISCOVERY_SESSION: OnceLock<Mutex<Option<DiscoverySession>>> = OnceLock::new();

fn get_session_mutex() -> &'static Mutex<Option<DiscoverySession>> {
    DISCOVERY_SESSION.get_or_init(|| Mutex::new(None))
}

/// Process an input event for the active discovery session (if any)
pub fn process_discovery_event(event: &InputEvent) {
    if let Some(mutex) = DISCOVERY_SESSION.get() {
        if let Ok(mut guard) = mutex.lock() {
            if let Some(session) = guard.as_mut() {
                session.handle_event(event);
            }
        }
    }
}

/// Discovery domain FFI implementation.
pub struct DiscoveryFfi;

impl FfiExportable for DiscoveryFfi {
    const DOMAIN: &'static str = "discovery";

    fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
        if ctx.has_domain(Self::DOMAIN) {
            return Err(FfiError::invalid_input(
                "discovery domain already initialized",
            ));
        }
        // Initialize common sinks
        set_session_update_sink(Some(std::sync::Arc::new(|update| {
            // Convert session update to FFI event
            let (event_type, payload) = match update {
                SessionUpdate::Progress(p) => {
                    (EventType::DiscoveryProgress, serde_json::to_value(p).ok())
                }
                SessionUpdate::Finished(s) => {
                    (EventType::DiscoverySummary, serde_json::to_value(s).ok())
                }
                SessionUpdate::Duplicate(d) => {
                    (EventType::DiscoveryDuplicate, serde_json::to_value(d).ok())
                }
                SessionUpdate::Ignored => (EventType::DiscoveryProgress, None), // Or skip
            };

            if let Some(payload) = payload {
                global_event_registry().invoke(event_type, &payload);
            }
        })));

        Ok(())
    }

    fn cleanup(ctx: &mut FfiContext) {
        // Remove discovery sink
        set_session_update_sink(None);
        // Clear session
        if let Some(mutex) = DISCOVERY_SESSION.get() {
            if let Ok(mut guard) = mutex.lock() {
                *guard = None;
            }
        }
        // Remove domain state
        ctx.remove_domain(Self::DOMAIN);
    }
}

// ─── Helpers ───────────────────────────────────────────────────────────────────

fn ffi_json<T: serde::Serialize>(result: FfiResult<T>) -> *mut c_char {
    let payload = serialize_ffi_result(&result).unwrap_or_else(|e| {
        format!("error:{{\"code\":\"SERIALIZATION_FAILED\",\"message\":\"{e}\"}}")
    });
    CString::new(payload)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

fn cstr_to_str(ptr: *const c_char) -> Result<&'static str, FfiError> {
    if ptr.is_null() {
        return Err(FfiError::internal("null pointer"));
    }
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .map_err(|_| FfiError::internal("invalid utf8"))
    }
}

// ─── FFI Exports ───────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct StartDiscoveryParams {
    device_id: String,
    rows: u8,
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_discovery_start_discovery(
    params_json: *const c_char,
) -> *mut c_char {
    let process = || -> FfiResult<DiscoverySummary> {
        let json_str = cstr_to_str(params_json)?;
        let params: StartDiscoveryParams = serde_json::from_str(json_str)
            .map_err(|e| FfiError::invalid_input(format!("invalid json: {}", e)))?;

        let (vendor, product) = parse_device_id(&params.device_id)?;
        let device_id = DeviceId::new(vendor, product);

        // Default to 32 columns if only rows provided (contract limitation)
        let cols_per_row = vec![32; params.rows as usize];

        let session = DiscoverySession::new(device_id, params.rows, cols_per_row)
            .map_err(|e| FfiError::invalid_input(e.to_string()))?;

        // Initialize sink if not already (redundant if init called, but safe)
        // Actually, init is called via context, but exports might skip it?
        // Let's ensure sink is hooked up.
        set_session_update_sink(Some(std::sync::Arc::new(|update| {
            let (event_type, payload) = match update {
                SessionUpdate::Progress(p) => {
                    (EventType::DiscoveryProgress, serde_json::to_value(p).ok())
                }
                SessionUpdate::Finished(s) => {
                    (EventType::DiscoverySummary, serde_json::to_value(s).ok())
                }
                SessionUpdate::Duplicate(d) => {
                    (EventType::DiscoveryDuplicate, serde_json::to_value(d).ok())
                }
                SessionUpdate::Ignored => (EventType::DiscoveryProgress, None),
            };
            if let Some(payload) = payload {
                global_event_registry().invoke(event_type, &payload);
            }
        })));

        let summary = session.summary();

        // Store session
        if let Ok(mut guard) = get_session_mutex().lock() {
            *guard = Some(session);
        }

        Ok(summary)
    };

    ffi_json(process())
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_discovery_stop_discovery() -> *mut c_char {
    let process = || -> FfiResult<DiscoverySummary> {
        if let Ok(mut guard) = get_session_mutex().lock() {
            if let Some(session) = guard.as_mut() {
                let summary = session.cancel("User requested stop");
                // The cancel method already potentially emits event if we were pumping events,
                // but here we proactively return summary.
                return Ok(summary);
            }
        }
        Err(FfiError::not_found("No active discovery session"))
    };
    ffi_json(process())
}

#[no_mangle]
pub unsafe extern "C" fn keyrx_discovery_get_discovery_status() -> *mut c_char {
    let process = || -> FfiResult<DiscoverySummary> {
        if let Ok(guard) = get_session_mutex().lock() {
            if let Some(session) = guard.as_ref() {
                return Ok(session.summary());
            }
        }
        Err(FfiError::not_found("No active discovery session"))
    };
    ffi_json(process())
}

fn parse_device_id(id: &str) -> FfiResult<(u16, u16)> {
    let parts: Vec<&str> = id.split(':').collect();
    if parts.len() != 2 {
        return Err(FfiError::invalid_input(
            "Invalid device ID format (expected vendor:product)",
        ));
    }
    let vendor = u16::from_str_radix(parts[0], 16)
        .map_err(|_| FfiError::invalid_input("Invalid vendor ID hex"))?;
    let product = u16::from_str_radix(parts[1], 16)
        .map_err(|_| FfiError::invalid_input("Invalid product ID hex"))?;
    Ok((vendor, product))
}
