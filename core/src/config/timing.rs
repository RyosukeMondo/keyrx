//! Timing constants for KeyRx decision-making.
//!
//! These constants control the timing behavior for tap-hold detection,
//! combo key windows, and hold delays.

/// Default tap timeout in milliseconds.
///
/// Duration to distinguish a tap from a hold. If a key is released within
/// this duration, it's considered a tap; otherwise, it becomes a hold.
///
/// **Valid range:** 50-1000 ms (typical: 150-250 ms)
///
/// **Default:** 200 ms
pub const DEFAULT_TAP_TIMEOUT_MS: u32 = 200;

/// Default combo timeout in milliseconds.
///
/// Window for detecting simultaneous keypresses as a combo. All keys in
/// a combo must be pressed within this duration of the first key.
///
/// **Valid range:** 10-200 ms (typical: 30-80 ms)
///
/// **Default:** 50 ms
pub const DEFAULT_COMBO_TIMEOUT_MS: u32 = 50;

/// Default hold delay in milliseconds.
///
/// Additional delay before considering a key press as a hold. This extends
/// the tap window beyond `tap_timeout_ms`.
///
/// **Valid range:** 0-500 ms (typical: 0 ms)
///
/// **Default:** 0 ms
pub const DEFAULT_HOLD_DELAY_MS: u32 = 0;

/// Microseconds per millisecond conversion factor.
///
/// Used for converting millisecond timing values to microsecond timestamps.
pub const MICROS_PER_MS: u64 = 1_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_have_expected_values() {
        assert_eq!(DEFAULT_TAP_TIMEOUT_MS, 200);
        assert_eq!(DEFAULT_COMBO_TIMEOUT_MS, 50);
        assert_eq!(DEFAULT_HOLD_DELAY_MS, 0);
        assert_eq!(MICROS_PER_MS, 1_000);
    }

    #[test]
    fn conversion_factor_is_correct() {
        // 1ms = 1000μs
        let ms: u64 = 200;
        let expected_us: u64 = 200_000;
        assert_eq!(ms * MICROS_PER_MS, expected_us);
    }
}
