//! Import resolution for multi-file Rhai configurations.
//!
//! This module handles resolving `import "path/to/file.rhai"` statements in Rhai scripts,
//! including recursive imports and circular dependency detection.

// Allow dead_code - import resolution will be used in task 24 for multi-file imports
#![allow(dead_code)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ParseError;

/// Import resolver for handling multi-file Rhai configurations.
///
/// Tracks visited files to detect circular imports and resolves relative paths
/// based on the current file's directory.
pub struct ImportResolver {
    /// Set of files that have been visited during import resolution.
    /// Used to detect circular imports.
    visited_files: HashSet<PathBuf>,
}

impl ImportResolver {
    /// Creates a new import resolver with empty state.
    pub fn new() -> Self {
        Self {
            visited_files: HashSet::new(),
        }
    }

    /// Resolves imports from a Rhai script file.
    ///
    /// Given a path to a Rhai script, this function:
    /// 1. Adds the path to visited files
    /// 2. Parses the file for import statements
    /// 3. Resolves each import relative to the current file's directory
    /// 4. Recursively resolves imports in imported files
    ///
    /// # Arguments
    /// * `path` - Path to the Rhai script file
    ///
    /// # Returns
    /// * `Ok(Vec<PathBuf>)` - List of all resolved import paths (in dependency order)
    /// * `Err(ParseError)` - If file not found, circular import detected, or other error
    ///
    /// # Errors
    /// * `ParseError::ImportNotFound` - If an imported file doesn't exist
    /// * `ParseError::CircularImport` - If a circular dependency is detected
    pub fn resolve_imports(&mut self, path: &Path) -> Result<Vec<PathBuf>, ParseError> {
        // Canonicalize the path to handle relative paths and resolve symlinks
        let canonical_path = path
            .canonicalize()
            .map_err(|_| ParseError::ImportNotFound {
                path: path.to_path_buf(),
                searched_paths: vec![path.to_path_buf()],
            })?;

        // Check for circular imports
        if self.visited_files.contains(&canonical_path) {
            // Build the import chain for error reporting
            let mut chain: Vec<PathBuf> = self.visited_files.iter().cloned().collect();
            chain.push(canonical_path.clone());
            return Err(ParseError::CircularImport { chain });
        }

        // Mark this file as visited
        self.visited_files.insert(canonical_path.clone());

        // Read the file content
        let content =
            fs::read_to_string(&canonical_path).map_err(|_| ParseError::ImportNotFound {
                path: path.to_path_buf(),
                searched_paths: vec![canonical_path.clone()],
            })?;

        // Find all import statements in the file
        let import_paths = self.extract_imports(&content, &canonical_path)?;

        // Recursively resolve imports
        let mut all_imports = Vec::new();
        for import_path in import_paths {
            // Resolve the import path relative to the current file's directory
            let resolved_path = self.resolve_path(&import_path, &canonical_path)?;

            // Recursively resolve imports in the imported file
            let nested_imports = self.resolve_imports(&resolved_path)?;
            all_imports.extend(nested_imports);

            // Add the import itself
            all_imports.push(resolved_path);
        }

        Ok(all_imports)
    }

