//! Import resolution for multi-file Rhai configurations.
//!
//! This module handles resolving load("path/to/file.rhai") calls in Rhai scripts,
//! searching through multiple stdlib paths.

use std::path::{Path, PathBuf};

use crate::error::ParseError;

/// Import resolver for handling multi-file Rhai configurations.
///
/// Resolves file paths through a cascading search path system.
pub struct ImportResolver;

impl ImportResolver {
    /// Creates a new import resolver.
    pub fn new() -> Self {
        Self
    }

    /// Resolves a load() path from a given directory.
    ///
    /// # Arguments
    /// * `import_path` - The import path string (e.g., "utils.rhai" or "shift.rhai")
    /// * `base_dir` - Base directory to resolve from
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Resolved absolute path
    /// * `Err(ParseError::ImportNotFound)` - If the resolved path doesn't exist
    pub fn resolve_path_from_dir(
        &self,
        import_path: &str,
        base_dir: &Path,
    ) -> Result<PathBuf, ParseError> {
        let mut searched_paths = Vec::new();

        // 1. Try relative to base directory
        let relative_path = base_dir.join(import_path);
        searched_paths.push(relative_path.clone());
        if relative_path.exists() {
            return Ok(relative_path);
        }

        // 2. Try ./stdlib/ relative to base directory
        let local_stdlib_path = base_dir.join("stdlib").join(import_path);
        searched_paths.push(local_stdlib_path.clone());
        if local_stdlib_path.exists() {
            return Ok(local_stdlib_path);
        }

        // 3. Try ~/.config/keyrx/stdlib/
        if let Some(home_dir) = std::env::var_os("HOME") {
            let user_stdlib_path = PathBuf::from(home_dir)
                .join(".config")
                .join("keyrx")
                .join("stdlib")
                .join(import_path);
            searched_paths.push(user_stdlib_path.clone());
            if user_stdlib_path.exists() {
                return Ok(user_stdlib_path);
            }
        }

        // 4. Try /usr/share/keyrx/stdlib/ (Linux only)
        #[cfg(target_os = "linux")]
        {
            let system_stdlib_path = PathBuf::from("/usr/share/keyrx/stdlib").join(import_path);
            searched_paths.push(system_stdlib_path.clone());
            if system_stdlib_path.exists() {
                return Ok(system_stdlib_path);
            }
        }

        // File not found in any location
        Err(ParseError::ImportNotFound {
            path: PathBuf::from(import_path),
            searched_paths,
            import_chain: Vec::new(),
        })
    }
}

impl Default for ImportResolver {
    fn default() -> Self {
        Self::new()
    }
}
