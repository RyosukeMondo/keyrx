//! Requirements coverage mapping and reporting.

use std::collections::{HashMap, HashSet};

use chrono::Utc;

use crate::uat::runner::{UatResults, UatTest};

/// Coverage status for a requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageStatus {
    /// All linked tests pass.
    Verified,
    /// Some linked tests fail.
    AtRisk,
    /// No linked tests.
    Uncovered,
}

/// Coverage information for a single requirement.
#[derive(Debug, Clone)]
pub struct RequirementCoverage {
    /// Requirement ID.
    pub id: String,
    /// Linked test names.
    pub linked_tests: Vec<String>,
    /// Coverage status.
    pub status: CoverageStatus,
    /// Last verification timestamp (ISO 8601 format).
    pub last_verified: Option<String>,
}

/// Map of requirement IDs to their coverage.
#[derive(Debug, Clone, Default)]
pub struct CoverageMap {
    /// Mapping of requirement ID to coverage info.
    pub requirements: HashMap<String, RequirementCoverage>,
}

impl CoverageMap {
    /// Create an empty coverage map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the total number of requirements tracked.
    pub fn total(&self) -> usize {
        self.requirements.len()
    }

    /// Get the number of verified requirements.
    pub fn verified_count(&self) -> usize {
        self.requirements
            .values()
            .filter(|r| r.status == CoverageStatus::Verified)
            .count()
    }

    /// Get the number of at-risk requirements.
    pub fn at_risk_count(&self) -> usize {
        self.requirements
            .values()
            .filter(|r| r.status == CoverageStatus::AtRisk)
            .count()
    }

    /// Get the number of uncovered requirements.
    pub fn uncovered_count(&self) -> usize {
        self.requirements
            .values()
            .filter(|r| r.status == CoverageStatus::Uncovered)
            .count()
    }

    /// Calculate coverage percentage (verified / total).
    pub fn coverage_percentage(&self) -> f64 {
        if self.requirements.is_empty() {
            return 1.0; // No requirements means 100% coverage
        }
        self.verified_count() as f64 / self.requirements.len() as f64
    }
}

/// Coverage report with summary statistics.
#[derive(Debug, Clone)]
pub struct CoverageReport {
    /// Coverage map.
    pub coverage: CoverageMap,
    /// Total requirements.
    pub total: usize,
    /// Verified requirements.
    pub verified: usize,
    /// At-risk requirements.
    pub at_risk: usize,
    /// Uncovered requirements.
    pub uncovered: usize,
    /// Coverage percentage (0.0-1.0).
    pub coverage_percentage: f64,
    /// Timestamp when the report was generated.
    pub generated_at: String,
}

/// Mapper for building coverage information from test metadata and results.
#[derive(Debug, Default)]
pub struct CoverageMapper;

impl CoverageMapper {
    /// Create a new coverage mapper.
    pub fn new() -> Self {
        Self
    }

    /// Build a coverage map from test metadata and results.
    ///
    /// Extracts requirement IDs from test `@requirement` metadata and links
    /// them to test results to determine coverage status.
    ///
    /// # Arguments
    /// * `tests` - Discovered UAT tests with requirement metadata
    /// * `results` - Execution results for the tests
    ///
    /// # Returns
    /// A `CoverageMap` linking requirements to tests with status information.
    pub fn build(&self, tests: &[UatTest], results: &UatResults) -> CoverageMap {
        // Build a map of test name -> passed status from results
        let test_results: HashMap<&str, bool> = results
            .results
            .iter()
            .map(|r| (r.test.name.as_str(), r.passed))
            .collect();

        // Collect all requirements and their linked tests
        let mut req_tests: HashMap<String, Vec<String>> = HashMap::new();

        for test in tests {
            for req_id in &test.requirements {
                req_tests
                    .entry(req_id.clone())
                    .or_default()
                    .push(test.name.clone());
            }
        }

        // Build coverage entries
        let timestamp = Utc::now().to_rfc3339();
        let mut requirements = HashMap::new();

        for (req_id, linked_tests) in req_tests {
            // Determine status based on test results
            let status = self.calculate_status(&linked_tests, &test_results);

            // Only set last_verified if all tests passed
            let last_verified = if status == CoverageStatus::Verified {
                Some(timestamp.clone())
            } else {
                None
            };

            requirements.insert(
                req_id.clone(),
                RequirementCoverage {
                    id: req_id,
                    linked_tests,
                    status,
                    last_verified,
                },
            );
        }

        tracing::info!(
            service = "keyrx",
            event = "coverage_map_built",
            component = "coverage_mapper",
            requirements = requirements.len(),
            "Built coverage map with {} requirements",
            requirements.len()
        );

        CoverageMap { requirements }
    }

