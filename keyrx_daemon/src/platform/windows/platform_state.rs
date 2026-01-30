///! Global state for Windows platform to enable cross-layer communication
///!
///! This module provides a way for the profile service to communicate with
///! the Windows platform layer for key blocking configuration.
///!
///! # Architecture Note
///!
///! This is a pragmatic solution to bridge the service layer (profile activation)
///! and platform layer (key blocking). A cleaner approach would use dependency
///! injection or callback patterns, but this works for now.

use std::sync::{Arc, Mutex, Once};
use keyrx_core::config::ConfigRoot;

use super::key_blocker::KeyBlocker;

static INIT: Once = Once::new();
static mut PLATFORM_STATE: Option<Arc<Mutex<PlatformState>>> = None;

/// Global platform state for Windows
pub struct PlatformState {
    pub key_blocker: Option<KeyBlocker>,
}

impl PlatformState {
    /// Initialize the global platform state (called once at startup)
    pub fn initialize(blocker: Option<KeyBlocker>) {
        INIT.call_once(|| {
            let state = PlatformState {
                key_blocker: blocker,
            };
            unsafe {
                PLATFORM_STATE = Some(Arc::new(Mutex::new(state)));
            }
        });
    }

    /// Get the global platform state
    pub fn get() -> Option<Arc<Mutex<PlatformState>>> {
        unsafe { PLATFORM_STATE.clone() }
    }

    /// Configure key blocking based on the active profile
    pub fn configure_blocking(config: Option<&ConfigRoot>) -> Result<(), String> {
        let Some(state_arc) = Self::get() else {
            log::warn!("Platform state not initialized");
            return Ok(());
        };

        let state = state_arc.lock().map_err(|e| format!("Lock error: {}", e))?;

        let Some(ref blocker) = state.key_blocker else {
            log::warn!("Key blocker not available");
            return Ok(());
        };

        // Clear existing blocks
        blocker.clear_all();

        let Some(config) = config else {
            log::info!("✓ Key blocking cleared (no active profile)");
            return Ok(());
        };

        // Extract and block all source keys
        let mut blocked_count = 0;
        for device_config in &config.devices {
            for mapping in &device_config.mappings {
                Self::extract_and_block_key(mapping, blocker, &mut blocked_count);
            }
        }

        // Verify the blocker actually has the keys
        let actual_count = blocker.blocked_count();
        log::info!("✓ Configured key blocking: {} keys extracted, {} actually blocked",
            blocked_count, actual_count);

        if actual_count != blocked_count {
            log::error!("✗ CRITICAL: Mismatch between extracted ({}) and blocked ({}) keys!",
                blocked_count, actual_count);
        }

        Ok(())
    }

    /// Recursively extract source key from mapping and block it
    fn extract_and_block_key(
        mapping: &keyrx_core::config::KeyMapping,
        blocker: &KeyBlocker,
        blocked_count: &mut usize,
    ) {
        use keyrx_core::config::{BaseKeyMapping, KeyMapping};
        use super::keycode::keycode_to_scancode;

        match mapping {
            KeyMapping::Base(base) => {
                let source_key = match base {
                    BaseKeyMapping::Simple { from, .. } => *from,
                    BaseKeyMapping::Modifier { from, .. } => *from,
                    BaseKeyMapping::Lock { from, .. } => *from,
                    BaseKeyMapping::TapHold { from, .. } => *from,
                    BaseKeyMapping::ModifiedOutput { from, .. } => *from,
                };

                if let Some(scan_code) = keycode_to_scancode(source_key) {
                    blocker.block_key(scan_code);
                    *blocked_count += 1;
                    log::trace!("Blocking {:?} (scan code: 0x{:04X})", source_key, scan_code);
                } else {
                    log::warn!("Failed to convert {:?} to scan code, won't be blocked", source_key);
                }
            }
            KeyMapping::Conditional { mappings, .. } => {
                for base_mapping in mappings {
                    let wrapped = KeyMapping::Base(base_mapping.clone());
                    Self::extract_and_block_key(&wrapped, blocker, blocked_count);
                }
            }
        }
    }
}
