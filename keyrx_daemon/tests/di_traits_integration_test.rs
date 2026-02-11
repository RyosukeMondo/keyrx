//! Integration tests for dependency injection traits.
//!
//! These tests demonstrate how EnvProvider and FileSystem traits enable
//! testable code without touching the real environment or filesystem.

use keyrx_daemon::traits::{EnvProvider, FileSystem, MockEnvProvider, MockFileSystem};
use std::path::Path;

#[test]
fn test_env_provider_mock_integration() {
    let mut env = MockEnvProvider::new();

    // Simulate environment setup
    env.set("KEYRX_CONFIG_DIR", "/tmp/keyrx_test");
    env.set("HOME", "/home/testuser");

    // Verify mock behavior
    assert_eq!(env.var("KEYRX_CONFIG_DIR").unwrap(), "/tmp/keyrx_test");
    assert_eq!(env.var("HOME").unwrap(), "/home/testuser");
    assert!(env.var("NONEXISTENT").is_err());
}

#[test]
fn test_filesystem_mock_integration() {
    let mut fs = MockFileSystem::new();

    // Create directory structure
    fs.add_dir("/config");
    fs.add_dir("/config/profiles");

    // Add files
    fs.add_file(
        "/config/profiles/default.rhai",
        r#"layer("base", #{
    "KEY_A": simple("KEY_B"),
});"#,
    );

    // Verify filesystem operations
    assert!(fs.exists(Path::new("/config")));
    assert!(fs.exists(Path::new("/config/profiles")));
    assert!(fs.exists(Path::new("/config/profiles/default.rhai")));

    let content = fs
        .read_to_string(Path::new("/config/profiles/default.rhai"))
        .unwrap();
    assert!(content.contains("KEY_A"));
    assert!(content.contains("KEY_B"));
}

#[test]
fn test_filesystem_write_and_modify() {
    let mut fs = MockFileSystem::new();
    fs.add_dir("/config");

    // Write initial content
    fs.write(Path::new("/config/test.rhai"), "initial content")
        .unwrap();

    // Read it back
    let content = fs.read_to_string(Path::new("/config/test.rhai")).unwrap();
    assert_eq!(content, "initial content");

    // Overwrite
    fs.write(Path::new("/config/test.rhai"), "updated content")
        .unwrap();

    let updated = fs.read_to_string(Path::new("/config/test.rhai")).unwrap();
    assert_eq!(updated, "updated content");
}

#[test]
fn test_filesystem_directory_operations() {
    let fs = MockFileSystem::new();

    // Create nested directories
    fs.create_dir_all(Path::new("/a/b/c/d")).unwrap();

    assert!(fs.exists(Path::new("/a")));
    assert!(fs.exists(Path::new("/a/b")));
    assert!(fs.exists(Path::new("/a/b/c")));
    assert!(fs.exists(Path::new("/a/b/c/d")));
}

#[test]
fn test_filesystem_file_operations() {
    let mut fs = MockFileSystem::new();
    fs.add_dir("/src");
    fs.add_dir("/dest");
    fs.add_file("/src/file.txt", "content");

    // Test copy
    fs.copy(Path::new("/src/file.txt"), Path::new("/dest/copy.txt"))
        .unwrap();

    assert!(fs.exists(Path::new("/src/file.txt")));
    assert!(fs.exists(Path::new("/dest/copy.txt")));
    assert_eq!(
        fs.read_to_string(Path::new("/dest/copy.txt")).unwrap(),
        "content"
    );

    // Test rename
    fs.rename(Path::new("/src/file.txt"), Path::new("/src/renamed.txt"))
        .unwrap();

    assert!(!fs.exists(Path::new("/src/file.txt")));
    assert!(fs.exists(Path::new("/src/renamed.txt")));
}

#[test]
fn test_filesystem_metadata() {
    let mut fs = MockFileSystem::new();
    fs.add_file("/file.txt", "data");
    fs.add_dir("/directory");

    let file_meta = fs.metadata(Path::new("/file.txt")).unwrap();
    assert!(!file_meta.is_dir);

    let dir_meta = fs.metadata(Path::new("/directory")).unwrap();
    assert!(dir_meta.is_dir);
}

#[test]
fn test_filesystem_read_dir() {
    let mut fs = MockFileSystem::new();
    fs.add_dir("/config");
    fs.add_file("/config/a.rhai", "content1");
    fs.add_file("/config/b.rhai", "content2");
    fs.add_dir("/config/subdir");

    let entries = fs.read_dir(Path::new("/config")).unwrap();
    assert_eq!(entries.len(), 3);

    // Verify all expected paths are present
    let paths: Vec<String> = entries
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    assert!(paths.contains(&"/config/a.rhai".to_string()));
    assert!(paths.contains(&"/config/b.rhai".to_string()));
    assert!(paths.contains(&"/config/subdir".to_string()));
}

#[test]
fn test_combined_env_and_fs() {
    // Simulate a complete config directory resolution scenario
    let mut env = MockEnvProvider::new();
    let mut fs = MockFileSystem::new();

    // Setup environment
    env.set("KEYRX_CONFIG_DIR", "/custom/config");

    // Setup filesystem
    let config_dir = env.var("KEYRX_CONFIG_DIR").unwrap();
    fs.add_dir(&config_dir);
    fs.add_dir(format!("{}/profiles", config_dir));
    fs.add_file(
        format!("{}/profiles/default.rhai", config_dir),
        "layer(\"base\", #{});",
    );

    // Verify the setup
    assert!(fs.exists(Path::new("/custom/config")));
    assert!(fs.exists(Path::new("/custom/config/profiles")));
    assert!(fs.exists(Path::new("/custom/config/profiles/default.rhai")));
}

#[test]
fn test_error_handling() {
    let fs = MockFileSystem::new();

    // Reading nonexistent file should error
    let result = fs.read_to_string(Path::new("/nonexistent.txt"));
    assert!(result.is_err());

    // Writing to nonexistent parent should error
    let result = fs.write(Path::new("/nonexistent/file.txt"), "data");
    assert!(result.is_err());

    // Removing nonexistent file should error
    let result = fs.remove_file(Path::new("/nonexistent.txt"));
    assert!(result.is_err());
}

#[test]
fn test_env_provider_remove_and_clear() {
    let mut env = MockEnvProvider::new();

    env.set("VAR1", "value1");
    env.set("VAR2", "value2");

    // Remove one variable
    env.remove("VAR1");
    assert!(env.var("VAR1").is_err());
    assert!(env.var("VAR2").is_ok());

    // Clear all
    env.clear();
    assert!(env.var("VAR2").is_err());
}

#[test]
fn test_filesystem_remove_operations() {
    let mut fs = MockFileSystem::new();
    fs.add_dir("/test");
    fs.add_file("/test/file.txt", "content");

    // Remove file
    fs.remove_file(Path::new("/test/file.txt")).unwrap();
    assert!(!fs.exists(Path::new("/test/file.txt")));

    // Directory should still exist
    assert!(fs.exists(Path::new("/test")));

    // Remove directory
    fs.remove_dir(Path::new("/test")).unwrap();
    assert!(!fs.exists(Path::new("/test")));
}
