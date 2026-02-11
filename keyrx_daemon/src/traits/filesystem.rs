//! Filesystem abstraction for dependency injection.
//!
//! This module provides traits and implementations for filesystem operations
//! in a testable way. The `FileSystem` trait abstracts common filesystem
//! operations allowing tests to use in-memory mock implementations.

use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// Metadata about a file or directory.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub is_dir: bool,
    pub modified: SystemTime,
}

/// Trait for filesystem operations.
///
/// This trait abstracts filesystem operations to enable testing
/// without accessing the real filesystem.
///
/// # Examples
///
/// ```
/// use keyrx_daemon::traits::filesystem::{FileSystem, RealFileSystem};
/// use std::path::Path;
///
/// let fs = RealFileSystem::new();
/// if fs.exists(Path::new("/tmp")) {
///     println!("/tmp exists");
/// }
/// ```
pub trait FileSystem: Send + Sync {
    /// Check if a path exists.
    fn exists(&self, path: &Path) -> bool;

    /// Read file contents as a string.
    fn read_to_string(&self, path: &Path) -> io::Result<String>;

    /// Write string contents to a file.
    fn write(&self, path: &Path, contents: &str) -> io::Result<()>;

    /// Create a directory and all parent directories.
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;

    /// Remove a file.
    fn remove_file(&self, path: &Path) -> io::Result<()>;

    /// Remove a directory.
    fn remove_dir(&self, path: &Path) -> io::Result<()>;

    /// Rename/move a file or directory.
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;

    /// Copy a file.
    fn copy(&self, from: &Path, to: &Path) -> io::Result<()>;

    /// Get file metadata.
    fn metadata(&self, path: &Path) -> io::Result<FileMetadata>;

    /// Read directory entries.
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
}

/// Production implementation that delegates to `std::fs`.
///
/// This is the default implementation used in production code.
///
/// # Examples
///
/// ```
/// use keyrx_daemon::traits::filesystem::{FileSystem, RealFileSystem};
/// use std::path::Path;
///
/// let fs = RealFileSystem::new();
/// let exists = fs.exists(Path::new("."));
/// assert!(exists); // Current directory always exists
/// ```
#[derive(Debug, Clone, Default)]
pub struct RealFileSystem;

impl RealFileSystem {
    /// Creates a new real filesystem provider.
    pub fn new() -> Self {
        Self
    }
}

impl FileSystem for RealFileSystem {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn write(&self, path: &Path, contents: &str) -> io::Result<()> {
        std::fs::write(path, contents)
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }

    fn remove_dir(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_dir(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(from, to)
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::copy(from, to).map(|_| ())
    }

    fn metadata(&self, path: &Path) -> io::Result<FileMetadata> {
        let meta = std::fs::metadata(path)?;
        Ok(FileMetadata {
            is_dir: meta.is_dir(),
            modified: meta.modified()?,
        })
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let entries: io::Result<Vec<PathBuf>> = std::fs::read_dir(path)?
            .map(|entry| entry.map(|e| e.path()))
            .collect();
        entries
    }
}

/// In-memory filesystem for testing.
///
/// Provides a complete in-memory filesystem that tracks files, directories,
/// and their contents. Thread-safe via internal `Arc<RwLock>`.
///
/// # Examples
///
/// ```
/// use keyrx_daemon::traits::filesystem::{FileSystem, MockFileSystem};
/// use std::path::Path;
///
/// let mut fs = MockFileSystem::new();
/// fs.add_file("/test.txt", "content");
///
/// assert!(fs.exists(Path::new("/test.txt")));
/// assert_eq!(fs.read_to_string(Path::new("/test.txt")).unwrap(), "content");
/// ```
#[derive(Debug, Clone)]
pub struct MockFileSystem {
    files: Arc<RwLock<HashMap<PathBuf, String>>>,
    dirs: Arc<RwLock<HashMap<PathBuf, SystemTime>>>,
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MockFileSystem {
    /// Creates a new mock filesystem.
    pub fn new() -> Self {
        let mut dirs = HashMap::new();
        // Root directory always exists
        dirs.insert(PathBuf::from("/"), SystemTime::now());

        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
            dirs: Arc::new(RwLock::new(dirs)),
        }
    }

    /// Adds a file to the mock filesystem.
    ///
    /// # Arguments
    ///
    /// * `path` - File path
    /// * `content` - File contents
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_daemon::traits::filesystem::MockFileSystem;
    ///
    /// let mut fs = MockFileSystem::new();
    /// fs.add_file("/config/test.rhai", "layer(\"base\", #{});");
    /// ```
    pub fn add_file(&mut self, path: impl AsRef<Path>, content: impl Into<String>) {
        if let Ok(mut files) = self.files.write() {
            let path_buf = path.as_ref().to_path_buf();

            // Ensure parent directory exists
            if let Some(parent) = path_buf.parent() {
                if let Ok(mut dirs) = self.dirs.write() {
                    dirs.insert(parent.to_path_buf(), SystemTime::now());
                }
            }

            files.insert(path_buf, content.into());
        }
    }

