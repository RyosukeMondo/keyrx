//! UAT test runner with discovery, filtering, and execution.

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use rhai::Engine;

/// Test priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Priority {
    /// Critical path tests that must always pass.
    P0,
    /// High priority tests.
    P1,
    /// Medium priority tests.
    #[default]
    P2,
}

impl FromStr for Priority {
    type Err = ();

    /// Parse priority from string (e.g., "P0", "P1", "P2").
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "P0" => Ok(Priority::P0),
            "P1" => Ok(Priority::P1),
            "P2" => Ok(Priority::P2),
            _ => Err(()),
        }
    }
}

/// A discovered UAT test.
#[derive(Debug, Clone)]
pub struct UatTest {
    /// Test name.
    pub name: String,
    /// Source file path.
    pub file: String,
    /// Test category.
    pub category: String,
    /// Test priority.
    pub priority: Priority,
    /// Linked requirement IDs.
    pub requirements: Vec<String>,
    /// Latency threshold in microseconds.
    pub latency_threshold: Option<u64>,
}

/// Filter for selecting UAT tests.
#[derive(Debug, Default, Clone)]
pub struct UatFilter {
    /// Filter by categories.
    pub categories: Vec<String>,
    /// Filter by priorities.
    pub priorities: Vec<Priority>,
    /// Filter by name pattern.
    pub pattern: Option<String>,
}

impl UatFilter {
    /// Check if a test matches this filter.
    ///
    /// All specified criteria must match (AND logic):
    /// - If categories are specified, test category must be in the list
    /// - If priorities are specified, test priority must be in the list
    /// - If pattern is specified, test name must contain the pattern
    pub fn matches(&self, test: &UatTest) -> bool {
        // Category filter (if any categories specified, test must match one)
        if !self.categories.is_empty() && !self.categories.contains(&test.category) {
            return false;
        }

        // Priority filter (if any priorities specified, test must match one)
        if !self.priorities.is_empty() && !self.priorities.contains(&test.priority) {
            return false;
        }

        // Pattern filter (substring match on test name)
        if let Some(ref pattern) = self.pattern {
            if !test.name.contains(pattern) {
                return false;
            }
        }

        true
    }
}

/// Result of a single UAT test.
#[derive(Debug, Clone)]
pub struct UatResult {
    /// Test that was run.
    pub test: UatTest,
    /// Whether the test passed.
    pub passed: bool,
    /// Duration in microseconds.
    pub duration_us: u64,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Aggregated results from a UAT run.
#[derive(Debug, Clone)]
pub struct UatResults {
    /// Total tests run.
    pub total: usize,
    /// Tests that passed.
    pub passed: usize,
    /// Tests that failed.
    pub failed: usize,
    /// Tests that were skipped.
    pub skipped: usize,
    /// Total duration in microseconds.
    pub duration_us: u64,
    /// Individual test results.
    pub results: Vec<UatResult>,
}

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

/// Recursively collect all `.rhai` files in a directory.
fn collect_rhai_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return files,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_rhai_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rhai") {
            files.push(path);
        }
    }

    files
}

/// Discover UAT tests in a single Rhai file.
fn discover_tests_in_file(file_path: &Path) -> Result<Vec<UatTest>, String> {
    let content =
        fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let engine = Engine::new();
    let ast = engine
        .compile(&content)
        .map_err(|e| format!("Failed to compile Rhai script: {}", e))?;

    let file_str = file_path.to_string_lossy().to_string();
    let mut tests = Vec::new();

    // Find all uat_* functions
    for fn_def in ast.iter_functions() {
        if fn_def.name.starts_with("uat_") {
            // Parse metadata from comments preceding the function
            let metadata = parse_function_metadata(&content, fn_def.name);

            tests.push(UatTest {
                name: fn_def.name.to_string(),
                file: file_str.clone(),
                category: metadata.category,
                priority: metadata.priority,
                requirements: metadata.requirements,
                latency_threshold: metadata.latency_threshold,
            });
        }
    }

    Ok(tests)
}

/// Parsed metadata from function comments.
struct FunctionMetadata {
    category: String,
    priority: Priority,
    requirements: Vec<String>,
    latency_threshold: Option<u64>,
}

