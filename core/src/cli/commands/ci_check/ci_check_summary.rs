//! CI check summary types and output generation.
//!
//! Contains the data structures for check phase results and summary,
//! as well as human-readable output formatting.

use crate::uat::GateResult;
use serde::Serialize;

/// Exit codes for CI check command.
pub mod exit_codes {
    /// All checks passed and gate passed.
    pub const SUCCESS: i32 = 0;
    /// Test failure (unit, integration, UAT, or regression).
    pub const TEST_FAIL: i32 = 1;
    /// Gate failure (tests passed but gate criteria not met).
    pub const GATE_FAIL: i32 = 2;
    /// Crash detected (in fuzz testing or panic).
    pub const CRASH: i32 = 3;
}

/// Result of a single check phase.
#[derive(Debug, Clone, Serialize)]
pub struct CheckPhaseResult {
    /// Phase name.
    pub name: String,
    /// Whether the phase passed.
    pub passed: bool,
    /// Duration in microseconds.
    pub duration_us: u64,
    /// Number of tests run (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tests_run: Option<usize>,
    /// Number of tests passed (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tests_passed: Option<usize>,
    /// Number of tests failed (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tests_failed: Option<usize>,
    /// Error message if phase failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Summary of all CI checks.
#[derive(Debug, Clone, Serialize)]
pub struct CiCheckSummary {
    /// Overall pass/fail status.
    pub passed: bool,
    /// Total duration in microseconds.
    pub total_duration_us: u64,
    /// Number of phases that passed.
    pub phases_passed: usize,
    /// Number of phases that failed.
    pub phases_failed: usize,
    /// Results for each phase.
    pub phases: Vec<CheckPhaseResult>,
    /// Gate result if a gate was specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_result: Option<GateResult>,
    /// Exit code that should be returned.
    pub exit_code: i32,
}

impl CiCheckSummary {
    /// Create a new empty summary.
    pub fn new() -> Self {
        Self {
            passed: true,
            total_duration_us: 0,
            phases_passed: 0,
            phases_failed: 0,
            phases: Vec::new(),
            gate_result: None,
            exit_code: exit_codes::SUCCESS,
        }
    }

    /// Add a phase result to the summary.
    pub fn add_phase(&mut self, result: CheckPhaseResult) {
        self.total_duration_us += result.duration_us;
        if result.passed {
            self.phases_passed += 1;
        } else {
            self.phases_failed += 1;
            self.passed = false;
            if self.exit_code == exit_codes::SUCCESS {
                self.exit_code = exit_codes::TEST_FAIL;
            }
        }
        self.phases.push(result);
    }