    /// Adds a directory to the mock filesystem.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path
    pub fn add_dir(&mut self, path: impl AsRef<Path>) {
        if let Ok(mut dirs) = self.dirs.write() {
            dirs.insert(path.as_ref().to_path_buf(), SystemTime::now());
        }
    }

    /// Clears all files and directories (except root).
    pub fn clear(&mut self) {
        if let Ok(mut files) = self.files.write() {
            files.clear();
        }
        if let Ok(mut dirs) = self.dirs.write() {
            dirs.clear();
            dirs.insert(PathBuf::from("/"), SystemTime::now());
        }
    }

    /// Gets a file's content for inspection.
    pub fn get_file_content(&self, path: &Path) -> Option<String> {
        self.files
            .read()
            .ok()
            .and_then(|files| files.get(path).cloned())
    }
}

impl FileSystem for MockFileSystem {
    fn exists(&self, path: &Path) -> bool {
        self.files
            .read()
            .ok()
            .map(|files| files.contains_key(path))
            .unwrap_or(false)
            || self
                .dirs
                .read()
                .ok()
                .map(|dirs| dirs.contains_key(path))
                .unwrap_or(false)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        self.files
            .read()
            .ok()
            .and_then(|files| files.get(path).cloned())
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "file not found"))
    }

    fn write(&self, path: &Path, contents: &str) -> io::Result<()> {
        // Check parent directory exists
        if let Some(parent) = path.parent() {
            if !self.exists(parent) {
                return Err(io::Error::new(
                    ErrorKind::NotFound,
                    "parent directory does not exist",
                ));
            }
        }

        self.files
            .write()
            .ok()
            .map(|mut files| {
                files.insert(path.to_path_buf(), contents.to_string());
            })
            .ok_or_else(|| io::Error::other("lock error"))
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        self.dirs
            .write()
            .ok()
            .map(|mut dirs| {
                // Create all parent directories
                let mut current = PathBuf::from("/");
                for component in path.components() {
                    current.push(component);
                    dirs.insert(current.clone(), SystemTime::now());
                }
            })
            .ok_or_else(|| io::Error::other("lock error"))
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        self.files
            .write()
            .ok()
            .and_then(|mut files| files.remove(path))
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "file not found"))
            .map(|_| ())
    }

    fn remove_dir(&self, path: &Path) -> io::Result<()> {
        self.dirs
            .write()
            .ok()
            .and_then(|mut dirs| dirs.remove(path))
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "directory not found"))
            .map(|_| ())
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        // Try file first
        if let Ok(mut files) = self.files.write() {
            if let Some(content) = files.remove(from) {
                files.insert(to.to_path_buf(), content);
                return Ok(());
            }
        }

        // Try directory
        if let Ok(mut dirs) = self.dirs.write() {
            if let Some(time) = dirs.remove(from) {
                dirs.insert(to.to_path_buf(), time);
                return Ok(());
            }
        }

        Err(io::Error::new(ErrorKind::NotFound, "path not found"))
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        let content = self.read_to_string(from)?;
        self.write(to, &content)
    }

    fn metadata(&self, path: &Path) -> io::Result<FileMetadata> {
        // Check if it's a file
        if let Ok(files) = self.files.read() {
            if files.contains_key(path) {
                return Ok(FileMetadata {
                    is_dir: false,
                    modified: SystemTime::now(),
                });
            }
        }

        // Check if it's a directory
        if let Ok(dirs) = self.dirs.read() {
            if let Some(&modified) = dirs.get(path) {
                return Ok(FileMetadata {
                    is_dir: true,
                    modified,
                });
            }
        }

        Err(io::Error::new(ErrorKind::NotFound, "path not found"))
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        // Check directory exists
        if !self
            .dirs
            .read()
            .ok()
            .map(|dirs| dirs.contains_key(path))
            .unwrap_or(false)
        {
            return Err(io::Error::new(ErrorKind::NotFound, "directory not found"));
        }

        let mut entries = Vec::new();

        // Find all files in this directory
        if let Ok(files) = self.files.read() {
            for file_path in files.keys() {
                if let Some(parent) = file_path.parent() {
                    if parent == path {
                        entries.push(file_path.clone());
                    }
                }
            }
        }

        // Find all subdirectories
        if let Ok(dirs) = self.dirs.read() {
            for dir_path in dirs.keys() {
                if let Some(parent) = dir_path.parent() {
                    if parent == path && dir_path != path {
                        entries.push(dir_path.clone());
                    }
                }
            }
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_fs_exists() {
        let fs = RealFileSystem::new();
        // Current directory should always exist
        assert!(fs.exists(Path::new(".")));
    }

    #[test]
    fn test_mock_fs_add_and_read_file() {
        let mut fs = MockFileSystem::new();
        fs.add_file("/test.txt", "hello world");

        assert!(fs.exists(Path::new("/test.txt")));
        assert_eq!(
            fs.read_to_string(Path::new("/test.txt")).unwrap(),
            "hello world"
        );
    }

    #[test]
    fn test_mock_fs_write_and_read() {
        let mut fs = MockFileSystem::new();
        fs.add_dir("/config");

        fs.write(Path::new("/config/test.rhai"), "layer(\"base\", #{});")
            .unwrap();

        let content = fs.read_to_string(Path::new("/config/test.rhai")).unwrap();
        assert_eq!(content, "layer(\"base\", #{});");
    }

    #[test]
    fn test_mock_fs_write_without_parent_fails() {
        let fs = MockFileSystem::new();

        let result = fs.write(Path::new("/nonexistent/test.txt"), "content");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_fs_create_dir_all() {
        let fs = MockFileSystem::new();

        fs.create_dir_all(Path::new("/a/b/c")).unwrap();

        assert!(fs.exists(Path::new("/a")));
        assert!(fs.exists(Path::new("/a/b")));
        assert!(fs.exists(Path::new("/a/b/c")));
    }

    #[test]
    fn test_mock_fs_remove_file() {
        let mut fs = MockFileSystem::new();
        fs.add_file("/test.txt", "content");

        assert!(fs.exists(Path::new("/test.txt")));

        fs.remove_file(Path::new("/test.txt")).unwrap();

        assert!(!fs.exists(Path::new("/test.txt")));
    }

    #[test]
    fn test_mock_fs_rename_file() {
        let mut fs = MockFileSystem::new();
        fs.add_file("/old.txt", "content");

        fs.rename(Path::new("/old.txt"), Path::new("/new.txt"))
            .unwrap();

        assert!(!fs.exists(Path::new("/old.txt")));
        assert!(fs.exists(Path::new("/new.txt")));
        assert_eq!(fs.read_to_string(Path::new("/new.txt")).unwrap(), "content");
    }

    #[test]
    fn test_mock_fs_copy_file() {
        let mut fs = MockFileSystem::new();
        fs.add_file("/source.txt", "data");
        fs.add_dir("/dest");

        fs.copy(Path::new("/source.txt"), Path::new("/dest/copy.txt"))
            .unwrap();

        assert!(fs.exists(Path::new("/source.txt")));
        assert!(fs.exists(Path::new("/dest/copy.txt")));
        assert_eq!(
            fs.read_to_string(Path::new("/dest/copy.txt")).unwrap(),
            "data"
        );
    }

    #[test]
    fn test_mock_fs_metadata() {
        let mut fs = MockFileSystem::new();
        fs.add_file("/file.txt", "content");
        fs.add_dir("/dir");

        let file_meta = fs.metadata(Path::new("/file.txt")).unwrap();
        assert!(!file_meta.is_dir);

        let dir_meta = fs.metadata(Path::new("/dir")).unwrap();
        assert!(dir_meta.is_dir);
    }

    #[test]
    fn test_mock_fs_read_dir() {
        let mut fs = MockFileSystem::new();
        fs.add_dir("/config");
        fs.add_file("/config/a.rhai", "content1");
        fs.add_file("/config/b.rhai", "content2");
        fs.add_dir("/config/subdir");

        let entries = fs.read_dir(Path::new("/config")).unwrap();
        assert_eq!(entries.len(), 3);

        let entry_strs: Vec<String> = entries
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        assert!(entry_strs.contains(&"/config/a.rhai".to_string()));
        assert!(entry_strs.contains(&"/config/b.rhai".to_string()));
        assert!(entry_strs.contains(&"/config/subdir".to_string()));
    }

    #[test]
    fn test_mock_fs_clear() {
        let mut fs = MockFileSystem::new();
        fs.add_file("/test.txt", "content");
        fs.add_dir("/dir");

        fs.clear();

        assert!(!fs.exists(Path::new("/test.txt")));
        assert!(!fs.exists(Path::new("/dir")));
        // Root should still exist
        assert!(fs.exists(Path::new("/")));
    }

    #[test]
    fn test_mock_fs_default() {
        let fs = MockFileSystem::default();
        assert!(fs.exists(Path::new("/")));
    }

    #[test]
    fn test_real_fs_default() {
        let fs = RealFileSystem::default();
        assert!(fs.exists(Path::new(".")));
    }
}
