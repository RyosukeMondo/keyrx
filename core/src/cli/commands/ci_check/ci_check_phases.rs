//! CI check phase execution.
//!
//! Contains individual phase runner functions for unit tests, integration tests,
//! UAT tests, regression tests, and performance tests.

use super::ci_check_summary::CheckPhaseResult;
use crate::uat::{GoldenSessionManager, PerformanceUat, UatFilter, UatResults, UatRunner};
use std::process::Command;
use std::time::Instant;

/// Run unit tests using cargo test.
pub fn run_unit_tests(json_output: bool) -> CheckPhaseResult {
    let start = Instant::now();

    if !json_output {
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
pub fn run_integration_tests(json_output: bool) -> CheckPhaseResult {
    let start = Instant::now();

    if !json_output {
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
///
/// Returns a tuple of the phase result and the UAT results for gate evaluation.
pub fn run_uat_tests(json_output: bool) -> (CheckPhaseResult, Option<UatResults>) {
    let start = Instant::now();

    if !json_output {
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
pub fn run_regression_tests(json_output: bool) -> CheckPhaseResult {
    let start = Instant::now();

    if !json_output {
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
pub fn run_performance_tests(json_output: bool) -> CheckPhaseResult {
    let start = Instant::now();

    if !json_output {
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
