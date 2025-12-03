//! UAT test runner with discovery, filtering, and execution.

use std::path::PathBuf;

use super::runner_discovery::{collect_rhai_files, discover_tests_in_file};
use super::runner_execution::execute_test;
pub use super::runner_types::{Priority, UatFilter, UatResult, UatResults, UatTest};

/// UAT test runner.
#[derive(Debug)]
pub struct UatRunner {
    /// Directory containing UAT test scripts.
    test_dir: PathBuf,
}

impl UatRunner {
    /// Create a new UAT runner with the default test directory (`tests/uat/`).
    pub fn new() -> Self {
        Self {
            test_dir: PathBuf::from("tests/uat"),
        }
    }

    /// Create a new UAT runner with a custom test directory.
    pub fn with_test_dir(test_dir: impl Into<PathBuf>) -> Self {
        Self {
            test_dir: test_dir.into(),
        }
    }

    /// Discover all UAT tests in the test directory.
    ///
    /// Scans all `.rhai` files in `tests/uat/` and its subdirectories,
    /// finding functions with the `uat_` prefix and parsing their metadata.
    ///
    /// # Returns
    /// A vector of discovered UAT tests, sorted by file path and name.
    pub fn discover(&self) -> Vec<UatTest> {
        let mut tests = Vec::new();

        if !self.test_dir.exists() {
            tracing::debug!(
                service = "keyrx",
                event = "uat_no_test_dir",
                component = "uat_runner",
                path = %self.test_dir.display(),
                "UAT test directory does not exist"
            );
            return tests;
        }

        // Collect all .rhai files recursively
        let rhai_files = collect_rhai_files(&self.test_dir);

        for file_path in rhai_files {
            match discover_tests_in_file(&file_path) {
                Ok(file_tests) => {
                    tests.extend(file_tests);
                }
                Err(e) => {
                    tracing::warn!(
                        service = "keyrx",
                        event = "uat_discovery_error",
                        component = "uat_runner",
                        file = %file_path.display(),
                        error = %e,
                        "Failed to discover tests in file"
                    );
                }
            }
        }

        // Sort by file path, then by name for consistent ordering
        tests.sort_by(|a, b| (&a.file, &a.name).cmp(&(&b.file, &b.name)));

        tracing::info!(
            service = "keyrx",
            event = "uat_discovery_complete",
            component = "uat_runner",
            test_count = tests.len(),
            "Discovered {} UAT tests",
            tests.len()
        );

        tests
    }

    /// Run UAT tests with the given filter.
    ///
    /// Discovers tests, applies the filter, and executes matching tests.
    ///
    /// # Arguments
    /// * `filter` - Filter to select which tests to run
    ///
    /// # Returns
    /// Aggregated results from the test run.
    pub fn run(&self, filter: &UatFilter) -> UatResults {
        self.run_internal(filter, false)
    }

    /// Run UAT tests with fail-fast mode.
    ///
    /// Stops execution on the first test failure.
    ///
    /// # Arguments
    /// * `filter` - Filter to select which tests to run
    ///
    /// # Returns
    /// Aggregated results from the test run.
    pub fn run_fail_fast(&self, filter: &UatFilter) -> UatResults {
        self.run_internal(filter, true)
    }

    /// Internal implementation of test execution.
    fn run_internal(&self, filter: &UatFilter, fail_fast: bool) -> UatResults {
        let start_time = std::time::Instant::now();
        let discovered = self.discover();

        // Apply filter
        let tests_to_run: Vec<_> = discovered
            .into_iter()
            .filter(|t| filter.matches(t))
            .collect();

        let total = tests_to_run.len();
        let mut results = Vec::with_capacity(total);
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        tracing::info!(
            service = "keyrx",
            event = "uat_run_start",
            component = "uat_runner",
            test_count = total,
            fail_fast = fail_fast,
            "Starting UAT run with {} tests",
            total
        );

        for test in tests_to_run {
            // Skip remaining tests in fail-fast mode after a failure
            if fail_fast && failed > 0 {
                skipped += 1;
                results.push(UatResult {
                    test,
                    passed: false,
                    duration_us: 0,
                    error: Some("Skipped due to fail-fast mode".to_string()),
                });
                continue;
            }

            let result = execute_test(&test);

            if result.passed {
                passed += 1;
            } else {
                failed += 1;
            }

            results.push(result);
        }

        let duration_us = start_time.elapsed().as_micros() as u64;

        tracing::info!(
            service = "keyrx",
            event = "uat_run_complete",
            component = "uat_runner",
            total = total,
            passed = passed,
            failed = failed,
            skipped = skipped,
            duration_us = duration_us,
            "UAT run complete: {} passed, {} failed, {} skipped",
            passed,
            failed,
            skipped
        );

        UatResults {
            total,
            passed,
            failed,
            skipped,
            duration_us,
            results,
        }
    }
}

impl Default for UatRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn discover_returns_empty_when_dir_missing() {
        let runner = UatRunner::with_test_dir("/nonexistent/path");
        let tests = runner.discover();
        assert!(tests.is_empty());
    }

    #[test]
    fn discover_finds_uat_functions() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
// @category: core
// @priority: P0
// @requirement: 1.1, 1.2
// @latency: 1000
fn uat_basic_test() {
    let x = 1;
}

fn helper() {
    // not a UAT test
}