    /// Build a coverage map with additional known requirements.
    ///
    /// This allows tracking requirements that don't have any linked tests yet,
    /// marking them as `Uncovered`.
    ///
    /// # Arguments
    /// * `tests` - Discovered UAT tests with requirement metadata
    /// * `results` - Execution results for the tests
    /// * `known_requirements` - Set of all known requirement IDs
    ///
    /// # Returns
    /// A `CoverageMap` including uncovered requirements.
    pub fn build_with_known_requirements(
        &self,
        tests: &[UatTest],
        results: &UatResults,
        known_requirements: &HashSet<String>,
    ) -> CoverageMap {
        let mut map = self.build(tests, results);

        // Add any known requirements that don't have linked tests
        for req_id in known_requirements {
            if !map.requirements.contains_key(req_id) {
                map.requirements.insert(
                    req_id.clone(),
                    RequirementCoverage {
                        id: req_id.clone(),
                        linked_tests: Vec::new(),
                        status: CoverageStatus::Uncovered,
                        last_verified: None,
                    },
                );
            }
        }

        map
    }

    /// Generate a coverage report from a coverage map.
    ///
    /// Creates a `CoverageReport` with summary statistics including:
    /// - Total requirements tracked
    /// - Verified, at-risk, and uncovered counts
    /// - Coverage percentage
    /// - Timestamp of report generation
    ///
    /// # Arguments
    /// * `map` - The coverage map to generate a report from
    ///
    /// # Returns
    /// A `CoverageReport` with all statistics calculated.
    pub fn report(&self, map: &CoverageMap) -> CoverageReport {
        let total = map.total();
        let verified = map.verified_count();
        let at_risk = map.at_risk_count();
        let uncovered = map.uncovered_count();
        let coverage_percentage = map.coverage_percentage();
        let generated_at = Utc::now().to_rfc3339();

        tracing::info!(
            service = "keyrx",
            event = "coverage_report_generated",
            component = "coverage_mapper",
            total = total,
            verified = verified,
            at_risk = at_risk,
            uncovered = uncovered,
            coverage_pct = format!("{:.1}%", coverage_percentage * 100.0),
            "Generated coverage report: {}/{} verified ({:.1}%)",
            verified,
            total,
            coverage_percentage * 100.0
        );

        CoverageReport {
            coverage: map.clone(),
            total,
            verified,
            at_risk,
            uncovered,
            coverage_percentage,
            generated_at,
        }
    }

