//! Performance test execution.
//!
//! This module handles the execution of individual performance tests,
//! including script compilation, iteration running, and latency measurement.

use std::fs;

use rhai::Engine;

use super::perf_types::{LatencyPercentiles, LatencyViolation, PerformanceResult};
use super::runner::UatTest;

/// Compile and run a performance test with multiple iterations.
///
/// This function handles the complete lifecycle of a single performance test:
/// 1. Read and compile the test script
/// 2. Execute the script to define functions
/// 3. Run the test function for the specified number of iterations
/// 4. Collect latency measurements and detect violations
///
/// # Arguments
/// * `test` - The UAT test metadata
/// * `iterations` - Number of iterations to run
///
/// # Returns
/// A `PerformanceResult` containing latency measurements and any violations.
pub fn run_perf_test(test: &UatTest, iterations: usize) -> PerformanceResult {
    tracing::debug!(
        service = "keyrx",
        event = "perf_test_start",
        component = "performance_uat",
        test_name = %test.name,
        test_file = %test.file,
        threshold_us = ?test.latency_threshold,
        iterations = iterations,
        "Running performance test"
    );

    let threshold_us = test.latency_threshold;

    // Read test file content
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
            return create_error_result(test, threshold_us, "read error");
        }
    };

    // Create Rhai engine and compile
    let engine = Engine::new();
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
            return create_error_result(test, threshold_us, "compile error");
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
        return create_error_result(test, threshold_us, "run error");
    }

    // Execute iterations and collect measurements
    let (latencies, violations) = execute_iterations(&engine, &ast, test, iterations);

    // Calculate percentiles
    let mut latencies_copy = latencies;
    let percentiles = LatencyPercentiles::from_samples(&mut latencies_copy);
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
        iterations = iterations,
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
        iterations,
        threshold_us,
        threshold_exceeded,
        violations,
    }
}

/// Execute test iterations and collect latency measurements.
fn execute_iterations(
    engine: &Engine,
    ast: &rhai::AST,
    test: &UatTest,
    iterations: usize,
) -> (Vec<u64>, Vec<LatencyViolation>) {
    let mut latencies = Vec::with_capacity(iterations);
    let mut violations = Vec::new();
    let threshold_us = test.latency_threshold;

    for i in 0..iterations {
        let start = std::time::Instant::now();
        let result = engine.call_fn::<()>(&mut rhai::Scope::new(), ast, &test.name, ());
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

    (latencies, violations)
}

/// Create an error result for a test that failed before execution.
fn create_error_result(
    test: &UatTest,
    threshold_us: Option<u64>,
    _error_type: &str,
) -> PerformanceResult {
    PerformanceResult {
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
    }
}

#[cfg(test)]
mod tests {
    use super::super::runner::Priority;
    use super::*;
    use tempfile::TempDir;

    fn create_test_script(dir: &TempDir, content: &str) -> UatTest {
        let test_file = dir.path().join("test.rhai");
        fs::write(&test_file, content).unwrap();

        UatTest {
            name: "uat_test_fn".to_string(),
            file: test_file.to_string_lossy().to_string(),
            category: String::new(),
            priority: Priority::P2,
            requirements: vec![],
            latency_threshold: Some(10_000_000), // 10ms
        }
    }

    #[test]
    fn run_perf_test_success() {
        let temp_dir = TempDir::new().unwrap();
        let script = r#"
fn uat_test_fn() {
    let x = 1 + 1;
}
"#;
        let test = create_test_script(&temp_dir, script);
        let result = run_perf_test(&test, 5);

        assert_eq!(result.test_name, "uat_test_fn");
        assert_eq!(result.iterations, 5);
        assert!(!result.threshold_exceeded);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn run_perf_test_with_threshold_violation() {
        let temp_dir = TempDir::new().unwrap();
        let script = r#"
fn uat_test_fn() {
    // Do some work
    let sum = 0;
    for i in 0..1000 {
        sum += i;
    }
}
"#;
        let test_file = temp_dir.path().join("test.rhai");
        fs::write(&test_file, script).unwrap();

        let test = UatTest {
            name: "uat_test_fn".to_string(),
            file: test_file.to_string_lossy().to_string(),
            category: String::new(),
            priority: Priority::P2,
            requirements: vec![],
            latency_threshold: Some(1), // 1µs - will definitely be exceeded
        };

        let result = run_perf_test(&test, 5);

        assert!(result.threshold_exceeded);
        assert!(!result.violations.is_empty());
    }

    #[test]
    fn run_perf_test_file_not_found() {
        let test = UatTest {
            name: "nonexistent".to_string(),
            file: "/nonexistent/path/test.rhai".to_string(),
            category: String::new(),
            priority: Priority::P2,
            requirements: vec![],
            latency_threshold: Some(1000),
        };

        let result = run_perf_test(&test, 5);

        assert!(result.threshold_exceeded);
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn run_perf_test_compile_error() {
        let temp_dir = TempDir::new().unwrap();
        let script = "fn invalid syntax {{{";
        let test_file = temp_dir.path().join("test.rhai");
        fs::write(&test_file, script).unwrap();

        let test = UatTest {
            name: "uat_test_fn".to_string(),
            file: test_file.to_string_lossy().to_string(),
            category: String::new(),
            priority: Priority::P2,
            requirements: vec![],
            latency_threshold: Some(1000),
        };

        let result = run_perf_test(&test, 5);

        assert!(result.threshold_exceeded);
        assert_eq!(result.iterations, 0);
    }
}
