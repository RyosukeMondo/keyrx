//! UAT test discovery functionality.
//!
//! This module handles discovering UAT tests in Rhai script files by:
//! - Recursively scanning directories for `.rhai` files
//! - Parsing function definitions for `uat_*` prefixed functions
//! - Extracting metadata from preceding comments

use std::fs;
use std::path::{Path, PathBuf};

use rhai::Engine;

use super::runner_types::{Priority, UatTest};

/// Parsed metadata from function comments.
pub(crate) struct FunctionMetadata {
    pub category: String,
    pub priority: Priority,
    pub requirements: Vec<String>,
    pub latency_threshold: Option<u64>,
}

/// Recursively collect all `.rhai` files in a directory.
pub(crate) fn collect_rhai_files(dir: &Path) -> Vec<PathBuf> {
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
pub(crate) fn discover_tests_in_file(file_path: &Path) -> Result<Vec<UatTest>, String> {
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

/// Parse metadata from comments preceding a function.
///
/// Looks for special comment tags:
/// - `@category: <value>`
/// - `@priority: P0|P1|P2`
/// - `@requirement: <id1>, <id2>, ...`
/// - `@latency: <microseconds>`
pub(crate) fn parse_function_metadata(content: &str, fn_name: &str) -> FunctionMetadata {
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
pub(crate) fn parse_metadata_line(
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
    fn collect_rhai_files_handles_subdirectories() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        fs::write(temp_dir.path().join("root.rhai"), "").unwrap();
        fs::write(sub_dir.join("nested.rhai"), "").unwrap();

        let files = collect_rhai_files(temp_dir.path());

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn discover_tests_in_file_finds_uat_functions() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rhai");

        let script = r#"
// @category: core
// @priority: P0
fn uat_basic_test() {
    let x = 1;
}

fn helper() {
    // not a UAT test
}
"#;
        fs::write(&test_file, script).unwrap();

        let tests = discover_tests_in_file(&test_file).unwrap();

        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "uat_basic_test");
        assert_eq!(tests[0].category, "core");
        assert_eq!(tests[0].priority, Priority::P0);
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
    fn parse_function_metadata_handles_missing_function() {
        let content = "fn other_function() {}";
        let metadata = parse_function_metadata(content, "nonexistent");

        assert_eq!(metadata.category, "default");
        assert_eq!(metadata.priority, Priority::P2);
        assert!(metadata.requirements.is_empty());
        assert_eq!(metadata.latency_threshold, None);
    }

    #[test]
    fn parse_function_metadata_extracts_all_tags() {
        let content = r#"
// @category: performance
// @priority: P1
// @requirement: REQ-1, REQ-2
// @latency: 1000
fn uat_perf_test() {
    let x = 1;
}
"#;
        let metadata = parse_function_metadata(content, "uat_perf_test");

        assert_eq!(metadata.category, "performance");
        assert_eq!(metadata.priority, Priority::P1);
        assert_eq!(metadata.requirements, vec!["REQ-1", "REQ-2"]);
        assert_eq!(metadata.latency_threshold, Some(1000));
    }
}
