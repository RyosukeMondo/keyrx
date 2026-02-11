# Dependency Injection Traits

This module provides abstraction traits for testability by decoupling code from external dependencies like environment variables and filesystem operations.

## Overview

The traits enable:
- **100% unit test coverage** without touching real filesystem or environment
- **Deterministic tests** - no flaky tests from environment state
- **Parallel test execution** - tests don't conflict via shared state
- **Simplified mocking** - no need for complex test setup/teardown

## Available Traits

### EnvProvider

Abstracts `std::env::var()` access.

**Implementations:**
- `RealEnvProvider` - Production use (delegates to `std::env`)
- `MockEnvProvider` - Testing (in-memory HashMap)

**Example:**

```rust
use keyrx_daemon::traits::{EnvProvider, MockEnvProvider};

let mut env = MockEnvProvider::new();
env.set("KEYRX_CONFIG_DIR", "/tmp/test");

assert_eq!(env.var("KEYRX_CONFIG_DIR").unwrap(), "/tmp/test");
```

### FileSystem

Abstracts filesystem operations: read, write, create_dir, remove, rename, copy, metadata, read_dir.

**Implementations:**
- `RealFileSystem` - Production use (delegates to `std::fs`)
- `MockFileSystem` - Testing (in-memory filesystem)

**Example:**

```rust
use keyrx_daemon::traits::{FileSystem, MockFileSystem};
use std::path::Path;

let mut fs = MockFileSystem::new();
fs.add_dir("/config");
fs.write(Path::new("/config/test.rhai"), "layer(\"base\", #{});").unwrap();

assert!(fs.exists(Path::new("/config/test.rhai")));
```

## Refactoring Pattern

### Before (Hard to Test)

```rust
pub struct ProfileManager {
    config_dir: PathBuf,
}

impl ProfileManager {
    pub fn new(config_dir: PathBuf) -> Result<Self, ProfileError> {
        // Direct filesystem access - hard to test!
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        Ok(Self { config_dir })
    }
}
```

### After (Testable)

```rust
use keyrx_daemon::traits::{EnvProvider, FileSystem};

pub struct ProfileManager<E: EnvProvider, F: FileSystem> {
    config_dir: PathBuf,
    env: E,
    fs: F,
}

impl<E: EnvProvider, F: FileSystem> ProfileManager<E, F> {
    pub fn new(config_dir: PathBuf, env: E, fs: F) -> Result<Self, ProfileError> {
        // Injected filesystem - easy to test!
        if !fs.exists(&config_dir) {
            fs.create_dir_all(&config_dir)?;
        }
        Ok(Self { config_dir, env, fs })
    }
}
```

### Production Usage

```rust
use keyrx_daemon::config::ProfileManager;
use keyrx_daemon::traits::{RealEnvProvider, RealFileSystem};

let manager = ProfileManager::new(
    config_dir,
    RealEnvProvider::new(),
    RealFileSystem::new()
)?;
```

### Test Usage

```rust
use keyrx_daemon::config::ProfileManager;
use keyrx_daemon::traits::{MockEnvProvider, MockFileSystem};

#[test]
fn test_profile_creation() {
    let mut env = MockEnvProvider::new();
    env.set("HOME", "/home/test");

    let mut fs = MockFileSystem::new();
    fs.add_dir("/home/test/.config/keyrx");

    let manager = ProfileManager::new(
        PathBuf::from("/home/test/.config/keyrx"),
        env,
        fs
    ).unwrap();

    // Test without touching real filesystem!
}
```

## Refactoring Checklist

For each service using `std::env` or `std::fs`:

1. **Add generic parameters** for `EnvProvider` and `FileSystem`
2. **Store trait instances** as struct fields
3. **Replace direct calls**:
   - `std::env::var()` → `self.env.var()`
   - `std::fs::*()` → `self.fs.*()`
4. **Update constructors** to accept injected dependencies
5. **Update production code** to inject `RealEnvProvider` and `RealFileSystem`
6. **Update tests** to inject `MockEnvProvider` and `MockFileSystem`

## Benefits

### Testability

```rust
// Before: Hard to test, requires real filesystem
#[test]
fn test_profile_manager() {
    let temp_dir = tempdir().unwrap(); // Requires cleanup
    let manager = ProfileManager::new(temp_dir.path().to_path_buf()).unwrap();
    // Test leaks state to filesystem
}

// After: Easy to test, fully isolated
#[test]
fn test_profile_manager() {
    let mut fs = MockFileSystem::new();
    fs.add_dir("/config");

    let manager = ProfileManager::new(
        PathBuf::from("/config"),
        MockEnvProvider::new(),
        fs
    ).unwrap();
    // No filesystem access, no cleanup needed
}
```

