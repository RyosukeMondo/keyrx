//! File writer module for generated Dart code
//!
//! This module handles writing generated code to files, including timestamp-based
//! change detection to avoid unnecessary regeneration.

use std::fs;
use std::io;
use std::path::Path;

/// Result of a write operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteResult {
    /// File was written successfully
    Written,
    /// File was skipped because content is identical
    Skipped,
}

/// Write content to a file, creating parent directories if needed.
///
/// Returns `WriteResult::Skipped` if the file already contains the same content,
/// avoiding unnecessary file modifications and timestamp changes.
pub fn write_if_changed(path: &Path, content: &str) -> io::Result<WriteResult> {
    if needs_write(path, content)? {
        write_file(path, content)?;
        Ok(WriteResult::Written)
    } else {
        Ok(WriteResult::Skipped)
    }
}

/// Check if a file needs to be written (doesn't exist or content differs)
fn needs_write(path: &Path, new_content: &str) -> io::Result<bool> {
    if !path.exists() {
        return Ok(true);
    }
    let existing_content = fs::read_to_string(path)?;
    Ok(existing_content != new_content)
}

/// Write content to a file, creating parent directories if needed
fn write_file(path: &Path, content: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

/// Write multiple files atomically (best-effort).
///
/// Writes all files, collecting errors. Returns the first error if any occurred.
pub fn write_files(files: &[(&Path, &str)]) -> io::Result<Vec<WriteResult>> {
    let mut results = Vec::with_capacity(files.len());
    let mut first_error: Option<io::Error> = None;

    for (path, content) in files {
        match write_if_changed(path, content) {
            Ok(result) => results.push(result),
            Err(e) => {
                if first_error.is_none() {
                    first_error = Some(e);
                }
                results.push(WriteResult::Written); // placeholder
            }
        }
    }

    if let Some(err) = first_error {
        return Err(err);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.dart");

        let result = write_if_changed(&file_path, "void main() {}").unwrap();

        assert_eq!(result, WriteResult::Written);
        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "void main() {}");
    }

    #[test]
    fn test_skip_unchanged_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.dart");
        let content = "void main() {}";

        fs::write(&file_path, content).unwrap();
        let result = write_if_changed(&file_path, content).unwrap();

        assert_eq!(result, WriteResult::Skipped);
    }

    #[test]
    fn test_overwrite_changed_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.dart");

        fs::write(&file_path, "old content").unwrap();
        let result = write_if_changed(&file_path, "new content").unwrap();

        assert_eq!(result, WriteResult::Written);
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "new content");
    }

    #[test]
    fn test_create_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nested").join("dir").join("test.dart");

        let result = write_if_changed(&file_path, "content").unwrap();

        assert_eq!(result, WriteResult::Written);
        assert!(file_path.exists());
    }

    #[test]
    fn test_write_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let path1 = temp_dir.path().join("file1.dart");
        let path2 = temp_dir.path().join("file2.dart");

        let files: Vec<(&Path, &str)> =
            vec![(path1.as_path(), "content1"), (path2.as_path(), "content2")];
        let results = write_files(&files).unwrap();

        assert_eq!(results.len(), 2);
        assert!(path1.exists());
        assert!(path2.exists());
    }

    #[test]
    fn test_needs_write_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.dart");

        assert!(needs_write(&file_path, "content").unwrap());
    }

    #[test]
    fn test_needs_write_same_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.dart");
        fs::write(&file_path, "content").unwrap();

        assert!(!needs_write(&file_path, "content").unwrap());
    }

    #[test]
    fn test_needs_write_different_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.dart");
        fs::write(&file_path, "old").unwrap();

        assert!(needs_write(&file_path, "new").unwrap());
    }
}
