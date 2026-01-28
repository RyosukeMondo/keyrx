//! Path validation and sanitization (VAL-002)
//!
//! Prevents path traversal attacks by:
//! - Using PathBuf::join() instead of string concatenation
//! - Validating paths with canonicalize()
//! - Ensuring paths are within allowed directories

use super::{ValidationError, ValidationResult};
use std::path::{Path, PathBuf};

/// Validates that a path is safe and within the allowed base directory.
///
/// This function prevents path traversal attacks by:
/// 1. Resolving the path to its canonical form (follows symlinks, resolves .. and .)
/// 2. Verifying the canonical path starts with the base directory
///
/// # Arguments
///
/// * `base_dir` - The allowed base directory (e.g., config directory)
/// * `user_path` - The user-supplied path component
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated canonical path
/// * `Err(ValidationError)` - If the path is invalid or outside base_dir
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use keyrx_daemon::validation::path::validate_path_within_base;
///
/// let config_dir = PathBuf::from("/home/user/.config/keyrx");
/// let profile_name = "my-profile";
///
/// // Safe: results in /home/user/.config/keyrx/profiles/my-profile.rhai
/// let safe_path = validate_path_within_base(
///     &config_dir.join("profiles"),
///     &format!("{}.rhai", profile_name)
/// );
/// assert!(safe_path.is_ok());
///
/// // Unsafe: attempts to escape with ../../../etc/passwd
/// let unsafe_path = validate_path_within_base(
///     &config_dir,
///     "../../../etc/passwd"
/// );
/// assert!(unsafe_path.is_err());
/// ```
pub fn validate_path_within_base<P1: AsRef<Path>, P2: AsRef<Path>>(
    base_dir: P1,
    user_path: P2,
) -> ValidationResult<PathBuf> {
    let base = base_dir.as_ref();
    let user = user_path.as_ref();

    // Construct the full path using PathBuf::join (safe method)
    let full_path = base.join(user);

    // Canonicalize both paths to resolve symlinks and relative components
    let canonical_base = base.canonicalize().map_err(|e| {
        ValidationError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to canonicalize base directory: {}", e),
        ))
    })?;

    let canonical_full = match full_path.canonicalize() {
        Ok(path) => path,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // For new files, check if parent directory is within base
            if let Some(parent) = full_path.parent() {
                if parent.exists() {
                    let canonical_parent = parent.canonicalize().map_err(|pe| {
                        ValidationError::Io(std::io::Error::new(
                            pe.kind(),
                            format!("Failed to canonicalize parent directory: {}", pe),
                        ))
                    })?;

                    if !canonical_parent.starts_with(&canonical_base) {
                        return Err(ValidationError::PathTraversal(format!(
                            "Path escapes base directory: {:?}",
                            full_path
                        )));
                    }

                    // Return the non-canonical path since file doesn't exist yet
                    return Ok(full_path);
                }
            }
            return Err(ValidationError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to canonicalize path: {}", e),
            )));
        }
        Err(e) => {
            return Err(ValidationError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to canonicalize path: {}", e),
            )));
        }
    };

    // Verify the canonical path is within the base directory
    if !canonical_full.starts_with(&canonical_base) {
        return Err(ValidationError::PathTraversal(format!(
            "Path traversal detected: {:?} is outside {:?}",
            canonical_full, canonical_base
        )));
    }

    Ok(canonical_full)
}

/// Safely constructs a path within a base directory without validation.
///
/// This is a convenience function for building paths when you know the
/// components are safe (e.g., from validated profile names).
///
/// # Arguments
///
/// * `base_dir` - Base directory path
/// * `component` - Path component to append
///
/// # Returns
///
/// A PathBuf with the component safely joined to the base
pub fn safe_join<P1: AsRef<Path>, P2: AsRef<Path>>(base_dir: P1, component: P2) -> PathBuf {
    base_dir.as_ref().join(component)
}

/// Validates a file path exists and is within the allowed directory.
///
/// # Arguments
///
/// * `base_dir` - The allowed base directory
/// * `file_path` - The file path to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` if valid
/// * `Err(ValidationError)` otherwise
pub fn validate_existing_file<P1: AsRef<Path>, P2: AsRef<Path>>(
    base_dir: P1,
    file_path: P2,
) -> ValidationResult<PathBuf> {
    let path = validate_path_within_base(base_dir, file_path)?;

    if !path.exists() {
        return Err(ValidationError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File does not exist: {:?}", path),
        )));
    }

    if !path.is_file() {
        return Err(ValidationError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is not a file: {:?}", path),
        )));
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_safe_path_within_base() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create a subdirectory
        let profiles_dir = base.join("profiles");
        fs::create_dir(&profiles_dir).unwrap();

        // Create a test file
        let test_file = profiles_dir.join("test.rhai");
        fs::write(&test_file, "// test content").unwrap();

        // Validate the safe path
        let result = validate_path_within_base(&profiles_dir, "test.rhai");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().file_name().unwrap(), "test.rhai");
    }

    #[test]
    fn test_path_traversal_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create subdirectory
        let profiles_dir = base.join("profiles");
        fs::create_dir(&profiles_dir).unwrap();

        // Create file outside profiles directory
        let outside_file = base.join("outside.txt");
        fs::write(&outside_file, "outside content").unwrap();

        // Attempt to access file outside using ../
        let result = validate_path_within_base(&profiles_dir, "../outside.txt");
        assert!(result.is_err());
        match result {
            Err(ValidationError::PathTraversal(_)) => (),
            _ => panic!("Expected PathTraversal error"),
        }
    }

    #[test]
    fn test_absolute_path_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Attempt to use absolute path
        #[cfg(unix)]
        let result = validate_path_within_base(base, "/etc/passwd");
        #[cfg(windows)]
        let result = validate_path_within_base(base, "C:\\Windows\\System32\\config\\SAM");

        // This should fail because absolute path won't be within base
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_join() {
        let base = PathBuf::from("/home/user/.config/keyrx");
        let component = "profiles";

        let result = safe_join(&base, component);
        assert_eq!(result, base.join(component));
    }

    #[test]
    fn test_validate_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create a test file
        let test_file = base.join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Validate existing file
        let result = validate_existing_file(base, "test.txt");
        assert!(result.is_ok());

        // Validate non-existent file
        let result = validate_existing_file(base, "nonexistent.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_directory_as_file() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create a subdirectory
        let subdir = base.join("subdir");
        fs::create_dir(&subdir).unwrap();

        // Validate directory as file (should fail)
        let result = validate_existing_file(base, "subdir");
        assert!(result.is_err());
    }

    #[test]
    fn test_new_file_parent_check() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create profiles directory
        let profiles_dir = base.join("profiles");
        fs::create_dir(&profiles_dir).unwrap();

        // Validate path for a new file (doesn't exist yet)
        let result = validate_path_within_base(&profiles_dir, "new_profile.rhai");
        assert!(result.is_ok());

        // Validate path traversal for new file
        let result = validate_path_within_base(&profiles_dir, "../outside.txt");
        assert!(result.is_err());
    }
}