// @category: layers
// @priority: P1
fn uat_layer_test() {
    let y = 2;
}
"#;
        fs::write(&test_file, script).unwrap();

        let runner = UatRunner::with_test_dir(temp_dir.path());
        let tests = runner.discover();

        assert_eq!(tests.len(), 2);

        // Check first test (sorted alphabetically by name)
        let basic = tests.iter().find(|t| t.name == "uat_basic_test").unwrap();
        assert_eq!(basic.category, "core");
        assert_eq!(basic.priority, Priority::P0);
        assert_eq!(basic.requirements, vec!["1.1", "1.2"]);
        assert_eq!(basic.latency_threshold, Some(1000));

        // Check second test
        let layer = tests.iter().find(|t| t.name == "uat_layer_test").unwrap();
        assert_eq!(layer.category, "layers");
        assert_eq!(layer.priority, Priority::P1);
        assert!(layer.requirements.is_empty());
        assert_eq!(layer.latency_threshold, None);
    }

    #[test]
    fn discover_handles_subdirectories() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        let test_file = sub_dir.join("nested.rhai");
        let script = r#"
fn uat_nested() {
    let x = 1;
}
"#;
        fs::write(&test_file, script).unwrap();

        let runner = UatRunner::with_test_dir(temp_dir.path());
        let tests = runner.discover();

        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "uat_nested");
        assert!(tests[0].file.contains("nested.rhai"));
    }

    #[test]
    fn discover_ignores_non_uat_functions() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
fn test_something() { }
fn helper_function() { }
fn uat_real_test() { }
"#;
        fs::write(&test_file, script).unwrap();

        let runner = UatRunner::with_test_dir(temp_dir.path());
        let tests = runner.discover();

        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "uat_real_test");
    }

    #[test]
    fn discover_uses_defaults_when_no_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
fn uat_no_metadata() {
    let x = 1;
}
"#;
        fs::write(&test_file, script).unwrap();

        let runner = UatRunner::with_test_dir(temp_dir.path());
        let tests = runner.discover();

        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].category, "default");
        assert_eq!(tests[0].priority, Priority::P2);
        assert!(tests[0].requirements.is_empty());
        assert_eq!(tests[0].latency_threshold, None);
    }

    #[test]
    fn uat_runner_default_test_dir() {
        let runner = UatRunner::new();
        assert_eq!(runner.test_dir, PathBuf::from("tests/uat"));
    }

    #[test]
    fn uat_runner_with_custom_dir() {
        let runner = UatRunner::with_test_dir("/custom/path");
        assert_eq!(runner.test_dir, PathBuf::from("/custom/path"));
    }

    #[test]
    fn run_executes_passing_tests() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
fn uat_passing() {
    let x = 1 + 1;
    // Test passes if no error is thrown
}
"#;
        fs::write(&test_file, script).unwrap();

        let runner = UatRunner::with_test_dir(temp_dir.path());
        let results = runner.run(&UatFilter::default());

        assert_eq!(results.total, 1);
        assert_eq!(results.passed, 1);
        assert_eq!(results.failed, 0);
        assert!(results.results[0].passed);
        assert!(results.results[0].error.is_none());
    }

    #[test]
    fn run_detects_failing_tests() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
fn uat_failing() {
    throw "Test failed intentionally";
}
"#;
        fs::write(&test_file, script).unwrap();

        let runner = UatRunner::with_test_dir(temp_dir.path());
        let results = runner.run(&UatFilter::default());

        assert_eq!(results.total, 1);
        assert_eq!(results.passed, 0);
        assert_eq!(results.failed, 1);
        assert!(!results.results[0].passed);
        assert!(results.results[0].error.is_some());
    }

    #[test]
    fn run_fail_fast_skips_remaining() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        // Two tests: first fails, second would pass
        let script = r#"
fn uat_a_fails() {
    throw "First test fails";
}

fn uat_b_passes() {
    let x = 1;
}
"#;
        fs::write(&test_file, script).unwrap();

        let runner = UatRunner::with_test_dir(temp_dir.path());
        let results = runner.run_fail_fast(&UatFilter::default());

        assert_eq!(results.total, 2);
        assert_eq!(results.failed, 1);
        assert_eq!(results.skipped, 1);
        // Second test should be skipped
        assert!(results.results[1]
            .error
            .as_ref()
            .map(|e| e.contains("fail-fast"))
            .unwrap_or(false));
    }

    #[test]
    fn run_applies_filter() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
// @category: core
fn uat_core_test() {
    let x = 1;
}

// @category: layers
fn uat_layer_test() {
    let y = 2;
}
"#;
        fs::write(&test_file, script).unwrap();

        let runner = UatRunner::with_test_dir(temp_dir.path());
        let filter = UatFilter {
            categories: vec!["core".to_string()],
            ..Default::default()
        };
        let results = runner.run(&filter);

        assert_eq!(results.total, 1);
        assert_eq!(results.results[0].test.name, "uat_core_test");
    }

    #[test]
    fn run_measures_duration() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
fn uat_timed() {
    let x = 1;
}
"#;
        fs::write(&test_file, script).unwrap();

        let runner = UatRunner::with_test_dir(temp_dir.path());
        let results = runner.run(&UatFilter::default());

        assert!(results.duration_us > 0);
        assert!(results.results[0].duration_us > 0);
    }
}
