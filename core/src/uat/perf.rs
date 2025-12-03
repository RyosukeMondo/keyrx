//! Performance UAT testing with latency measurement.
//!
//! This module provides latency measurement and threshold enforcement for UAT tests.
//! It measures p50, p95, p99, and max latencies and detects threshold violations.
//! It also supports baseline comparison for regression detection.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use rhai::Engine;
use serde::{Deserialize, Serialize};

use super::runner::{UatFilter, UatTest};

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

/// Default regression threshold in microseconds (100µs).
const DEFAULT_REGRESSION_THRESHOLD_US: u64 = 100;

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

/// Number of iterations to run for each performance test.
const DEFAULT_ITERATIONS: usize = 100;

/// Default baseline file path relative to project root.
const BASELINE_FILE: &str = "target/perf-baseline.json";

/// Performance UAT runner.
///
/// Executes tests with `@latency` thresholds and measures p50/p95/p99/max latencies.
#[derive(Debug)]
pub struct PerformanceUat {
    /// Directory containing UAT test scripts.
    test_dir: PathBuf,
    /// Number of iterations to run per test.
    iterations: usize,
}

impl PerformanceUat {
    /// Create a new performance UAT runner with default settings.
    pub fn new() -> Self {
        Self {
            test_dir: PathBuf::from("tests/uat"),
            iterations: DEFAULT_ITERATIONS,
        }
    }

    /// Create a new performance UAT runner with a custom test directory.
    pub fn with_test_dir(test_dir: impl Into<PathBuf>) -> Self {
        Self {
            test_dir: test_dir.into(),
            iterations: DEFAULT_ITERATIONS,
        }
    }

    /// Set the number of iterations per test.
    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    /// Run performance UAT tests.
    ///
    /// Only runs tests that have a `@latency` threshold defined.
    /// Measures latencies over multiple iterations and reports percentiles.
    ///
    /// # Arguments
    /// * `filter` - Filter to select which tests to run
    ///
    /// # Returns
    /// Aggregated performance results with latency metrics.
    pub fn run(&self, filter: &UatFilter) -> PerfResults {
        let start_time = std::time::Instant::now();
        let tests = self.discover_perf_tests(filter);

        if tests.is_empty() {
            tracing::info!(
                service = "keyrx",
                event = "perf_uat_no_tests",
                component = "performance_uat",
                "No performance tests found (tests with @latency threshold)"
            );
            return PerfResults::default();
        }

        tracing::info!(
            service = "keyrx",
            event = "perf_uat_start",
            component = "performance_uat",
            test_count = tests.len(),
            iterations = self.iterations,
            "Starting performance UAT with {} tests, {} iterations each",
            tests.len(),
            self.iterations
        );

        let mut results = Vec::with_capacity(tests.len());
        let mut all_latencies = Vec::new();
        let mut all_violations = Vec::new();
        let mut passed = 0;
        let mut failed = 0;

        for test in &tests {
            let result = self.run_perf_test(test);

            // Collect all latencies for aggregate percentiles
            // Use the test's latencies indirectly via iterations
            all_latencies.push(result.p50_us);

            if result.threshold_exceeded {
                failed += 1;
                all_violations.extend(result.violations.clone());
            } else {
                passed += 1;
            }

            results.push(result);
        }

        // Calculate aggregate percentiles from all test p50s
        let aggregate_percentiles = if !results.is_empty() {
            let mut all_p50s: Vec<u64> = results.iter().map(|r| r.p50_us).collect();
            let mut all_p95s: Vec<u64> = results.iter().map(|r| r.p95_us).collect();
            let mut all_p99s: Vec<u64> = results.iter().map(|r| r.p99_us).collect();
            let mut all_maxs: Vec<u64> = results.iter().map(|r| r.max_us).collect();

            all_p50s.sort_unstable();
            all_p95s.sort_unstable();
            all_p99s.sort_unstable();
            all_maxs.sort_unstable();

            let len = results.len();
            (
                all_p50s[len / 2],
                all_p95s[(len as f64 * 0.95) as usize].min(*all_p95s.last().unwrap_or(&0)),
                all_p99s[(len as f64 * 0.99) as usize].min(*all_p99s.last().unwrap_or(&0)),
                *all_maxs.last().unwrap_or(&0),
            )
        } else {
            (0, 0, 0, 0)
        };

        let total_duration_us = start_time.elapsed().as_micros() as u64;

        tracing::info!(
            service = "keyrx",
            event = "perf_uat_complete",
            component = "performance_uat",
            total = tests.len(),
            passed = passed,
            failed = failed,
            p50_us = aggregate_percentiles.0,
            p95_us = aggregate_percentiles.1,
            p99_us = aggregate_percentiles.2,
            max_us = aggregate_percentiles.3,
            duration_us = total_duration_us,
            "Performance UAT complete: {} passed, {} failed",
            passed,
            failed
        );

        PerfResults {
            total: tests.len(),
            passed,
            failed,
            aggregate_p50_us: aggregate_percentiles.0,
            aggregate_p95_us: aggregate_percentiles.1,
            aggregate_p99_us: aggregate_percentiles.2,
            aggregate_max_us: aggregate_percentiles.3,
            total_duration_us,
            results,
            all_violations,
        }
    }

