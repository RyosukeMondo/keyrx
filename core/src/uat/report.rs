//! Report generation for UAT results.
//!
//! Generates comprehensive reports in HTML, Markdown, and JSON formats.

use std::collections::HashMap;

use chrono::Utc;

use super::coverage::{CoverageReport, CoverageStatus};
use super::gates::GateResult;
use super::perf::PerfResults;
use super::runner::{Priority, UatResults};

/// Aggregated data for report generation.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Default)]
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
        let mut html = String::with_capacity(16384);

        // HTML header with embedded CSS
        html.push_str(&self.html_header(&data.title));

        // Main content
        html.push_str("<div class=\"container\">\n");

        // Title and timestamp
        html.push_str(&format!(
            "<header>\n<h1>{}</h1>\n<p class=\"timestamp\">Generated: {}</p>\n</header>\n",
            escape_html(&data.title),
            escape_html(&data.generated_at)
        ));

        // Summary section
        html.push_str(&self.generate_summary_section(data));

        // Quality gate section (if available)
        if let Some(ref gate_result) = data.gate_result {
            html.push_str(&self.generate_gate_section(gate_result));
        }

        // Test results by category
        html.push_str(&self.generate_category_section(data));

        // Test results by priority
        html.push_str(&self.generate_priority_section(data));

        // Failed tests section
        html.push_str(&self.generate_failed_tests_section(data));

        // Coverage section (if available)
        if let Some(ref coverage) = data.coverage {
            html.push_str(&self.generate_coverage_section(coverage));
        }

        // Performance section (if available)
        if let Some(ref perf) = data.performance {
            html.push_str(&self.generate_performance_section(perf));
        }

        // All tests table
        html.push_str(&self.generate_all_tests_section(data));

        html.push_str("</div>\n");

        // HTML footer
        html.push_str(&self.html_footer());

        html
    }

    /// Generate HTML header with embedded CSS.
    fn html_header(&self, title: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{}</title>
<style>
:root {{
  --color-pass: #22c55e;
  --color-fail: #ef4444;
  --color-skip: #f59e0b;
  --color-bg: #f8fafc;
  --color-card: #ffffff;
  --color-border: #e2e8f0;
  --color-text: #1e293b;
  --color-text-muted: #64748b;
}}
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: var(--color-bg);
  color: var(--color-text);
  line-height: 1.6;
  padding: 2rem;
}}
.container {{ max-width: 1200px; margin: 0 auto; }}
header {{ margin-bottom: 2rem; }}
h1 {{ font-size: 2rem; font-weight: 700; }}
h2 {{ font-size: 1.5rem; font-weight: 600; margin: 1.5rem 0 1rem; }}
h3 {{ font-size: 1.25rem; font-weight: 600; margin: 1rem 0 0.5rem; }}
.timestamp {{ color: var(--color-text-muted); font-size: 0.875rem; }}
.card {{
  background: var(--color-card);
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
  padding: 1.5rem;
  margin-bottom: 1.5rem;
}}
.summary-grid {{
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 1rem;
}}
.stat {{
  text-align: center;
  padding: 1rem;
  border-radius: 0.5rem;
  background: var(--color-bg);
}}
.stat-value {{ font-size: 2rem; font-weight: 700; }}
.stat-label {{ font-size: 0.875rem; color: var(--color-text-muted); }}
.stat.pass {{ border-left: 4px solid var(--color-pass); }}
.stat.fail {{ border-left: 4px solid var(--color-fail); }}
.stat.skip {{ border-left: 4px solid var(--color-skip); }}
.stat.total {{ border-left: 4px solid #3b82f6; }}
.badge {{
  display: inline-block;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  font-weight: 600;
}}
.badge-pass {{ background: #dcfce7; color: #166534; }}
.badge-fail {{ background: #fee2e2; color: #991b1b; }}
.badge-skip {{ background: #fef3c7; color: #92400e; }}
.badge-p0 {{ background: #fee2e2; color: #991b1b; }}
.badge-p1 {{ background: #fef3c7; color: #92400e; }}
.badge-p2 {{ background: #e0e7ff; color: #3730a3; }}
table {{
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}}
th, td {{
  padding: 0.75rem;
  text-align: left;
  border-bottom: 1px solid var(--color-border);
}}
th {{ font-weight: 600; background: var(--color-bg); }}
tr:hover {{ background: var(--color-bg); }}
.progress-bar {{
  height: 8px;
  background: var(--color-border);
  border-radius: 4px;
  overflow: hidden;
}}
.progress-fill {{
  height: 100%;
  background: var(--color-pass);
  transition: width 0.3s;
}}
.gate-pass {{ color: var(--color-pass); }}
.gate-fail {{ color: var(--color-fail); }}
.violation {{ background: #fee2e2; padding: 0.5rem; border-radius: 0.25rem; margin: 0.25rem 0; }}
.coverage-verified {{ color: var(--color-pass); }}
.coverage-atrisk {{ color: var(--color-fail); }}
.coverage-uncovered {{ color: var(--color-text-muted); }}
.perf-metric {{ display: inline-block; margin-right: 1.5rem; }}
.perf-value {{ font-size: 1.5rem; font-weight: 600; }}
.perf-label {{ font-size: 0.75rem; color: var(--color-text-muted); }}
.error-msg {{ color: var(--color-fail); font-family: monospace; font-size: 0.8rem; }}
</style>
</head>
<body>
"#,
            escape_html(title)
        )
    }

    /// Generate HTML footer.
    fn html_footer(&self) -> String {
        r#"</body>
</html>
"#
        .to_string()
    }

    /// Generate summary section.
    fn generate_summary_section(&self, data: &ReportData) -> String {
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
    fn generate_gate_section(&self, gate_result: &GateResult) -> String {
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
    fn generate_category_section(&self, data: &ReportData) -> String {
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
    fn generate_priority_section(&self, data: &ReportData) -> String {
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
    fn generate_failed_tests_section(&self, data: &ReportData) -> String {
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

    /// Generate coverage section.
    fn generate_coverage_section(&self, coverage: &CoverageReport) -> String {
        let mut html = format!(
            r#"<section class="card">
<h2>Requirements Coverage</h2>
<div class="summary-grid">
<div class="stat total">
<div class="stat-value">{}</div>
<div class="stat-label">Total Requirements</div>
</div>
<div class="stat pass">
<div class="stat-value">{}</div>
<div class="stat-label">Verified</div>
</div>
<div class="stat fail">
<div class="stat-value">{}</div>
<div class="stat-label">At Risk</div>
</div>
<div class="stat skip">
<div class="stat-value">{}</div>
<div class="stat-label">Uncovered</div>
</div>
<div class="stat">
<div class="stat-value">{:.1}%</div>
<div class="stat-label">Coverage</div>
</div>
</div>
"#,
            coverage.total,
            coverage.verified,
            coverage.at_risk,
            coverage.uncovered,
            coverage.coverage_percentage * 100.0
        );

        // Coverage matrix table
        if !coverage.coverage.requirements.is_empty() {
            html.push_str(
                r#"<h3>Coverage Matrix</h3>
<table>
<thead><tr><th>Requirement</th><th>Status</th><th>Linked Tests</th></tr></thead>
<tbody>
"#,
            );

            let mut requirements: Vec<_> = coverage.coverage.requirements.iter().collect();
            requirements.sort_by_key(|(id, _)| id.as_str());

            for (id, req) in requirements {
                let (status_class, status_text) = match req.status {
                    CoverageStatus::Verified => ("coverage-verified", "Verified"),
                    CoverageStatus::AtRisk => ("coverage-atrisk", "At Risk"),
                    CoverageStatus::Uncovered => ("coverage-uncovered", "Uncovered"),
                };
                let tests = if req.linked_tests.is_empty() {
                    "-".to_string()
                } else {
                    req.linked_tests.join(", ")
                };

                html.push_str(&format!(
                    "<tr><td>{}</td><td class=\"{}\">{}</td><td>{}</td></tr>\n",
                    escape_html(id),
                    status_class,
                    status_text,
                    escape_html(&tests)
                ));
            }

            html.push_str("</tbody></table>\n");
        }

        html.push_str("</section>\n");
        html
    }

    /// Generate performance section.
    fn generate_performance_section(&self, perf: &PerfResults) -> String {
        let mut html = format!(
            r#"<section class="card">
<h2>Performance Metrics</h2>
<div style="margin-bottom: 1rem;">
<span class="perf-metric"><span class="perf-value">{}µs</span><br><span class="perf-label">P50 Latency</span></span>
<span class="perf-metric"><span class="perf-value">{}µs</span><br><span class="perf-label">P95 Latency</span></span>
<span class="perf-metric"><span class="perf-value">{}µs</span><br><span class="perf-label">P99 Latency</span></span>
<span class="perf-metric"><span class="perf-value">{}µs</span><br><span class="perf-label">Max Latency</span></span>
</div>
<div class="summary-grid">
<div class="stat total">
<div class="stat-value">{}</div>
<div class="stat-label">Total Perf Tests</div>
</div>
<div class="stat pass">
<div class="stat-value">{}</div>
<div class="stat-label">Passed</div>
</div>
<div class="stat fail">
<div class="stat-value">{}</div>
<div class="stat-label">Failed</div>
</div>
</div>
"#,
            perf.aggregate_p50_us,
            perf.aggregate_p95_us,
            perf.aggregate_p99_us,
            perf.aggregate_max_us,
            perf.total,
            perf.passed,
            perf.failed
        );

        // Violations
        if !perf.all_violations.is_empty() {
            html.push_str("<h3>Latency Violations</h3>\n");
            for violation in &perf.all_violations {
                html.push_str(&format!(
                    "<div class=\"violation\"><strong>{}:</strong> Expected ≤{}µs, got {}µs (iteration {})</div>\n",
                    escape_html(&violation.test_name),
                    violation.threshold_us,
                    violation.actual_us,
                    violation.iteration
                ));
            }
        }

        // Per-test results
        if !perf.results.is_empty() {
            html.push_str(
                r#"<h3>Per-Test Results</h3>
<table>
<thead><tr><th>Test</th><th>P50</th><th>P95</th><th>P99</th><th>Max</th><th>Threshold</th><th>Status</th></tr></thead>
<tbody>
"#,
            );

            for result in &perf.results {
                let status = if result.threshold_exceeded {
                    "<span class=\"badge badge-fail\">FAIL</span>"
                } else {
                    "<span class=\"badge badge-pass\">PASS</span>"
                };
                let threshold_str = result
                    .threshold_us
                    .map(|t| format!("{}µs", t))
                    .unwrap_or_else(|| "-".to_string());

                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}µs</td><td>{}µs</td><td>{}µs</td><td>{}µs</td><td>{}</td><td>{}</td></tr>\n",
                    escape_html(&result.test_name),
                    result.p50_us,
                    result.p95_us,
                    result.p99_us,
                    result.max_us,
                    threshold_str,
                    status
                ));
            }

            html.push_str("</tbody></table>\n");
        }

        html.push_str("</section>\n");
        html
    }

    /// Generate all tests section.
    fn generate_all_tests_section(&self, data: &ReportData) -> String {
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
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}µs</td></tr>\n",
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
}

/// Escape HTML special characters.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uat::coverage::{CoverageMap, RequirementCoverage};
    use crate::uat::gates::GateViolation;
    use crate::uat::perf::{LatencyViolation, PerformanceResult};
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
            violations: vec![GateViolation::new("pass_rate", "≥95.0%", "80.0%")],
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
        assert!(html.contains("100µs")); // P50
        assert!(html.contains("500µs")); // Max
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

    #[test]
    fn escape_html_works() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_html("it's"), "it&#39;s");
        assert_eq!(escape_html("normal text"), "normal text");
    }
}