    /// Extracts import statements from Rhai script content.
    ///
    /// Searches for lines matching `import "path/to/file.rhai"` pattern.
    ///
    /// # Arguments
    /// * `content` - The Rhai script content
    /// * `current_file` - Path to the current file (for error reporting)
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of import path strings
    /// * `Err(ParseError)` - If import syntax is invalid
    fn extract_imports(
        &self,
        content: &str,
        _current_file: &Path,
    ) -> Result<Vec<String>, ParseError> {
        let mut imports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Look for import statements: import "path/to/file.rhai"
            if trimmed.starts_with("import ") {
                // Extract the path from quotes
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed[start + 1..].find('"') {
                        let path = &trimmed[start + 1..start + 1 + end];
                        imports.push(path.to_string());
                    }
                }
            }
        }

        Ok(imports)
    }

    /// Resolves an import path relative to the current file's directory.
    ///
    /// # Arguments
    /// * `import_path` - The import path string (e.g., "utils.rhai" or "../common/keys.rhai")
    /// * `current_file` - Path to the current file
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Resolved absolute path
    /// * `Err(ParseError::ImportNotFound)` - If the resolved path doesn't exist
    fn resolve_path(&self, import_path: &str, current_file: &Path) -> Result<PathBuf, ParseError> {
        // Get the directory of the current file
        let current_dir = current_file
            .parent()
            .ok_or_else(|| ParseError::ImportNotFound {
                path: PathBuf::from(import_path),
                searched_paths: vec![current_file.to_path_buf()],
            })?;

        // Resolve the import path relative to the current directory
        let resolved_path = current_dir.join(import_path);

        // Check if the file exists
        if !resolved_path.exists() {
            return Err(ParseError::ImportNotFound {
                path: PathBuf::from(import_path),
                searched_paths: vec![resolved_path.clone(), current_dir.join(import_path)],
            });
        }

        Ok(resolved_path)
    }

    /// Detects circular imports in an import chain.
    ///
    /// # Arguments
    /// * `chain` - The import chain to check
    ///
    /// # Returns
    /// * `Ok(())` - If no circular import detected
    /// * `Err(ParseError::CircularImport)` - If a circular import is detected
    pub fn detect_circular_imports(chain: &[PathBuf]) -> Result<(), ParseError> {
        let mut seen = HashSet::new();

        for path in chain {
            if seen.contains(path) {
                // Found a duplicate - circular import detected
                return Err(ParseError::CircularImport {
                    chain: chain.to_vec(),
                });
            }
            seen.insert(path.clone());
        }

        Ok(())
    }
}

impl Default for ImportResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper to create a temporary Rhai file with content.
    fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let file_path = dir.path().join(name);
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }

    #[test]
    fn test_simple_import_resolution() {
        let temp_dir = TempDir::new().unwrap();

        // Create main.rhai that imports utils.rhai
        let utils_content = r#"
// utils.rhai - utility functions
fn helper() { }
"#;
        create_temp_file(&temp_dir, "utils.rhai", utils_content);

        let main_content = r#"
import "utils.rhai"

device("Keyboard") {
    map("A", "VK_B");
}
"#;
        let main_path = create_temp_file(&temp_dir, "main.rhai", main_content);

        // Resolve imports
        let mut resolver = ImportResolver::new();
        let imports = resolver.resolve_imports(&main_path).unwrap();

        // Should find utils.rhai
        assert_eq!(imports.len(), 1);
        assert!(imports[0].ends_with("utils.rhai"));
    }

    #[test]
    fn test_recursive_imports() {
        let temp_dir = TempDir::new().unwrap();

        // Create a chain: main -> utils -> common
        let common_content = "// common.rhai";
        create_temp_file(&temp_dir, "common.rhai", common_content);

        let utils_content = r#"
import "common.rhai"
// utils.rhai
"#;
        create_temp_file(&temp_dir, "utils.rhai", utils_content);

        let main_content = r#"
import "utils.rhai"
// main.rhai
"#;
        let main_path = create_temp_file(&temp_dir, "main.rhai", main_content);

        // Resolve imports
        let mut resolver = ImportResolver::new();
        let imports = resolver.resolve_imports(&main_path).unwrap();

        // Should find both common.rhai and utils.rhai (in dependency order)
        assert_eq!(imports.len(), 2);
        assert!(imports[0].ends_with("common.rhai"));
        assert!(imports[1].ends_with("utils.rhai"));
    }

    #[test]
    fn test_circular_import_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Create circular dependency: a -> b -> c -> a
        let c_content = r#"
import "a.rhai"
// c.rhai
"#;
        create_temp_file(&temp_dir, "c.rhai", c_content);

        let b_content = r#"
import "c.rhai"
// b.rhai
"#;
        create_temp_file(&temp_dir, "b.rhai", b_content);

        let a_content = r#"
