# Dependency Injection Traits - Quick Reference

## Import

```rust
use keyrx_daemon::traits::{
    EnvProvider, RealEnvProvider, MockEnvProvider,
    FileSystem, RealFileSystem, MockFileSystem,
};
```

## EnvProvider

### Production
```rust
let env = RealEnvProvider::new();
let home = env.var("HOME")?;
```

### Testing
```rust
let mut env = MockEnvProvider::new();
env.set("HOME", "/home/test");
env.set("KEYRX_CONFIG_DIR", "/custom");

assert_eq!(env.var("HOME")?, "/home/test");
env.remove("HOME");
env.clear(); // Remove all
```

## FileSystem

### Production
```rust
let fs = RealFileSystem::new();
fs.create_dir_all(Path::new("/config"))?;
fs.write(Path::new("/config/test.txt"), "content")?;
```

### Testing
```rust
let mut fs = MockFileSystem::new();
fs.add_dir("/config");
fs.add_file("/config/test.txt", "content");

assert!(fs.exists(Path::new("/config/test.txt")));
let content = fs.read_to_string(Path::new("/config/test.txt"))?;
```

## Refactoring Pattern

### Before
```rust
pub struct MyService {
    config_dir: PathBuf,
}

impl MyService {
    pub fn new(dir: PathBuf) -> Self {
        fs::create_dir_all(&dir).unwrap();
        Self { config_dir: dir }
    }

    pub fn load(&self) -> String {
        fs::read_to_string(&self.config_dir.join("file.txt")).unwrap()
    }
}
```

### After
```rust
use crate::traits::FileSystem;

pub struct MyService<F: FileSystem> {
    config_dir: PathBuf,
    fs: F,
}

impl<F: FileSystem> MyService<F> {
    pub fn new(dir: PathBuf, fs: F) -> Self {
        fs.create_dir_all(&dir).unwrap();
        Self { config_dir: dir, fs }
    }

    pub fn load(&self) -> String {
        self.fs.read_to_string(&self.config_dir.join("file.txt")).unwrap()
    }
}
```

### Usage

**Production:**
```rust
let service = MyService::new(
    PathBuf::from("/config"),
    RealFileSystem::new()
);
```

**Testing:**
```rust
let mut fs = MockFileSystem::new();
fs.add_dir("/config");
fs.add_file("/config/file.txt", "test content");

let service = MyService::new(PathBuf::from("/config"), fs);
assert_eq!(service.load(), "test content");
```

## Type Aliases

```rust
// Add to your module for convenience
pub type RealMyService = MyService<RealFileSystem>;
pub type MockMyService = MyService<MockFileSystem>;
```

## Complete Example

```rust
use keyrx_daemon::traits::{EnvProvider, FileSystem, MockEnvProvider, MockFileSystem};
use std::path::{Path, PathBuf};

struct ConfigManager<E: EnvProvider, F: FileSystem> {
    env: E,
    fs: F,
}

impl<E: EnvProvider, F: FileSystem> ConfigManager<E, F> {
    fn new(env: E, fs: F) -> Self {
        Self { env, fs }
    }

    fn get_config_dir(&self) -> Result<PathBuf, String> {
        if let Ok(dir) = self.env.var("KEYRX_CONFIG_DIR") {
            Ok(PathBuf::from(dir))
        } else {
            let home = self.env.var("HOME")
                .map_err(|_| "HOME not set")?;
            Ok(PathBuf::from(home).join(".config/keyrx"))
        }
    }

    fn ensure_config_exists(&self) -> Result<(), std::io::Error> {
        let dir = self.get_config_dir().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::NotFound, e)
        })?;

        if !self.fs.exists(&dir) {
            self.fs.create_dir_all(&dir)?;
        }

        Ok(())
    }
}

#[test]
fn test_config_manager() {
    let mut env = MockEnvProvider::new();
    env.set("HOME", "/home/test");

    let fs = MockFileSystem::new();

    let manager = ConfigManager::new(env, fs.clone());

    manager.ensure_config_exists().unwrap();

    assert!(fs.exists(Path::new("/home/test/.config/keyrx")));
}
```

## All FileSystem Methods

| Method | Description |
|--------|-------------|
| `exists(path)` | Check if path exists |
| `read_to_string(path)` | Read file as string |
| `write(path, content)` | Write string to file |
| `create_dir_all(path)` | Create directory and parents |
| `remove_file(path)` | Delete file |
| `remove_dir(path)` | Delete directory |
| `rename(from, to)` | Move/rename file or directory |
| `copy(from, to)` | Copy file |
| `metadata(path)` | Get file metadata (is_dir, modified) |
| `read_dir(path)` | List directory contents |

## Testing Best Practices

### ✅ Do
- Use mocks in unit tests
- Test error conditions easily
- Run tests in parallel
- Keep tests fast and deterministic

### ❌ Don't
- Use `tempdir()` in unit tests (use mocks instead)
- Modify `std::env` directly in tests
- Rely on filesystem state
- Use `serial_test` unless necessary

## Documentation

- **Full guide:** `src/traits/README.md`
- **Refactoring steps:** `docs/development/di-refactoring-guide.md`
- **Phase summary:** `docs/development/PHASE_1.2_SUMMARY.md`
