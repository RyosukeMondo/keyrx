//! Engine control FFI exports.
//!
//! **DEPRECATED**: This module is deprecated. Use `domains::engine` instead.
//!
//! This file now delegates to the new domain-based architecture for backward compatibility.
//! New code should use the EngineFfi domain directly.
#![allow(unsafe_code)]
#![allow(deprecated)]

// Re-export the new implementation
pub use crate::ffi::domains::engine::{
    publish_state_change, publish_state_snapshot, publish_state_snapshot_legacy,
};

use crate::drivers::emergency_exit::{is_bypass_active, set_bypass_mode};

/// Check if emergency bypass mode is currently active.
///
/// When bypass mode is active, all key remapping is disabled.
///
/// **Deprecated**: Use the new `keyrx_is_bypass_mode_active()` function from EngineFfi.
///
/// # Safety
/// This function is safe to call from any thread.
#[deprecated(note = "Use keyrx_is_bypass_mode_active() from EngineFfi")]
#[no_mangle]
pub extern "C" fn keyrx_is_bypass_active() -> bool {
    is_bypass_active()
}

/// Set the emergency bypass mode state.
///
/// **Deprecated**: Use the new `keyrx_set_bypass_mode_state()` function from EngineFfi.
///
/// # Arguments
///
/// * `active` - If true, enable bypass mode (disable remapping).
///   If false, disable bypass mode (re-enable remapping).
///
/// # Safety
/// This function is safe to call from any thread.
#[deprecated(note = "Use keyrx_set_bypass_mode_state() from EngineFfi")]
#[no_mangle]
pub extern "C" fn keyrx_set_bypass(active: bool) {
    set_bypass_mode(active);
}

// keyrx_on_state is now in compat/engine_compat.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::emergency_exit::reset_bypass_mode;

    #[test]
    fn bypass_active_returns_current_state() {
        // Ensure clean state
        reset_bypass_mode();
        assert!(!keyrx_is_bypass_active());

        // Activate bypass
        keyrx_set_bypass(true);
        assert!(keyrx_is_bypass_active());

        // Deactivate bypass
        keyrx_set_bypass(false);
        assert!(!keyrx_is_bypass_active());

        // Clean up
        reset_bypass_mode();
    }
}
