//! Configuration for resource enforcement limits.
//!
//! Provides defaults and serde support for engine resource caps.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configurable resource limits for engine operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceLimits {
    /// Maximum duration allowed for a single execution before timing out.
    #[serde(default = "default_execution_timeout")]
    pub execution_timeout: Duration,
    /// Maximum memory usage in bytes before enforcement triggers.
    #[serde(default = "default_memory_limit")]
    pub memory_limit: usize,
    /// Maximum queue depth before events are dropped or rejected.
    #[serde(default = "default_queue_limit")]
    pub queue_limit: usize,
}

impl ResourceLimits {
    /// Create a new set of resource limits.
    pub fn new(execution_timeout: Duration, memory_limit: usize, queue_limit: usize) -> Self {
        Self {
            execution_timeout,
            memory_limit,
            queue_limit,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            execution_timeout: default_execution_timeout(),
            memory_limit: default_memory_limit(),
            queue_limit: default_queue_limit(),
        }
    }
}

fn default_execution_timeout() -> Duration {
    Duration::from_millis(100)
}

fn default_memory_limit() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_queue_limit() -> usize {
    1000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_design_expectations() {
        let limits = ResourceLimits::default();

        assert_eq!(limits.execution_timeout, Duration::from_millis(100));
        assert_eq!(limits.memory_limit, 10 * 1024 * 1024);
        assert_eq!(limits.queue_limit, 1000);
    }

    #[test]
    fn serde_roundtrip_preserves_limits() {
        let limits = ResourceLimits::new(Duration::from_millis(250), 5 * 1024 * 1024, 50);

        let serialized = serde_json::to_string(&limits).expect("serialize limits");
        let deserialized: ResourceLimits =
            serde_json::from_str(&serialized).expect("deserialize limits");

        assert_eq!(limits, deserialized);
    }
}
