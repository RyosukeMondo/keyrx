//! Emergency exit handler for bypassing key remapping.
//!
//! This module provides a critical safety mechanism that allows users to disable
//! all key remapping when they press Ctrl+Alt+Shift+Escape. This is essential for
//! recovering from broken configurations that might otherwise lock the user out.
//!
//! The bypass mode is implemented using [`BypassController`] which uses an atomic
//! flag for thread-safety. The functions here delegate to a global controller for
//! backward compatibility, but new code should use [`BypassController`] directly
//! for testability.

use std::sync::OnceLock;
use tracing::warn;

use super::bypass::BypassController;
use crate::drivers::keycodes::KeyCode;
use crate::engine::{ModifierState, StandardModifier};

pub use super::bypass::BypassCallback;

/// Global bypass controller for backward compatibility.
///
/// New code should inject [`BypassController`] instead of using these functions.
static GLOBAL_CONTROLLER: OnceLock<BypassController> = OnceLock::new();

/// Static callback holder for bypass mode notifications (legacy).
static BYPASS_CALLBACK: OnceLock<BypassCallback> = OnceLock::new();

fn global_controller() -> &'static BypassController {
    GLOBAL_CONTROLLER.get_or_init(BypassController::new)
}

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
    let was_active = global_controller().is_active();
    global_controller().activate();
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
    let was_active = global_controller().is_active();
    global_controller().deactivate();
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
    let new_state = !global_controller().is_active();
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
    global_controller().is_active()
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
    global_controller().deactivate();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Modifier;
    use serial_test::serial;

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
    #[serial]
    fn bypass_mode_activate_deactivate() {
        reset_bypass_mode();
        assert!(!is_bypass_active());

        activate_bypass_mode();
        assert!(is_bypass_active());

        deactivate_bypass_mode();
        assert!(!is_bypass_active());
    }

    #[test]
    #[serial]
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
    #[serial]
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
    #[serial]
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
