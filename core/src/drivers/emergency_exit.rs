//! Emergency exit handler for bypassing key remapping.
//!
//! This module provides a critical safety mechanism that allows users to disable
//! all key remapping when they press Ctrl+Alt+Shift+Escape. This is essential for
//! recovering from broken configurations that might otherwise lock the user out.
//!
//! The bypass mode is implemented using an atomic flag to ensure thread-safety
//! across all driver callbacks.

use std::sync::atomic::{AtomicBool, Ordering};
use tracing::warn;

use crate::drivers::keycodes::KeyCode;
use crate::engine::{ModifierState, StandardModifier};

/// Static atomic flag indicating whether bypass mode is active.
///
/// When true, all remapping should be skipped and keys should pass through directly.
static BYPASS_MODE: AtomicBool = AtomicBool::new(false);

/// Callback type for notifying UI about bypass mode changes.
pub type BypassCallback = Box<dyn Fn(bool) + Send + Sync>;

/// Static callback holder for bypass mode notifications.
static BYPASS_CALLBACK: std::sync::OnceLock<BypassCallback> = std::sync::OnceLock::new();

/// Register a callback to be invoked when bypass mode changes.
///
/// The callback receives `true` when bypass is activated and `false` when deactivated.
/// Only the first registered callback is used; subsequent calls are ignored.
pub fn register_bypass_callback(callback: BypassCallback) {
    let _ = BYPASS_CALLBACK.set(callback);
}

/// Notify any registered callback about bypass state change.
fn notify_callback(active: bool) {
    if let Some(cb) = BYPASS_CALLBACK.get() {
        cb(active);
    }
}

/// Check if the given key and modifier state form the emergency exit combo.
///
/// The emergency exit combo is: Ctrl + Alt + Shift + Escape
///
/// This function should be called at the very start of keyboard event processing,
/// before any remapping logic runs.
///
/// # Arguments
///
/// * `key` - The key that was pressed
/// * `modifiers` - The current modifier state
///
/// # Returns
///
/// `true` if the emergency exit combo was detected (key is Escape with Ctrl+Alt+Shift),
/// `false` otherwise.
#[inline]
pub fn check_emergency_exit(key: KeyCode, modifiers: &ModifierState) -> bool {
    if key != KeyCode::Escape {
        return false;
    }

    let std_mods = modifiers.standard();
    std_mods.is_active(StandardModifier::Control)
        && std_mods.is_active(StandardModifier::Alt)
        && std_mods.is_active(StandardModifier::Shift)
}

/// Activate bypass mode, disabling all key remapping.
///
/// When bypass mode is active, all keys should pass through unchanged.
/// This function logs a warning to alert the user that remapping has been disabled.
pub fn activate_bypass_mode() {
    let was_active = BYPASS_MODE.swap(true, Ordering::SeqCst);
    if !was_active {
        warn!(
            "Emergency exit triggered: bypass mode ACTIVATED. \
             All key remapping is now disabled. \
             Press Ctrl+Alt+Shift+Escape again to re-enable."
        );
        notify_callback(true);
    }
}

/// Deactivate bypass mode, re-enabling key remapping.
///
/// After calling this, normal key remapping will resume.
pub fn deactivate_bypass_mode() {
    let was_active = BYPASS_MODE.swap(false, Ordering::SeqCst);
    if was_active {
        warn!("Bypass mode DEACTIVATED. Key remapping is now re-enabled.");
        notify_callback(false);
    }
}

/// Toggle bypass mode.
///
/// If bypass is currently active, it will be deactivated.
/// If bypass is currently inactive, it will be activated.
///
/// # Returns
///
/// The new state of bypass mode after toggling.
pub fn toggle_bypass_mode() -> bool {
    let new_state = !BYPASS_MODE.load(Ordering::SeqCst);
    if new_state {
        activate_bypass_mode();
    } else {
        deactivate_bypass_mode();
    }
    new_state
}

/// Check if bypass mode is currently active.
///
/// # Returns
///
/// `true` if bypass mode is active (remapping disabled), `false` otherwise.
#[inline]
pub fn is_bypass_active() -> bool {
    BYPASS_MODE.load(Ordering::SeqCst)
}

