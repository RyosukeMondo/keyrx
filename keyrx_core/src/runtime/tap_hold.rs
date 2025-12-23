//! Tap-hold state machine implementation.
//!
//! This module provides the core tap-hold functionality where a key can act as
//! one key when tapped (quick press and release) and a modifier when held
//! (pressed beyond a threshold).
//!
//! # State Machine
//!
//! ```text
//!                    Press
//!     ┌─────────────────────────────────────┐
//!     │                                     ▼
//!  ┌──────┐    ┌─────────┐  timeout    ┌────────┐
//!  │ Idle │───▶│ Pending │────────────▶│  Hold  │
//!  └──────┘    └─────────┘             └────────┘
//!     ▲             │                       │
//!     │   quick     │    other key          │
//!     │   release   │    pressed            │
//!     │   (tap)     │  (permissive hold)    │
//!     │             ▼                       │
//!     │         emit tap                    │
//!     │         key event                   │
//!     │                                     │
//!     └─────────────────────────────────────┘
//!                   Release
//! ```
//!
//! # Example
//!
//! ```rust
//! use keyrx_core::runtime::tap_hold::{TapHoldPhase, TapHoldState, TapHoldConfig};
//! use keyrx_core::config::KeyCode;
//!
//! // Configure CapsLock as tap=Escape, hold=Ctrl (modifier 0)
//! let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
//!
//! // Create initial state
//! let state = TapHoldState::new(KeyCode::CapsLock, config);
//!
//! assert_eq!(state.phase(), TapHoldPhase::Idle);
//! assert_eq!(state.key(), KeyCode::CapsLock);
//! ```

use crate::config::KeyCode;

/// Phase of the tap-hold state machine.
///
/// # Phases
///
/// - `Idle`: No key activity, waiting for press
/// - `Pending`: Key pressed, waiting to determine tap vs hold
/// - `Hold`: Key held past threshold, modifier active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TapHoldPhase {
    /// No activity, waiting for key press
    #[default]
    Idle,
    /// Key pressed, waiting for release or timeout
    Pending,
    /// Key held, modifier is active
    Hold,
}

impl TapHoldPhase {
    /// Returns true if the phase is Idle.
    pub const fn is_idle(&self) -> bool {
        matches!(self, TapHoldPhase::Idle)
    }

    /// Returns true if the phase is Pending.
    pub const fn is_pending(&self) -> bool {
        matches!(self, TapHoldPhase::Pending)
    }

    /// Returns true if the phase is Hold.
    pub const fn is_hold(&self) -> bool {
        matches!(self, TapHoldPhase::Hold)
    }
}

/// Configuration for a tap-hold key.
///
/// Contains the behavior settings for a single tap-hold key:
/// - What key to emit on tap
/// - What modifier to activate on hold
/// - Threshold time in microseconds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TapHoldConfig {
    /// Key to emit when tapped (quick press and release)
    tap_key: KeyCode,
    /// Modifier ID to activate when held (0-254)
    hold_modifier: u8,
    /// Threshold time in microseconds (tap vs hold boundary)
    threshold_us: u64,
}

impl TapHoldConfig {
    /// Creates a new tap-hold configuration.
    ///
    /// # Arguments
    ///
    /// * `tap_key` - Key to emit on tap
    /// * `hold_modifier` - Modifier ID to activate on hold (0-254)
    /// * `threshold_us` - Time in microseconds to distinguish tap from hold
    ///
    /// # Example
    ///
    /// ```rust
    /// use keyrx_core::runtime::tap_hold::TapHoldConfig;
    /// use keyrx_core::config::KeyCode;
    ///
    /// // CapsLock: tap=Escape, hold=Ctrl (200ms threshold)
    /// let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
    ///
    /// assert_eq!(config.tap_key(), KeyCode::Escape);
    /// assert_eq!(config.hold_modifier(), 0);
    /// assert_eq!(config.threshold_us(), 200_000);
    /// ```
    pub const fn new(tap_key: KeyCode, hold_modifier: u8, threshold_us: u64) -> Self {
        Self {
            tap_key,
            hold_modifier,
            threshold_us,
        }
    }