    /// Set the gate result.
    pub fn set_gate_result(&mut self, result: GateResult) {
        if !result.passed && self.exit_code == exit_codes::SUCCESS {
            self.exit_code = exit_codes::GATE_FAIL;
            self.passed = false;
        }
        self.gate_result = Some(result);
    }
}

impl Default for CiCheckSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Output a single phase result in human-readable format.
pub fn output_phase_result(result: &CheckPhaseResult) {
    let status = if result.passed { "✓" } else { "✗" };
    let duration_ms = result.duration_us as f64 / 1000.0;

    match (
        &result.tests_run,
        &result.tests_passed,
        &result.tests_failed,
    ) {
        (Some(total), Some(passed), Some(failed)) => {
            println!(
                "        {} {} ({}/{} passed, {} failed) [{:.1}ms]",
                status, result.name, passed, total, failed, duration_ms
            );
        }
        _ => {
            println!("        {} {} [{:.1}ms]", status, result.name, duration_ms);
        }
    }

    if !result.passed {
        if let Some(ref error) = result.error {
            let error_preview = if error.len() > 100 {
                format!("{}...", &error[..100])
            } else {
                error.clone()
            };
            println!("          └─ {}", error_preview);
        }
    }
}

/// Output human-readable summary.
pub fn output_human_summary(summary: &CiCheckSummary) {
    println!("\n{}", "═".repeat(60));
    println!("  CI Check Summary");
    println!("{}", "═".repeat(60));

    let duration_ms = summary.total_duration_us as f64 / 1000.0;
    println!(
        "\n  Phases: {} passed, {} failed ({:.1}ms total)",
        summary.phases_passed, summary.phases_failed, duration_ms
    );

    // Gate result
    if let Some(ref gate) = summary.gate_result {
        println!();
        if gate.passed {
            println!("  Quality Gate: PASSED ✓");
        } else {
            println!("  Quality Gate: FAILED ✗");
            for violation in &gate.violations {
                println!(
                    "    - {}: expected {}, got {}",
                    violation.criterion, violation.expected, violation.actual
                );
            }
        }
    }

    // Final status
    println!();
    match summary.exit_code {
        exit_codes::SUCCESS => {
            println!("  Status: PASSED ✓");
        }
        exit_codes::TEST_FAIL => {
            println!("  Status: FAILED (test failures)");
        }
        exit_codes::GATE_FAIL => {
            println!("  Status: FAILED (gate violations)");
        }
        exit_codes::CRASH => {
            println!("  Status: FAILED (crash detected)");
        }
        _ => {
            println!("  Status: FAILED (unknown error)");
        }
    }

    println!("{}", "═".repeat(60));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_correct() {
        assert_eq!(exit_codes::SUCCESS, 0);
        assert_eq!(exit_codes::TEST_FAIL, 1);
        assert_eq!(exit_codes::GATE_FAIL, 2);
        assert_eq!(exit_codes::CRASH, 3);
    }

    #[test]
    fn ci_check_summary_new() {
        let summary = CiCheckSummary::new();
        assert!(summary.passed);
        assert_eq!(summary.total_duration_us, 0);
        assert_eq!(summary.phases_passed, 0);
        assert_eq!(summary.phases_failed, 0);
        assert!(summary.phases.is_empty());
        assert!(summary.gate_result.is_none());
        assert_eq!(summary.exit_code, exit_codes::SUCCESS);
    }

    #[test]
    fn ci_check_summary_add_passed_phase() {
        let mut summary = CiCheckSummary::new();
        summary.add_phase(CheckPhaseResult {
            name: "Test".to_string(),
            passed: true,
            duration_us: 1000,
            tests_run: Some(10),
            tests_passed: Some(10),
            tests_failed: Some(0),
            error: None,
        });

        assert!(summary.passed);
        assert_eq!(summary.phases_passed, 1);
        assert_eq!(summary.phases_failed, 0);
        assert_eq!(summary.total_duration_us, 1000);
        assert_eq!(summary.exit_code, exit_codes::SUCCESS);
    }

    #[test]
    fn ci_check_summary_add_failed_phase() {
        let mut summary = CiCheckSummary::new();
        summary.add_phase(CheckPhaseResult {
            name: "Test".to_string(),
            passed: false,
            duration_us: 1000,
            tests_run: Some(10),
            tests_passed: Some(8),
            tests_failed: Some(2),
            error: Some("2 tests failed".to_string()),
        });

        assert!(!summary.passed);
        assert_eq!(summary.phases_passed, 0);
        assert_eq!(summary.phases_failed, 1);
        assert_eq!(summary.exit_code, exit_codes::TEST_FAIL);
    }

    #[test]
    fn ci_check_summary_set_gate_result_passed() {
        let mut summary = CiCheckSummary::new();
        summary.set_gate_result(GateResult {
            passed: true,
            violations: vec![],
        });

        assert!(summary.passed);
        assert!(summary.gate_result.is_some());
        assert_eq!(summary.exit_code, exit_codes::SUCCESS);
    }

    #[test]
    fn ci_check_summary_set_gate_result_failed() {
        use crate::uat::GateViolation;

        let mut summary = CiCheckSummary::new();
        summary.set_gate_result(GateResult {
            passed: false,
            violations: vec![GateViolation::new("pass_rate", "≥95%", "90%")],
        });

        assert!(!summary.passed);
        assert!(summary.gate_result.is_some());
        assert_eq!(summary.exit_code, exit_codes::GATE_FAIL);
    }

    #[test]
    fn ci_check_summary_test_fail_takes_priority_over_gate_fail() {
        use crate::uat::GateViolation;

        let mut summary = CiCheckSummary::new();

        // Add a failed test phase first
        summary.add_phase(CheckPhaseResult {
            name: "Test".to_string(),
            passed: false,
            duration_us: 1000,
            tests_run: Some(10),
            tests_passed: Some(8),
            tests_failed: Some(2),
            error: Some("2 tests failed".to_string()),
        });

        // Then add a failed gate
        summary.set_gate_result(GateResult {
            passed: false,
            violations: vec![GateViolation::new("pass_rate", "≥95%", "90%")],
        });

        // Test failure should take priority
        assert_eq!(summary.exit_code, exit_codes::TEST_FAIL);
    }

    #[test]
    fn check_phase_result_serializes() {
        let result = CheckPhaseResult {
            name: "Unit Tests".to_string(),
            passed: true,
            duration_us: 5000,
            tests_run: Some(100),
            tests_passed: Some(100),
            tests_failed: Some(0),
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"name\":\"Unit Tests\""));
        assert!(json.contains("\"passed\":true"));
        assert!(json.contains("\"tests_run\":100"));
        assert!(!json.contains("error")); // Should be skipped when None
    }

    #[test]
    fn check_phase_result_with_error_serializes() {
        let result = CheckPhaseResult {
            name: "Unit Tests".to_string(),
            passed: false,
            duration_us: 5000,
            tests_run: None,
            tests_passed: None,
            tests_failed: None,
            error: Some("Build failed".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"error\":\"Build failed\""));
        assert!(!json.contains("tests_run")); // Should be skipped when None
    }

    #[test]
    fn ci_check_summary_serializes() {
        let mut summary = CiCheckSummary::new();
        summary.add_phase(CheckPhaseResult {
            name: "Test".to_string(),
            passed: true,
            duration_us: 1000,
            tests_run: Some(10),
            tests_passed: Some(10),
            tests_failed: Some(0),
            error: None,
        });

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"passed\":true"));
        assert!(json.contains("\"phases_passed\":1"));
        assert!(json.contains("\"phases_failed\":0"));
        assert!(json.contains("\"exit_code\":0"));
    }
}
