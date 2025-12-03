//! CI check command for unified test execution.
//!
//! Runs all test types: unit tests, integration tests, UAT tests,
//! regression tests, and performance tests with quality gate enforcement.

use crate::cli::{OutputFormat, OutputWriter};
use crate::uat::{
    GateResult, GoldenSessionManager, PerformanceUat, QualityGateEnforcer, UatFilter, UatResults,
    UatRunner,
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::process::Command;
use std::time::Instant;

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
    fn new() -> Self {
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

    fn add_phase(&mut self, result: CheckPhaseResult) {
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

    fn set_gate_result(&mut self, result: GateResult) {
        if !result.passed && self.exit_code == exit_codes::SUCCESS {
            self.exit_code = exit_codes::GATE_FAIL;
            self.passed = false;
        }
        self.gate_result = Some(result);
    }
}

/// CI check command for running all tests in CI pipeline.
pub struct CiCheckCommand {
    /// Quality gate name to enforce.
    pub gate: Option<String>,
    /// Output in JSON format.
    pub json: bool,
    /// Skip unit tests.
    pub skip_unit: bool,
    /// Skip integration tests.
    pub skip_integration: bool,
    /// Skip UAT tests.
    pub skip_uat: bool,
    /// Skip regression tests.
    pub skip_regression: bool,
    /// Skip performance tests.
    pub skip_perf: bool,
    /// Output writer.
    pub output: OutputWriter,
}

impl CiCheckCommand {
    /// Create a new CI check command.
    pub fn new(format: OutputFormat) -> Self {
        Self {
            gate: None,
            json: matches!(format, OutputFormat::Json),
            skip_unit: false,
            skip_integration: false,
            skip_uat: false,
            skip_regression: false,
            skip_perf: false,
            output: OutputWriter::new(format),
        }
    }

    /// Set quality gate.
    pub fn with_gate(mut self, gate: Option<String>) -> Self {
        self.gate = gate;
        self
    }

    /// Enable JSON output.
    pub fn with_json(mut self, json: bool) -> Self {
        self.json = json;
        if json {
            self.output = OutputWriter::new(OutputFormat::Json);
        }
        self
    }

    /// Skip unit tests.
    pub fn with_skip_unit(mut self, skip: bool) -> Self {
        self.skip_unit = skip;
        self
    }

    /// Skip integration tests.
    pub fn with_skip_integration(mut self, skip: bool) -> Self {
        self.skip_integration = skip;
        self
    }

    /// Skip UAT tests.
    pub fn with_skip_uat(mut self, skip: bool) -> Self {
        self.skip_uat = skip;
        self
    }

    /// Skip regression tests.
    pub fn with_skip_regression(mut self, skip: bool) -> Self {
        self.skip_regression = skip;
        self
    }

    /// Skip performance tests.
    pub fn with_skip_perf(mut self, skip: bool) -> Self {
        self.skip_perf = skip;
        self
    }

    /// Run the CI check command.
    ///
    /// Returns the exit code:
    /// - 0: All checks passed and gate passed
    /// - 1: Test failure
    /// - 2: Gate failure
    /// - 3: Crash detected
    pub fn run(&self) -> Result<i32> {
        let mut summary = CiCheckSummary::new();

        if !self.json {
            println!("\n{}", "═".repeat(60));
            println!("  KeyRx CI Check");
            println!("{}", "═".repeat(60));
        }

        // Phase 1: Unit tests
        if !self.skip_unit {
            let result = self.run_unit_tests();
            self.output_phase_result(&result);
            summary.add_phase(result);
        }

        // Phase 2: Integration tests
        if !self.skip_integration {
            let result = self.run_integration_tests();
            self.output_phase_result(&result);
            summary.add_phase(result);
        }

        // Phase 3: UAT tests
        if !self.skip_uat {
            let (result, uat_results) = self.run_uat_tests();
            self.output_phase_result(&result);
            summary.add_phase(result);

            // Evaluate quality gate if specified
            if let Some(ref gate_name) = self.gate {
                if let Some(ref results) = uat_results {
                    let gate_result = self.evaluate_gate(gate_name, results)?;
                    summary.set_gate_result(gate_result);
                }
            }
        }

        // Phase 4: Regression tests
        if !self.skip_regression {
            let result = self.run_regression_tests();
            self.output_phase_result(&result);
            summary.add_phase(result);
        }

        // Phase 5: Performance tests
        if !self.skip_perf {
            let result = self.run_performance_tests();
            self.output_phase_result(&result);
            summary.add_phase(result);
        }

        // Output final summary
        if self.json {
            self.output.data(&summary)?;
        } else {
            self.output_human_summary(&summary);
        }

        Ok(summary.exit_code)
    }

    /// Run unit tests using cargo test.
    fn run_unit_tests(&self) -> CheckPhaseResult {
        let start = Instant::now();

        if !self.json {
            println!("\n  [1/5] Running unit tests...");
        }

        let result = Command::new("cargo")
            .args(["test", "--lib", "--quiet"])
            .output();

        let duration_us = start.elapsed().as_micros() as u64;

        match result {
            Ok(output) => {
                let passed = output.status.success();
                let error = if !passed {
                    Some(String::from_utf8_lossy(&output.stderr).to_string())
                } else {
                    None
                };

                CheckPhaseResult {
                    name: "Unit Tests".to_string(),
                    passed,
                    duration_us,
                    tests_run: None,
                    tests_passed: None,
                    tests_failed: None,
                    error,
                }
            }
            Err(e) => CheckPhaseResult {
                name: "Unit Tests".to_string(),
                passed: false,
                duration_us,
                tests_run: None,
                tests_passed: None,
                tests_failed: None,
                error: Some(format!("Failed to run cargo test: {}", e)),
            },
        }
    }

    /// Run integration tests using cargo test.
    fn run_integration_tests(&self) -> CheckPhaseResult {
        let start = Instant::now();

        if !self.json {
            println!("\n  [2/5] Running integration tests...");
        }

        let result = Command::new("cargo")
            .args(["test", "--test", "*", "--quiet"])
            .output();

        let duration_us = start.elapsed().as_micros() as u64;

        match result {
            Ok(output) => {
                let passed = output.status.success();
                let error = if !passed {
                    Some(String::from_utf8_lossy(&output.stderr).to_string())
                } else {
                    None
                };

                CheckPhaseResult {
                    name: "Integration Tests".to_string(),
                    passed,
                    duration_us,
                    tests_run: None,
                    tests_passed: None,
                    tests_failed: None,
                    error,
                }
            }
            Err(e) => CheckPhaseResult {
                name: "Integration Tests".to_string(),
                passed: false,
                duration_us,
                tests_run: None,
                tests_passed: None,
                tests_failed: None,
                error: Some(format!("Failed to run integration tests: {}", e)),
            },
        }
    }

    /// Run UAT tests.
    fn run_uat_tests(&self) -> (CheckPhaseResult, Option<UatResults>) {
        let start = Instant::now();

        if !self.json {
            println!("\n  [3/5] Running UAT tests...");
        }

        let runner = UatRunner::new();
        let filter = UatFilter::default();
        let results = runner.run(&filter);

        let duration_us = start.elapsed().as_micros() as u64;
        let passed = results.failed == 0;

        let result = CheckPhaseResult {
            name: "UAT Tests".to_string(),
            passed,
            duration_us,
            tests_run: Some(results.total),
            tests_passed: Some(results.passed),
            tests_failed: Some(results.failed),
            error: if !passed {
                Some(format!("{} UAT test(s) failed", results.failed))
            } else {
                None
            },
        };

        (result, Some(results))
    }

    /// Run regression tests.
    fn run_regression_tests(&self) -> CheckPhaseResult {
        let start = Instant::now();

        if !self.json {
            println!("\n  [4/5] Running regression tests...");
        }

        let manager = GoldenSessionManager::new();
        let sessions = match manager.list_sessions() {
            Ok(s) => s,
            Err(e) => {
                return CheckPhaseResult {
                    name: "Regression Tests".to_string(),
                    passed: false,
                    duration_us: start.elapsed().as_micros() as u64,
                    tests_run: None,
                    tests_passed: None,
                    tests_failed: None,
                    error: Some(format!("Failed to list golden sessions: {}", e)),
                };
            }
        };

        if sessions.is_empty() {
            return CheckPhaseResult {
                name: "Regression Tests".to_string(),
                passed: true,
                duration_us: start.elapsed().as_micros() as u64,
                tests_run: Some(0),
                tests_passed: Some(0),
                tests_failed: Some(0),
                error: None,
            };
        }

        let mut passed_count = 0;
        let mut failed_count = 0;

        for session in &sessions {
            match manager.verify(session) {
                Ok(result) => {
                    if result.passed {
                        passed_count += 1;
                    } else {
                        failed_count += 1;
                    }
                }
                Err(_) => {
                    failed_count += 1;
                }
            }
        }

        let duration_us = start.elapsed().as_micros() as u64;
        let passed = failed_count == 0;

        CheckPhaseResult {
            name: "Regression Tests".to_string(),
            passed,
            duration_us,
            tests_run: Some(sessions.len()),
            tests_passed: Some(passed_count),
            tests_failed: Some(failed_count),
            error: if !passed {
                Some(format!(
                    "{} golden session(s) failed verification",
                    failed_count
                ))
            } else {
                None
            },
        }
    }

    /// Run performance tests.
    fn run_performance_tests(&self) -> CheckPhaseResult {
        let start = Instant::now();

        if !self.json {
            println!("\n  [5/5] Running performance tests...");
        }

        let perf_runner = PerformanceUat::new();
        let filter = UatFilter::default();
        let results = perf_runner.run(&filter);

        let duration_us = start.elapsed().as_micros() as u64;
        let passed = results.failed == 0;

        CheckPhaseResult {
            name: "Performance Tests".to_string(),
            passed,
            duration_us,
            tests_run: Some(results.total),
            tests_passed: Some(results.passed),
            tests_failed: Some(results.failed),
            error: if !passed {
                Some(format!(
                    "{} performance test(s) exceeded latency threshold",
                    results.failed
                ))
            } else {
                None
            },
        }
    }

    /// Evaluate quality gate against UAT results.
    fn evaluate_gate(&self, gate_name: &str, results: &UatResults) -> Result<GateResult> {
        let enforcer = QualityGateEnforcer::new();
        let gate = enforcer
            .load(Some(gate_name))
            .context(format!("Failed to load quality gate '{}'", gate_name))?;
        Ok(enforcer.evaluate(&gate, results))
    }

    /// Output a single phase result (human-readable).
    fn output_phase_result(&self, result: &CheckPhaseResult) {
        if self.json {
            return;
        }

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
    fn output_human_summary(&self, summary: &CiCheckSummary) {
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
    fn ci_check_command_new() {
        let cmd = CiCheckCommand::new(OutputFormat::Human);
        assert!(cmd.gate.is_none());
        assert!(!cmd.json);
        assert!(!cmd.skip_unit);
        assert!(!cmd.skip_integration);
        assert!(!cmd.skip_uat);
        assert!(!cmd.skip_regression);
        assert!(!cmd.skip_perf);
    }

    #[test]
    fn ci_check_command_builder_methods() {
        let cmd = CiCheckCommand::new(OutputFormat::Human)
            .with_gate(Some("beta".to_string()))
            .with_json(true)
            .with_skip_unit(true)
            .with_skip_integration(true)
            .with_skip_uat(true)
            .with_skip_regression(true)
            .with_skip_perf(true);

        assert_eq!(cmd.gate, Some("beta".to_string()));
        assert!(cmd.json);
        assert!(cmd.skip_unit);
        assert!(cmd.skip_integration);
        assert!(cmd.skip_uat);
        assert!(cmd.skip_regression);
        assert!(cmd.skip_perf);
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
