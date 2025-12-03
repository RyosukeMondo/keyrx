//! Performance UAT type definitions.
//!
//! This module contains all type definitions for performance testing,
//! including latency percentiles, performance results, and baseline comparison types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Latency percentiles collected from test execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    /// P50 (median) latency in microseconds.
    pub p50_us: u64,
    /// P95 latency in microseconds.
    pub p95_us: u64,
    /// P99 latency in microseconds.
    pub p99_us: u64,
    /// Maximum latency in microseconds.
    pub max_us: u64,
    /// Minimum latency in microseconds.
    pub min_us: u64,
    /// Number of samples.
    pub sample_count: usize,
}

impl LatencyPercentiles {
    /// Calculate percentiles from a slice of latency measurements.
    ///
    /// Uses nearest-rank method for percentile calculation.
    pub fn from_samples(samples: &mut [u64]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        samples.sort_unstable();

        let len = samples.len();

        // Nearest-rank percentile calculation:
        // Index = ceil(percentile / 100 * n) - 1 (0-based index)
        // For p50: ceil(0.5 * n) - 1
        // For p95: ceil(0.95 * n) - 1
        // For p99: ceil(0.99 * n) - 1
        let p50_idx = ((0.50 * len as f64).ceil() as usize).saturating_sub(1);
        let p95_idx = ((0.95 * len as f64).ceil() as usize).saturating_sub(1);
        let p99_idx = ((0.99 * len as f64).ceil() as usize).saturating_sub(1);

        Self {
            p50_us: samples[p50_idx.min(len - 1)],
            p95_us: samples[p95_idx.min(len - 1)],
            p99_us: samples[p99_idx.min(len - 1)],
            max_us: samples[len - 1],
            min_us: samples[0],
            sample_count: len,
        }
    }
}

/// A latency threshold violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyViolation {
    /// Test name that violated the threshold.
    pub test_name: String,
    /// Threshold in microseconds.
    pub threshold_us: u64,
    /// Actual measured latency in microseconds.
    pub actual_us: u64,
    /// Iteration index where the violation occurred.
    pub iteration: usize,
}

/// Result of a single performance test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceResult {
    /// Test name.
    pub test_name: String,
    /// Test file.
    pub test_file: String,
    /// P50 latency in microseconds.
    pub p50_us: u64,
    /// P95 latency in microseconds.
    pub p95_us: u64,
    /// P99 latency in microseconds.
    pub p99_us: u64,
    /// Maximum latency in microseconds.
    pub max_us: u64,
    /// Minimum latency in microseconds.
    pub min_us: u64,
    /// Number of iterations run.
    pub iterations: usize,
    /// Latency threshold from test metadata (if any).
    pub threshold_us: Option<u64>,
    /// Whether the threshold was exceeded.
    pub threshold_exceeded: bool,
    /// Violations (iterations that exceeded threshold).
    pub violations: Vec<LatencyViolation>,
}

impl PerformanceResult {
    /// Check if the sample count is valid (iterations > 0).
    pub fn sample_count_valid(&self) -> bool {
        self.iterations > 0
    }
}

/// Aggregated results from a performance UAT run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerfResults {
    /// Total tests run.
    pub total: usize,
    /// Tests that passed (no threshold violations).
    pub passed: usize,
    /// Tests that failed (threshold exceeded).
    pub failed: usize,
    /// Aggregate p50 latency across all tests.
    pub aggregate_p50_us: u64,
    /// Aggregate p95 latency across all tests.
    pub aggregate_p95_us: u64,
    /// Aggregate p99 latency across all tests.
    pub aggregate_p99_us: u64,
    /// Aggregate max latency across all tests.
    pub aggregate_max_us: u64,
    /// Total duration in microseconds.
    pub total_duration_us: u64,
    /// Individual test results.
    pub results: Vec<PerformanceResult>,
    /// All violations across all tests.
    pub all_violations: Vec<LatencyViolation>,
}

/// Baseline performance data for comparison.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaselineData {
    /// Branch name this baseline was captured from.
    pub branch: String,
    /// Commit hash at time of capture.
    pub commit: String,
    /// Timestamp when baseline was captured.
    pub captured_at: String,
    /// Per-test latency data.
    pub tests: HashMap<String, BaselineTestData>,
    /// Aggregate p50 latency.
    pub aggregate_p50_us: u64,
    /// Aggregate p95 latency.
    pub aggregate_p95_us: u64,
    /// Aggregate p99 latency.
    pub aggregate_p99_us: u64,
    /// Aggregate max latency.
    pub aggregate_max_us: u64,
}

/// Baseline latency data for a single test.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaselineTestData {
    /// Test name.
    pub test_name: String,
    /// P50 latency in microseconds.
    pub p50_us: u64,
    /// P95 latency in microseconds.
    pub p95_us: u64,
    /// P99 latency in microseconds.
    pub p99_us: u64,
    /// Max latency in microseconds.
    pub max_us: u64,
}

/// Result of comparing current performance against baseline.
#[derive(Debug, Clone, Default)]
pub struct PerfComparison {
    /// Baseline branch name.
    pub baseline_branch: String,
    /// Baseline commit hash.
    pub baseline_commit: String,
    /// Current commit hash.
    pub current_commit: String,
    /// Whether any regressions exceed the threshold.
    pub has_regression: bool,
    /// Regression threshold in microseconds.
    pub threshold_us: u64,
    /// Per-test regressions.
    pub regressions: Vec<BaselineRegression>,
    /// Aggregate p50 delta (current - baseline).
    pub aggregate_p50_delta_us: i64,
    /// Aggregate p95 delta.
    pub aggregate_p95_delta_us: i64,
    /// Aggregate p99 delta.
    pub aggregate_p99_delta_us: i64,
    /// Aggregate max delta.
    pub aggregate_max_delta_us: i64,
}