    /// Creates a config from milliseconds threshold.
    ///
    /// Convenience constructor for common millisecond-based thresholds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use keyrx_core::runtime::tap_hold::TapHoldConfig;
    /// use keyrx_core::config::KeyCode;
    ///
    /// let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    /// assert_eq!(config.threshold_us(), 200_000);
    /// ```
    pub const fn from_ms(tap_key: KeyCode, hold_modifier: u8, threshold_ms: u16) -> Self {
        Self::new(tap_key, hold_modifier, threshold_ms as u64 * 1000)
    }

    /// Returns the tap key.
    pub const fn tap_key(&self) -> KeyCode {
        self.tap_key
    }

    /// Returns the hold modifier ID.
    pub const fn hold_modifier(&self) -> u8 {
        self.hold_modifier
    }

    /// Returns the threshold in microseconds.
    pub const fn threshold_us(&self) -> u64 {
        self.threshold_us
    }
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
        self.config.tap_key
    }

    /// Returns the hold modifier from config.
    pub const fn hold_modifier(&self) -> u8 {
        self.config.hold_modifier
    }

    /// Returns the threshold in microseconds.
    pub const fn threshold_us(&self) -> u64 {
        self.config.threshold_us
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
        current_time.saturating_sub(self.press_time) >= self.config.threshold_us
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
        self.phase = TapHoldPhase::Pending;
        self.press_time = timestamp;
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
        self.phase = TapHoldPhase::Hold;
    }

    /// Transitions back to Idle.
    ///
    /// Called on key release in any active phase.
    pub fn transition_to_idle(&mut self) {
        self.phase = TapHoldPhase::Idle;
        self.press_time = 0;
    }

    /// Resets the state to Idle.
    ///
    /// Same as `transition_to_idle()` but more explicit naming.
    pub fn reset(&mut self) {
        self.transition_to_idle();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- TapHoldPhase Tests ---

    #[test]
    fn test_phase_default_is_idle() {
        let phase = TapHoldPhase::default();
        assert_eq!(phase, TapHoldPhase::Idle);
    }

    #[test]
    fn test_phase_is_idle() {
        assert!(TapHoldPhase::Idle.is_idle());
        assert!(!TapHoldPhase::Pending.is_idle());
        assert!(!TapHoldPhase::Hold.is_idle());
    }

    #[test]
    fn test_phase_is_pending() {
        assert!(!TapHoldPhase::Idle.is_pending());
        assert!(TapHoldPhase::Pending.is_pending());
        assert!(!TapHoldPhase::Hold.is_pending());
    }

    #[test]
    fn test_phase_is_hold() {
        assert!(!TapHoldPhase::Idle.is_hold());
        assert!(!TapHoldPhase::Pending.is_hold());
        assert!(TapHoldPhase::Hold.is_hold());
    }

    // --- TapHoldConfig Tests ---

    #[test]
    fn test_config_new() {
        let config = TapHoldConfig::new(KeyCode::Escape, 5, 200_000);

        assert_eq!(config.tap_key(), KeyCode::Escape);
        assert_eq!(config.hold_modifier(), 5);
        assert_eq!(config.threshold_us(), 200_000);
    }

    #[test]
    fn test_config_from_ms() {
        let config = TapHoldConfig::from_ms(KeyCode::Tab, 0, 150);

        assert_eq!(config.tap_key(), KeyCode::Tab);
        assert_eq!(config.hold_modifier(), 0);
        assert_eq!(config.threshold_us(), 150_000);
    }

    #[test]
    fn test_config_from_ms_max_value() {
        // u16::MAX = 65535ms = 65,535,000μs
        let config = TapHoldConfig::from_ms(KeyCode::A, 254, u16::MAX);
        assert_eq!(config.threshold_us(), 65_535_000);
    }

    // --- TapHoldState Tests ---

    #[test]
    fn test_state_new() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let state = TapHoldState::new(KeyCode::CapsLock, config);

        assert_eq!(state.key(), KeyCode::CapsLock);
        assert_eq!(state.phase(), TapHoldPhase::Idle);
        assert_eq!(state.press_time(), 0);
        assert_eq!(state.tap_key(), KeyCode::Escape);
        assert_eq!(state.hold_modifier(), 0);
        assert_eq!(state.threshold_us(), 200_000);
    }

    #[test]
    fn test_state_transition_idle_to_pending() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        state.transition_to_pending(1000);

        assert_eq!(state.phase(), TapHoldPhase::Pending);
        assert_eq!(state.press_time(), 1000);
    }

    #[test]
    fn test_state_transition_pending_to_hold() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        state.transition_to_pending(1000);
        state.transition_to_hold();

        assert_eq!(state.phase(), TapHoldPhase::Hold);
        assert_eq!(state.press_time(), 1000); // press_time preserved
    }

    #[test]
    fn test_state_transition_to_idle() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        // From Pending
        state.transition_to_pending(1000);
        state.transition_to_idle();
        assert_eq!(state.phase(), TapHoldPhase::Idle);
        assert_eq!(state.press_time(), 0);

        // From Hold
        state.transition_to_pending(2000);
        state.transition_to_hold();
        state.transition_to_idle();
        assert_eq!(state.phase(), TapHoldPhase::Idle);
        assert_eq!(state.press_time(), 0);
    }

    #[test]
    fn test_state_reset() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        state.transition_to_pending(5000);
        state.reset();

        assert_eq!(state.phase(), TapHoldPhase::Idle);
        assert_eq!(state.press_time(), 0);
    }

    #[test]
    fn test_is_threshold_exceeded() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000); // 200ms
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        state.transition_to_pending(1_000_000); // pressed at 1s

        // Before threshold
        assert!(!state.is_threshold_exceeded(1_100_000)); // 100ms elapsed
        assert!(!state.is_threshold_exceeded(1_199_999)); // just under

        // At threshold
        assert!(state.is_threshold_exceeded(1_200_000)); // exactly 200ms

        // After threshold
        assert!(state.is_threshold_exceeded(1_300_000)); // 300ms elapsed
    }

    #[test]
    fn test_elapsed() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        state.transition_to_pending(1_000_000);

        assert_eq!(state.elapsed(1_000_000), 0);
        assert_eq!(state.elapsed(1_100_000), 100_000);
        assert_eq!(state.elapsed(1_500_000), 500_000);
    }

    #[test]
    fn test_elapsed_saturates_on_underflow() {
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        state.transition_to_pending(1_000_000);

        // Current time before press time (shouldn't happen, but handle gracefully)
        assert_eq!(state.elapsed(500_000), 0);
    }

    #[test]
    fn test_tap_scenario() {
        let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        // Key pressed
        state.transition_to_pending(0);
        assert!(state.phase().is_pending());

        // Quick release (100ms < 200ms threshold)
        let release_time = 100_000; // 100ms
        assert!(!state.is_threshold_exceeded(release_time));

        // Would emit tap key (Escape) - transition back to idle
        state.transition_to_idle();
        assert!(state.phase().is_idle());
    }

    #[test]
    fn test_hold_scenario() {
        let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        // Key pressed
        state.transition_to_pending(0);
        assert!(state.phase().is_pending());

        // Threshold exceeded (300ms > 200ms)
        let check_time = 300_000;
        assert!(state.is_threshold_exceeded(check_time));

        // Transition to hold
        state.transition_to_hold();
        assert!(state.phase().is_hold());

        // Key released
        state.transition_to_idle();
        assert!(state.phase().is_idle());
    }

    #[test]
    fn test_permissive_hold_scenario() {
        let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
        let mut state = TapHoldState::new(KeyCode::CapsLock, config);

        // Key pressed
        state.transition_to_pending(0);

        // Another key pressed before threshold (50ms < 200ms)
        // This triggers permissive hold
        let interrupt_time = 50_000;
        assert!(!state.is_threshold_exceeded(interrupt_time));

        // Immediately transition to hold (permissive hold behavior)
        state.transition_to_hold();
        assert!(state.phase().is_hold());
    }

    #[test]
    fn test_config_accessors() {
        let config = TapHoldConfig::new(KeyCode::Tab, 3, 150_000);
        let state = TapHoldState::new(KeyCode::Space, config);

        assert_eq!(state.config().tap_key(), KeyCode::Tab);
        assert_eq!(state.config().hold_modifier(), 3);
        assert_eq!(state.config().threshold_us(), 150_000);
    }
}