/// Parse metadata from comments preceding a function.
///
/// Looks for special comment tags:
/// - `@category: <value>`
/// - `@priority: P0|P1|P2`
/// - `@requirement: <id1>, <id2>, ...`
/// - `@latency: <microseconds>`
fn parse_function_metadata(content: &str, fn_name: &str) -> FunctionMetadata {
    // Find the function definition and look for comments before it
    let fn_pattern = format!("fn {}", fn_name);
    let fn_pos = match content.find(&fn_pattern) {
        Some(pos) => pos,
        None => {
            return FunctionMetadata {
                category: "default".to_string(),
                priority: Priority::default(),
                requirements: Vec::new(),
                latency_threshold: None,
            };
        }
    };

    // Extract comments before the function (look back up to 500 chars)
    let start_pos = fn_pos.saturating_sub(500);
    let prefix = &content[start_pos..fn_pos];

    // Collect comment lines (both // and /* */)
    let mut category = "default".to_string();
    let mut priority = Priority::default();
    let mut requirements = Vec::new();
    let mut latency_threshold = None;

    for line in prefix.lines().rev() {
        let trimmed = line.trim();

        // Handle single-line comments
        if let Some(comment) = trimmed.strip_prefix("//") {
            parse_metadata_line(
                comment.trim(),
                &mut category,
                &mut priority,
                &mut requirements,
                &mut latency_threshold,
            );
        }

        // Stop if we hit a non-comment, non-empty line (likely end of previous function)
        if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("/*") {
            // But allow blank lines between comments and function
            if !trimmed.chars().all(|c| c.is_whitespace()) {
                break;
            }
        }
    }

    FunctionMetadata {
        category,
        priority,
        requirements,
        latency_threshold,
    }
}

/// Parse a single metadata line from a comment.
fn parse_metadata_line(
    line: &str,
    category: &mut String,
    priority: &mut Priority,
    requirements: &mut Vec<String>,
    latency_threshold: &mut Option<u64>,
) {
    // Parse @category: <value>
    if let Some(value) = line.strip_prefix("@category:") {
        *category = value.trim().to_string();
    }
    // Parse @priority: P0|P1|P2
    else if let Some(value) = line.strip_prefix("@priority:") {
        if let Ok(p) = value.parse::<Priority>() {
            *priority = p;
        }
    }
    // Parse @requirement: <id1>, <id2>, ...
    else if let Some(value) = line.strip_prefix("@requirement:") {
        let ids: Vec<String> = value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        requirements.extend(ids);
    }
    // Parse @latency: <microseconds>
    else if let Some(value) = line.strip_prefix("@latency:") {
        if let Ok(us) = value.trim().parse::<u64>() {
            *latency_threshold = Some(us);
        }
    }
}

