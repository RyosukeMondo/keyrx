//! HTML report generation for UAT results.
//!
//! This module contains functions for generating standalone HTML reports
//! with embedded CSS for viewing in web browsers.

use super::gates::GateResult;
use super::report_data::ReportData;
use super::report_html_sections::{html_coverage_section, html_performance_section};
use super::report_html_styles::REPORT_CSS;
use super::runner::Priority;

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
pub fn generate_html(data: &ReportData) -> String {
    let mut html = String::with_capacity(16384);

    // HTML header with embedded CSS
    html.push_str(&html_header(&data.title));

    // Main content
    html.push_str("<div class=\"container\">\n");

    // Title and timestamp
    html.push_str(&format!(
        "<header>\n<h1>{}</h1>\n<p class=\"timestamp\">Generated: {}</p>\n</header>\n",
        escape_html(&data.title),
        escape_html(&data.generated_at)
    ));

    // Summary section
    html.push_str(&html_summary_section(data));

    // Quality gate section (if available)
    if let Some(ref gate_result) = data.gate_result {
        html.push_str(&html_gate_section(gate_result));
    }

    // Test results by category
    html.push_str(&html_category_section(data));

    // Test results by priority
    html.push_str(&html_priority_section(data));

    // Failed tests section
    html.push_str(&html_failed_tests_section(data));

    // Coverage section (if available)
    if let Some(ref coverage) = data.coverage {
        html.push_str(&html_coverage_section(coverage));
    }

    // Performance section (if available)
    if let Some(ref perf) = data.performance {
        html.push_str(&html_performance_section(perf));
    }

    // All tests table
    html.push_str(&html_all_tests_section(data));

    html.push_str("</div>\n");

    // HTML footer
    html.push_str(&html_footer());

    html
}

/// Generate HTML header with embedded CSS.
pub fn html_header(title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{}</title>
<style>
{}
</style>
</head>
<body>
"#,
        escape_html(title),
        REPORT_CSS
    )
}

/// Generate HTML footer.
pub fn html_footer() -> String {
    r#"</body>
</html>
"#
    .to_string()
}

/// Generate summary section.
pub fn html_summary_section(data: &ReportData) -> String {
    let pass_rate = data.pass_rate();
    let duration_ms = data.uat_results.duration_us / 1000;

    format!(
        r#"<section class="card">
<h2>Summary</h2>
<div class="summary-grid">
<div class="stat total">
<div class="stat-value">{}</div>
<div class="stat-label">Total Tests</div>
</div>
<div class="stat pass">
<div class="stat-value">{}</div>
<div class="stat-label">Passed</div>
</div>
<div class="stat fail">
<div class="stat-value">{}</div>
<div class="stat-label">Failed</div>
</div>
<div class="stat skip">
<div class="stat-value">{}</div>
<div class="stat-label">Skipped</div>
</div>
<div class="stat">
<div class="stat-value">{:.1}%</div>
<div class="stat-label">Pass Rate</div>
</div>
<div class="stat">
<div class="stat-value">{}ms</div>
<div class="stat-label">Duration</div>
</div>
</div>
<div style="margin-top: 1rem;">
<div class="progress-bar">
<div class="progress-fill" style="width: {:.1}%;"></div>
</div>
</div>
</section>
"#,
        data.uat_results.total,
        data.uat_results.passed,
        data.uat_results.failed,
        data.uat_results.skipped,
        pass_rate,
        duration_ms,
        pass_rate
    )
}

/// Generate quality gate section.
pub fn html_gate_section(gate_result: &GateResult) -> String {
    let status_class = if gate_result.passed {
        "gate-pass"
    } else {
        "gate-fail"
    };
    let status_text = if gate_result.passed {
        "PASSED"
    } else {
        "FAILED"
    };

    let mut html = format!(
        r#"<section class="card">
<h2>Quality Gate</h2>
<p class="{}"><strong>Status: {}</strong></p>
"#,
        status_class, status_text
    );

    if !gate_result.violations.is_empty() {
        html.push_str("<h3>Violations</h3>\n");
        for violation in &gate_result.violations {
            html.push_str(&format!(
                "<div class=\"violation\"><strong>{}:</strong> Expected {}, got {}</div>\n",
                escape_html(&violation.criterion),
                escape_html(&violation.expected),
                escape_html(&violation.actual)
            ));
        }
    }

    html.push_str("</section>\n");
    html
}

