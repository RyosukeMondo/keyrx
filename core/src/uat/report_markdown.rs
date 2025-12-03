//! Markdown report generation for UAT results.
//!
//! This module contains functions for generating GitHub-flavored markdown reports
//! suitable for PR comments and documentation.

use super::coverage::{CoverageReport, CoverageStatus};
use super::gates::GateResult;
use super::perf::PerfResults;
use super::report_data::ReportData;
use super::runner::Priority;

/// Generate a Markdown report for PR comments.
///
/// Creates a GitHub-flavored markdown report suitable for PR comments with:
/// - Summary section with pass/fail counts
/// - Test results grouped by category
/// - Coverage summary (if available)
/// - Performance summary (if available)
/// - Quality gate status (if available)
/// - Failed tests list
pub fn generate_markdown(data: &ReportData) -> String {
    let mut md = String::with_capacity(4096);

    // Title and summary
    md.push_str(&format!("# {}\n\n", data.title));
    md.push_str(&format!("_Generated: {}_\n\n", data.generated_at));

    // Summary section
    md.push_str(&markdown_summary_section(data));

    // Quality gate section (if available)
    if let Some(ref gate_result) = data.gate_result {
        md.push_str(&markdown_gate_section(gate_result));
    }

    // Failed tests section
    md.push_str(&markdown_failed_tests_section(data));

    // Results by category
    md.push_str(&markdown_category_section(data));

    // Results by priority
    md.push_str(&markdown_priority_section(data));

    // Coverage section (if available)
    if let Some(ref coverage) = data.coverage {
        md.push_str(&markdown_coverage_section(coverage));
    }

    // Performance section (if available)
    if let Some(ref perf) = data.performance {
        md.push_str(&markdown_performance_section(perf));
    }

    md
}

/// Generate markdown summary section.
pub fn markdown_summary_section(data: &ReportData) -> String {
    let pass_rate = data.pass_rate();
    let duration_ms = data.uat_results.duration_us / 1000;
    let status_emoji = if data.uat_results.failed == 0 {
        "✅"
    } else {
        "❌"
    };

    format!(
        "## {} Summary\n\n\
        | Metric | Value |\n\
        |--------|-------|\n\
        | **Total Tests** | {} |\n\
        | **Passed** | {} |\n\
        | **Failed** | {} |\n\
        | **Skipped** | {} |\n\
        | **Pass Rate** | {:.1}% |\n\
        | **Duration** | {}ms |\n\n",
        status_emoji,
        data.uat_results.total,
        data.uat_results.passed,
        data.uat_results.failed,
        data.uat_results.skipped,
        pass_rate,
        duration_ms
    )
}

/// Generate markdown quality gate section.
pub fn markdown_gate_section(gate_result: &GateResult) -> String {
    let (status_emoji, status_text) = if gate_result.passed {
        ("✅", "PASSED")
    } else {
        ("❌", "FAILED")
    };

    let mut md = format!("## {} Quality Gate: {}\n\n", status_emoji, status_text);

    if !gate_result.violations.is_empty() {
        md.push_str("### Violations\n\n");
        for violation in &gate_result.violations {
            md.push_str(&format!(
                "- **{}**: Expected {}, got {}\n",
                violation.criterion, violation.expected, violation.actual
            ));
        }
        md.push('\n');
    }

    md
}

/// Generate markdown failed tests section.
pub fn markdown_failed_tests_section(data: &ReportData) -> String {
    let failed: Vec<_> = data
        .uat_results
        .results
        .iter()
        .filter(|r| !r.passed)
        .collect();

    if failed.is_empty() {
        return String::new();
    }

    let mut md = String::from("## ❌ Failed Tests\n\n");
    md.push_str("| Test | Category | Priority | Error |\n");
    md.push_str("|------|----------|----------|-------|\n");

    for result in failed {
        let priority = match result.test.priority {
            Priority::P0 => "🔴 P0",
            Priority::P1 => "🟠 P1",
            Priority::P2 => "🔵 P2",
        };
        let error = result
            .error
            .as_deref()
            .unwrap_or("Unknown error")
            .replace('\n', " ");
        // Truncate long errors for table readability
        let error_display = if error.len() > 50 {
            format!("{}...", &error[..47])
        } else {
            error
        };

        md.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            result.test.name, result.test.category, priority, error_display
        ));
    }
    md.push('\n');

    md
}

