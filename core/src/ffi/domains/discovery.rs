//! Discovery domain FFI implementation.
#![allow(dead_code)] // TODO: Remove when #[ffi_export] is uncommented (task 20)
//!
//! Implements the FfiExportable trait for discovery session management.
//! This module replaces the global static pattern from exports_discovery.rs
//! with instance-scoped state management through FfiContext.
#![allow(unsafe_code)]

use crate::discovery::{session::set_session_update_sink, SessionUpdate};
use crate::ffi::context::FfiContext;
use crate::ffi::error::{FfiError, FfiResult};
use crate::ffi::events::{EventRegistry, EventType};
use crate::ffi::traits::FfiExportable;
// use keyrx_ffi_macros::ffi_export; // TODO: Uncomment when exports_*.rs files are removed (task 20)
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};

/// Discovery domain FFI implementation.
pub struct DiscoveryFfi;

/// Discovery session state for FFI.
#[derive(Debug)]
pub struct DiscoverySessionState {
    /// Active discovery session
    pub session: crate::discovery::DiscoverySession,
    /// Device path for the session
    pub device_path: PathBuf,
    /// Cancellation flag for the session
    pub cancel_flag: Arc<AtomicBool>,
}

impl FfiExportable for DiscoveryFfi {
    const DOMAIN: &'static str = "discovery";

    fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
        if ctx.has_domain(Self::DOMAIN) {
            return Err(FfiError::invalid_input(
                "discovery domain already initialized",
            ));
        }

        // Initialize discovery domain state
        // We don't create a session yet - it's created when start_discovery is called
        ctx.set_domain(Self::DOMAIN, Option::<DiscoverySessionState>::None);

        // Set up the discovery sink to route updates to callbacks
        refresh_discovery_sink();

        Ok(())
    }

    fn cleanup(ctx: &mut FfiContext) {
        // Cancel any active session before cleanup
        if let Some(mut state_guard) =
            ctx.get_domain_mut::<Option<DiscoverySessionState>>(Self::DOMAIN)
        {
            if let Some(state) = state_guard.downcast_mut::<Option<DiscoverySessionState>>() {
                if let Some(mut session_state) = state.take() {
                    // Cancel the session
                    session_state.cancel_flag.store(true, Ordering::SeqCst);
                    let summary = session_state.session.cancel("cleanup");
                    // Publish cancellation
                    crate::discovery::session::publish_session_update(&SessionUpdate::Finished(
                        summary,
                    ));
                }
            }
        }

        // Remove discovery sink
        set_session_update_sink(None);

        // Remove domain state
        ctx.remove_domain(Self::DOMAIN);
    }
}

impl DiscoveryFfi {
    // No helper methods needed yet - will be added when transitioning to handle-based access
}

/// Discovery start result for FFI JSON output.
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub struct DiscoveryStartResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(rename = "totalKeys", skip_serializing_if = "Option::is_none")]
    pub total_keys: Option<usize>,
}

/// Refresh the discovery sink based on registered callbacks.
pub(crate) fn refresh_discovery_sink() {
    let registry = global_event_registry();
    // Check if any discovery callbacks are registered
    if registry.is_registered(EventType::DiscoveryProgress)
        || registry.is_registered(EventType::DiscoveryDuplicate)
        || registry.is_registered(EventType::DiscoverySummary)
    {
        set_session_update_sink(Some(discovery_sink()));
    } else {
        set_session_update_sink(None);
    }
}

/// Create the discovery sink closure that routes updates to callbacks.
fn discovery_sink() -> Arc<dyn Fn(&SessionUpdate) + Send + Sync + 'static> {
    Arc::new(|update| {
        let registry = global_event_registry();
        match update {
            SessionUpdate::Ignored => {}
            SessionUpdate::Progress(progress) => {
                registry.invoke(EventType::DiscoveryProgress, progress);
            }
            SessionUpdate::Duplicate(dup) => {
                registry.invoke(EventType::DiscoveryDuplicate, dup);
            }
            SessionUpdate::Finished(summary) => {
                registry.invoke(EventType::DiscoverySummary, summary);
            }
        }
    })
}

// Note: Callback registration functions (keyrx_on_discovery_progress, etc.)
// are in exports_discovery.rs and now use EventRegistry for unified callback management.

// ─── FFI Exports ───────────────────────────────────────────────────────────

// Temporary global state - will be replaced with handle-based access in later tasks
use std::sync::{Mutex, OnceLock};

pub(crate) fn global_discovery_context() -> &'static Mutex<Option<FfiContext>> {
    static CONTEXT: OnceLock<Mutex<Option<FfiContext>>> = OnceLock::new();
    CONTEXT.get_or_init(|| Mutex::new(Some(FfiContext::new())))
}

