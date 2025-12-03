//! Report generation for UAT results.
//!
//! Generates comprehensive reports in HTML, Markdown, and JSON formats.

use super::report_html;
use super::report_markdown;

// Re-export from report_data module
pub use super::report_data::{CategoryStats, ReportData};

/// Report generator for multiple output formats.
#[derive(Debug, Default)]
pub struct ReportGenerator;

impl ReportGenerator {
    /// Create a new report generator.
    pub fn new() -> Self {
        Self
    }

    /// Generate an HTML report.
    ///
    /// Creates a comprehensive HTML report with:
    /// - Summary section with pass/fail counts
    /// - Test results grouped by category
    /// - Coverage matrix (if available)
    /// - Performance metrics (if available)
    /// - Quality gate status (if available)
    /// - Trend comparison (if available)
    ///
    /// The HTML includes embedded CSS for standalone viewing.
    pub fn generate_html(&self, data: &ReportData) -> String {
        report_html::generate_html(data)
    }

    /// Generate a Markdown report for PR comments.
    ///
    /// Creates a GitHub-flavored markdown report suitable for PR comments with:
    /// - Summary section with pass/fail counts
    /// - Test results grouped by category
    /// - Coverage summary (if available)
    /// - Performance summary (if available)
    /// - Quality gate status (if available)
    /// - Failed tests list
    pub fn generate_markdown(&self, data: &ReportData) -> String {
        report_markdown::generate_markdown(data)
    }