    /// Discover tests that have a latency threshold defined.
    fn discover_perf_tests(&self, filter: &UatFilter) -> Vec<UatTest> {
        let runner = super::runner::UatRunner::with_test_dir(&self.test_dir);
        let all_tests = runner.discover();

        // Filter to only tests with latency thresholds
        all_tests
            .into_iter()
            .filter(|t| t.latency_threshold.is_some())
            .filter(|t| filter.matches(t))
            .collect()
    }

    /// Run a single performance test with multiple iterations.
    fn run_perf_test(&self, test: &UatTest) -> PerformanceResult {
        tracing::debug!(
            service = "keyrx",
            event = "perf_test_start",
            component = "performance_uat",
            test_name = %test.name,
            test_file = %test.file,
            threshold_us = ?test.latency_threshold,
            iterations = self.iterations,
            "Running performance test"
        );

        let threshold_us = test.latency_threshold;
        let mut latencies = Vec::with_capacity(self.iterations);
        let mut violations = Vec::new();

        // Read test file content once
        let content = match fs::read_to_string(&test.file) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    service = "keyrx",
                    event = "perf_test_read_error",
                    component = "performance_uat",
                    test_name = %test.name,
                    error = %e,
                    "Failed to read test file"
                );
                return PerformanceResult {
                    test_name: test.name.clone(),
                    test_file: test.file.clone(),
                    p50_us: 0,
                    p95_us: 0,
                    p99_us: 0,
                    max_us: 0,
                    min_us: 0,
                    iterations: 0,
                    threshold_us,
                    threshold_exceeded: true,
                    violations: vec![LatencyViolation {
                        test_name: test.name.clone(),
                        threshold_us: threshold_us.unwrap_or(0),
                        actual_us: 0,
                        iteration: 0,
                    }],
                };
            }
        };

        // Create Rhai engine
        let engine = Engine::new();

        // Compile once, run multiple times
        let ast = match engine.compile(&content) {
            Ok(ast) => ast,
            Err(e) => {
                tracing::warn!(
                    service = "keyrx",
                    event = "perf_test_compile_error",
                    component = "performance_uat",
                    test_name = %test.name,
                    error = %e,
                    "Failed to compile test file"
                );
                return PerformanceResult {
                    test_name: test.name.clone(),
                    test_file: test.file.clone(),
                    p50_us: 0,
                    p95_us: 0,
                    p99_us: 0,
                    max_us: 0,
                    min_us: 0,
                    iterations: 0,
                    threshold_us,
                    threshold_exceeded: true,
                    violations: vec![LatencyViolation {
                        test_name: test.name.clone(),
                        threshold_us: threshold_us.unwrap_or(0),
                        actual_us: 0,
                        iteration: 0,
                    }],
                };
            }
        };

        // Run the script once to define functions
        if let Err(e) = engine.run_ast(&ast) {
            tracing::warn!(
                service = "keyrx",
                event = "perf_test_run_error",
                component = "performance_uat",
                test_name = %test.name,
                error = %e,
                "Failed to run test script"
            );
            return PerformanceResult {
                test_name: test.name.clone(),
                test_file: test.file.clone(),
                p50_us: 0,
                p95_us: 0,
                p99_us: 0,
                max_us: 0,
                min_us: 0,
                iterations: 0,
                threshold_us,
                threshold_exceeded: true,
                violations: vec![LatencyViolation {
                    test_name: test.name.clone(),
                    threshold_us: threshold_us.unwrap_or(0),
                    actual_us: 0,
                    iteration: 0,
                }],
            };
        }

        // Run iterations
        for i in 0..self.iterations {
            let start = std::time::Instant::now();

            let result = engine.call_fn::<()>(&mut rhai::Scope::new(), &ast, &test.name, ());

            let duration_us = start.elapsed().as_micros() as u64;
            latencies.push(duration_us);

            if let Err(e) = result {
                tracing::debug!(
                    service = "keyrx",
                    event = "perf_test_iteration_error",
                    component = "performance_uat",
                    test_name = %test.name,
                    iteration = i,
                    error = %e,
                    "Test iteration failed"
                );
                // Record as a violation if there's a threshold
                if let Some(threshold) = threshold_us {
                    violations.push(LatencyViolation {
                        test_name: test.name.clone(),
                        threshold_us: threshold,
                        actual_us: duration_us,
                        iteration: i,
                    });
                }
                continue;
            }

            // Check threshold violation
            if let Some(threshold) = threshold_us {
                if duration_us > threshold {
                    violations.push(LatencyViolation {
                        test_name: test.name.clone(),
                        threshold_us: threshold,
                        actual_us: duration_us,
                        iteration: i,
                    });
                }
            }
        }

        // Calculate percentiles
        let percentiles = LatencyPercentiles::from_samples(&mut latencies);

        let threshold_exceeded = !violations.is_empty();

        tracing::debug!(
            service = "keyrx",
            event = "perf_test_complete",
            component = "performance_uat",
            test_name = %test.name,
            p50_us = percentiles.p50_us,
            p95_us = percentiles.p95_us,
            p99_us = percentiles.p99_us,
            max_us = percentiles.max_us,
            iterations = self.iterations,
            violations = violations.len(),
            "Performance test complete"
        );

        PerformanceResult {
            test_name: test.name.clone(),
            test_file: test.file.clone(),
            p50_us: percentiles.p50_us,
            p95_us: percentiles.p95_us,
            p99_us: percentiles.p99_us,
            max_us: percentiles.max_us,
            min_us: percentiles.min_us,
            iterations: self.iterations,
            threshold_us,
            threshold_exceeded,
            violations,
        }
    }

    /// Compare current performance against a baseline from another branch.
    ///
    /// Fetches baseline performance data from the specified branch (typically `main`)
    /// and compares it against current test results. Fails if any test shows a
    /// regression greater than the specified threshold (default: 100µs).
    ///
    /// # Arguments
    /// * `branch` - The branch name to compare against (e.g., "main")
    ///
    /// # Returns
    /// A `PerfComparison` with detailed regression information.
    pub fn compare_baseline(&self, branch: &str) -> Result<PerfComparison, BaselineError> {
        self.compare_baseline_with_threshold(branch, DEFAULT_REGRESSION_THRESHOLD_US)
    }

    /// Compare with a custom regression threshold.
    pub fn compare_baseline_with_threshold(
        &self,
        branch: &str,
        threshold_us: u64,
    ) -> Result<PerfComparison, BaselineError> {
        tracing::info!(
            service = "keyrx",
            event = "baseline_compare_start",
            component = "performance_uat",
            branch = %branch,
            threshold_us = threshold_us,
            "Starting baseline comparison"
        );

        // Get current branch name to ensure we're not comparing against ourselves
        let current_branch = Self::get_current_branch()?;
        if current_branch == branch {
            tracing::warn!(
                service = "keyrx",
                event = "baseline_same_branch",
                component = "performance_uat",
                branch = %branch,
                "Cannot compare against the same branch"
            );
            return Err(BaselineError::SameBranch);
        }

        // Get current commit hash
        let current_commit = Self::get_current_commit()?;

        // Fetch baseline data from target branch
        let baseline = Self::fetch_baseline_from_branch(branch)?;

        // Run current performance tests
        let current_results = self.run(&UatFilter::default());

        // Compare results
        let comparison =
            Self::compare_results(&baseline, &current_results, &current_commit, threshold_us);

        if comparison.has_regression {
            tracing::warn!(
                service = "keyrx",
                event = "baseline_regression_detected",
                component = "performance_uat",
                branch = %branch,
                regression_count = comparison.regressions.len(),
                threshold_us = threshold_us,
                "Performance regressions detected"
            );
        } else {
            tracing::info!(
                service = "keyrx",
                event = "baseline_compare_pass",
                component = "performance_uat",
                branch = %branch,
                "No performance regressions detected"
            );
        }

        Ok(comparison)
    }

    /// Get the current Git branch name.
    fn get_current_branch() -> Result<String, BaselineError> {
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
    fn get_current_commit() -> Result<String, BaselineError> {
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
    fn fetch_baseline_from_branch(branch: &str) -> Result<BaselineData, BaselineError> {
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
    fn compare_results(
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
        let aggregate_p50_delta_us =
            current.aggregate_p50_us as i64 - baseline.aggregate_p50_us as i64;
        let aggregate_p95_delta_us =
            current.aggregate_p95_us as i64 - baseline.aggregate_p95_us as i64;
        let aggregate_p99_delta_us =
            current.aggregate_p99_us as i64 - baseline.aggregate_p99_us as i64;
        let aggregate_max_delta_us =
            current.aggregate_max_us as i64 - baseline.aggregate_max_us as i64;

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

    /// Save current performance results as a baseline.
    ///
    /// This should be called on the main branch after tests pass to establish
    /// the baseline for future comparisons.
    pub fn save_baseline(&self, results: &PerfResults) -> Result<(), BaselineError> {
        let branch = Self::get_current_branch()?;
        let commit = Self::get_current_commit()?;

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

        let baseline = BaselineData {
            branch,
            commit,
            captured_at: chrono::Utc::now().to_rfc3339(),
            tests,
            aggregate_p50_us: results.aggregate_p50_us,
            aggregate_p95_us: results.aggregate_p95_us,
            aggregate_p99_us: results.aggregate_p99_us,
            aggregate_max_us: results.aggregate_max_us,
        };

        // Ensure target directory exists
        if let Some(parent) = PathBuf::from(BASELINE_FILE).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| BaselineError::GitError(format!("Failed to create directory: {e}")))?;
        }

        let json = serde_json::to_string_pretty(&baseline)
            .map_err(|e| BaselineError::ParseError(format!("Failed to serialize baseline: {e}")))?;

        fs::write(BASELINE_FILE, json)
            .map_err(|e| BaselineError::GitError(format!("Failed to write baseline file: {e}")))?;

        tracing::info!(
            service = "keyrx",
            event = "baseline_saved",
            component = "performance_uat",
            path = %BASELINE_FILE,
            "Baseline saved successfully"
        );

        Ok(())
    }
}

impl Default for PerformanceUat {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
    fn performance_uat_new_defaults() {
        let perf = PerformanceUat::new();
        assert_eq!(perf.test_dir, PathBuf::from("tests/uat"));
        assert_eq!(perf.iterations, DEFAULT_ITERATIONS);
    }

    #[test]
    fn performance_uat_with_custom_dir() {
        let perf = PerformanceUat::with_test_dir("/custom/path");
        assert_eq!(perf.test_dir, PathBuf::from("/custom/path"));
    }

    #[test]
    fn performance_uat_with_iterations() {
        let perf = PerformanceUat::new().with_iterations(50);
        assert_eq!(perf.iterations, 50);
    }

    #[test]
    fn performance_uat_returns_empty_when_no_tests() {
        let temp_dir = TempDir::new().unwrap();
        let perf = PerformanceUat::with_test_dir(temp_dir.path());
        let results = perf.run(&UatFilter::default());

        assert_eq!(results.total, 0);
        assert_eq!(results.passed, 0);
        assert_eq!(results.failed, 0);
    }

    #[test]
    fn performance_uat_only_runs_tests_with_latency() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        // One test with latency, one without
        let script = r#"
// @latency: 100000
fn uat_with_latency() {
    let x = 1;
}

fn uat_without_latency() {
    let y = 2;
}
"#;
        fs::write(&test_file, script).unwrap();

        let perf = PerformanceUat::with_test_dir(temp_dir.path()).with_iterations(5);
        let results = perf.run(&UatFilter::default());

        // Only the test with @latency should run
        assert_eq!(results.total, 1);
        assert_eq!(results.results[0].test_name, "uat_with_latency");
    }

    #[test]
    fn performance_uat_detects_threshold_violations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        // Test with very low threshold that will be exceeded
        let script = r#"
// @latency: 1
fn uat_slow_test() {
    // Do some work that takes more than 1 microsecond
    let sum = 0;
    for i in 0..100 {
        sum += i;
    }
}
"#;
        fs::write(&test_file, script).unwrap();

        let perf = PerformanceUat::with_test_dir(temp_dir.path()).with_iterations(10);
        let results = perf.run(&UatFilter::default());

        assert_eq!(results.total, 1);
        // The test should fail because it exceeds 1µs threshold
        assert_eq!(results.failed, 1);
        assert!(results.results[0].threshold_exceeded);
        assert!(!results.all_violations.is_empty());
    }

    #[test]
    fn performance_uat_passes_when_within_threshold() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        // Test with very high threshold
        let script = r#"
