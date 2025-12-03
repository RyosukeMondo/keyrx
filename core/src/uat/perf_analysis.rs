//! Performance analysis and regression detection.
//!
//! This module provides statistical analysis functions for performance testing,
//! including baseline comparison and regression detection.

use std::process::Command;

use super::perf_types::{
    BaselineData, BaselineError, BaselineRegression, BaselineTestData, PerfComparison, PerfResults,
};

/// Default baseline file path relative to project root.
const BASELINE_FILE: &str = "target/perf-baseline.json";

/// Get the current Git branch name.
pub fn get_current_branch() -> Result<String, BaselineError> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| BaselineError::GitError(format!("Failed to run git: {e}")))?;

    if !output.status.success() {
        return Err(BaselineError::GitError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get the current Git commit hash.
pub fn get_current_commit() -> Result<String, BaselineError> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| BaselineError::GitError(format!("Failed to run git: {e}")))?;

    if !output.status.success() {
        return Err(BaselineError::GitError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Fetch baseline data from a Git branch.
pub fn fetch_baseline_from_branch(branch: &str) -> Result<BaselineData, BaselineError> {
    // Use git show to get the baseline file content from the target branch
    let output = Command::new("git")
        .args(["show", &format!("{branch}:{BASELINE_FILE}")])
        .output()
        .map_err(|e| BaselineError::GitError(format!("Failed to run git: {e}")))?;

    if !output.status.success() {
        // Check if it's a "file not found" error
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("does not exist") || stderr.contains("not found") {
            return Err(BaselineError::BaselineNotFound(branch.to_string()));
        }
        return Err(BaselineError::GitError(stderr.to_string()));
    }

    let content = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&content)
        .map_err(|e| BaselineError::ParseError(format!("Invalid baseline JSON: {e}")))
}

/// Compare current results against baseline and detect regressions.
///
/// This function compares each test's p50 latency against the baseline
/// and flags regressions that exceed the specified threshold.
///
/// # Arguments
/// * `baseline` - The baseline performance data to compare against
/// * `current` - Current performance test results
/// * `current_commit` - The current Git commit hash
/// * `threshold_us` - Regression threshold in microseconds
///
/// # Returns
/// A `PerfComparison` with detailed regression information.
pub fn compare_results(
    baseline: &BaselineData,
    current: &PerfResults,
    current_commit: &str,
    threshold_us: u64,
) -> PerfComparison {
    let mut regressions = Vec::new();
    let mut has_regression = false;

    // Compare per-test results
    for result in &current.results {
        if let Some(baseline_test) = baseline.tests.get(&result.test_name) {
            let delta_us = result.p50_us as i64 - baseline_test.p50_us as i64;
            let exceeds_threshold = delta_us > threshold_us as i64;

            if exceeds_threshold {
                has_regression = true;
            }

            // Record all tests with positive delta (performance got worse)
            if delta_us > 0 {
                regressions.push(BaselineRegression {
                    test_name: result.test_name.clone(),
                    baseline_p50_us: baseline_test.p50_us,
                    current_p50_us: result.p50_us,
                    delta_us,
                    threshold_us,
                    exceeds_threshold,
                });
            }
        }
    }

    // Calculate aggregate deltas
    let aggregate_p50_delta_us = current.aggregate_p50_us as i64 - baseline.aggregate_p50_us as i64;
    let aggregate_p95_delta_us = current.aggregate_p95_us as i64 - baseline.aggregate_p95_us as i64;
    let aggregate_p99_delta_us = current.aggregate_p99_us as i64 - baseline.aggregate_p99_us as i64;
    let aggregate_max_delta_us = current.aggregate_max_us as i64 - baseline.aggregate_max_us as i64;

    PerfComparison {
        baseline_branch: baseline.branch.clone(),
        baseline_commit: baseline.commit.clone(),
        current_commit: current_commit.to_string(),
        has_regression,
        threshold_us,
        regressions,
        aggregate_p50_delta_us,
        aggregate_p95_delta_us,
        aggregate_p99_delta_us,
        aggregate_max_delta_us,
    }
}

/// Create baseline data from performance results.
///
/// Converts `PerfResults` into `BaselineData` suitable for saving.
///
/// # Arguments
/// * `results` - The performance results to convert
/// * `branch` - The current branch name
/// * `commit` - The current commit hash
///
/// # Returns
/// A `BaselineData` struct ready to be serialized and saved.
pub fn create_baseline_data(results: &PerfResults, branch: String, commit: String) -> BaselineData {
    use std::collections::HashMap;

    let mut tests = HashMap::new();
    for result in &results.results {
        tests.insert(
            result.test_name.clone(),
            BaselineTestData {
                test_name: result.test_name.clone(),
                p50_us: result.p50_us,
                p95_us: result.p95_us,
                p99_us: result.p99_us,
                max_us: result.max_us,
            },
        );
    }

    BaselineData {
        branch,
        commit,
        captured_at: chrono::Utc::now().to_rfc3339(),
        tests,
        aggregate_p50_us: results.aggregate_p50_us,
        aggregate_p95_us: results.aggregate_p95_us,
        aggregate_p99_us: results.aggregate_p99_us,
        aggregate_max_us: results.aggregate_max_us,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::uat::perf_types::PerformanceResult;

    #[test]
    fn compare_results_detects_regression() {
        let mut tests = HashMap::new();
        tests.insert(
            "test1".to_string(),
            BaselineTestData {
                test_name: "test1".to_string(),
                p50_us: 100,
                p95_us: 150,
                p99_us: 200,
                max_us: 250,
            },
        );

        let baseline = BaselineData {
            branch: "main".to_string(),
            commit: "abc123".to_string(),
            captured_at: "2025-01-01T00:00:00Z".to_string(),
            tests,
            aggregate_p50_us: 100,
            aggregate_p95_us: 150,
            aggregate_p99_us: 200,
            aggregate_max_us: 250,
        };

        // Current results with regression (p50 increased by 150µs)
        let current = PerfResults {
            total: 1,
            passed: 1,
            failed: 0,
            aggregate_p50_us: 250,
            aggregate_p95_us: 300,
            aggregate_p99_us: 350,
            aggregate_max_us: 400,
            total_duration_us: 1000,
            results: vec![PerformanceResult {
                test_name: "test1".to_string(),
                test_file: "test.rhai".to_string(),
                p50_us: 250,
                p95_us: 300,
                p99_us: 350,
                max_us: 400,
                min_us: 50,
                iterations: 100,
                threshold_us: Some(1000),
                threshold_exceeded: false,
                violations: vec![],
            }],
            all_violations: vec![],
        };

        let comparison = compare_results(&baseline, &current, "def456", 100);

        assert!(comparison.has_regression);
        assert_eq!(comparison.regressions.len(), 1);
        assert_eq!(comparison.regressions[0].test_name, "test1");
        assert_eq!(comparison.regressions[0].baseline_p50_us, 100);
        assert_eq!(comparison.regressions[0].current_p50_us, 250);
        assert_eq!(comparison.regressions[0].delta_us, 150);
        assert!(comparison.regressions[0].exceeds_threshold);
    }

    #[test]
    fn compare_results_no_regression_within_threshold() {
        let mut tests = HashMap::new();
        tests.insert(
            "test1".to_string(),
            BaselineTestData {
                test_name: "test1".to_string(),
                p50_us: 100,
                p95_us: 150,
                p99_us: 200,
                max_us: 250,
            },
        );

        let baseline = BaselineData {
            branch: "main".to_string(),
            commit: "abc123".to_string(),
            captured_at: "2025-01-01T00:00:00Z".to_string(),
            tests,
            aggregate_p50_us: 100,
            aggregate_p95_us: 150,
            aggregate_p99_us: 200,
            aggregate_max_us: 250,
        };

        // Current results with small increase (50µs, within 100µs threshold)
        let current = PerfResults {
            total: 1,
            passed: 1,
            failed: 0,
            aggregate_p50_us: 150,
            aggregate_p95_us: 200,
            aggregate_p99_us: 250,
            aggregate_max_us: 300,
            total_duration_us: 1000,
            results: vec![PerformanceResult {
                test_name: "test1".to_string(),
                test_file: "test.rhai".to_string(),
                p50_us: 150,
                p95_us: 200,
                p99_us: 250,
                max_us: 300,
                min_us: 50,
                iterations: 100,
                threshold_us: Some(1000),
                threshold_exceeded: false,
                violations: vec![],
            }],
            all_violations: vec![],
        };

        let comparison = compare_results(&baseline, &current, "def456", 100);

        // Has regression entry but doesn't exceed threshold
        assert!(!comparison.has_regression);
        assert_eq!(comparison.regressions.len(), 1);
        assert!(!comparison.regressions[0].exceeds_threshold);
        assert_eq!(comparison.regressions[0].delta_us, 50);
    }

    #[test]
    fn compare_results_improvement_no_regression() {
        let mut tests = HashMap::new();
        tests.insert(
            "test1".to_string(),
            BaselineTestData {
                test_name: "test1".to_string(),
                p50_us: 200,
                p95_us: 250,
                p99_us: 300,
                max_us: 350,
            },
        );

        let baseline = BaselineData {
            branch: "main".to_string(),
            commit: "abc123".to_string(),
            captured_at: "2025-01-01T00:00:00Z".to_string(),
            tests,
            aggregate_p50_us: 200,
            aggregate_p95_us: 250,
            aggregate_p99_us: 300,
            aggregate_max_us: 350,
        };

        // Current results with improvement (faster)
        let current = PerfResults {
            total: 1,
            passed: 1,
            failed: 0,
            aggregate_p50_us: 100,
            aggregate_p95_us: 150,
            aggregate_p99_us: 200,
            aggregate_max_us: 250,
            total_duration_us: 1000,
            results: vec![PerformanceResult {
                test_name: "test1".to_string(),
                test_file: "test.rhai".to_string(),
                p50_us: 100,
                p95_us: 150,
                p99_us: 200,
                max_us: 250,
                min_us: 50,
                iterations: 100,
                threshold_us: Some(1000),
                threshold_exceeded: false,
                violations: vec![],
            }],
            all_violations: vec![],
        };

        let comparison = compare_results(&baseline, &current, "def456", 100);

        // Performance improved, no regressions
        assert!(!comparison.has_regression);
        assert!(comparison.regressions.is_empty());
        assert_eq!(comparison.aggregate_p50_delta_us, -100);
    }

    #[test]
    fn compare_results_handles_new_tests() {
        // Baseline with one test
        let mut tests = HashMap::new();
        tests.insert(
            "test1".to_string(),
            BaselineTestData {
                test_name: "test1".to_string(),
                p50_us: 100,
                p95_us: 150,
                p99_us: 200,
                max_us: 250,
            },
        );

        let baseline = BaselineData {
            branch: "main".to_string(),
            commit: "abc123".to_string(),
            captured_at: "2025-01-01T00:00:00Z".to_string(),
            tests,
            aggregate_p50_us: 100,
            aggregate_p95_us: 150,
            aggregate_p99_us: 200,
            aggregate_max_us: 250,
        };

        // Current has a new test not in baseline
        let current = PerfResults {
            total: 2,
            passed: 2,
            failed: 0,
            aggregate_p50_us: 125,
            aggregate_p95_us: 175,
            aggregate_p99_us: 225,
            aggregate_max_us: 275,
            total_duration_us: 1000,
            results: vec![
                PerformanceResult {
                    test_name: "test1".to_string(),
                    test_file: "test.rhai".to_string(),
                    p50_us: 100,
                    p95_us: 150,
                    p99_us: 200,
                    max_us: 250,
                    min_us: 50,
                    iterations: 100,
                    threshold_us: Some(1000),
                    threshold_exceeded: false,
                    violations: vec![],
                },
                PerformanceResult {
                    test_name: "test2_new".to_string(),
                    test_file: "test2.rhai".to_string(),
                    p50_us: 150,
                    p95_us: 200,
                    p99_us: 250,
                    max_us: 300,
                    min_us: 100,
                    iterations: 100,
                    threshold_us: Some(1000),
                    threshold_exceeded: false,
                    violations: vec![],
                },
            ],
            all_violations: vec![],
        };

        let comparison = compare_results(&baseline, &current, "def456", 100);

        // No regression because new test isn't in baseline (nothing to compare against)
        assert!(!comparison.has_regression);
    }

    #[test]
    fn create_baseline_data_from_results() {
        let results = PerfResults {
            total: 1,
            passed: 1,
            failed: 0,
            aggregate_p50_us: 100,
            aggregate_p95_us: 150,
            aggregate_p99_us: 200,
            aggregate_max_us: 250,
            total_duration_us: 1000,
            results: vec![PerformanceResult {
                test_name: "test1".to_string(),
                test_file: "test.rhai".to_string(),
                p50_us: 100,
                p95_us: 150,
                p99_us: 200,
                max_us: 250,
                min_us: 50,
                iterations: 100,
                threshold_us: Some(1000),
                threshold_exceeded: false,
                violations: vec![],
            }],
            all_violations: vec![],
        };

        let baseline =
            create_baseline_data(&results, "feature-branch".to_string(), "abc123".to_string());

        assert_eq!(baseline.branch, "feature-branch");
        assert_eq!(baseline.commit, "abc123");
        assert_eq!(baseline.aggregate_p50_us, 100);
        assert_eq!(baseline.tests.len(), 1);
        assert_eq!(baseline.tests["test1"].p50_us, 100);
    }
}