    /// Generate a JSON report for machine parsing.
    ///
    /// Creates a machine-readable JSON output containing all report data
    /// for programmatic consumption by CI/CD systems and other tools.
    ///
    /// # Returns
    /// A JSON string containing all report data.
    pub fn generate_json(&self, data: &ReportData) -> String {
        serde_json::to_string_pretty(data)
            .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize report: {}\"}}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uat::coverage::{CoverageMap, CoverageReport, CoverageStatus, RequirementCoverage};
    use crate::uat::gates::{GateResult, GateViolation};
    use crate::uat::perf::{LatencyViolation, PerfResults, PerformanceResult};
    use crate::uat::runner::{Priority, UatResult, UatResults, UatTest};

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
    fn generate_html_basic() {
        let results = create_uat_results();
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let html = generator.generate_html(&data);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html"));
        assert!(html.contains("</html>"));
        assert!(html.contains("UAT Report"));
        assert!(html.contains("Summary"));
        assert!(html.contains("10")); // total tests
        assert!(html.contains("8")); // passed
        assert!(html.contains("2")); // failed
    }

    #[test]
    fn generate_html_with_gate_result() {
        let results = create_uat_results();
        let gate_result = GateResult {
            passed: false,
            violations: vec![GateViolation::new("pass_rate", ">=95.0%", "80.0%")],
        };
        let data = ReportData::new(results).with_gate_result(gate_result);
        let generator = ReportGenerator::new();

        let html = generator.generate_html(&data);

        assert!(html.contains("Quality Gate"));
        assert!(html.contains("FAILED"));
        assert!(html.contains("pass_rate"));
        assert!(html.contains("95.0%"));
    }

    #[test]
    fn generate_html_with_coverage() {
        let results = create_uat_results();
        let mut coverage_map = CoverageMap::new();
        coverage_map.requirements.insert(
            "1.1".to_string(),
            RequirementCoverage {
                id: "1.1".to_string(),
                linked_tests: vec!["test1".to_string()],
                status: CoverageStatus::Verified,
                last_verified: Some("2025-01-01".to_string()),
            },
        );
        coverage_map.requirements.insert(
            "1.2".to_string(),
            RequirementCoverage {
                id: "1.2".to_string(),
                linked_tests: vec!["test3".to_string()],
                status: CoverageStatus::AtRisk,
                last_verified: None,
            },
        );
        let coverage = CoverageReport {
            coverage: coverage_map,
            total: 2,
            verified: 1,
            at_risk: 1,
            uncovered: 0,
            coverage_percentage: 0.5,
            generated_at: "2025-01-01".to_string(),
        };
        let data = ReportData::new(results).with_coverage(coverage);
        let generator = ReportGenerator::new();

        let html = generator.generate_html(&data);

        assert!(html.contains("Requirements Coverage"));
        assert!(html.contains("1.1"));
        assert!(html.contains("Verified"));
        assert!(html.contains("At Risk"));
    }

    #[test]
    fn generate_html_with_performance() {
        let results = create_uat_results();
        let perf = PerfResults {
            total: 2,
            passed: 1,
            failed: 1,
            aggregate_p50_us: 100,
            aggregate_p95_us: 200,
            aggregate_p99_us: 300,
            aggregate_max_us: 500,
            total_duration_us: 10000,
            results: vec![
                PerformanceResult {
                    test_name: "perf_test1".to_string(),
                    test_file: "test.rhai".to_string(),
                    p50_us: 100,
                    p95_us: 200,
                    p99_us: 300,
                    max_us: 400,
                    min_us: 50,
                    iterations: 100,
                    threshold_us: Some(1000),
                    threshold_exceeded: false,
                    violations: vec![],
                },
                PerformanceResult {
                    test_name: "perf_test2".to_string(),
                    test_file: "test.rhai".to_string(),
                    p50_us: 500,
                    p95_us: 800,
                    p99_us: 1100,
                    max_us: 1500,
                    min_us: 200,
                    iterations: 100,
                    threshold_us: Some(1000),
                    threshold_exceeded: true,
                    violations: vec![LatencyViolation {
                        test_name: "perf_test2".to_string(),
                        threshold_us: 1000,
                        actual_us: 1500,
                        iteration: 50,
                    }],
                },
            ],
            all_violations: vec![LatencyViolation {
                test_name: "perf_test2".to_string(),
                threshold_us: 1000,
                actual_us: 1500,
                iteration: 50,
            }],
        };
        let data = ReportData::new(results).with_performance(perf);
        let generator = ReportGenerator::new();

        let html = generator.generate_html(&data);

        assert!(html.contains("Performance Metrics"));
        assert!(html.contains("100us")); // P50
        assert!(html.contains("500us")); // Max
        assert!(html.contains("Latency Violations"));
        assert!(html.contains("perf_test2"));
    }

    #[test]
    fn generate_html_escapes_special_chars() {
        let mut results = create_uat_results();
        results.results.push(UatResult {
            test: UatTest {
                name: "test_with_<html>".to_string(),
                file: "test.rhai".to_string(),
                category: "special & chars".to_string(),
                priority: Priority::P1,
                requirements: vec![],
                latency_threshold: None,
            },
            passed: false,
            duration_us: 100,
            error: Some("<script>alert('xss')</script>".to_string()),
        });
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let html = generator.generate_html(&data);

        assert!(html.contains("&lt;html&gt;"));
        assert!(html.contains("special &amp; chars"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>alert"));
    }

    #[test]
    fn generate_html_empty_results() {
        let results = UatResults {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_us: 0,
            results: vec![],
        };
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let html = generator.generate_html(&data);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Summary"));
        assert!(html.contains("0")); // Total tests
    }

    // Markdown report tests

    #[test]
    fn generate_markdown_basic() {
        let results = create_uat_results();
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let md = generator.generate_markdown(&data);

        assert!(md.contains("# UAT Report"));
        assert!(md.contains("## \u{274c} Summary")); // Has failures
        assert!(md.contains("| **Total Tests** | 10 |"));
        assert!(md.contains("| **Passed** | 8 |"));
        assert!(md.contains("| **Failed** | 2 |"));
        assert!(md.contains("| **Pass Rate** | 80.0% |"));
    }

    #[test]
    fn generate_markdown_all_passed() {
        let results = UatResults {
            total: 5,
            passed: 5,
            failed: 0,
            skipped: 0,
            duration_us: 1000,
            results: vec![
                create_test_result("test1", "core", Priority::P0, true),
                create_test_result("test2", "core", Priority::P1, true),
                create_test_result("test3", "core", Priority::P2, true),
                create_test_result("test4", "layers", Priority::P1, true),
                create_test_result("test5", "layers", Priority::P2, true),
            ],
        };
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let md = generator.generate_markdown(&data);

        assert!(md.contains("## \u{2705} Summary")); // All passed
        assert!(!md.contains("## \u{274c} Failed Tests")); // No failures section
    }

    #[test]
    fn generate_markdown_with_gate_result() {
        let results = create_uat_results();
        let gate_result = GateResult {
            passed: false,
            violations: vec![GateViolation::new("pass_rate", ">=95.0%", "80.0%")],
        };
        let data = ReportData::new(results).with_gate_result(gate_result);
        let generator = ReportGenerator::new();

        let md = generator.generate_markdown(&data);

        assert!(md.contains("## \u{274c} Quality Gate: FAILED"));
        assert!(md.contains("### Violations"));
        assert!(md.contains("**pass_rate**: Expected >=95.0%, got 80.0%"));
    }

    #[test]
    fn generate_markdown_with_gate_passed() {
        let results = create_uat_results();
        let gate_result = GateResult {
            passed: true,
            violations: vec![],
        };
        let data = ReportData::new(results).with_gate_result(gate_result);
        let generator = ReportGenerator::new();

        let md = generator.generate_markdown(&data);

        assert!(md.contains("## \u{2705} Quality Gate: PASSED"));
        assert!(!md.contains("### Violations"));
    }

    #[test]
    fn generate_markdown_failed_tests() {
        let results = create_uat_results();
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let md = generator.generate_markdown(&data);

        assert!(md.contains("## \u{274c} Failed Tests"));
        assert!(md.contains("| Test | Category | Priority | Error |"));
        assert!(md.contains("test3"));
        assert!(md.contains("test7"));
    }

    #[test]
    fn generate_markdown_category_breakdown() {
        let results = create_uat_results();
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let md = generator.generate_markdown(&data);

        assert!(md.contains("## Results by Category"));
        assert!(md.contains("| Category | Total | Passed | Failed | Pass Rate |"));
        assert!(md.contains("| core |"));
        assert!(md.contains("| layers |"));
        assert!(md.contains("| combos |"));
    }

    #[test]
    fn generate_markdown_priority_breakdown() {
        let results = create_uat_results();
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let md = generator.generate_markdown(&data);

        assert!(md.contains("## Results by Priority"));
        assert!(md.contains("\u{1f534} P0 (Critical)"));
        assert!(md.contains("\u{1f7e0} P1 (High)"));
        assert!(md.contains("\u{1f535} P2 (Normal)"));
    }

    #[test]
    fn generate_markdown_with_coverage() {
        let results = create_uat_results();
        let mut coverage_map = CoverageMap::new();
        coverage_map.requirements.insert(
            "1.1".to_string(),
            RequirementCoverage {
                id: "1.1".to_string(),
                linked_tests: vec!["test1".to_string()],
                status: CoverageStatus::Verified,
                last_verified: Some("2025-01-01".to_string()),
            },
        );
        coverage_map.requirements.insert(
            "1.2".to_string(),
            RequirementCoverage {
                id: "1.2".to_string(),
                linked_tests: vec!["test3".to_string()],
                status: CoverageStatus::AtRisk,
                last_verified: None,
            },
        );
        coverage_map.requirements.insert(
            "1.3".to_string(),
            RequirementCoverage {
                id: "1.3".to_string(),
                linked_tests: vec![],
                status: CoverageStatus::Uncovered,
                last_verified: None,
            },
        );
        let coverage = CoverageReport {
            coverage: coverage_map,
            total: 3,
            verified: 1,
            at_risk: 1,
            uncovered: 1,
            coverage_percentage: 0.333,
            generated_at: "2025-01-01".to_string(),
        };
        let data = ReportData::new(results).with_coverage(coverage);
        let generator = ReportGenerator::new();

        let md = generator.generate_markdown(&data);

        assert!(md.contains("## Requirements Coverage"));
        assert!(md.contains("| **Total Requirements** | 3 |"));
        assert!(md.contains("| **Verified** | 1 |"));
        assert!(md.contains("| **At Risk** | 1 |"));
        assert!(md.contains("| **Uncovered** | 1 |"));
        assert!(md.contains("### \u{26a0}\u{fe0f} At Risk Requirements"));
        assert!(md.contains("**1.2**"));
        assert!(md.contains("### \u{1f6ab} Uncovered Requirements"));
        assert!(md.contains("**1.3**"));
    }

    #[test]
    fn generate_markdown_with_performance() {
        let results = create_uat_results();
        let perf = PerfResults {
            total: 2,
            passed: 1,
            failed: 1,
            aggregate_p50_us: 100,
            aggregate_p95_us: 200,
            aggregate_p99_us: 300,
            aggregate_max_us: 500,
            total_duration_us: 10000,
            results: vec![],
            all_violations: vec![LatencyViolation {
                test_name: "perf_test2".to_string(),
                threshold_us: 1000,
                actual_us: 1500,
                iteration: 50,
            }],
        };
        let data = ReportData::new(results).with_performance(perf);
        let generator = ReportGenerator::new();

        let md = generator.generate_markdown(&data);

        assert!(md.contains("## Performance Metrics"));
        assert!(md.contains("| **P50** | 100\u{b5}s |"));
        assert!(md.contains("| **P95** | 200\u{b5}s |"));
        assert!(md.contains("| **P99** | 300\u{b5}s |"));
        assert!(md.contains("| **Max** | 500\u{b5}s |"));
        assert!(md.contains("### \u{26a0}\u{fe0f} Latency Violations"));
        assert!(md.contains("**perf_test2**: Expected \u{2264}1000\u{b5}s, got 1500\u{b5}s"));
    }

    #[test]
    fn generate_markdown_empty_results() {
        let results = UatResults {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_us: 0,
            results: vec![],
        };
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let md = generator.generate_markdown(&data);

        assert!(md.contains("# UAT Report"));
        assert!(md.contains("## \u{2705} Summary")); // No failures = pass emoji
        assert!(md.contains("| **Total Tests** | 0 |"));
        assert!(md.contains("| **Pass Rate** | 100.0% |"));
    }

    #[test]
    fn generate_markdown_truncates_long_errors() {
        let mut results = create_uat_results();
        results.results.push(UatResult {
            test: UatTest {
                name: "test_long_error".to_string(),
                file: "test.rhai".to_string(),
                category: "core".to_string(),
                priority: Priority::P0,
                requirements: vec![],
                latency_threshold: None,
            },
            passed: false,
            duration_us: 100,
            error: Some("This is a very long error message that should be truncated because it exceeds the maximum length for table display".to_string()),
        });
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let md = generator.generate_markdown(&data);

        // Truncates at 47 chars + "..." = 50 chars total
        assert!(md.contains("This is a very long error message that should b..."));
        assert!(!md.contains("exceeds the maximum length"));
    }

    // JSON report tests

    #[test]
    fn generate_json_basic() {
        let results = create_uat_results();
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let json = generator.generate_json(&data);

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        // Check structure
        assert!(parsed.get("uat_results").is_some());
        assert!(parsed.get("title").is_some());
        assert!(parsed.get("generated_at").is_some());

        // Check values
        assert_eq!(parsed["uat_results"]["total"], 10);
        assert_eq!(parsed["uat_results"]["passed"], 8);
        assert_eq!(parsed["uat_results"]["failed"], 2);
        assert_eq!(parsed["title"], "UAT Report");
    }

    #[test]
    fn generate_json_with_gate_result() {
        let results = create_uat_results();
        let gate_result = GateResult {
            passed: false,
            violations: vec![GateViolation::new("pass_rate", ">=95.0%", "80.0%")],
        };
        let data = ReportData::new(results).with_gate_result(gate_result);
        let generator = ReportGenerator::new();

        let json = generator.generate_json(&data);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        assert!(parsed.get("gate_result").is_some());
        assert_eq!(parsed["gate_result"]["passed"], false);
        assert!(parsed["gate_result"]["violations"].is_array());
        assert_eq!(
            parsed["gate_result"]["violations"][0]["criterion"],
            "pass_rate"
        );
    }

    #[test]
    fn generate_json_with_coverage() {
        let results = create_uat_results();
        let mut coverage_map = CoverageMap::new();
        coverage_map.requirements.insert(
            "1.1".to_string(),
            RequirementCoverage {
                id: "1.1".to_string(),
                linked_tests: vec!["test1".to_string()],
                status: CoverageStatus::Verified,
                last_verified: Some("2025-01-01".to_string()),
            },
        );
        let coverage = CoverageReport {
            coverage: coverage_map,
            total: 1,
            verified: 1,
            at_risk: 0,
            uncovered: 0,
            coverage_percentage: 1.0,
            generated_at: "2025-01-01".to_string(),
        };
        let data = ReportData::new(results).with_coverage(coverage);
        let generator = ReportGenerator::new();

        let json = generator.generate_json(&data);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        assert!(parsed.get("coverage").is_some());
        assert_eq!(parsed["coverage"]["total"], 1);
        assert_eq!(parsed["coverage"]["verified"], 1);
        assert_eq!(parsed["coverage"]["coverage_percentage"], 1.0);
    }

    #[test]
    fn generate_json_with_performance() {
        let results = create_uat_results();
        let perf = PerfResults {
            total: 2,
            passed: 1,
            failed: 1,
            aggregate_p50_us: 100,
            aggregate_p95_us: 200,
            aggregate_p99_us: 300,
            aggregate_max_us: 500,
            total_duration_us: 10000,
            results: vec![],
            all_violations: vec![LatencyViolation {
                test_name: "perf_test".to_string(),
                threshold_us: 1000,
                actual_us: 1500,
                iteration: 50,
            }],
        };
        let data = ReportData::new(results).with_performance(perf);
        let generator = ReportGenerator::new();

        let json = generator.generate_json(&data);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        assert!(parsed.get("performance").is_some());
        assert_eq!(parsed["performance"]["aggregate_p50_us"], 100);
        assert_eq!(parsed["performance"]["aggregate_p95_us"], 200);
        assert_eq!(parsed["performance"]["aggregate_p99_us"], 300);
        assert_eq!(parsed["performance"]["aggregate_max_us"], 500);
        assert!(parsed["performance"]["all_violations"].is_array());
    }

    #[test]
    fn generate_json_empty_results() {
        let results = UatResults {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_us: 0,
            results: vec![],
        };
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let json = generator.generate_json(&data);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        assert_eq!(parsed["uat_results"]["total"], 0);
        assert_eq!(parsed["uat_results"]["passed"], 0);
        assert_eq!(
            parsed["uat_results"]["results"].as_array().unwrap().len(),
            0
        );
    }

    #[test]
    fn generate_json_includes_test_details() {
        let results = create_uat_results();
        let data = ReportData::new(results);
        let generator = ReportGenerator::new();

        let json = generator.generate_json(&data);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        let test_results = parsed["uat_results"]["results"].as_array().unwrap();
        assert_eq!(test_results.len(), 10);

        // Check first test structure
        let first_test = &test_results[0];
        assert!(first_test.get("test").is_some());
        assert!(first_test.get("passed").is_some());
        assert!(first_test.get("duration_us").is_some());

        // Check test metadata
        assert!(first_test["test"].get("name").is_some());
        assert!(first_test["test"].get("category").is_some());
        assert!(first_test["test"].get("priority").is_some());
    }
}
