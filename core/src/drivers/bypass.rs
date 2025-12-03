//! Injectable bypass controller for testing.
//!
//! Provides a testable alternative to global bypass state.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Callback type for notifying about bypass mode changes.
pub type BypassCallback = Box<dyn Fn(bool) + Send + Sync>;

/// Injectable bypass controller that manages bypass mode state.
///
/// Unlike the global bypass state in `emergency_exit`, this controller
/// can be instantiated per-test, enabling parallel test execution.
#[derive(Clone)]
pub struct BypassController {
    active: Arc<AtomicBool>,
    callback: Arc<Option<BypassCallback>>,
}

impl Default for BypassController {
    fn default() -> Self {
        Self::new()
    }
}

impl BypassController {
    /// Create a new bypass controller with inactive state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            callback: Arc::new(None),
        }
    }

    /// Create a bypass controller with a callback for state changes.
    #[must_use]
    pub fn with_callback(callback: BypassCallback) -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            callback: Arc::new(Some(callback)),
        }
    }

    /// Check if bypass mode is currently active.
    #[inline]
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    /// Activate bypass mode.
    pub fn activate(&self) {
        let was_active = self.active.swap(true, Ordering::SeqCst);
        if !was_active {
            self.notify(true);
        }
    }

    /// Deactivate bypass mode.
    pub fn deactivate(&self) {
        let was_active = self.active.swap(false, Ordering::SeqCst);
        if was_active {
            self.notify(false);
        }
    }

    /// Toggle bypass mode and return the new state.
    pub fn toggle(&self) -> bool {
        let new_state = !self.active.load(Ordering::SeqCst);
        if new_state {
            self.activate();
        } else {
            self.deactivate();
        }
        new_state
    }

    /// Set bypass mode to a specific state.
    pub fn set(&self, active: bool) {
        if active {
            self.activate();
        } else {
            self.deactivate();
        }
    }

    fn notify(&self, active: bool) {
        if let Some(ref cb) = *self.callback {
            cb(active);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[test]
    fn new_controller_is_inactive() {
        let ctrl = BypassController::new();
        assert!(!ctrl.is_active());
    }

    #[test]
    fn activate_sets_active() {
        let ctrl = BypassController::new();
        ctrl.activate();
        assert!(ctrl.is_active());
    }

    #[test]
    fn deactivate_clears_active() {
        let ctrl = BypassController::new();
        ctrl.activate();
        ctrl.deactivate();
        assert!(!ctrl.is_active());
    }

    #[test]
    fn toggle_flips_state() {
        let ctrl = BypassController::new();
        assert!(!ctrl.is_active());

        let state = ctrl.toggle();
        assert!(state);
        assert!(ctrl.is_active());

        let state = ctrl.toggle();
        assert!(!state);
        assert!(!ctrl.is_active());
    }

    #[test]
    fn callback_invoked_on_state_change() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        let ctrl = BypassController::with_callback(Box::new(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));

        ctrl.activate();
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        ctrl.deactivate();
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn callback_not_invoked_on_redundant_activation() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        let ctrl = BypassController::with_callback(Box::new(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));

        ctrl.activate();
        ctrl.activate(); // Should not trigger callback
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn clone_shares_state() {
        let ctrl1 = BypassController::new();
        let ctrl2 = ctrl1.clone();

        ctrl1.activate();
        assert!(ctrl2.is_active());

        ctrl2.deactivate();
        assert!(!ctrl1.is_active());
    }

    #[test]
    fn parallel_controllers_independent() {
        let ctrl1 = BypassController::new();
        let ctrl2 = BypassController::new();

        ctrl1.activate();
        assert!(ctrl1.is_active());
        assert!(!ctrl2.is_active()); // Independent instance
    }
}