/// Global event registry for Discovery domain.
/// This will be moved into FfiContext in a future refactor.
pub(crate) fn global_event_registry() -> &'static EventRegistry {
    static REGISTRY: OnceLock<EventRegistry> = OnceLock::new();
    REGISTRY.get_or_init(EventRegistry::new)
}

/// Start a discovery session for a device.
///
/// # Arguments
/// * `device_id` - Device identifier as "vendorId:productId" (e.g., "1234:5678")
/// * `rows` - Number of rows in the keyboard layout
/// * `cols_per_row_json` - JSON array of column counts per row (e.g., "[14, 14, 13, 12, 8]")
///
/// Returns JSON: `ok:{success: bool, error?: string, totalKeys?: number}`
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
fn start_discovery(
    device_id: &str,
    rows: u8,
    cols_per_row_json: &str,
) -> FfiResult<DiscoveryStartResult> {
    // Parse device ID (format: "vendorId:productId")
    let parts: Vec<&str> = device_id.split(':').collect();
    let (vendor_id, product_id) = if parts.len() == 2 {
        match (
            u16::from_str_radix(parts[0], 16),
            u16::from_str_radix(parts[1], 16),
        ) {
            (Ok(v), Ok(p)) => (v, p),
            _ => {
                return Ok(DiscoveryStartResult {
                    success: false,
                    error: Some("invalid vendor_id or product_id hex format".to_string()),
                    total_keys: None,
                });
            }
        }
    } else {
        return Ok(DiscoveryStartResult {
            success: false,
            error: Some("device_id must be 'vendorId:productId' (hex)".to_string()),
            total_keys: None,
        });
    };

    // Parse cols_per_row JSON array
    let cols_per_row: Vec<u8> = match serde_json::from_str(cols_per_row_json) {
        Ok(cols) => cols,
        Err(e) => {
            return Ok(DiscoveryStartResult {
                success: false,
                error: Some(format!("invalid cols_per_row JSON: {e}")),
                total_keys: None,
            });
        }
    };

    // Find device by vendor/product ID
    let devices = match crate::drivers::list_keyboards() {
        Ok(d) => d,
        Err(e) => {
            return Ok(DiscoveryStartResult {
                success: false,
                error: Some(format!("failed to list devices: {e}")),
                total_keys: None,
            });
        }
    };

    let device = devices
        .iter()
        .find(|d| d.vendor_id == vendor_id && d.product_id == product_id);

    let device_path = match device {
        Some(d) => d.path.clone(),
        None => {
            return Ok(DiscoveryStartResult {
                success: false,
                error: Some(format!(
                    "device {:04x}:{:04x} not found",
                    vendor_id, product_id
                )),
                total_keys: None,
            });
        }
    };

    // Create discovery session
    let dev_id = crate::discovery::DeviceId::new(vendor_id, product_id);
    let session = match crate::discovery::DiscoverySession::new(dev_id, rows, cols_per_row) {
        Ok(s) => s,
        Err(e) => {
            return Ok(DiscoveryStartResult {
                success: false,
                error: Some(format!("invalid layout: {e}")),
                total_keys: None,
            });
        }
    };

    let total_keys = session.progress().total;

    // Store session state in context
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let session_state = DiscoverySessionState {
        session,
        device_path,
        cancel_flag,
    };

    let mut ctx_guard = global_discovery_context()
        .lock()
        .map_err(|_| FfiError::internal("context lock poisoned"))?;

    if let Some(ctx) = ctx_guard.as_mut() {
        // Initialize domain if needed
        if !ctx.has_domain(DiscoveryFfi::DOMAIN) {
            DiscoveryFfi::init(ctx)?;
        }

        // Set the session state
        if let Some(mut state_guard) =
            ctx.get_domain_mut::<Option<DiscoverySessionState>>(DiscoveryFfi::DOMAIN)
        {
            if let Some(state_opt) = state_guard.downcast_mut::<Option<DiscoverySessionState>>() {
                *state_opt = Some(session_state);
            }
        }
    }

    Ok(DiscoveryStartResult {
        success: true,
        error: None,
        total_keys: Some(total_keys),
    })
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
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
fn process_discovery_event(scan_code: u16, pressed: bool, timestamp_us: u64) -> FfiResult<i32> {
    let mut ctx_guard = global_discovery_context()
        .lock()
        .map_err(|_| FfiError::internal("context lock poisoned"))?;

    let ctx = ctx_guard
        .as_mut()
        .ok_or_else(|| FfiError::internal("no context"))?;

    if !ctx.has_domain(DiscoveryFfi::DOMAIN) {
        return Ok(-1);
    }

    let mut state_guard = ctx
        .get_domain_mut::<Option<DiscoverySessionState>>(DiscoveryFfi::DOMAIN)
        .ok_or_else(|| FfiError::internal("domain not found"))?;

    let state_opt = state_guard
        .downcast_mut::<Option<DiscoverySessionState>>()
        .ok_or_else(|| FfiError::internal("type mismatch"))?;

    let state = match state_opt.as_mut() {
        Some(s) => s,
        None => return Ok(-1),
    };

    // Check cancellation
    if state.cancel_flag.load(Ordering::SeqCst) {
        return Ok(-2);
    }

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
            *state_opt = None;
            Ok(1)
        }
        _ => Ok(0),
    }
}