// @latency: 10000000
fn uat_fast_test() {
    let x = 1 + 1;
}
"#;
        fs::write(&test_file, script).unwrap();

        let perf = PerformanceUat::with_test_dir(temp_dir.path()).with_iterations(5);
        let results = perf.run(&UatFilter::default());

        assert_eq!(results.total, 1);
        assert_eq!(results.passed, 1);
        assert_eq!(results.failed, 0);
        assert!(!results.results[0].threshold_exceeded);
        assert!(results.all_violations.is_empty());
    }

    #[test]
    fn performance_uat_reports_percentiles() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        // Use a script that does some actual work to ensure measurable time
        let script = r#"
// @latency: 10000000
fn uat_perf_test() {
    // Do some work to ensure measurable execution time
    let sum = 0;
    for i in 0..50 {
        sum += i;
    }
}
"#;
        fs::write(&test_file, script).unwrap();

        let perf = PerformanceUat::with_test_dir(temp_dir.path()).with_iterations(20);
        let results = perf.run(&UatFilter::default());

        assert_eq!(results.total, 1);
        let result = &results.results[0];

        // Verify iterations were run
        assert_eq!(result.iterations, 20);

        // Percentiles should be calculated (may be 0 for very fast operations)
        // Just verify the structure is populated
        assert!(result.sample_count_valid());

        // min <= p50 <= p95 <= p99 <= max should always hold
        assert!(result.min_us <= result.p50_us || result.p50_us == 0);
        assert!(result.p50_us <= result.p95_us || result.p95_us == 0);
        assert!(result.p95_us <= result.p99_us || result.p99_us == 0);
        assert!(result.p99_us <= result.max_us || result.max_us == 0);
    }

    #[test]
    fn performance_uat_applies_filter() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
// @category: core
// @latency: 10000000
fn uat_core_perf() {
    let x = 1;
}

// @category: layers
// @latency: 10000000
fn uat_layer_perf() {
    let y = 2;
}
"#;
        fs::write(&test_file, script).unwrap();

        let perf = PerformanceUat::with_test_dir(temp_dir.path()).with_iterations(5);
        let filter = UatFilter {
            categories: vec!["core".to_string()],
            ..Default::default()
        };
        let results = perf.run(&filter);

        assert_eq!(results.total, 1);
        assert_eq!(results.results[0].test_name, "uat_core_perf");
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

    // Baseline comparison tests

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

        let comparison = PerformanceUat::compare_results(&baseline, &current, "def456", 100);

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

        let comparison = PerformanceUat::compare_results(&baseline, &current, "def456", 100);

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

        let comparison = PerformanceUat::compare_results(&baseline, &current, "def456", 100);

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

        let comparison = PerformanceUat::compare_results(&baseline, &current, "def456", 100);

        // No regression because new test isn't in baseline (nothing to compare against)
        assert!(!comparison.has_regression);
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
