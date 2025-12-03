//! HTML section generators for coverage and performance reports.
//!
//! This module contains functions for generating specialized HTML sections
//! for requirements coverage and performance metrics.

use super::coverage::{CoverageReport, CoverageStatus};
use super::perf::PerfResults;
use super::report_html::escape_html;

/// Generate coverage section.
pub fn html_coverage_section(coverage: &CoverageReport) -> String {
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
pub fn html_performance_section(perf: &PerfResults) -> String {
    let mut html = format!(
        r#"<section class="card">
<h2>Performance Metrics</h2>
<div style="margin-bottom: 1rem;">
<span class="perf-metric"><span class="perf-value">{}us</span><br><span class="perf-label">P50 Latency</span></span>
<span class="perf-metric"><span class="perf-value">{}us</span><br><span class="perf-label">P95 Latency</span></span>
<span class="perf-metric"><span class="perf-value">{}us</span><br><span class="perf-label">P99 Latency</span></span>
<span class="perf-metric"><span class="perf-value">{}us</span><br><span class="perf-label">Max Latency</span></span>
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
                "<div class=\"violation\"><strong>{}:</strong> Expected \u{2264}{}us, got {}us (iteration {})</div>\n",
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
                .map(|t| format!("{}us", t))
                .unwrap_or_else(|| "-".to_string());

            html.push_str(&format!(
                "<tr><td>{}</td><td>{}us</td><td>{}us</td><td>{}us</td><td>{}us</td><td>{}</td><td>{}</td></tr>\n",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uat::coverage::{CoverageMap, RequirementCoverage};
    use crate::uat::perf::{LatencyViolation, PerformanceResult};

    #[test]
    fn html_coverage_section_basic() {
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

        let html = html_coverage_section(&coverage);

        assert!(html.contains("Requirements Coverage"));
        assert!(html.contains("1.1"));
        assert!(html.contains("Verified"));
    }

    #[test]
    fn html_performance_section_basic() {
        let perf = PerfResults {
            total: 1,
            passed: 1,
            failed: 0,
            aggregate_p50_us: 100,
            aggregate_p95_us: 200,
            aggregate_p99_us: 300,
            aggregate_max_us: 500,
            total_duration_us: 10000,
            results: vec![PerformanceResult {
                test_name: "perf_test".to_string(),
                test_file: "test.rhai".to_string(),
                p50_us: 100,
                p95_us: 200,
                p99_us: 300,
                max_us: 500,
                min_us: 50,
                iterations: 100,
                threshold_us: Some(1000),
                threshold_exceeded: false,
                violations: vec![],
            }],
            all_violations: vec![],
        };

        let html = html_performance_section(&perf);

        assert!(html.contains("Performance Metrics"));
        assert!(html.contains("100us"));
        assert!(html.contains("perf_test"));
    }

    #[test]
    fn html_performance_section_with_violations() {
        let perf = PerfResults {
            total: 1,
            passed: 0,
            failed: 1,
            aggregate_p50_us: 100,
            aggregate_p95_us: 200,
            aggregate_p99_us: 300,
            aggregate_max_us: 1500,
            total_duration_us: 10000,
            results: vec![],
            all_violations: vec![LatencyViolation {
                test_name: "slow_test".to_string(),
                threshold_us: 1000,
                actual_us: 1500,
                iteration: 50,
            }],
        };

        let html = html_performance_section(&perf);

        assert!(html.contains("Latency Violations"));
        assert!(html.contains("slow_test"));
        assert!(html.contains("1000"));
        assert!(html.contains("1500"));
    }
}
