//! Capacity and threshold limit constants.
//!
//! This module defines limits used throughout the KeyRx engine for capacity bounds,
//! validation ranges, and performance thresholds.

/// Maximum number of pending decisions (tap-hold and combo) to track in the queue.
///
/// This prevents unbounded growth of the decision queue when many keys are held
/// simultaneously. A value of 32 is sufficient for typical keyboard usage patterns.
pub const MAX_PENDING_DECISIONS: usize = 32;

/// Minimum number of keys required to form a combo.
///
/// Combos require at least 2 keys to be pressed within the combo timeout window.
pub const MIN_COMBO_KEYS: usize = 2;

/// Maximum number of keys allowed in a combo.
///
/// Combos are limited to 4 keys to keep the matching logic efficient and to match
/// typical usage patterns for keyboard shortcuts.
pub const MAX_COMBO_KEYS: usize = 4;

/// Maximum virtual modifier ID (0-255 range for u8).
///
/// Virtual modifiers are stored in a 256-bit bitmap, allowing IDs from 0 to 255.
pub const MAX_MODIFIER_ID: u8 = 255;

/// Maximum timeout value in milliseconds for timing configuration.
///
/// This applies to tap_timeout_ms, combo_timeout_ms, and hold_delay_ms settings.
/// Values above 5000ms are considered unreasonable for interactive use.
pub const MAX_TIMEOUT_MS: i64 = 5000;

/// Default inter-event gap in microseconds for simulated events.
///
/// When simulating keyboard events, this is the default time gap between
/// consecutive events if no explicit timing is provided.
pub const DEFAULT_EVENT_GAP_US: u64 = 1_000;

/// Latency warning threshold in nanoseconds.
///
/// If the average event processing latency exceeds 1ms (1,000,000ns), a warning
/// is displayed in benchmark results. This threshold indicates potential
/// performance issues that could affect typing responsiveness.
pub const LATENCY_THRESHOLD_NS: u64 = 1_000_000;

/// Default regression threshold in microseconds for performance comparisons.
///
/// When comparing against a baseline, a latency regression of more than 100μs
/// is flagged as a potential performance regression.
pub const DEFAULT_REGRESSION_THRESHOLD_US: u64 = 100;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_have_expected_values() {
        assert_eq!(MAX_PENDING_DECISIONS, 32);
        assert_eq!(MIN_COMBO_KEYS, 2);
        assert_eq!(MAX_COMBO_KEYS, 4);
        assert_eq!(MAX_MODIFIER_ID, 255);
        assert_eq!(MAX_TIMEOUT_MS, 5000);
        assert_eq!(DEFAULT_EVENT_GAP_US, 1_000);
        assert_eq!(LATENCY_THRESHOLD_NS, 1_000_000);
        assert_eq!(DEFAULT_REGRESSION_THRESHOLD_US, 100);
    }

    #[test]
    fn combo_keys_range_is_valid() {
        // Min must be at least 2 (a combo requires multiple keys)
        assert!(MIN_COMBO_KEYS >= 2);
        // Max must be greater than or equal to min
        assert!(MAX_COMBO_KEYS >= MIN_COMBO_KEYS);
        // Max should be reasonable for typical usage
        assert!(MAX_COMBO_KEYS <= 10);
    }

    #[test]
    fn pending_decisions_capacity_is_reasonable() {
        // Should be power of 2 for efficient allocation
        assert!(MAX_PENDING_DECISIONS.is_power_of_two());
        // Should be sufficient for typical usage (10 fingers, simultaneous combos)
        assert!(MAX_PENDING_DECISIONS >= 16);
        // Should not be excessive (memory efficiency)
        assert!(MAX_PENDING_DECISIONS <= 256);
    }

    #[test]
    fn modifier_id_uses_full_u8_range() {
        // Should use full u8 range (0-255) for bitmap storage
        assert_eq!(MAX_MODIFIER_ID, u8::MAX);
    }

    #[test]
    fn timeout_limit_is_reasonable() {
        // 5 seconds is a reasonable upper bound for interactive use
        assert_eq!(MAX_TIMEOUT_MS, 5000);
        // Should be positive
        assert!(MAX_TIMEOUT_MS > 0);
    }

    #[test]
    fn latency_thresholds_are_consistent() {
        // Threshold should be 1ms (1,000,000ns = 1ms)
        assert_eq!(LATENCY_THRESHOLD_NS, 1_000_000);
        // Regression threshold should be less than latency threshold
        assert!(DEFAULT_REGRESSION_THRESHOLD_US < LATENCY_THRESHOLD_NS / 1000);
    }

    #[test]
    fn event_gap_is_reasonable() {
        // 1ms is a reasonable default gap between events
        assert_eq!(DEFAULT_EVENT_GAP_US, 1_000);
        // Should be at least 100μs for timing accuracy
        assert!(DEFAULT_EVENT_GAP_US >= 100);
        // Should not exceed 10ms (would feel sluggish)
        assert!(DEFAULT_EVENT_GAP_US <= 10_000);
    }
}