/// A detected performance regression against baseline.
#[derive(Debug, Clone)]
pub struct BaselineRegression {
    /// Test name.
    pub test_name: String,
    /// Baseline p50 latency in microseconds.
    pub baseline_p50_us: u64,
    /// Current p50 latency in microseconds.
    pub current_p50_us: u64,
    /// Delta (current - baseline) in microseconds.
    pub delta_us: i64,
    /// Regression threshold.
    pub threshold_us: u64,
    /// Whether this regression exceeds the threshold.
    pub exceeds_threshold: bool,
}

/// Error type for baseline comparison operations.
#[derive(Debug)]
pub enum BaselineError {
    /// Git command failed.
    GitError(String),
    /// Baseline file not found on the target branch.
    BaselineNotFound(String),
    /// Failed to parse baseline data.
    ParseError(String),
    /// Current branch is the same as baseline branch.
    SameBranch,
    /// Failed to run performance tests.
    TestError(String),
}

impl std::fmt::Display for BaselineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitError(msg) => write!(f, "Git error: {msg}"),
            Self::BaselineNotFound(branch) => {
                write!(f, "Baseline not found on branch '{branch}'")
            }
            Self::ParseError(msg) => write!(f, "Failed to parse baseline: {msg}"),
            Self::SameBranch => write!(f, "Current branch is the same as baseline branch"),
            Self::TestError(msg) => write!(f, "Test error: {msg}"),
        }
    }
}

impl std::error::Error for BaselineError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latency_percentiles_from_empty() {
        let mut samples: Vec<u64> = vec![];
        let percentiles = LatencyPercentiles::from_samples(&mut samples);
        assert_eq!(percentiles.sample_count, 0);
        assert_eq!(percentiles.p50_us, 0);
    }

    #[test]
    fn latency_percentiles_from_single() {
        let mut samples = vec![100];
        let percentiles = LatencyPercentiles::from_samples(&mut samples);
        assert_eq!(percentiles.sample_count, 1);
        assert_eq!(percentiles.p50_us, 100);
        assert_eq!(percentiles.max_us, 100);
        assert_eq!(percentiles.min_us, 100);
    }

    #[test]
    fn latency_percentiles_calculates_correctly() {
        // 100 samples: 1, 2, 3, ..., 100
        let mut samples: Vec<u64> = (1..=100).collect();
        let percentiles = LatencyPercentiles::from_samples(&mut samples);

        assert_eq!(percentiles.sample_count, 100);
        assert_eq!(percentiles.min_us, 1);
        assert_eq!(percentiles.max_us, 100);
        assert_eq!(percentiles.p50_us, 50); // median
        assert_eq!(percentiles.p95_us, 95);
        assert_eq!(percentiles.p99_us, 99);
    }

    #[test]
    fn latency_percentiles_handles_unsorted_input() {
        let mut samples = vec![50, 10, 90, 30, 70, 20, 80, 40, 60, 100];
        let percentiles = LatencyPercentiles::from_samples(&mut samples);

        assert_eq!(percentiles.min_us, 10);
        assert_eq!(percentiles.max_us, 100);
        assert_eq!(percentiles.p50_us, 50);
    }

    #[test]
    fn perf_results_default() {
        let results = PerfResults::default();
        assert_eq!(results.total, 0);
        assert_eq!(results.passed, 0);
        assert_eq!(results.failed, 0);
        assert!(results.results.is_empty());
        assert!(results.all_violations.is_empty());
    }

    #[test]
    fn baseline_data_serialization() {
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

        // Verify serialization works
        let json = serde_json::to_string(&baseline).unwrap();
        assert!(json.contains("main"));
        assert!(json.contains("abc123"));

        // Verify deserialization works
        let restored: BaselineData = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.branch, "main");
        assert_eq!(restored.commit, "abc123");
        assert_eq!(restored.tests.len(), 1);
        assert_eq!(restored.tests["test1"].p50_us, 100);
    }

    #[test]
    fn baseline_test_data_default() {
        let data = BaselineTestData::default();
        assert_eq!(data.test_name, "");
        assert_eq!(data.p50_us, 0);
        assert_eq!(data.p95_us, 0);
        assert_eq!(data.p99_us, 0);
        assert_eq!(data.max_us, 0);
    }

    #[test]
    fn perf_comparison_default() {
        let comparison = PerfComparison::default();
        assert_eq!(comparison.baseline_branch, "");
        assert!(!comparison.has_regression);
        assert!(comparison.regressions.is_empty());
    }

    #[test]
    fn baseline_error_display() {
        let err = BaselineError::GitError("command failed".to_string());
        assert!(err.to_string().contains("Git error"));

        let err = BaselineError::BaselineNotFound("main".to_string());
        assert!(err.to_string().contains("main"));

        let err = BaselineError::ParseError("invalid json".to_string());
        assert!(err.to_string().contains("parse"));

        let err = BaselineError::SameBranch;
        let msg = err.to_string().to_lowercase();
        assert!(msg.contains("same") && msg.contains("branch"));

        let err = BaselineError::TestError("test failed".to_string());
        assert!(err.to_string().contains("Test error"));
    }
}
