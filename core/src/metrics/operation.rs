//! Operation types for metrics tracking.
//!
//! This module defines the operations that can be profiled in the KeyRx engine.

use serde::{Deserialize, Serialize};

/// Operations that can be profiled and tracked.
///
/// Each variant represents a distinct operation in the KeyRx engine that
/// can have latency and performance metrics recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operation {
    /// Event processing latency - time from receiving input to starting processing
    EventProcess,
    /// Rule matching latency - time to find matching rules
    RuleMatch,
    /// Action execution latency - time to execute matched actions
    ActionExecute,
    /// Driver read latency - time to read input from OS driver
    DriverRead,
    /// Driver write latency - time to write output to OS driver
    DriverWrite,
}

impl Operation {
    /// Returns a human-readable name for the operation.
    pub fn name(&self) -> &'static str {
        match self {
            Operation::EventProcess => "event_process",
            Operation::RuleMatch => "rule_match",
            Operation::ActionExecute => "action_execute",
            Operation::DriverRead => "driver_read",
            Operation::DriverWrite => "driver_write",
        }
    }

    /// Returns all operation variants.
    pub fn all() -> &'static [Operation] {
        &[
            Operation::EventProcess,
            Operation::RuleMatch,
            Operation::ActionExecute,
            Operation::DriverRead,
            Operation::DriverWrite,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_name() {
        assert_eq!(Operation::EventProcess.name(), "event_process");
        assert_eq!(Operation::RuleMatch.name(), "rule_match");
        assert_eq!(Operation::ActionExecute.name(), "action_execute");
        assert_eq!(Operation::DriverRead.name(), "driver_read");
        assert_eq!(Operation::DriverWrite.name(), "driver_write");
    }

    #[test]
    fn test_operation_all() {
        let all = Operation::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&Operation::EventProcess));
        assert!(all.contains(&Operation::RuleMatch));
        assert!(all.contains(&Operation::ActionExecute));
        assert!(all.contains(&Operation::DriverRead));
        assert!(all.contains(&Operation::DriverWrite));
    }

    #[test]
    fn test_operation_eq() {
        assert_eq!(Operation::EventProcess, Operation::EventProcess);
        assert_ne!(Operation::EventProcess, Operation::RuleMatch);
    }

    #[test]
    fn test_operation_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(Operation::EventProcess, 100);
        map.insert(Operation::RuleMatch, 200);
        assert_eq!(map.get(&Operation::EventProcess), Some(&100));
        assert_eq!(map.get(&Operation::RuleMatch), Some(&200));
    }
}
