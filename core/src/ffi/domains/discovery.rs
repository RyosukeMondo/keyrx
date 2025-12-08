//! Discovery domain FFI implementation.
//!
//! Implements the FfiExportable trait for discovery session management.
//! This module replaces the global static pattern from exports_discovery.rs
//! with instance-scoped state management through FfiContext.
#![allow(unsafe_code)]

use crate::discovery::session::set_session_update_sink;
use crate::ffi::context::FfiContext;
use crate::ffi::error::FfiError;
use crate::ffi::events::EventRegistry;
use crate::ffi::traits::FfiExportable;
use std::sync::OnceLock;

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
        // Domain initialized but empty (legacy features removed)
        Ok(())
    }

    fn cleanup(ctx: &mut FfiContext) {
        // Remove discovery sink
        set_session_update_sink(None);
        // Remove domain state
        ctx.remove_domain(Self::DOMAIN);
    }
}

// ─── FFI Exports ───────────────────────────────────────────────────────────

/// Global event registry for Discovery (and other non-engine) domains.
pub(crate) fn global_event_registry() -> &'static EventRegistry {
    static REGISTRY: OnceLock<EventRegistry> = OnceLock::new();
    REGISTRY.get_or_init(EventRegistry::new)
}

// NOTE: Legacy discovery functions (start_discovery, etc.) have been removed
// to enforce Single Source of Truth (SSOT) via DeviceRegistry.

/// Refresh the discovery sink based on registered callbacks.
///
/// This is now a no-op as legacy discovery has been removed.
/// Kept to satisfy calls from exports.rs.
pub(crate) fn refresh_discovery_sink() {
    // No-op
}
