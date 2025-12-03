//! Report data structures for UAT results.
//!
//! This module contains the core data structures used for aggregating
//! and organizing UAT test results for report generation.

use std::collections::HashMap;

use chrono::Utc;
use serde::Serialize;

use super::coverage::CoverageReport;
use super::gates::GateResult;
use super::perf::PerfResults;
use super::runner::{Priority, UatResults};

/// Aggregated data for report generation.
#[derive(Debug, Clone, Serialize)]
pub struct ReportData {
    /// UAT test results.
    pub uat_results: UatResults,
    /// Coverage report (optional).
    pub coverage: Option<CoverageReport>,
    /// Performance results (optional).
    pub performance: Option<PerfResults>,
    /// Quality gate result (optional).
    pub gate_result: Option<GateResult>,
    /// Report title.
    pub title: String,
    /// Timestamp when report was generated.
    pub generated_at: String,
}

impl ReportData {
    /// Create a new report data with just UAT results.
    pub fn new(uat_results: UatResults) -> Self {
        Self {
            uat_results,
            coverage: None,
            performance: None,
            gate_result: None,
            title: "UAT Report".to_string(),
            generated_at: Utc::now().to_rfc3339(),
        }
    }

    /// Set the report title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Add coverage report.
    pub fn with_coverage(mut self, coverage: CoverageReport) -> Self {
        self.coverage = Some(coverage);
        self
    }

    /// Add performance results.
    pub fn with_performance(mut self, performance: PerfResults) -> Self {
        self.performance = Some(performance);
        self
    }

    /// Add gate result.
    pub fn with_gate_result(mut self, gate_result: GateResult) -> Self {
        self.gate_result = Some(gate_result);
        self
    }

    /// Calculate pass rate as percentage.
    pub fn pass_rate(&self) -> f64 {
        if self.uat_results.total == 0 {
            return 100.0;
        }
        (self.uat_results.passed as f64 / self.uat_results.total as f64) * 100.0
    }

    /// Group test results by category.
    pub fn results_by_category(&self) -> HashMap<String, CategoryStats> {
        let mut by_category: HashMap<String, CategoryStats> = HashMap::new();

        for result in &self.uat_results.results {
            let entry = by_category
                .entry(result.test.category.clone())
                .or_insert_with(|| CategoryStats {
                    total: 0,
                    passed: 0,
                    failed: 0,
                });
            entry.total += 1;
            if result.passed {
                entry.passed += 1;
            } else {
                entry.failed += 1;
            }
        }

        by_category
    }

    /// Group test results by priority.
    pub fn results_by_priority(&self) -> HashMap<Priority, CategoryStats> {
        let mut by_priority: HashMap<Priority, CategoryStats> = HashMap::new();

        for result in &self.uat_results.results {
            let entry = by_priority
                .entry(result.test.priority)
                .or_insert_with(|| CategoryStats {
                    total: 0,
                    passed: 0,
                    failed: 0,
                });
            entry.total += 1;
            if result.passed {
                entry.passed += 1;
            } else {
                entry.failed += 1;
            }
        }

        by_priority
    }
}

/// Statistics for a category or priority group.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CategoryStats {
    /// Total tests in this group.
    pub total: usize,
    /// Passed tests.
    pub passed: usize,
    /// Failed tests.
    pub failed: usize,
}

impl CategoryStats {
    /// Calculate pass rate as percentage.
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.passed as f64 / self.total as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uat::runner::{UatResult, UatTest};

    fn create_test_result(
        name: &str,
        category: &str,
        priority: Priority,
        passed: bool,
    ) -> UatResult {
        UatResult {
            test: UatTest {
                name: name.to_string(),
                file: "test.rhai".to_string(),
                category: category.to_string(),
                priority,
                requirements: vec![],
                latency_threshold: None,
            },
            passed,
            duration_us: 100,
            error: if passed {
                None
            } else {
                Some("Test failed".to_string())
            },
        }
    }

    fn create_uat_results() -> UatResults {
        UatResults {
            total: 10,
            passed: 8,
            failed: 2,
            skipped: 0,
            duration_us: 5000,
            results: vec![
                create_test_result("test1", "core", Priority::P0, true),
                create_test_result("test2", "core", Priority::P1, true),
                create_test_result("test3", "layers", Priority::P1, false),
                create_test_result("test4", "layers", Priority::P2, true),
                create_test_result("test5", "combos", Priority::P0, true),
                create_test_result("test6", "combos", Priority::P1, true),
                create_test_result("test7", "combos", Priority::P2, false),
                create_test_result("test8", "core", Priority::P2, true),
                create_test_result("test9", "core", Priority::P2, true),
                create_test_result("test10", "layers", Priority::P2, true),
            ],
        }
    }

    #[test]
    fn report_data_new() {
        let results = create_uat_results();
        let data = ReportData::new(results);
        assert_eq!(data.title, "UAT Report");
        assert!(!data.generated_at.is_empty());
    }

    #[test]
    fn report_data_with_title() {
        let results = create_uat_results();
        let data = ReportData::new(results).with_title("Custom Report");
        assert_eq!(data.title, "Custom Report");
    }

    #[test]
    fn report_data_pass_rate() {
        let results = create_uat_results();
        let data = ReportData::new(results);
        assert!((data.pass_rate() - 80.0).abs() < f64::EPSILON);
    }

    #[test]
    fn report_data_pass_rate_empty() {
        let results = UatResults {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_us: 0,
            results: vec![],
        };
        let data = ReportData::new(results);
        assert_eq!(data.pass_rate(), 100.0);
    }

    #[test]
    fn report_data_results_by_category() {
        let results = create_uat_results();
        let data = ReportData::new(results);
        let by_category = data.results_by_category();

        assert_eq!(by_category.len(), 3);
        assert_eq!(by_category.get("core").unwrap().total, 4);
        assert_eq!(by_category.get("core").unwrap().passed, 4);
        assert_eq!(by_category.get("layers").unwrap().total, 3);
        assert_eq!(by_category.get("layers").unwrap().failed, 1);
        assert_eq!(by_category.get("combos").unwrap().total, 3);
    }

    #[test]
    fn report_data_results_by_priority() {
        let results = create_uat_results();
        let data = ReportData::new(results);
        let by_priority = data.results_by_priority();

        assert_eq!(by_priority.len(), 3);
        assert_eq!(by_priority.get(&Priority::P0).unwrap().total, 2);
        assert_eq!(by_priority.get(&Priority::P0).unwrap().passed, 2);
        assert_eq!(by_priority.get(&Priority::P1).unwrap().total, 3);
        assert_eq!(by_priority.get(&Priority::P1).unwrap().failed, 1);
    }

    #[test]
    fn category_stats_pass_rate() {
        let stats = CategoryStats {
            total: 10,
            passed: 7,
            failed: 3,
        };
        assert!((stats.pass_rate() - 70.0).abs() < f64::EPSILON);
    }

    #[test]
    fn category_stats_pass_rate_empty() {
        let stats = CategoryStats::default();
        assert_eq!(stats.pass_rate(), 100.0);
    }
}