/// Set bypass mode to a specific state.
///
/// This is useful for FFI callers that want to directly control bypass state
/// rather than toggling.
///
/// # Arguments
///
/// * `active` - Whether bypass mode should be active
pub fn set_bypass_mode(active: bool) {
    if active {
        activate_bypass_mode();
    } else {
        deactivate_bypass_mode();
    }
}

/// Reset bypass mode to inactive state.
///
/// This is primarily useful for testing to ensure a clean state.
/// Does not trigger the callback.
#[cfg(test)]
pub fn reset_bypass_mode() {
    BYPASS_MODE.store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Modifier;

    fn make_modifiers_state(ctrl: bool, alt: bool, shift: bool) -> ModifierState {
        let mut state = ModifierState::new();
        if ctrl {
            state.activate(Modifier::Standard(StandardModifier::Control));
        }
        if alt {
            state.activate(Modifier::Standard(StandardModifier::Alt));
        }
        if shift {
            state.activate(Modifier::Standard(StandardModifier::Shift));
        }
        state
    }

    #[test]
    fn check_emergency_exit_with_all_mods() {
        let mods = make_modifiers_state(true, true, true);
        assert!(check_emergency_exit(KeyCode::Escape, &mods));
    }

    #[test]
    fn check_emergency_exit_missing_ctrl() {
        let mods = make_modifiers_state(false, true, true);
        assert!(!check_emergency_exit(KeyCode::Escape, &mods));
    }

    #[test]
    fn check_emergency_exit_missing_alt() {
        let mods = make_modifiers_state(true, false, true);
        assert!(!check_emergency_exit(KeyCode::Escape, &mods));
    }

    #[test]
    fn check_emergency_exit_missing_shift() {
        let mods = make_modifiers_state(true, true, false);
        assert!(!check_emergency_exit(KeyCode::Escape, &mods));
    }

    #[test]
    fn check_emergency_exit_wrong_key() {
        let mods = make_modifiers_state(true, true, true);
        assert!(!check_emergency_exit(KeyCode::A, &mods));
    }

    #[test]
    fn check_emergency_exit_no_mods() {
        let mods = ModifierState::new();
        assert!(!check_emergency_exit(KeyCode::Escape, &mods));
    }

    #[test]
    fn bypass_mode_activate_deactivate() {
        reset_bypass_mode();
        assert!(!is_bypass_active());

        activate_bypass_mode();
        assert!(is_bypass_active());

        deactivate_bypass_mode();
        assert!(!is_bypass_active());
    }

    #[test]
    fn bypass_mode_toggle() {
        reset_bypass_mode();
        assert!(!is_bypass_active());

        let state = toggle_bypass_mode();
        assert!(state);
        assert!(is_bypass_active());

        let state = toggle_bypass_mode();
        assert!(!state);
        assert!(!is_bypass_active());
    }

    #[test]
    fn bypass_mode_idempotent_activation() {
        reset_bypass_mode();

        activate_bypass_mode();
        assert!(is_bypass_active());

        // Activating again should keep it active
        activate_bypass_mode();
        assert!(is_bypass_active());

        deactivate_bypass_mode();
        assert!(!is_bypass_active());

        // Deactivating again should keep it inactive
        deactivate_bypass_mode();
        assert!(!is_bypass_active());
    }

    #[test]
    fn bypass_mode_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        reset_bypass_mode();

        let barrier = Arc::new(std::sync::Barrier::new(4));
        let mut handles = vec![];

        // Spawn multiple threads to concurrently access bypass mode
        for i in 0..4 {
            let b = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                b.wait();
                for _ in 0..100 {
                    if i % 2 == 0 {
                        activate_bypass_mode();
                    } else {
                        deactivate_bypass_mode();
                    }
                    let _ = is_bypass_active();
                }
            }));
        }

        for h in handles {
            h.join().expect("thread should not panic");
        }

        // Reset to known state after concurrent access
        reset_bypass_mode();
        assert!(!is_bypass_active());
    }
}
