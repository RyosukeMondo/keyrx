//! Discovery domain FFI implementation.
//!
//! Implements the FfiExportable trait for discovery session management.
//! This module replaces the global static pattern from exports_discovery.rs
//! with instance-scoped state management through FfiContext.
#![allow(unsafe_code)]

use crate::discovery::{session::set_session_update_sink, SessionUpdate};
use crate::ffi::callbacks::callback_registry;
use crate::ffi::context::FfiContext;
use crate::ffi::error::FfiError;
use crate::ffi::traits::FfiExportable;
use serde::Serialize;
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
    // Helper methods for accessing session state from context
    // These will be used by the FFI export methods in the next task
    #[allow(dead_code)]
    fn get_session_state_mut(
        ctx: &FfiContext,
    ) -> Option<std::sync::RwLockWriteGuard<'_, Box<dyn std::any::Any + Send + Sync>>> {
        ctx.get_domain_mut::<Option<DiscoverySessionState>>(Self::DOMAIN)
    }

    #[allow(dead_code)]
    fn get_session_state(
        ctx: &FfiContext,
    ) -> Option<std::sync::RwLockReadGuard<'_, Box<dyn std::any::Any + Send + Sync>>> {
        ctx.get_domain::<Option<DiscoverySessionState>>(Self::DOMAIN)
    }
}

/// Discovery start result for FFI JSON output.
#[derive(Serialize)]
pub struct DiscoveryStartResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(rename = "totalKeys", skip_serializing_if = "Option::is_none")]
    pub total_keys: Option<usize>,
}

/// Refresh the discovery sink based on registered callbacks.
fn refresh_discovery_sink() {
    if callback_registry().has_any_discovery_callback() {
        set_session_update_sink(Some(discovery_sink()));
    } else {
        set_session_update_sink(None);
    }
}

/// Create the discovery sink closure that routes updates to callbacks.
fn discovery_sink() -> Arc<dyn Fn(&SessionUpdate) + Send + Sync + 'static> {
    Arc::new(|update| {
        let registry = callback_registry();
        match update {
            SessionUpdate::Ignored => {}
            SessionUpdate::Progress(progress) => {
                registry.invoke_discovery(registry.progress(), progress, "progress");
            }
            SessionUpdate::Duplicate(dup) => {
                registry.invoke_discovery(registry.duplicate(), dup, "duplicate");
            }
            SessionUpdate::Finished(summary) => {
                registry.invoke_discovery(registry.summary(), summary, "summary");
            }
        }
    })
}

// Note: Callback registration functions (keyrx_on_discovery_progress, etc.)
// are still in exports_discovery.rs and will be migrated to EventRegistry in task 10.

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
