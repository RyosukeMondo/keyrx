//! Tap-hold state machine implementation.
//!
//! This module contains the [`TapHoldState`] struct which represents the state
//! of a single tap-hold key as it transitions through the Idle → Pending → Hold
//! state machine.

use super::types::{TapHoldConfig, TapHoldPhase};
use crate::config::KeyCode;

/// Logs a tap-hold state transition in debug builds.
///
/// This macro is a no-op in release builds to ensure zero overhead.
#[cfg(debug_assertions)]
macro_rules! log_transition {
    ($key:expr, $old_state:expr, $new_state:expr, $elapsed_us:expr, $reason:expr) => {
        log::trace!(
            "tap-hold: {:?} {} -> {} (elapsed: {}us, reason: {})",
            $key,
            $old_state,
            $new_state,
            $elapsed_us,
            $reason
        );
    };
    ($key:expr, $old_state:expr, $new_state:expr, $reason:expr) => {
        log::trace!(
            "tap-hold: {:?} {} -> {} (reason: {})",
            $key,
            $old_state,
            $new_state,
            $reason
        );
    };
}

/// No-op in release builds.
#[cfg(not(debug_assertions))]
macro_rules! log_transition {
    ($key:expr, $old_state:expr, $new_state:expr, $elapsed_us:expr, $reason:expr) => {};
    ($key:expr, $old_state:expr, $new_state:expr, $reason:expr) => {};
}

/// State for a single tap-hold key.
///
/// Tracks the current phase, timing, and configuration for one tap-hold key.
/// Multiple instances can be tracked simultaneously via `PendingKeyRegistry`.
///
/// # Example
///
/// ```rust
/// use keyrx_core::runtime::tap_hold::{TapHoldPhase, TapHoldState, TapHoldConfig};
/// use keyrx_core::config::KeyCode;
///
/// let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
/// let mut state = TapHoldState::new(KeyCode::CapsLock, config);
///
/// // Initially idle
/// assert!(state.phase().is_idle());
///
/// // Transition to pending on press
/// state.transition_to_pending(1000);
/// assert!(state.phase().is_pending());
/// assert_eq!(state.press_time(), 1000);
///
/// // Check if threshold exceeded
/// assert!(!state.is_threshold_exceeded(100_000)); // 99ms < 200ms
/// assert!(state.is_threshold_exceeded(300_000));  // 299ms > 200ms
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TapHoldState {
    /// The physical key that triggers this tap-hold
    key: KeyCode,
    /// Current state machine phase
    phase: TapHoldPhase,
    /// Configuration for tap/hold behavior
    config: TapHoldConfig,
    /// Timestamp when key was pressed (microseconds)
    press_time: u64,
}

impl TapHoldState {
    /// Creates a new tap-hold state in Idle phase.
    ///
    /// # Arguments
    ///
    /// * `key` - The physical key that triggers this tap-hold
    /// * `config` - Configuration for tap/hold behavior
    pub const fn new(key: KeyCode, config: TapHoldConfig) -> Self {
        Self {
            key,
            phase: TapHoldPhase::Idle,
            config,
            press_time: 0,
        }
    }

    /// Returns the physical key.
    pub const fn key(&self) -> KeyCode {
        self.key
    }

    /// Returns the current phase.
    pub const fn phase(&self) -> TapHoldPhase {
        self.phase
    }

    /// Returns the configuration.
    pub const fn config(&self) -> &TapHoldConfig {
        &self.config
    }

    /// Returns the press timestamp.
    pub const fn press_time(&self) -> u64 {
        self.press_time
    }

    /// Returns the tap key from config.
    pub const fn tap_key(&self) -> KeyCode {
        self.config.tap_key()
    }

    /// Returns the hold modifier from config.
    pub const fn hold_modifier(&self) -> u8 {
        self.config.hold_modifier()
    }

    /// The real key HOLD presses, if any (see `TapHoldConfig::hold_key`).
    pub const fn hold_key(&self) -> Option<KeyCode> {
        self.config.hold_key()
    }

    /// Returns the threshold in microseconds.
    pub const fn threshold_us(&self) -> u64 {
        self.config.threshold_us()
    }

    /// Checks if the threshold has been exceeded at the given time.
    ///
    /// # Arguments
    ///
    /// * `current_time` - Current timestamp in microseconds
    ///
    /// # Returns
    ///
    /// `true` if (current_time - press_time) >= threshold
    pub const fn is_threshold_exceeded(&self, current_time: u64) -> bool {
        current_time.saturating_sub(self.press_time) >= self.config.threshold_us()
    }

    /// Calculates elapsed time since press.
    ///
    /// # Arguments
    ///
    /// * `current_time` - Current timestamp in microseconds
    pub const fn elapsed(&self, current_time: u64) -> u64 {
        current_time.saturating_sub(self.press_time)
    }

    // --- State Transitions ---

    /// Transitions from Idle to Pending on key press.
    ///
    /// Records the press timestamp for later threshold checking.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Press event timestamp in microseconds
    ///
    /// # Panics
    ///
    /// Debug asserts that current phase is Idle.
    pub fn transition_to_pending(&mut self, timestamp: u64) {
        debug_assert!(
            self.phase.is_idle(),
            "transition_to_pending called from non-Idle phase: {:?}",
            self.phase
        );
        #[cfg(debug_assertions)]
        let old_phase = self.phase;
        self.phase = TapHoldPhase::Pending;
        self.press_time = timestamp;
        log_transition!(self.key, old_phase, self.phase, "key pressed");
    }

    /// Transitions from Pending to Hold.
    ///
    /// Called when threshold is exceeded or another key interrupts (permissive hold).
    ///
    /// # Panics
    ///
    /// Debug asserts that current phase is Pending.
    pub fn transition_to_hold(&mut self) {
        debug_assert!(
            self.phase.is_pending(),
            "transition_to_hold called from non-Pending phase: {:?}",
            self.phase
        );
        #[cfg(debug_assertions)]
        let old_phase = self.phase;
        self.phase = TapHoldPhase::Hold;
        log_transition!(
            self.key,
            old_phase,
            self.phase,
            "threshold exceeded or permissive hold"
        );
    }

    /// Transitions back to Idle.
    ///
    /// Called on key release in any active phase.
    pub fn transition_to_idle(&mut self) {
        #[cfg(debug_assertions)]
        let old_phase = self.phase;
        self.phase = TapHoldPhase::Idle;
        self.press_time = 0;
        log_transition!(self.key, old_phase, self.phase, "key released");
    }

    /// Resets the state to Idle.
    ///
    /// Same as `transition_to_idle()` but more explicit naming.
    pub fn reset(&mut self) {
        self.transition_to_idle();
    }
}