import "b.rhai"
// a.rhai
"#;
        let a_path = create_temp_file(&temp_dir, "a.rhai", a_content);

        // Resolve imports - should detect circular dependency
        let mut resolver = ImportResolver::new();
        let result = resolver.resolve_imports(&a_path);

        assert!(result.is_err());
        match result {
            Err(ParseError::CircularImport { chain }) => {
                // Chain should contain at least 2 files (the cycle participants)
                assert!(chain.len() >= 2);
                // Verify that at least one of the expected files is in the chain
                let has_expected_file = chain.iter().any(|p| {
                    p.ends_with("a.rhai") || p.ends_with("b.rhai") || p.ends_with("c.rhai")
                });
                assert!(
                    has_expected_file,
                    "Chain should contain one of the circular import files"
                );
            }
            _ => panic!("Expected CircularImport error"),
        }
    }

    #[test]
    fn test_file_not_found_error() {
        let temp_dir = TempDir::new().unwrap();

        // Create main.rhai that imports non-existent file
        let main_content = r#"
import "missing.rhai"
"#;
        let main_path = create_temp_file(&temp_dir, "main.rhai", main_content);

        // Resolve imports - should fail with ImportNotFound
        let mut resolver = ImportResolver::new();
        let result = resolver.resolve_imports(&main_path);

        assert!(result.is_err());
        match result {
            Err(ParseError::ImportNotFound {
                path,
                searched_paths,
            }) => {
                assert!(path.to_str().unwrap().contains("missing.rhai"));
                assert!(!searched_paths.is_empty());
            }
            _ => panic!("Expected ImportNotFound error"),
        }
    }

    #[test]
    fn test_relative_path_resolution() {
        let temp_dir = TempDir::new().unwrap();

        // Create subdirectory structure
        let subdir = temp_dir.path().join("lib");
        fs::create_dir(&subdir).unwrap();

        // Create lib/common.rhai
        let common_content = "// common.rhai";
        let mut common_file = fs::File::create(subdir.join("common.rhai")).unwrap();
        common_file.write_all(common_content.as_bytes()).unwrap();

        // Create main.rhai that imports lib/common.rhai
        let main_content = r#"
import "lib/common.rhai"
"#;
        let main_path = create_temp_file(&temp_dir, "main.rhai", main_content);

        // Resolve imports
        let mut resolver = ImportResolver::new();
        let imports = resolver.resolve_imports(&main_path).unwrap();

        // Should find lib/common.rhai
        assert_eq!(imports.len(), 1);
        assert!(imports[0].ends_with("common.rhai"));
    }

    #[test]
    fn test_detect_circular_imports_function() {
        // Test the standalone detect_circular_imports function
        let path_a = PathBuf::from("a.rhai");
        let path_b = PathBuf::from("b.rhai");
        let path_c = PathBuf::from("c.rhai");

        // No circular import
        let chain = vec![path_a.clone(), path_b.clone(), path_c.clone()];
        assert!(ImportResolver::detect_circular_imports(&chain).is_ok());

        // Circular import: a -> b -> c -> a
        let circular_chain = vec![
            path_a.clone(),
            path_b.clone(),
            path_c.clone(),
            path_a.clone(),
        ];
        let result = ImportResolver::detect_circular_imports(&circular_chain);
        assert!(result.is_err());
        match result {
            Err(ParseError::CircularImport { chain }) => {
                assert_eq!(chain.len(), 4);
            }
            _ => panic!("Expected CircularImport error"),
        }
    }

    #[test]
    fn test_extract_imports() {
        let resolver = ImportResolver::new();
        let current_file = PathBuf::from("test.rhai");

        let content = r#"
// Comment
import "utils.rhai"
import "common.rhai"

device("Keyboard") {
    map("A", "VK_B");
}

import "extra.rhai"
"#;

        let imports = resolver.extract_imports(content, &current_file).unwrap();

        assert_eq!(imports.len(), 3);
        assert_eq!(imports[0], "utils.rhai");
        assert_eq!(imports[1], "common.rhai");
        assert_eq!(imports[2], "extra.rhai");
    }

    #[test]
    fn test_default_trait() {
        let resolver = ImportResolver::default();
        assert_eq!(resolver.visited_files.len(), 0);
    }
}
