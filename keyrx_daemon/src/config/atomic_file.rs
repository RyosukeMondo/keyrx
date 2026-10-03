//! The one way the daemon replaces a file it owns (profiles, the `.active`
//! pointer, the device registry).
//!
//! `fs::write` truncates the target first, so a full disk or a crash
//! half-way leaves an empty or partial file - for `.active` that means the
//! keyboard silently stops being remapped after the next restart. Writing to
//! a sibling temp file and renaming makes the replacement all-or-nothing, and
//! a failed attempt removes its temp file instead of leaving litter behind.

use std::io::Write;
use std::path::{Path, PathBuf};

/// The sibling temp file for `path` (same directory, so the rename is atomic).
fn temp_sibling(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    path.with_file_name(name)
}

/// Replaces `path` with `bytes`, all or nothing.
///
/// # Errors
/// Any I/O error (disk full, read-only filesystem, permissions); `path` is
/// then untouched and no temp file remains.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temp = temp_sibling(path);
    let result = (|| {
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_the_file_and_leaves_no_temp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.json");
        write_atomic(&path, b"one").unwrap();
        write_atomic(&path, b"two").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"two");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    /// A failure keeps the previous content and cleans up (here the rename
    /// fails because the target is a directory).
    #[test]
    fn a_failed_write_keeps_the_old_target_and_removes_the_temp() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("t");
        std::fs::create_dir(&target).unwrap();
        assert!(write_atomic(&target, b"x").is_err());
        assert!(target.is_dir());
        assert!(!temp_sibling(&target).exists());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn a_full_disk_leaves_the_previous_content() {
        // /dev/full accepts the open and fails every write with ENOSPC.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("keep");
        write_atomic(&path, b"good").unwrap();
        let temp = temp_sibling(&path);
        std::os::unix::fs::symlink("/dev/full", &temp).unwrap();
        let err = write_atomic(&path, b"new content").unwrap_err();
        assert_eq!(err.raw_os_error(), Some(28), "{err}");
        assert_eq!(std::fs::read(&path).unwrap(), b"good");
    }
}
