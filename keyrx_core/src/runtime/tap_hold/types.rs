//! Core types for tap-hold functionality.
//!
//! This module contains the foundational types used by the tap-hold state machine:
//! - [`TapHoldPhase`] - The current state of a tap-hold key
//! - [`TapHoldConfig`] - Configuration for tap and hold behavior
//! - [`TapHoldOutput`] - Output events produced by the state machine

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

    /// Returns the phase name as a static string.
    ///
    /// Used for logging and debugging purposes.
    pub const fn as_str(&self) -> &'static str {
        match self {
            TapHoldPhase::Idle => "Idle",
            TapHoldPhase::Pending => "Pending",
            TapHoldPhase::Hold => "Hold",
        }
    }
}

impl core::fmt::Display for TapHoldPhase {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
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
    /// Whether tap output is suppressed (hold_only mode)
    tap_suppressed: bool,
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
            tap_suppressed: false,
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

    /// Creates a hold-only configuration (tap suppressed).
    ///
    /// The tap key is set to a dummy value since it won't be used.
    pub const fn hold_only(hold_modifier: u8, threshold_us: u64) -> Self {
        Self {
            tap_key: KeyCode::A, // Dummy, won't be used when tap_suppressed
            hold_modifier,
            threshold_us,
            tap_suppressed: true,
        }
    }

    /// Creates a hold-only configuration from milliseconds threshold.
    pub const fn hold_only_ms(hold_modifier: u8, threshold_ms: u16) -> Self {
        Self::hold_only(hold_modifier, threshold_ms as u64 * 1000)
    }

    /// Returns true if tap output is suppressed (hold_only mode).
    pub const fn is_tap_suppressed(&self) -> bool {
        self.tap_suppressed
    }
}

/// Output event from the tap-hold state machine.
///
/// Represents actions that should be taken in response to input events.
/// The processor produces these outputs which are then translated into
/// actual key events or modifier state changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapHoldOutput {
    /// Emit a key event (press or release)
    KeyEvent {
        /// The key to emit
        key: KeyCode,
        /// True for press, false for release
        is_press: bool,
        /// Timestamp to use for the output event
        timestamp_us: u64,
    },
    /// Activate a hold modifier
    ActivateModifier {
        /// The modifier ID to activate
        modifier_id: u8,
    },
    /// Deactivate a hold modifier
    DeactivateModifier {
        /// The modifier ID to deactivate
        modifier_id: u8,
    },
}

impl TapHoldOutput {
    /// Creates a key press output event.
    pub const fn key_press(key: KeyCode, timestamp_us: u64) -> Self {
        Self::KeyEvent {
            key,
            is_press: true,
            timestamp_us,
        }
    }

    /// Creates a key release output event.
    pub const fn key_release(key: KeyCode, timestamp_us: u64) -> Self {
        Self::KeyEvent {
            key,
            is_press: false,
            timestamp_us,
        }
    }

    /// Creates a modifier activation output.
    pub const fn activate_modifier(modifier_id: u8) -> Self {
        Self::ActivateModifier { modifier_id }
    }

    /// Creates a modifier deactivation output.
    pub const fn deactivate_modifier(modifier_id: u8) -> Self {
        Self::DeactivateModifier { modifier_id }
    }
}