/// Generate markdown category breakdown section.
pub fn markdown_category_section(data: &ReportData) -> String {
    let by_category = data.results_by_category();

    if by_category.is_empty() {
        return String::new();
    }

    let mut md = String::from("## Results by Category\n\n");
    md.push_str("| Category | Total | Passed | Failed | Pass Rate |\n");
    md.push_str("|----------|-------|--------|--------|----------|\n");

    let mut categories: Vec<_> = by_category.iter().collect();
    categories.sort_by_key(|(name, _)| name.as_str());

    for (category, stats) in categories {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {:.1}% |\n",
            category,
            stats.total,
            stats.passed,
            stats.failed,
            stats.pass_rate()
        ));
    }
    md.push('\n');

    md
}

/// Generate markdown priority breakdown section.
pub fn markdown_priority_section(data: &ReportData) -> String {
    let by_priority = data.results_by_priority();

    if by_priority.is_empty() {
        return String::new();
    }

    let mut md = String::from("## Results by Priority\n\n");
    md.push_str("| Priority | Total | Passed | Failed | Pass Rate |\n");
    md.push_str("|----------|-------|--------|--------|----------|\n");

    let mut priorities: Vec<_> = by_priority.iter().collect();
    priorities.sort_by_key(|(priority, _)| match priority {
        Priority::P0 => 0,
        Priority::P1 => 1,
        Priority::P2 => 2,
    });

    for (priority, stats) in priorities {
        let priority_str = match priority {
            Priority::P0 => "🔴 P0 (Critical)",
            Priority::P1 => "🟠 P1 (High)",
            Priority::P2 => "🔵 P2 (Normal)",
        };
        md.push_str(&format!(
            "| {} | {} | {} | {} | {:.1}% |\n",
            priority_str,
            stats.total,
            stats.passed,
            stats.failed,
            stats.pass_rate()
        ));
    }
    md.push('\n');

    md
}

/// Generate markdown coverage section.
pub fn markdown_coverage_section(coverage: &CoverageReport) -> String {
    let mut md = format!(
        "## Requirements Coverage\n\n\
        | Metric | Value |\n\
        |--------|-------|\n\
        | **Total Requirements** | {} |\n\
        | **Verified** | {} |\n\
        | **At Risk** | {} |\n\
        | **Uncovered** | {} |\n\
        | **Coverage** | {:.1}% |\n\n",
        coverage.total,
        coverage.verified,
        coverage.at_risk,
        coverage.uncovered,
        coverage.coverage_percentage * 100.0
    );

    // Show at-risk and uncovered requirements
    let at_risk: Vec<_> = coverage
        .coverage
        .requirements
        .iter()
        .filter(|(_, r)| matches!(r.status, CoverageStatus::AtRisk))
        .collect();

    let uncovered: Vec<_> = coverage
        .coverage
        .requirements
        .iter()
        .filter(|(_, r)| matches!(r.status, CoverageStatus::Uncovered))
        .collect();

    if !at_risk.is_empty() {
        md.push_str("### ⚠️ At Risk Requirements\n\n");
        for (id, req) in &at_risk {
            md.push_str(&format!("- **{}**: {}\n", id, req.linked_tests.join(", ")));
        }
        md.push('\n');
    }

    if !uncovered.is_empty() {
        md.push_str("### 🚫 Uncovered Requirements\n\n");
        for (id, _) in &uncovered {
            md.push_str(&format!("- **{}**\n", id));
        }
        md.push('\n');
    }

    md
}