/// Generate category breakdown section.
pub fn html_category_section(data: &ReportData) -> String {
    let by_category = data.results_by_category();

    if by_category.is_empty() {
        return String::new();
    }

    let mut html = String::from(
        r#"<section class="card">
<h2>Results by Category</h2>
<table>
<thead><tr><th>Category</th><th>Total</th><th>Passed</th><th>Failed</th><th>Pass Rate</th></tr></thead>
<tbody>
"#,
    );

    let mut categories: Vec<_> = by_category.iter().collect();
    categories.sort_by_key(|(name, _)| name.as_str());

    for (category, stats) in categories {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.1}%</td></tr>\n",
            escape_html(category),
            stats.total,
            stats.passed,
            stats.failed,
            stats.pass_rate()
        ));
    }

    html.push_str("</tbody></table>\n</section>\n");
    html
}

/// Generate priority breakdown section.
pub fn html_priority_section(data: &ReportData) -> String {
    let by_priority = data.results_by_priority();

    if by_priority.is_empty() {
        return String::new();
    }

    let mut html = String::from(
        r#"<section class="card">
<h2>Results by Priority</h2>
<table>
<thead><tr><th>Priority</th><th>Total</th><th>Passed</th><th>Failed</th><th>Pass Rate</th></tr></thead>
<tbody>
"#,
    );

    // Sort by priority order: P0, P1, P2
    let mut priorities: Vec<_> = by_priority.iter().collect();
    priorities.sort_by_key(|(priority, _)| match priority {
        Priority::P0 => 0,
        Priority::P1 => 1,
        Priority::P2 => 2,
    });

    for (priority, stats) in priorities {
        let priority_str = match priority {
            Priority::P0 => "P0 (Critical)",
            Priority::P1 => "P1 (High)",
            Priority::P2 => "P2 (Normal)",
        };
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.1}%</td></tr>\n",
            priority_str,
            stats.total,
            stats.passed,
            stats.failed,
            stats.pass_rate()
        ));
    }

    html.push_str("</tbody></table>\n</section>\n");
    html
}

/// Generate failed tests section.
pub fn html_failed_tests_section(data: &ReportData) -> String {
    let failed: Vec<_> = data
        .uat_results
        .results
        .iter()
        .filter(|r| !r.passed)
        .collect();

    if failed.is_empty() {
        return String::new();
    }

    let mut html = String::from(
        r#"<section class="card">
<h2>Failed Tests</h2>
<table>
<thead><tr><th>Test</th><th>Category</th><th>Priority</th><th>Error</th></tr></thead>
<tbody>
"#,
    );

    for result in failed {
        let priority_badge = match result.test.priority {
            Priority::P0 => "<span class=\"badge badge-p0\">P0</span>",
            Priority::P1 => "<span class=\"badge badge-p1\">P1</span>",
            Priority::P2 => "<span class=\"badge badge-p2\">P2</span>",
        };
        let error = result.error.as_deref().unwrap_or("Unknown error");

        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td class=\"error-msg\">{}</td></tr>\n",
            escape_html(&result.test.name),
            escape_html(&result.test.category),
            priority_badge,
            escape_html(error)
        ));
    }

    html.push_str("</tbody></table>\n</section>\n");
    html
}

/// Generate all tests section.
pub fn html_all_tests_section(data: &ReportData) -> String {
    if data.uat_results.results.is_empty() {
        return String::new();
    }

    let mut html = String::from(
        r#"<section class="card">
<h2>All Tests</h2>
<table>
<thead><tr><th>Test</th><th>Category</th><th>Priority</th><th>Status</th><th>Duration</th></tr></thead>
<tbody>
"#,
    );

    for result in &data.uat_results.results {
        let priority_badge = match result.test.priority {
            Priority::P0 => "<span class=\"badge badge-p0\">P0</span>",
            Priority::P1 => "<span class=\"badge badge-p1\">P1</span>",
            Priority::P2 => "<span class=\"badge badge-p2\">P2</span>",
        };
        let status_badge = if result.passed {
            "<span class=\"badge badge-pass\">PASS</span>"
        } else {
            "<span class=\"badge badge-fail\">FAIL</span>"
        };

        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}us</td></tr>\n",
            escape_html(&result.test.name),
            escape_html(&result.test.category),
            priority_badge,
            status_badge,
            result.duration_us
        ));
    }

    html.push_str("</tbody></table>\n</section>\n");
    html
}

/// Escape HTML special characters.
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uat::coverage::{CoverageMap, CoverageReport, CoverageStatus, RequirementCoverage};
    use crate::uat::gates::GateViolation;
    use crate::uat::perf::{LatencyViolation, PerfResults, PerformanceResult};
    use crate::uat::runner::{UatResult, UatResults, UatTest};

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

        let html = generate_html(&data);

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

        let html = generate_html(&data);

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

        let html = generate_html(&data);

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

        let html = generate_html(&data);

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

        let html = generate_html(&data);

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

        let html = generate_html(&data);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Summary"));
        assert!(html.contains("0")); // Total tests
    }

    #[test]
    fn escape_html_works() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_html("it's"), "it&#39;s");
        assert_eq!(escape_html("normal text"), "normal text");
    }
}