    /// Calculate coverage status based on linked test results.
    fn calculate_status(
        &self,
        linked_tests: &[String],
        test_results: &HashMap<&str, bool>,
    ) -> CoverageStatus {
        if linked_tests.is_empty() {
            return CoverageStatus::Uncovered;
        }

        let mut any_passed = false;
        let mut any_failed = false;

        for test_name in linked_tests {
            match test_results.get(test_name.as_str()) {
                Some(true) => any_passed = true,
                Some(false) => any_failed = true,
                None => {
                    // Test was discovered but not run (skipped or filtered out)
                    // We consider this as not contributing to coverage
                }
            }
        }

        if any_failed {
            CoverageStatus::AtRisk
        } else if any_passed {
            CoverageStatus::Verified
        } else {
            // All tests were skipped/not run
            CoverageStatus::Uncovered
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uat::runner::{Priority, UatResult};

    fn create_test(name: &str, requirements: Vec<&str>) -> UatTest {
        UatTest {
            name: name.to_string(),
            file: "test.rhai".to_string(),
            category: "default".to_string(),
            priority: Priority::P2,
            requirements: requirements.into_iter().map(String::from).collect(),
            latency_threshold: None,
        }
    }

    fn create_result(test: UatTest, passed: bool) -> UatResult {
        UatResult {
            test,
            passed,
            duration_us: 100,
            error: if passed {
                None
            } else {
                Some("Test failed".to_string())
            },
        }
    }

    fn create_results(results: Vec<UatResult>) -> UatResults {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        UatResults {
            total,
            passed,
            failed: total - passed,
            skipped: 0,
            duration_us: 1000,
            results,
        }
    }

    #[test]
    fn build_empty_tests() {
        let mapper = CoverageMapper::new();
        let tests: Vec<UatTest> = vec![];
        let results = create_results(vec![]);

        let map = mapper.build(&tests, &results);

        assert!(map.requirements.is_empty());
        assert_eq!(map.total(), 0);
    }

    #[test]
    fn build_single_requirement_verified() {
        let mapper = CoverageMapper::new();
        let test = create_test("uat_test1", vec!["1.1"]);
        let tests = vec![test.clone()];
        let results = create_results(vec![create_result(test, true)]);

        let map = mapper.build(&tests, &results);

        assert_eq!(map.total(), 1);
        let req = map.requirements.get("1.1").unwrap();
        assert_eq!(req.id, "1.1");
        assert_eq!(req.linked_tests, vec!["uat_test1"]);
        assert_eq!(req.status, CoverageStatus::Verified);
        assert!(req.last_verified.is_some());
    }

    #[test]
    fn build_single_requirement_at_risk() {
        let mapper = CoverageMapper::new();
        let test = create_test("uat_test1", vec!["1.1"]);
        let tests = vec![test.clone()];
        let results = create_results(vec![create_result(test, false)]);

        let map = mapper.build(&tests, &results);

        assert_eq!(map.total(), 1);
        let req = map.requirements.get("1.1").unwrap();
        assert_eq!(req.status, CoverageStatus::AtRisk);
        assert!(req.last_verified.is_none());
    }

    #[test]
    fn build_multiple_tests_same_requirement() {
        let mapper = CoverageMapper::new();
        let test1 = create_test("uat_test1", vec!["1.1"]);
        let test2 = create_test("uat_test2", vec!["1.1"]);
        let tests = vec![test1.clone(), test2.clone()];
        let results = create_results(vec![create_result(test1, true), create_result(test2, true)]);

        let map = mapper.build(&tests, &results);

        assert_eq!(map.total(), 1);
        let req = map.requirements.get("1.1").unwrap();
        assert_eq!(req.linked_tests.len(), 2);
        assert!(req.linked_tests.contains(&"uat_test1".to_string()));
        assert!(req.linked_tests.contains(&"uat_test2".to_string()));
        assert_eq!(req.status, CoverageStatus::Verified);
    }

    #[test]
    fn build_multiple_tests_one_fails() {
        let mapper = CoverageMapper::new();
        let test1 = create_test("uat_test1", vec!["1.1"]);
        let test2 = create_test("uat_test2", vec!["1.1"]);
        let tests = vec![test1.clone(), test2.clone()];
        let results = create_results(vec![
            create_result(test1, true),
            create_result(test2, false), // One fails
        ]);

        let map = mapper.build(&tests, &results);

        let req = map.requirements.get("1.1").unwrap();
        assert_eq!(req.status, CoverageStatus::AtRisk);
    }

    #[test]
    fn build_test_with_multiple_requirements() {
        let mapper = CoverageMapper::new();
        let test = create_test("uat_test1", vec!["1.1", "2.3", "4.5"]);
        let tests = vec![test.clone()];
        let results = create_results(vec![create_result(test, true)]);

        let map = mapper.build(&tests, &results);

        assert_eq!(map.total(), 3);
        assert!(map.requirements.contains_key("1.1"));
        assert!(map.requirements.contains_key("2.3"));
        assert!(map.requirements.contains_key("4.5"));

        for req in map.requirements.values() {
            assert_eq!(req.status, CoverageStatus::Verified);
            assert_eq!(req.linked_tests, vec!["uat_test1"]);
        }
    }

    #[test]
    fn build_with_known_requirements_adds_uncovered() {
        let mapper = CoverageMapper::new();
        let test = create_test("uat_test1", vec!["1.1"]);
        let tests = vec![test.clone()];
        let results = create_results(vec![create_result(test, true)]);

        let known: HashSet<String> = ["1.1", "2.1", "3.1"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let map = mapper.build_with_known_requirements(&tests, &results, &known);

        assert_eq!(map.total(), 3);
        assert_eq!(
            map.requirements.get("1.1").unwrap().status,
            CoverageStatus::Verified
        );
        assert_eq!(
            map.requirements.get("2.1").unwrap().status,
            CoverageStatus::Uncovered
        );
        assert_eq!(
            map.requirements.get("3.1").unwrap().status,
            CoverageStatus::Uncovered
        );
    }

    #[test]
    fn build_test_not_run() {
        let mapper = CoverageMapper::new();
        let test = create_test("uat_test1", vec!["1.1"]);
        let tests = vec![test]; // Test discovered but not in results
        let results = create_results(vec![]); // No results

        let map = mapper.build(&tests, &results);

        let req = map.requirements.get("1.1").unwrap();
        assert_eq!(req.status, CoverageStatus::Uncovered);
    }

    #[test]
    fn coverage_map_statistics() {
        let mut map = CoverageMap::new();
        map.requirements.insert(
            "1.1".to_string(),
            RequirementCoverage {
                id: "1.1".to_string(),
                linked_tests: vec!["test1".to_string()],
                status: CoverageStatus::Verified,
                last_verified: Some("2024-01-01".to_string()),
            },
        );
        map.requirements.insert(
            "1.2".to_string(),
            RequirementCoverage {
                id: "1.2".to_string(),
                linked_tests: vec!["test2".to_string()],
                status: CoverageStatus::AtRisk,
                last_verified: None,
            },
        );
        map.requirements.insert(
            "1.3".to_string(),
            RequirementCoverage {
                id: "1.3".to_string(),
                linked_tests: vec![],
                status: CoverageStatus::Uncovered,
                last_verified: None,
            },
        );

        assert_eq!(map.total(), 3);
        assert_eq!(map.verified_count(), 1);
        assert_eq!(map.at_risk_count(), 1);
        assert_eq!(map.uncovered_count(), 1);
        assert!((map.coverage_percentage() - 1.0 / 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn coverage_map_empty_percentage() {
        let map = CoverageMap::new();
        assert_eq!(map.coverage_percentage(), 1.0);
    }

    #[test]
    fn coverage_status_equality() {
        assert_eq!(CoverageStatus::Verified, CoverageStatus::Verified);
        assert_ne!(CoverageStatus::Verified, CoverageStatus::AtRisk);
        assert_ne!(CoverageStatus::AtRisk, CoverageStatus::Uncovered);
    }

    #[test]
    fn report_empty_map() {
        let mapper = CoverageMapper::new();
        let map = CoverageMap::new();

        let report = mapper.report(&map);

        assert_eq!(report.total, 0);
        assert_eq!(report.verified, 0);
        assert_eq!(report.at_risk, 0);
        assert_eq!(report.uncovered, 0);
        assert_eq!(report.coverage_percentage, 1.0);
        assert!(!report.generated_at.is_empty());
    }

    #[test]
    fn report_with_mixed_statuses() {
        let mapper = CoverageMapper::new();
        let mut map = CoverageMap::new();

        map.requirements.insert(
            "1.1".to_string(),
            RequirementCoverage {
                id: "1.1".to_string(),
                linked_tests: vec!["test1".to_string()],
                status: CoverageStatus::Verified,
                last_verified: Some("2024-01-01".to_string()),
            },
        );
        map.requirements.insert(
            "1.2".to_string(),
            RequirementCoverage {
                id: "1.2".to_string(),
                linked_tests: vec!["test2".to_string()],
                status: CoverageStatus::Verified,
                last_verified: Some("2024-01-01".to_string()),
            },
        );
        map.requirements.insert(
            "1.3".to_string(),
            RequirementCoverage {
                id: "1.3".to_string(),
                linked_tests: vec!["test3".to_string()],
                status: CoverageStatus::AtRisk,
                last_verified: None,
            },
        );
        map.requirements.insert(
            "1.4".to_string(),
            RequirementCoverage {
                id: "1.4".to_string(),
                linked_tests: vec![],
                status: CoverageStatus::Uncovered,
                last_verified: None,
            },
        );

        let report = mapper.report(&map);

        assert_eq!(report.total, 4);
        assert_eq!(report.verified, 2);
        assert_eq!(report.at_risk, 1);
        assert_eq!(report.uncovered, 1);
        assert_eq!(report.coverage_percentage, 0.5);
        assert!(!report.generated_at.is_empty());
        // Verify report includes the coverage map
        assert_eq!(report.coverage.total(), 4);
    }

    #[test]
    fn report_all_verified() {
        let mapper = CoverageMapper::new();
        let test = create_test("uat_test1", vec!["1.1", "1.2"]);
        let tests = vec![test.clone()];
        let results = create_results(vec![create_result(test, true)]);

        let map = mapper.build(&tests, &results);
        let report = mapper.report(&map);

        assert_eq!(report.total, 2);
        assert_eq!(report.verified, 2);
        assert_eq!(report.at_risk, 0);
        assert_eq!(report.uncovered, 0);
        assert_eq!(report.coverage_percentage, 1.0);
    }

    #[test]
    fn report_timestamp_format() {
        let mapper = CoverageMapper::new();
        let map = CoverageMap::new();

        let report = mapper.report(&map);

        // Verify timestamp is valid RFC 3339 format
        assert!(report.generated_at.contains("T"));
        assert!(
            report.generated_at.ends_with("Z")
                || report.generated_at.contains("+")
                || report.generated_at.contains("-")
        );
    }
}