/// Generate markdown performance section.
pub fn markdown_performance_section(perf: &PerfResults) -> String {
    let mut md = format!(
        "## Performance Metrics\n\n\
        | Percentile | Latency |\n\
        |------------|--------|\n\
        | **P50** | {}µs |\n\
        | **P95** | {}µs |\n\
        | **P99** | {}µs |\n\
        | **Max** | {}µs |\n\n",
        perf.aggregate_p50_us, perf.aggregate_p95_us, perf.aggregate_p99_us, perf.aggregate_max_us
    );

    if !perf.all_violations.is_empty() {
        md.push_str("### ⚠️ Latency Violations\n\n");
        for violation in &perf.all_violations {
            md.push_str(&format!(
                "- **{}**: Expected ≤{}µs, got {}µs (iteration {})\n",
                violation.test_name,
                violation.threshold_us,
                violation.actual_us,
                violation.iteration
            ));
        }
        md.push('\n');
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uat::coverage::{CoverageMap, CoverageReport, RequirementCoverage};
    use crate::uat::gates::GateViolation;
    use crate::uat::perf::LatencyViolation;
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
    fn generate_markdown_basic() {
        let results = create_uat_results();
        let data = ReportData::new(results);

        let md = generate_markdown(&data);

        assert!(md.contains("# UAT Report"));
        assert!(md.contains("## ❌ Summary")); // Has failures
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

        let md = generate_markdown(&data);

        assert!(md.contains("## ✅ Summary")); // All passed
        assert!(!md.contains("## ❌ Failed Tests")); // No failures section
    }

    #[test]
    fn generate_markdown_with_gate_result() {
        let results = create_uat_results();
        let gate_result = GateResult {
            passed: false,
            violations: vec![GateViolation::new("pass_rate", "≥95.0%", "80.0%")],
        };
        let data = ReportData::new(results).with_gate_result(gate_result);

        let md = generate_markdown(&data);

        assert!(md.contains("## ❌ Quality Gate: FAILED"));
        assert!(md.contains("### Violations"));
        assert!(md.contains("**pass_rate**: Expected ≥95.0%, got 80.0%"));
    }

    #[test]
    fn generate_markdown_with_gate_passed() {
        let results = create_uat_results();
        let gate_result = GateResult {
            passed: true,
            violations: vec![],
        };
        let data = ReportData::new(results).with_gate_result(gate_result);

        let md = generate_markdown(&data);

        assert!(md.contains("## ✅ Quality Gate: PASSED"));
        assert!(!md.contains("### Violations"));
    }

    #[test]
    fn generate_markdown_failed_tests() {
        let results = create_uat_results();
        let data = ReportData::new(results);

        let md = generate_markdown(&data);

        assert!(md.contains("## ❌ Failed Tests"));
        assert!(md.contains("| Test | Category | Priority | Error |"));
        assert!(md.contains("test3"));
        assert!(md.contains("test7"));
    }

    #[test]
    fn generate_markdown_category_breakdown() {
        let results = create_uat_results();
        let data = ReportData::new(results);

        let md = generate_markdown(&data);

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

        let md = generate_markdown(&data);

        assert!(md.contains("## Results by Priority"));
        assert!(md.contains("🔴 P0 (Critical)"));
        assert!(md.contains("🟠 P1 (High)"));
        assert!(md.contains("🔵 P2 (Normal)"));
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

        let md = generate_markdown(&data);

        assert!(md.contains("## Requirements Coverage"));
        assert!(md.contains("| **Total Requirements** | 3 |"));
        assert!(md.contains("| **Verified** | 1 |"));
        assert!(md.contains("| **At Risk** | 1 |"));
        assert!(md.contains("| **Uncovered** | 1 |"));
        assert!(md.contains("### ⚠️ At Risk Requirements"));
        assert!(md.contains("**1.2**"));
        assert!(md.contains("### 🚫 Uncovered Requirements"));
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

        let md = generate_markdown(&data);

        assert!(md.contains("## Performance Metrics"));
        assert!(md.contains("| **P50** | 100µs |"));
        assert!(md.contains("| **P95** | 200µs |"));
        assert!(md.contains("| **P99** | 300µs |"));
        assert!(md.contains("| **Max** | 500µs |"));
        assert!(md.contains("### ⚠️ Latency Violations"));
        assert!(md.contains("**perf_test2**: Expected ≤1000µs, got 1500µs"));
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

        let md = generate_markdown(&data);

        assert!(md.contains("# UAT Report"));
        assert!(md.contains("## ✅ Summary")); // No failures = pass emoji
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

        let md = generate_markdown(&data);

        // Truncates at 47 chars + "..." = 50 chars total
        assert!(md.contains("This is a very long error message that should b..."));
        assert!(!md.contains("exceeds the maximum length"));
    }
}
