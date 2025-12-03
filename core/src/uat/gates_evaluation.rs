//! Quality gate evaluation logic.
//!
//! This module contains the evaluation methods for checking UAT results
//! against quality gates.

use super::gates_definitions::{EvaluationContext, GateResult, GateViolation, QualityGate};
use super::runner::{Priority, UatResults};

/// Evaluate UAT results against a quality gate.
///
/// Checks all gate criteria and returns a result with any violations.
///
/// # Arguments
/// * `gate` - Quality gate configuration to evaluate against
/// * `results` - UAT test results
///
/// # Returns
/// A `GateResult` indicating pass/fail and listing any violations.
pub fn evaluate(gate: &QualityGate, results: &UatResults) -> GateResult {
    evaluate_with_context(gate, results, &EvaluationContext::default())
}

/// Evaluate UAT results against a quality gate with additional context.
///
/// # Arguments
/// * `gate` - Quality gate configuration to evaluate against
/// * `results` - UAT test results
/// * `context` - Additional evaluation context (coverage, latency metrics)
///
/// # Returns
/// A `GateResult` indicating pass/fail and listing any violations.
pub fn evaluate_with_context(
    gate: &QualityGate,
    results: &UatResults,
    context: &EvaluationContext,
) -> GateResult {
    let mut violations = Vec::new();

    // Check pass rate
    let pass_rate = if results.total > 0 {
        results.passed as f64 / results.total as f64
    } else {
        1.0 // No tests means 100% pass rate
    };

    if pass_rate < gate.pass_rate {
        violations.push(GateViolation::new(
            "pass_rate",
            format!("≥{:.1}%", gate.pass_rate * 100.0),
            format!("{:.1}%", pass_rate * 100.0),
        ));
    }

    // Count P0 failures
    let p0_failures = results
        .results
        .iter()
        .filter(|r| !r.passed && r.test.priority == Priority::P0)
        .count();

    if p0_failures > gate.p0_open {
        violations.push(GateViolation::new(
            "p0_open",
            format!("≤{}", gate.p0_open),
            p0_failures.to_string(),
        ));
    }

    // Count P1 failures
    let p1_failures = results
        .results
        .iter()
        .filter(|r| !r.passed && r.test.priority == Priority::P1)
        .count();

    if p1_failures > gate.p1_open {
        violations.push(GateViolation::new(
            "p1_open",
            format!("≤{}", gate.p1_open),
            p1_failures.to_string(),
        ));
    }

    // Check max latency from results
    let max_latency_from_results = results.results.iter().map(|r| r.duration_us).max();
    let max_latency = context
        .max_latency_us
        .or(max_latency_from_results)
        .unwrap_or(0);

    if max_latency > gate.max_latency_us {
        violations.push(GateViolation::new(
            "max_latency_us",
            format!("≤{}µs", gate.max_latency_us),
            format!("{}µs", max_latency),
        ));
    }

    // Check coverage (only if provided in context)
    if let Some(coverage) = context.coverage {
        if coverage < gate.coverage_min {
            violations.push(GateViolation::new(
                "coverage_min",
                format!("≥{:.1}%", gate.coverage_min * 100.0),
                format!("{:.1}%", coverage * 100.0),
            ));
        }
    }

    if violations.is_empty() {
        tracing::info!(
            service = "keyrx",
            event = "gate_evaluation_passed",
            component = "quality_gate",
            pass_rate = %format!("{:.1}%", pass_rate * 100.0),
            p0_failures = p0_failures,
            p1_failures = p1_failures,
            "Quality gate passed"
        );
        GateResult::pass()
    } else {
        tracing::warn!(
            service = "keyrx",
            event = "gate_evaluation_failed",
            component = "quality_gate",
            violation_count = violations.len(),
            "Quality gate failed with {} violation(s)",
            violations.len()
        );
        GateResult::fail(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uat::runner::{UatResult, UatTest};

    // Helper function to create test results
    fn create_test_results(
        total: usize,
        passed: usize,
        p0_failures: usize,
        p1_failures: usize,
        max_duration_us: u64,
    ) -> UatResults {
        let mut results = Vec::new();
        let mut remaining_passed = passed;
        let mut remaining_p0 = p0_failures;
        let mut remaining_p1 = p1_failures;

        for i in 0..total {
            let (priority, is_passed) = if remaining_p0 > 0 {
                remaining_p0 -= 1;
                (Priority::P0, false)
            } else if remaining_p1 > 0 {
                remaining_p1 -= 1;
                (Priority::P1, false)
            } else if remaining_passed > 0 {
                remaining_passed -= 1;
                (Priority::P2, true)
            } else {
                (Priority::P2, false)
            };

            results.push(UatResult {
                test: UatTest {
                    name: format!("test_{}", i),
                    file: "test.rhai".to_string(),
                    category: "default".to_string(),
                    priority,
                    requirements: vec![],
                    latency_threshold: None,
                },
                passed: is_passed,
                duration_us: if i == 0 { max_duration_us } else { 100 },
                error: if is_passed {
                    None
                } else {
                    Some("Test failed".to_string())
                },
            });
        }

        UatResults {
            total,
            passed,
            failed: total - passed,
            skipped: 0,
            duration_us: max_duration_us + (total as u64 - 1) * 100,
            results,
        }
    }

    #[test]
    fn evaluate_passes_when_all_criteria_met() {
        let gate = QualityGate::default(); // 95% pass rate, 0 P0, 2 P1, 1000µs, 80% coverage

        // 100 tests, 96 passed, 0 P0 failures, 2 P1 failures, 500µs max latency
        let results = create_test_results(100, 96, 0, 2, 500);

        let result = evaluate(&gate, &results);

        assert!(result.passed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn evaluate_fails_on_low_pass_rate() {
        let gate = QualityGate::default(); // 95% pass rate

        // 100 tests, only 90 passed (90%)
        let results = create_test_results(100, 90, 0, 0, 500);

        let result = evaluate(&gate, &results);

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].criterion, "pass_rate");
        assert!(result.violations[0].expected.contains("95.0%"));
        assert!(result.violations[0].actual.contains("90.0%"));
    }

    #[test]
    fn evaluate_fails_on_p0_failures() {
        let gate = QualityGate::default(); // 0 P0 open

        // 100 tests, 98 passed, 1 P0 failure
        let results = create_test_results(100, 98, 1, 0, 500);

        let result = evaluate(&gate, &results);

        assert!(!result.passed);
        let p0_violation = result
            .violations
            .iter()
            .find(|v| v.criterion == "p0_open")
            .unwrap();
        assert_eq!(p0_violation.actual, "1");
    }

    #[test]
    fn evaluate_fails_on_too_many_p1_failures() {
        let gate = QualityGate::default(); // 2 P1 open max

        // 100 tests, 96 passed, 0 P0 failures, 3 P1 failures (exceeds limit)
        let results = create_test_results(100, 96, 0, 3, 500);

        let result = evaluate(&gate, &results);

        assert!(!result.passed);
        let p1_violation = result
            .violations
            .iter()
            .find(|v| v.criterion == "p1_open")
            .unwrap();
        assert_eq!(p1_violation.actual, "3");
    }

    #[test]
    fn evaluate_fails_on_high_latency() {
        let gate = QualityGate::default(); // 1000µs max

        // 100 tests, all pass, but max latency is 1500µs
        let results = create_test_results(100, 100, 0, 0, 1500);

        let result = evaluate(&gate, &results);

        assert!(!result.passed);
        let latency_violation = result
            .violations
            .iter()
            .find(|v| v.criterion == "max_latency_us")
            .unwrap();
        assert!(latency_violation.actual.contains("1500"));
    }

    #[test]
    fn evaluate_with_context_checks_coverage() {
        let gate = QualityGate::default(); // 80% coverage min

        let results = create_test_results(100, 100, 0, 0, 500);
        let context = EvaluationContext {
            coverage: Some(0.70), // Only 70% coverage
            max_latency_us: None,
        };

        let result = evaluate_with_context(&gate, &results, &context);

        assert!(!result.passed);
        let coverage_violation = result
            .violations
            .iter()
            .find(|v| v.criterion == "coverage_min")
            .unwrap();
        assert!(coverage_violation.expected.contains("80.0%"));
        assert!(coverage_violation.actual.contains("70.0%"));
    }

    #[test]
    fn evaluate_skips_coverage_when_not_provided() {
        let gate = QualityGate::default();

        let results = create_test_results(100, 100, 0, 0, 500);
        // No coverage in context
        let context = EvaluationContext::default();

        let result = evaluate_with_context(&gate, &results, &context);

        assert!(result.passed);
        assert!(!result
            .violations
            .iter()
            .any(|v| v.criterion == "coverage_min"));
    }

    #[test]
    fn evaluate_with_context_uses_provided_latency() {
        let gate = QualityGate::default(); // 1000µs max

        let results = create_test_results(100, 100, 0, 0, 500); // Results show 500µs
        let context = EvaluationContext {
            coverage: None,
            max_latency_us: Some(2000), // But context says 2000µs
        };

        let result = evaluate_with_context(&gate, &results, &context);

        assert!(!result.passed);
        let latency_violation = result
            .violations
            .iter()
            .find(|v| v.criterion == "max_latency_us")
            .unwrap();
        assert!(latency_violation.actual.contains("2000"));
    }

    #[test]
    fn evaluate_passes_with_no_tests() {
        let gate = QualityGate::default();

        let results = UatResults {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_us: 0,
            results: vec![],
        };

        let result = evaluate(&gate, &results);

        // No tests means 100% pass rate, 0 failures
        assert!(result.passed);
    }

    #[test]
    fn evaluate_collects_multiple_violations() {
        let gate = QualityGate::ga(); // Strictest: 100% pass, 0 P0/P1, 500µs, 90% coverage

        // Many violations: low pass rate, P0 failure, P1 failure, high latency
        let results = create_test_results(100, 80, 2, 3, 1000);
        let context = EvaluationContext {
            coverage: Some(0.50),
            max_latency_us: None,
        };

        let result = evaluate_with_context(&gate, &results, &context);

        assert!(!result.passed);
        // Should have violations for: pass_rate, p0_open, p1_open, max_latency_us, coverage_min
        assert!(result.violations.len() >= 4); // At least these should fail
        assert!(result.violations.iter().any(|v| v.criterion == "pass_rate"));
        assert!(result.violations.iter().any(|v| v.criterion == "p0_open"));
        assert!(result.violations.iter().any(|v| v.criterion == "p1_open"));
        assert!(result
            .violations
            .iter()
            .any(|v| v.criterion == "coverage_min"));
    }

    #[test]
    fn evaluate_with_alpha_gate_is_more_lenient() {
        let gate = QualityGate::alpha(); // 80% pass rate, 5 P1 open, 2000µs

        // Would fail default gate but passes alpha
        let results = create_test_results(100, 85, 0, 4, 1500);

        let result = evaluate(&gate, &results);

        assert!(result.passed);
    }
}
