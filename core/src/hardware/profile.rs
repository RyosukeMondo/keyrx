use serde::{Deserialize, Serialize};

const DEFAULT_DEBOUNCE_MS: u32 = 5;
const DEFAULT_REPEAT_DELAY_MS: u32 = 250;
const DEFAULT_REPEAT_RATE_MS: u32 = 33;
const DEFAULT_SCAN_INTERVAL_US: u32 = 1000;

/// Source for a profile so consumers can reason about trust level and mutability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileSource {
    Builtin,
    Community,
    Calibrated,
    Custom,
}

/// Timing characteristics for a specific piece of hardware.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimingConfig {
    pub debounce_ms: u32,
    pub repeat_delay_ms: u32,
    pub repeat_rate_ms: u32,
    pub scan_interval_us: u32,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            debounce_ms: DEFAULT_DEBOUNCE_MS,
            repeat_delay_ms: DEFAULT_REPEAT_DELAY_MS,
            repeat_rate_ms: DEFAULT_REPEAT_RATE_MS,
            scan_interval_us: DEFAULT_SCAN_INTERVAL_US,
        }
    }
}

/// Hardware profile with timing configuration and metadata for lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: String,
    pub timing: TimingConfig,
    pub source: ProfileSource,
}

impl HardwareProfile {
    /// Build a profile from identifiers and timing configuration.
    pub fn new(
        vendor_id: u16,
        product_id: u16,
        name: impl Into<String>,
        timing: TimingConfig,
        source: ProfileSource,
    ) -> Self {
        Self {
            vendor_id,
            product_id,
            name: name.into(),
            timing,
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_config_has_sane_defaults() {
        let timing = TimingConfig::default();
        assert_eq!(timing.debounce_ms, DEFAULT_DEBOUNCE_MS);
        assert_eq!(timing.repeat_delay_ms, DEFAULT_REPEAT_DELAY_MS);
        assert_eq!(timing.repeat_rate_ms, DEFAULT_REPEAT_RATE_MS);
        assert_eq!(timing.scan_interval_us, DEFAULT_SCAN_INTERVAL_US);
    }

    #[test]
    fn hardware_profile_round_trips_via_serde() {
        let profile = HardwareProfile::new(
            0x1234,
            0x5678,
            "Test Board",
            TimingConfig {
                debounce_ms: 3,
                repeat_delay_ms: 200,
                repeat_rate_ms: 25,
                scan_interval_us: 800,
            },
            ProfileSource::Custom,
        );

        let encoded = serde_json::to_string(&profile).expect("serialize");
        let decoded: HardwareProfile = serde_json::from_str(&encoded).expect("deserialize");
        assert_eq!(decoded, profile);
    }
}
