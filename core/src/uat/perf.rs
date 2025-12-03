//! Performance UAT testing with latency measurement.
//!
//! This module provides latency measurement and threshold enforcement for UAT tests.
//! It measures p50, p95, p99, and max latencies and detects threshold violations.

use std::fs;
use std::path::PathBuf;

use rhai::Engine;

use super::runner::{UatFilter, UatTest};

/// Latency percentiles collected from test execution.
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Default)]
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

/// Number of iterations to run for each performance test.
const DEFAULT_ITERATIONS: usize = 100;

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
}