/// Execute a single UAT test and return the result.
fn execute_test(test: &UatTest) -> UatResult {
    let start_time = std::time::Instant::now();

    tracing::debug!(
        service = "keyrx",
        event = "uat_test_start",
        component = "uat_runner",
        test_name = %test.name,
        test_file = %test.file,
        category = %test.category,
        "Executing UAT test"
    );

    // Read the test file
    let content = match fs::read_to_string(&test.file) {
        Ok(c) => c,
        Err(e) => {
            let duration_us = start_time.elapsed().as_micros() as u64;
            return UatResult {
                test: test.clone(),
                passed: false,
                duration_us,
                error: Some(format!("Failed to read test file: {}", e)),
            };
        }
    };

    // Create Rhai engine
    let engine = Engine::new();

    // Compile and run the test
    let result = (|| -> Result<(), String> {
        let ast = engine
            .compile(&content)
            .map_err(|e| format!("Compilation error: {}", e))?;

        // Run the script to define functions
        engine
            .run_ast(&ast)
            .map_err(|e| format!("Script error: {}", e))?;

        // Call the test function
        engine
            .call_fn::<()>(&mut rhai::Scope::new(), &ast, &test.name, ())
            .map_err(|e| format!("Test execution error: {}", e))?;

        Ok(())
    })();

    let duration_us = start_time.elapsed().as_micros() as u64;

    match result {
        Ok(()) => {
            tracing::debug!(
                service = "keyrx",
                event = "uat_test_pass",
                component = "uat_runner",
                test_name = %test.name,
                duration_us = duration_us,
                "UAT test passed"
            );
            UatResult {
                test: test.clone(),
                passed: true,
                duration_us,
                error: None,
            }
        }
        Err(e) => {
            tracing::debug!(
                service = "keyrx",
                event = "uat_test_fail",
                component = "uat_runner",
                test_name = %test.name,
                duration_us = duration_us,
                error = %e,
                "UAT test failed"
            );
            UatResult {
                test: test.clone(),
                passed: false,
                duration_us,
                error: Some(e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn priority_from_str_parses_valid_values() {
        assert_eq!("P0".parse::<Priority>(), Ok(Priority::P0));
        assert_eq!("P1".parse::<Priority>(), Ok(Priority::P1));
        assert_eq!("P2".parse::<Priority>(), Ok(Priority::P2));
        assert_eq!("p0".parse::<Priority>(), Ok(Priority::P0));
        assert_eq!(" P1 ".parse::<Priority>(), Ok(Priority::P1));
    }

    #[test]
    fn priority_from_str_returns_err_for_invalid() {
        assert!("P3".parse::<Priority>().is_err());
        assert!("".parse::<Priority>().is_err());
        assert!("high".parse::<Priority>().is_err());
    }

    #[test]
    fn priority_default_is_p2() {
        assert_eq!(Priority::default(), Priority::P2);
    }

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
    fn parse_metadata_line_parses_category() {
        let mut category = String::new();
        let mut priority = Priority::P2;
        let mut requirements = Vec::new();
        let mut latency = None;

        parse_metadata_line(
            "@category: core",
            &mut category,
            &mut priority,
            &mut requirements,
            &mut latency,
        );

        assert_eq!(category, "core");
    }

    #[test]
    fn parse_metadata_line_parses_priority() {
        let mut category = String::new();
        let mut priority = Priority::P2;
        let mut requirements = Vec::new();
        let mut latency = None;

        parse_metadata_line(
            "@priority: P0",
            &mut category,
            &mut priority,
            &mut requirements,
            &mut latency,
        );

        assert_eq!(priority, Priority::P0);
    }

    #[test]
    fn parse_metadata_line_parses_requirements() {
        let mut category = String::new();
        let mut priority = Priority::P2;
        let mut requirements = Vec::new();
        let mut latency = None;

        parse_metadata_line(
            "@requirement: 1.1, 2.3, 4.5",
            &mut category,
            &mut priority,
            &mut requirements,
            &mut latency,
        );

        assert_eq!(requirements, vec!["1.1", "2.3", "4.5"]);
    }

    #[test]
    fn parse_metadata_line_parses_latency() {
        let mut category = String::new();
        let mut priority = Priority::P2;
        let mut requirements = Vec::new();
        let mut latency = None;

        parse_metadata_line(
            "@latency: 500",
            &mut category,
            &mut priority,
            &mut requirements,
            &mut latency,
        );

        assert_eq!(latency, Some(500));
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
    fn collect_rhai_files_finds_only_rhai() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("test.rhai"), "").unwrap();
        fs::write(temp_dir.path().join("test.rs"), "").unwrap();
        fs::write(temp_dir.path().join("test.txt"), "").unwrap();

        let files = collect_rhai_files(temp_dir.path());

        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().ends_with(".rhai"));
    }

    #[test]
    fn filter_matches_all_when_empty() {
        let filter = UatFilter::default();
        let test = UatTest {
            name: "uat_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P1,
            requirements: vec![],
            latency_threshold: None,
        };
        assert!(filter.matches(&test));
    }

    #[test]
    fn filter_matches_by_category() {
        let filter = UatFilter {
            categories: vec!["core".to_string()],
            ..Default::default()
        };

        let matching = UatTest {
            name: "uat_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P1,
            requirements: vec![],
            latency_threshold: None,
        };

        let non_matching = UatTest {
            name: "uat_test2".to_string(),
            file: "test.rhai".to_string(),
            category: "layers".to_string(),
            priority: Priority::P1,
            requirements: vec![],
            latency_threshold: None,
        };

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&non_matching));
    }

    #[test]
    fn filter_matches_by_priority() {
        let filter = UatFilter {
            priorities: vec![Priority::P0, Priority::P1],
            ..Default::default()
        };

        let matching = UatTest {
            name: "uat_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P0,
            requirements: vec![],
            latency_threshold: None,
        };

        let non_matching = UatTest {
            name: "uat_test2".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P2,
            requirements: vec![],
            latency_threshold: None,
        };

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&non_matching));
    }

    #[test]
    fn filter_matches_by_pattern() {
        let filter = UatFilter {
            pattern: Some("layer".to_string()),
            ..Default::default()
        };

        let matching = UatTest {
            name: "uat_layer_switch".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P1,
            requirements: vec![],
            latency_threshold: None,
        };

        let non_matching = UatTest {
            name: "uat_basic_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P1,
            requirements: vec![],
            latency_threshold: None,
        };

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&non_matching));
    }

    #[test]
    fn filter_uses_and_logic() {
        let filter = UatFilter {
            categories: vec!["core".to_string()],
            priorities: vec![Priority::P0],
            pattern: Some("basic".to_string()),
        };

        // Matches all criteria
        let matching = UatTest {
            name: "uat_basic_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P0,
            requirements: vec![],
            latency_threshold: None,
        };

        // Wrong category
        let wrong_category = UatTest {
            name: "uat_basic_test".to_string(),
            file: "test.rhai".to_string(),
            category: "layers".to_string(),
            priority: Priority::P0,
            requirements: vec![],
            latency_threshold: None,
        };

        // Wrong priority
        let wrong_priority = UatTest {
            name: "uat_basic_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P1,
            requirements: vec![],
            latency_threshold: None,
        };

        // Wrong pattern
        let wrong_pattern = UatTest {
            name: "uat_advanced_test".to_string(),
            file: "test.rhai".to_string(),
            category: "core".to_string(),
            priority: Priority::P0,
            requirements: vec![],
            latency_threshold: None,
        };

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&wrong_category));
        assert!(!filter.matches(&wrong_priority));
        assert!(!filter.matches(&wrong_pattern));
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
