use crate::config::{DEFAULT_COMBO_TIMEOUT_MS, DEFAULT_HOLD_DELAY_MS, DEFAULT_TAP_TIMEOUT_MS};
use serde::{Deserialize, Serialize};

/// Timing configuration for decision-making.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimingConfig {
    /// Duration (ms) to distinguish tap from hold. Default: 200.
    pub tap_timeout_ms: u32,
    /// Window (ms) for detecting simultaneous keypresses. Default: 50.
    pub combo_timeout_ms: u32,
    /// Delay (ms) before considering a hold. Default: 0.
    pub hold_delay_ms: u32,
    /// If true, emit tap immediately and correct if becomes hold. Default: false.
    pub eager_tap: bool,
    /// If true, consider as hold if another key pressed during hold. Default: true.
    pub permissive_hold: bool,
    /// If true, release tap even if interrupted. Default: false.
    pub retro_tap: bool,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            tap_timeout_ms: DEFAULT_TAP_TIMEOUT_MS,
            combo_timeout_ms: DEFAULT_COMBO_TIMEOUT_MS,
            hold_delay_ms: DEFAULT_HOLD_DELAY_MS,
            eager_tap: false,
            permissive_hold: true,
            retro_tap: false,
        }
    }
}

impl TimingConfig {
    /// Start building a timing configuration with default values.
    pub fn builder() -> TimingConfigBuilder {
        TimingConfigBuilder::new()
    }
}

/// Fluent builder for `TimingConfig`.
#[derive(Debug)]
pub struct TimingConfigBuilder {
    config: TimingConfig,
}

impl Default for TimingConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TimingConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: TimingConfig::default(),
        }
    }

    pub fn tap_timeout_ms(mut self, tap_timeout_ms: u32) -> Self {
        self.config.tap_timeout_ms = tap_timeout_ms;
        self
    }

    pub fn combo_timeout_ms(mut self, combo_timeout_ms: u32) -> Self {
        self.config.combo_timeout_ms = combo_timeout_ms;
        self
    }

    pub fn hold_delay_ms(mut self, hold_delay_ms: u32) -> Self {
        self.config.hold_delay_ms = hold_delay_ms;
        self
    }

    pub fn eager_tap(mut self, eager_tap: bool) -> Self {
        self.config.eager_tap = eager_tap;
        self
    }

    pub fn permissive_hold(mut self, permissive_hold: bool) -> Self {
        self.config.permissive_hold = permissive_hold;
        self
    }

    pub fn retro_tap(mut self, retro_tap: bool) -> Self {
        self.config.retro_tap = retro_tap;
        self
    }

    pub fn build(self) -> TimingConfig {
        self.config
    }
}
