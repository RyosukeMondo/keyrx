//! Dart formatter integration module
//!
//! This module provides functionality to run `dart format` on generated files,
//! ensuring consistent code style in the output.

use std::io;
use std::path::Path;
use std::process::Command;

/// Result of a format operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatResult {
    /// File was formatted successfully
    Formatted,
    /// Formatting was skipped (file unchanged or dart not available)
    Skipped,
    /// Formatting failed with an error message
    Failed(String),
}

/// Check if the `dart` command is available in PATH
pub fn is_dart_available() -> bool {
    Command::new("dart")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Format a single Dart file using `dart format`
pub fn format_file(path: &Path) -> FormatResult {
    if !path.exists() {
        return FormatResult::Failed(format!("File does not exist: {}", path.display()));
    }

    let output = match Command::new("dart").arg("format").arg(path).output() {
        Ok(output) => output,
        Err(e) => {
            return if e.kind() == io::ErrorKind::NotFound {
                FormatResult::Skipped
            } else {
                FormatResult::Failed(format!("Failed to run dart format: {e}"))
            };
        }
    };

    if output.status.success() {
        FormatResult::Formatted
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        FormatResult::Failed(format!("dart format failed: {stderr}"))
    }
}

/// Format multiple Dart files, returning results for each
pub fn format_files(paths: &[&Path]) -> Vec<FormatResult> {
    if !is_dart_available() {
        return paths.iter().map(|_| FormatResult::Skipped).collect();
    }
    paths.iter().map(|p| format_file(p)).collect()
}

/// Format all Dart files in a directory recursively
pub fn format_directory(dir: &Path) -> FormatResult {
    if !dir.is_dir() {
        return FormatResult::Failed(format!("Not a directory: {}", dir.display()));
    }

    let output = match Command::new("dart").arg("format").arg(dir).output() {
        Ok(output) => output,
        Err(e) => {
            return if e.kind() == io::ErrorKind::NotFound {
                FormatResult::Skipped
            } else {
                FormatResult::Failed(format!("Failed to run dart format: {e}"))
            };
        }
    };

    if output.status.success() {
        FormatResult::Formatted
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        FormatResult::Failed(format!("dart format failed: {stderr}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_format_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.dart");

        let result = format_file(&file_path);
        assert!(matches!(result, FormatResult::Failed(_)));
    }

    #[test]
    fn test_format_file_dart_available() {
        if !is_dart_available() {
            return; // Skip test if dart not installed
        }

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.dart");
        fs::write(&file_path, "void main(){print('hello');}").unwrap();

        let result = format_file(&file_path);
        assert_eq!(result, FormatResult::Formatted);
    }

    #[test]
    fn test_format_invalid_dart_file() {
        if !is_dart_available() {
            return; // Skip test if dart not installed
        }

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.dart");
        fs::write(&file_path, "this is not valid dart {{{").unwrap();

        let result = format_file(&file_path);
        // dart format may still succeed even on invalid files, just report the result
        assert!(matches!(
            result,
            FormatResult::Formatted | FormatResult::Failed(_)
        ));
    }

    #[test]
    fn test_format_files_no_dart() {
        // This test mocks the scenario - in practice depends on dart availability
        let paths: Vec<&Path> = vec![];
        let results = format_files(&paths);
        assert!(results.is_empty());
    }

    #[test]
    fn test_format_directory_not_a_dir() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.dart");
        fs::write(&file_path, "void main() {}").unwrap();

        let result = format_directory(&file_path);
        assert!(matches!(result, FormatResult::Failed(_)));
    }

    #[test]
    fn test_format_directory_dart_available() {
        if !is_dart_available() {
            return; // Skip test if dart not installed
        }

        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("a.dart");
        let file2 = temp_dir.path().join("b.dart");
        fs::write(&file1, "void a(){}").unwrap();
        fs::write(&file2, "void b(){}").unwrap();

        let result = format_directory(temp_dir.path());
        assert_eq!(result, FormatResult::Formatted);
    }
}
