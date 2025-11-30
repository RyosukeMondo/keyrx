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
            tap_timeout_ms: 200,
            combo_timeout_ms: 50,
            hold_delay_ms: 0,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let config = TimingConfig::default();
        assert_eq!(config.tap_timeout_ms, 200);
        assert_eq!(config.combo_timeout_ms, 50);
        assert_eq!(config.hold_delay_ms, 0);
        assert!(!config.eager_tap);
        assert!(config.permissive_hold);
        assert!(!config.retro_tap);
    }

    #[test]
    fn builder_overrides_fields() {
        let config = TimingConfig::builder()
            .tap_timeout_ms(150)
            .combo_timeout_ms(30)
            .hold_delay_ms(10)
            .eager_tap(true)
            .permissive_hold(false)
            .retro_tap(true)
            .build();

        assert_eq!(
            config,
            TimingConfig {
                tap_timeout_ms: 150,
                combo_timeout_ms: 30,
                hold_delay_ms: 10,
                eager_tap: true,
                permissive_hold: false,
                retro_tap: true,
            }
        );
    }

    #[test]
    fn serde_roundtrip_preserves_values() {
        let config = TimingConfig::builder()
            .tap_timeout_ms(175)
            .combo_timeout_ms(60)
            .hold_delay_ms(5)
            .eager_tap(true)
            .permissive_hold(true)
            .retro_tap(true)
            .build();

        let json = serde_json::to_string(&config).expect("serialize");
        let decoded: TimingConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, config);
    }
}
