//! Validation domain FFI implementation.
//!
//! Implements the FfiExportable trait for script validation and key suggestions.
//! Unlike Discovery, validation is stateless and doesn't require callbacks.
//!
//! Note: The actual FFI functions (keyrx_validate_script, etc.) remain in
//! exports_validation.rs for now. They will be migrated here in a future task.
#![allow(unsafe_code)]

use crate::ffi::context::FfiContext;
use crate::ffi::error::FfiError;
use crate::ffi::traits::FfiExportable;

/// Validation domain FFI implementation.
pub struct ValidationFfi;

impl FfiExportable for ValidationFfi {
    const DOMAIN: &'static str = "validation";

    fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
        if ctx.has_domain(Self::DOMAIN) {
            return Err(FfiError::invalid_input(
                "validation domain already initialized",
            ));
        }

        // Validation is stateless - no domain state needed
        ctx.set_domain(Self::DOMAIN, ());

        Ok(())
    }

    fn cleanup(ctx: &mut FfiContext) {
        // Validation is stateless - just remove domain marker
        ctx.remove_domain(Self::DOMAIN);
    }
}

// ValidationFfi is stateless - no helper methods needed
