//! Configuration validation and value clamping.
//!
//! This module provides validation logic for configuration values,
//! ensuring they fall within acceptable ranges and logging warnings
//! when values are clamped.

use super::Config;

// =============================================================================
// Validation
// =============================================================================

/// Validate configuration and clamp values to valid ranges.
///
/// This function modifies the config in place, clamping any out-of-range
/// values and logging warnings for invalid inputs.
pub fn validate_and_clamp(config: &mut Config) {
    // Timing validation
    config.timing.tap_timeout_ms =
        clamp_with_warning(config.timing.tap_timeout_ms, 50, 1000, "tap_timeout_ms");

    config.timing.combo_timeout_ms =
        clamp_with_warning(config.timing.combo_timeout_ms, 10, 200, "combo_timeout_ms");

    config.timing.hold_delay_ms =
        clamp_with_warning(config.timing.hold_delay_ms, 0, 500, "hold_delay_ms");

    // UI validation
    config.ui.max_events_history =
        clamp_with_warning(config.ui.max_events_history, 50, 1000, "max_events_history");

    config.ui.animation_duration_ms = clamp_with_warning(
        config.ui.animation_duration_ms,
        50,
        500,
        "animation_duration_ms",
    );

    // Performance validation
    config.performance.regression_threshold_us = clamp_with_warning(
        config.performance.regression_threshold_us,
        10,
        1000,
        "regression_threshold_us",
    );

    // Ensure caution < warning for latency thresholds
    if config.performance.latency_caution_us >= config.performance.latency_warning_us {
        tracing::warn!(
            service = "keyrx",
            event = "config_validation",
            component = "config",
            field = "latency_caution_us",
            "latency_caution_us must be less than latency_warning_us, adjusting"
        );
        config.performance.latency_caution_us = config.performance.latency_warning_us / 2;
    }
}

/// Clamp a value to a range and log a warning if clamping occurred.
fn clamp_with_warning<T: Ord + Copy + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    field_name: &str,
) -> T {
    if value < min {
        tracing::warn!(
            service = "keyrx",
            event = "config_clamped",
            component = "config",
            field = field_name,
            value = %value,
            min = %min,
            "Value below minimum, clamping to minimum"
        );
        min
    } else if value > max {
        tracing::warn!(
            service = "keyrx",
            event = "config_clamped",
            component = "config",
            field = field_name,
            value = %value,
            max = %max,
            "Value above maximum, clamping to maximum"
        );
        max
    } else {
        value
    }
}

// =============================================================================
// CLI Overrides
// =============================================================================

/// Merge CLI argument overrides into configuration.
///
/// Any `Some` value will override the corresponding config value.
/// `None` values leave the config unchanged.
///
/// # Arguments
///
/// * `config` - Configuration to modify
/// * `tap_timeout_ms` - Override for tap timeout
/// * `combo_timeout_ms` - Override for combo timeout
/// * `hold_delay_ms` - Override for hold delay
///
/// # Example
///
/// ```
/// use keyrx_core::config::{Config, merge_cli_overrides};
///
/// let mut config = Config::default();
/// merge_cli_overrides(&mut config, Some(150), None, Some(10));
///
/// assert_eq!(config.timing.tap_timeout_ms, 150);
/// assert_eq!(config.timing.combo_timeout_ms, 50); // unchanged
/// assert_eq!(config.timing.hold_delay_ms, 10);
/// ```
pub fn merge_cli_overrides(
    config: &mut Config,
    tap_timeout_ms: Option<u32>,
    combo_timeout_ms: Option<u32>,
    hold_delay_ms: Option<u32>,
) {
    if let Some(tap) = tap_timeout_ms {
        config.timing.tap_timeout_ms = tap;
    }
    if let Some(combo) = combo_timeout_ms {
        config.timing.combo_timeout_ms = combo;
    }
    if let Some(hold) = hold_delay_ms {
        config.timing.hold_delay_ms = hold;
    }

    // Re-validate after merging CLI overrides
    validate_and_clamp(config);
}