### Determinism

```rust
// Before: Flaky - depends on actual environment
#[test]
fn test_config_dir() {
    std::env::set_var("HOME", "/test"); // Affects other tests!
    let dir = get_config_dir();
    std::env::remove_var("HOME"); // Cleanup required
}

// After: Deterministic - isolated environment
#[test]
fn test_config_dir() {
    let mut env = MockEnvProvider::new();
    env.set("HOME", "/test");

    let dir = get_config_dir(&env);
    // No global state, no cleanup
}
```

### Speed

- **MockFileSystem** operations are ~100x faster than real disk I/O
- Tests can run in parallel without conflicts
- No need for temp directory creation/cleanup

## Architecture Notes

### Thread Safety

Both `MockEnvProvider` and `MockFileSystem` are thread-safe via `Arc<RwLock<_>>`:

```rust
#[derive(Clone)]
pub struct MockEnvProvider {
    vars: Arc<RwLock<HashMap<String, String>>>,
}
```

This allows:
- Cloning for use across threads
- Concurrent reads (via `RwLock`)
- Safe mutation in tests

### Type Erasure

For cases where generics are inconvenient, use trait objects:

```rust
pub struct ProfileManager {
    env: Box<dyn EnvProvider>,
    fs: Box<dyn FileSystem>,
}

impl ProfileManager {
    pub fn new(
        env: Box<dyn EnvProvider>,
        fs: Box<dyn FileSystem>
    ) -> Self {
        Self { env, fs }
    }
}
```

**Trade-offs:**
- ✅ Simpler API (no generics in signatures)
- ❌ Slight runtime overhead (dynamic dispatch)
- ❌ No compile-time optimization

Prefer generics for performance-critical code, trait objects for API simplicity.

## Migration Examples

### config_dir.rs

**Before:**

```rust
pub fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(dir) = std::env::var("KEYRX_CONFIG_DIR") {
        return Ok(PathBuf::from(dir));
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Could not determine home directory")?;

    Ok(PathBuf::from(home).join(".config").join("keyrx"))
}
```

**After:**

```rust
use crate::traits::EnvProvider;

pub fn get_config_dir<E: EnvProvider>(env: &E) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(dir) = env.var("KEYRX_CONFIG_DIR") {
        return Ok(PathBuf::from(dir));
    }

    let home = env.var("HOME")
        .or_else(|_| env.var("USERPROFILE"))
        .map_err(|_| "Could not determine home directory")?;

    Ok(PathBuf::from(home).join(".config").join("keyrx"))
}
```

### ProfileManager

See `src/config/profile_manager.rs` for full refactoring example (to be implemented in next phase).

## Testing Guidelines

### Unit Tests

Use mocks exclusively:

```rust
#[test]
fn test_create_profile() {
    let mut fs = MockFileSystem::new();
    fs.add_dir("/config/profiles");

    let mut manager = ProfileManager::new(
        PathBuf::from("/config"),
        MockEnvProvider::new(),
        fs
    ).unwrap();

    manager.create("test", ProfileTemplate::Blank).unwrap();

    // Verify via mock filesystem
    assert!(manager.fs.exists(Path::new("/config/profiles/test.rhai")));
}
```

### Integration Tests

Use real implementations for end-to-end validation:

```rust
#[test]
fn test_profile_workflow_e2e() {
    let temp_dir = tempdir().unwrap();

    let manager = ProfileManager::new(
        temp_dir.path().to_path_buf(),
        RealEnvProvider::new(),
        RealFileSystem::new()
    ).unwrap();

    // Real filesystem operations
    manager.create("production", ProfileTemplate::Gaming).unwrap();
}
```

## Future Enhancements

Potential additional traits:

- **TimeProvider** - Abstract `SystemTime::now()` for time-dependent tests
- **NetworkProvider** - Abstract HTTP/WebSocket for network tests
- **ProcessProvider** - Abstract process spawning for subprocess tests

## References

- [Dependency Injection in Rust](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- [Test Doubles in Rust](https://rust-lang-nursery.github.io/rust-cookbook/testing/mocking.html)
- [SOLID Principles](https://en.wikipedia.org/wiki/SOLID)