/// Cancel an ongoing discovery session.
///
/// Returns:
/// - 0: Discovery cancelled successfully
/// - -1: No active discovery session
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
fn cancel_discovery() -> FfiResult<i32> {
    let mut ctx_guard = global_discovery_context()
        .lock()
        .map_err(|_| FfiError::internal("context lock poisoned"))?;

    let ctx = ctx_guard
        .as_mut()
        .ok_or_else(|| FfiError::internal("no context"))?;

    if !ctx.has_domain(DiscoveryFfi::DOMAIN) {
        return Ok(-1);
    }

    let mut state_guard = ctx
        .get_domain_mut::<Option<DiscoverySessionState>>(DiscoveryFfi::DOMAIN)
        .ok_or_else(|| FfiError::internal("domain not found"))?;

    let state_opt = state_guard
        .downcast_mut::<Option<DiscoverySessionState>>()
        .ok_or_else(|| FfiError::internal("type mismatch"))?;

    match state_opt.take() {
        Some(mut state) => {
            state.cancel_flag.store(true, Ordering::SeqCst);
            let summary = state.session.cancel("cancelled by user");
            // Publish the cancellation through callbacks
            crate::discovery::session::publish_session_update(&SessionUpdate::Finished(summary));
            Ok(0)
        }
        None => Ok(-1),
    }
}

/// Discovery progress result for FFI.
#[derive(Clone, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
struct DiscoveryProgressResult {
    captured: usize,
    total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    next: Option<crate::discovery::ExpectedPosition>,
}

/// Get the current discovery progress.
///
/// Returns JSON: `ok:{captured, total, next?: {row, col}}`
// #[ffi_export] // TODO: Uncomment when exports_*.rs files are removed (task 20)
fn get_discovery_progress() -> FfiResult<DiscoveryProgressResult> {
    let ctx_guard = global_discovery_context()
        .lock()
        .map_err(|_| FfiError::internal("context lock poisoned"))?;

    let ctx = ctx_guard
        .as_ref()
        .ok_or_else(|| FfiError::internal("no context"))?;

    if !ctx.has_domain(DiscoveryFfi::DOMAIN) {
        return Err(FfiError::not_found("no active discovery session"));
    }

    let state_guard = ctx
        .get_domain::<Option<DiscoverySessionState>>(DiscoveryFfi::DOMAIN)
        .ok_or_else(|| FfiError::internal("domain not found"))?;

    let state_opt = state_guard
        .downcast_ref::<Option<DiscoverySessionState>>()
        .ok_or_else(|| FfiError::internal("type mismatch"))?;

    let state = state_opt
        .as_ref()
        .ok_or_else(|| FfiError::not_found("no active discovery session"))?;

    let progress = state.session.progress();
    Ok(DiscoveryProgressResult {
        captured: progress.captured,
        total: progress.total,
        next: progress.next,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_ffi_init() {
        let mut ctx = FfiContext::new();
        let result = DiscoveryFfi::init(&mut ctx);
        assert!(result.is_ok());
        assert!(ctx.has_domain(DiscoveryFfi::DOMAIN));
    }

    #[test]
    fn test_discovery_ffi_double_init_fails() {
        let mut ctx = FfiContext::new();
        DiscoveryFfi::init(&mut ctx).unwrap();

        // Second init should fail
        let result = DiscoveryFfi::init(&mut ctx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "INVALID_INPUT");
    }

    #[test]
    fn test_discovery_ffi_cleanup() {
        let mut ctx = FfiContext::new();
        DiscoveryFfi::init(&mut ctx).unwrap();
        assert!(ctx.has_domain(DiscoveryFfi::DOMAIN));

        DiscoveryFfi::cleanup(&mut ctx);
        assert!(!ctx.has_domain(DiscoveryFfi::DOMAIN));
    }

    #[test]
    fn test_discovery_state_storage() {
        let mut ctx = FfiContext::new();
        DiscoveryFfi::init(&mut ctx).unwrap();

        // Verify we can access the state
        let state = ctx.get_domain::<Option<DiscoverySessionState>>(DiscoveryFfi::DOMAIN);
        assert!(state.is_some());

        // Verify initial state is None (no active session)
        let guard = state.unwrap();
        let session_state = guard
            .downcast_ref::<Option<DiscoverySessionState>>()
            .unwrap();
        assert!(session_state.is_none());
    }
}
