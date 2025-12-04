//! Configuration for event coalescing.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for event coalescing behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoalescingConfig {
    /// Maximum number of events to batch before forcing a flush.
    pub max_batch_size: usize,

    /// Maximum time to wait before flushing buffered events.
    pub flush_timeout: Duration,

    /// Whether to coalesce consecutive repeat events.
    pub coalesce_repeats: bool,
}

impl Default for CoalescingConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 10,
            flush_timeout: Duration::from_millis(5),
            coalesce_repeats: true,
        }
    }
}

impl CoalescingConfig {
    /// Create a new config with custom values.
    pub fn new(max_batch_size: usize, flush_timeout: Duration, coalesce_repeats: bool) -> Self {
        Self {
            max_batch_size,
            flush_timeout,
            coalesce_repeats,
        }
    }

    /// Create a config with coalescing disabled (passthrough mode).
    pub fn passthrough() -> Self {
        Self {
            max_batch_size: 1,
            flush_timeout: Duration::ZERO,
            coalesce_repeats: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sensible_values() {
        let config = CoalescingConfig::default();
        assert_eq!(config.max_batch_size, 10);
        assert_eq!(config.flush_timeout, Duration::from_millis(5));
        assert!(config.coalesce_repeats);
    }

    #[test]
    fn passthrough_config_disables_batching() {
        let config = CoalescingConfig::passthrough();
        assert_eq!(config.max_batch_size, 1);
        assert_eq!(config.flush_timeout, Duration::ZERO);
        assert!(!config.coalesce_repeats);
    }

    #[test]
    fn custom_config_respects_values() {
        let config = CoalescingConfig::new(20, Duration::from_millis(10), false);
        assert_eq!(config.max_batch_size, 20);
        assert_eq!(config.flush_timeout, Duration::from_millis(10));
        assert!(!config.coalesce_repeats);
    }
}
