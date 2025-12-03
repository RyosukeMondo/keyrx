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
}
