//! UAT test runner type definitions.
//!
//! This module contains the core types used by the UAT test runner:
//! - Priority levels for test categorization
//! - UatTest representing a discovered test
//! - UatFilter for selecting tests to run
//! - UatResult and UatResults for test outcomes

use std::str::FromStr;

/// Test priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize)]
pub enum Priority {
    /// Critical path tests that must always pass.
    P0,
    /// High priority tests.
    P1,
    /// Medium priority tests.
    #[default]
    P2,
}

impl FromStr for Priority {
    type Err = ();

    /// Parse priority from string (e.g., "P0", "P1", "P2").
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "P0" => Ok(Priority::P0),
            "P1" => Ok(Priority::P1),
            "P2" => Ok(Priority::P2),
            _ => Err(()),
        }
    }
}

/// A discovered UAT test.
#[derive(Debug, Clone, serde::Serialize)]
pub struct UatTest {
    /// Test name.
    pub name: String,
    /// Source file path.
    pub file: String,
    /// Test category.
    pub category: String,
    /// Test priority.
    pub priority: Priority,
    /// Linked requirement IDs.
    pub requirements: Vec<String>,
    /// Latency threshold in microseconds.
    pub latency_threshold: Option<u64>,
}

/// Filter for selecting UAT tests.
#[derive(Debug, Default, Clone)]
pub struct UatFilter {
    /// Filter by categories.
    pub categories: Vec<String>,
    /// Filter by priorities.
    pub priorities: Vec<Priority>,
    /// Filter by name pattern.
    pub pattern: Option<String>,
}

impl UatFilter {
    /// Check if a test matches this filter.
    ///
    /// All specified criteria must match (AND logic):
    /// - If categories are specified, test category must be in the list
    /// - If priorities are specified, test priority must be in the list
    /// - If pattern is specified, test name must contain the pattern
    pub fn matches(&self, test: &UatTest) -> bool {
        // Category filter (if any categories specified, test must match one)
        if !self.categories.is_empty() && !self.categories.contains(&test.category) {
            return false;
        }

        // Priority filter (if any priorities specified, test must match one)
        if !self.priorities.is_empty() && !self.priorities.contains(&test.priority) {
            return false;
        }

        // Pattern filter (substring match on test name)
        if let Some(ref pattern) = self.pattern {
            if !test.name.contains(pattern) {
                return false;
            }
        }

        true
    }
}

/// Result of a single UAT test.
#[derive(Debug, Clone, serde::Serialize)]
pub struct UatResult {
    /// Test that was run.
    pub test: UatTest,
    /// Whether the test passed.
    pub passed: bool,
    /// Duration in microseconds.
    pub duration_us: u64,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Aggregated results from a UAT run.
#[derive(Debug, Clone, serde::Serialize)]
pub struct UatResults {
    /// Total tests run.
    pub total: usize,
    /// Tests that passed.
    pub passed: usize,
    /// Tests that failed.
    pub failed: usize,
    /// Tests that were skipped.
    pub skipped: usize,
    /// Total duration in microseconds.
    pub duration_us: u64,
    /// Individual test results.
    pub results: Vec<UatResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_from_str_parses_valid_values() {
        assert_eq!("P0".parse::<Priority>(), Ok(Priority::P0));
        assert_eq!("P1".parse::<Priority>(), Ok(Priority::P1));
        assert_eq!("P2".parse::<Priority>(), Ok(Priority::P2));
        assert_eq!("p0".parse::<Priority>(), Ok(Priority::P0));
        assert_eq!(" P1 ".parse::<Priority>(), Ok(Priority::P1));
    }

    #[test]
    fn priority_from_str_returns_err_for_invalid() {
        assert!("P3".parse::<Priority>().is_err());
        assert!("".parse::<Priority>().is_err());
        assert!("high".parse::<Priority>().is_err());
    }

    #[test]
    fn priority_default_is_p2() {
        assert_eq!(Priority::default(), Priority::P2);
    }

    #[test]
    fn filter_matches_all_when_empty() {
        let filter = UatFilter::default();
        let test = UatTest {
            name: "uat_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P1,
            requirements: vec![],
            latency_threshold: None,
        };
        assert!(filter.matches(&test));
    }

    #[test]
    fn filter_matches_by_category() {
        let filter = UatFilter {
            categories: vec!["core".to_string()],
            ..Default::default()
        };

        let matching = UatTest {
            name: "uat_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P1,
            requirements: vec![],
            latency_threshold: None,
        };

        let non_matching = UatTest {
            name: "uat_test2".to_string(),
            file: "test.rhai".to_string(),
            category: "layers".to_string(),
            priority: Priority::P1,
            requirements: vec![],
            latency_threshold: None,
        };

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&non_matching));
    }

    #[test]
    fn filter_matches_by_priority() {
        let filter = UatFilter {
            priorities: vec![Priority::P0, Priority::P1],
            ..Default::default()
        };

        let matching = UatTest {
            name: "uat_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P0,
            requirements: vec![],
            latency_threshold: None,
        };

        let non_matching = UatTest {
            name: "uat_test2".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P2,
            requirements: vec![],
            latency_threshold: None,
        };

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&non_matching));
    }

    #[test]
    fn filter_matches_by_pattern() {
        let filter = UatFilter {
            pattern: Some("layer".to_string()),
            ..Default::default()
        };

        let matching = UatTest {
            name: "uat_layer_switch".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P1,
            requirements: vec![],
            latency_threshold: None,
        };

        let non_matching = UatTest {
            name: "uat_basic_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P1,
            requirements: vec![],
            latency_threshold: None,
        };

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&non_matching));
    }

    #[test]
    fn filter_uses_and_logic() {
        let filter = UatFilter {
            categories: vec!["core".to_string()],
            priorities: vec![Priority::P0],
            pattern: Some("basic".to_string()),
        };

        // Matches all criteria
        let matching = UatTest {
            name: "uat_basic_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P0,
            requirements: vec![],
            latency_threshold: None,
        };

        // Wrong category
        let wrong_category = UatTest {
            name: "uat_basic_test".to_string(),
            file: "test.rhai".to_string(),
            category: "layers".to_string(),
            priority: Priority::P0,
            requirements: vec![],
            latency_threshold: None,
        };

        // Wrong priority
        let wrong_priority = UatTest {
            name: "uat_basic_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P1,
            requirements: vec![],
            latency_threshold: None,
        };

        // Wrong pattern
        let wrong_pattern = UatTest {
            name: "uat_advanced_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P0,
            requirements: vec![],
            latency_threshold: None,
        };

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&wrong_category));
        assert!(!filter.matches(&wrong_priority));
        assert!(!filter.matches(&wrong_pattern));
    }
}
