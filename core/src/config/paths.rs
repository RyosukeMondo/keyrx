//! File system path constants.
//!
//! This module provides constants for paths used throughout KeyRx,
//! including device paths, config directories, and file names.
//!
//! # Path Categories
//!
//! - **Device paths**: System device files (uinput on Linux)
//! - **Config paths**: User configuration directories and file names
//! - **Runtime paths**: Temporary files and history
//!
//! # XDG Base Directory Support
//!
//! Helper functions follow the XDG Base Directory Specification where applicable,
//! falling back to sensible defaults when XDG variables are not set.

use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;
use std::sync::RwLock;

// =============================================================================
// Linux Device Paths
// =============================================================================

/// Path to the uinput device file on Linux.
///
/// This is the kernel interface for creating virtual input devices.
/// Requires write access; see `LinuxInput::check_uinput_accessible()` for
/// permission requirements.
///
/// # Platform
/// Linux only.
#[cfg(target_os = "linux")]
pub const UINPUT_PATH: &str = "/dev/uinput";

/// Name of the virtual keyboard device created via uinput.
///
/// This is the device name shown in `/dev/input/by-id/` and in tools
/// like `evtest`. Used to identify KeyRx's virtual keyboard.
///
/// # Platform
/// Linux only.
#[cfg(target_os = "linux")]
pub const UINPUT_DEVICE_NAME: &str = "KeyRx Virtual Keyboard";

// =============================================================================
// Config File Names
// =============================================================================

/// Default configuration file name.
///
/// KeyRx looks for this file in the config directory to load user settings.
/// If not found, built-in defaults are used.
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// Default directory name for user scripts.
///
/// Scripts are loaded from `{config_dir}/scripts/` by default.
pub const SCRIPTS_DIR: &str = "scripts";

/// Default directory name for runtime caches.
pub const CACHE_DIR_NAME: &str = ".keyrx_cache";

/// Subdirectory for compiled script caches.
pub const SCRIPT_CACHE_DIR: &str = "scripts";

// =============================================================================
// History and Temporary Files
// =============================================================================

/// REPL command history file name.
///
/// Stored in the user's home directory to persist command history
/// across REPL sessions.
pub const REPL_HISTORY_FILE: &str = ".keyrx_repl_history";

/// Performance baseline file path (relative to project root).
///
/// Used by the performance UAT system to store baseline measurements
/// for regression detection.
pub const PERF_BASELINE_FILE: &str = "target/perf-baseline.json";

// =============================================================================
// Path Resolution Functions
// =============================================================================

lazy_static! {
    static ref CONFIG_ROOT_OVERRIDE: RwLock<Option<PathBuf>> = RwLock::new(None);
}

/// Set a runtime override for the configuration root directory.
///
/// This allows the embedding application to specify a custom location for
/// configuration files, bypassing standard XDG/system paths.
///
/// # Arguments
///
/// * `path` - The new configuration root directory path.
pub fn set_config_root(path: PathBuf) {
    if let Ok(mut lock) = CONFIG_ROOT_OVERRIDE.write() {
        *lock = Some(path);
    }
}

/// Reset the configuration root directory override.
///
/// This clears any override set by `set_config_root`, causing the configuration
/// system to fall back to standard XDG/system paths.
///
/// # Testing
/// This is primarily useful for cleanup during testing.
pub fn clear_config_root() {
    if let Ok(mut lock) = CONFIG_ROOT_OVERRIDE.write() {
        *lock = None;
    }
}

/// Resolve the KeyRx configuration directory.
///
/// Preference order:
/// 1. Runtime override (set via `set_config_root`)
/// 2. `$XDG_CONFIG_HOME/keyrx`
/// 3. Platform-specific config dir (e.g. `%APPDATA%` on Windows, `~/.config` on Linux)
/// 4. `$HOME/.config/keyrx` (Legacy fallback)
/// 5. `.config/keyrx` relative to CWD (last-resort fallback)
///
/// # Returns
///
/// Path to the KeyRx configuration directory.
///
/// # Example
///
/// ```
/// use keyrx_core::config::config_dir;
///
/// let dir = config_dir();
/// println!("Config directory: {}", dir.display());
/// ```
pub fn config_dir() -> PathBuf {
    // Check for runtime override first
    if let Ok(lock) = CONFIG_ROOT_OVERRIDE.read() {
        if let Some(path) = lock.as_ref() {
            return path.clone();
        }
    }

    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("keyrx");
    }

    // Use dirs crate for robust platform-standard paths
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("keyrx");
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(".config").join("keyrx");
    }

    PathBuf::from(".").join(".config").join("keyrx")
}

/// Resolve the device profiles directory.
///
/// Preference order:
/// 1. `$XDG_CONFIG_HOME/keyrx/devices`
/// 2. `$HOME/.config/keyrx/devices`
/// 3. `.config/keyrx/devices` relative to CWD (last-resort fallback)
///
/// This is a convenience wrapper that appends "devices" to [`config_dir()`].
///
/// # Returns
///
/// Path to the device profiles directory.
pub fn device_profiles_dir() -> PathBuf {
    config_dir().join("devices")
}

/// Resolve the scripts directory.
///
/// Returns `{config_dir}/scripts` where user Rhai scripts are stored.
///
/// # Returns
///
/// Path to the scripts directory.
pub fn scripts_dir() -> PathBuf {
    config_dir().join(SCRIPTS_DIR)
}

/// Resolve the cache root directory.
///
/// Returns `$HOME/.keyrx_cache` when a home directory exists, otherwise
/// falls back to a relative `.keyrx_cache` path.
fn cache_dir() -> PathBuf {
    home_dir()
        .map(|h| h.join(CACHE_DIR_NAME))
        .unwrap_or_else(|| PathBuf::from(CACHE_DIR_NAME))
}

/// Resolve the cache directory for compiled scripts.
///
/// Returns `$HOME/.keyrx_cache/scripts` when a home directory exists,
/// otherwise falls back to a relative `.keyrx_cache/scripts` path.
pub fn script_cache_dir() -> PathBuf {
    cache_dir().join(SCRIPT_CACHE_DIR)
}

/// Get the user's home directory.
///
/// Preference order:
/// 1. `$HOME` (Unix/Linux/macOS)
/// 2. `$USERPROFILE` (Windows)
/// 3. `None` if neither is set
///
/// # Returns
///
/// Optional path to the user's home directory.
pub fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Get the REPL history file path.
///
/// Returns `$HOME/.keyrx_repl_history` if home directory is available,
/// otherwise `None`.
///
/// # Returns
///
/// Optional path to the REPL history file.
pub fn repl_history_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(REPL_HISTORY_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::tempdir;

    #[test]
    #[serial]
    fn config_dir_prefers_override() {
        // Reset override for clean state (though serial test helps)
        if let Ok(mut lock) = CONFIG_ROOT_OVERRIDE.write() {
            *lock = None;
        }

        let temp = tempdir().unwrap();
        let override_path = temp.path().join("override");

        set_config_root(override_path.clone());

        assert_eq!(config_dir(), override_path);

        // Clean up
        if let Ok(mut lock) = CONFIG_ROOT_OVERRIDE.write() {
            *lock = None;
        }
    }

    #[test]
    #[serial]
    fn config_dir_prefers_xdg_config_home() {
        // Ensure override is cleared
        if let Ok(mut lock) = CONFIG_ROOT_OVERRIDE.write() {
            *lock = None;
        }

        let temp = tempdir().unwrap();
        let prev_xdg = env::var("XDG_CONFIG_HOME").ok();
        let prev_home = env::var("HOME").ok();

        unsafe {
            env::set_var("XDG_CONFIG_HOME", temp.path());
            env::remove_var("HOME");
        }

        let path = config_dir();

        #[cfg(windows)]
        assert!(path
            .to_string_lossy()
            .to_lowercase()
            .starts_with(&temp.path().to_string_lossy().to_lowercase()));
        #[cfg(not(windows))]
        assert!(path.starts_with(temp.path()));
        assert!(path.ends_with("keyrx"));

        unsafe {
            match prev_xdg {
                Some(val) => env::set_var("XDG_CONFIG_HOME", val),
                None => env::remove_var("XDG_CONFIG_HOME"),
            }
        }
        if let Some(home) = prev_home {
            unsafe {
                env::set_var("HOME", home);
            }
        }
    }

    #[test]
    #[serial]
    #[cfg(not(windows))]
    fn config_dir_falls_back_to_home() {
        // Ensure override is cleared
        if let Ok(mut lock) = CONFIG_ROOT_OVERRIDE.write() {
            *lock = None;
        }
        let temp = tempdir().unwrap();
        let prev_xdg = env::var("XDG_CONFIG_HOME").ok();
        let prev_home = env::var("HOME").ok();

        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
            env::set_var("HOME", temp.path());
        }

        let path = config_dir();

        #[cfg(windows)]
        assert!(path
            .to_string_lossy()
            .to_lowercase()
            .starts_with(&temp.path().to_string_lossy().to_lowercase()));
        #[cfg(not(windows))]
        assert!(path.starts_with(temp.path()));
        assert!(path.ends_with(PathBuf::from(".config").join("keyrx")));

        if let Some(xdg) = prev_xdg {
            unsafe {
                env::set_var("XDG_CONFIG_HOME", xdg);
            }
        }
        unsafe {
            match prev_home {
                Some(val) => env::set_var("HOME", val),
                None => env::remove_var("HOME"),
            }
        }
    }

    #[test]
    #[serial]
    fn device_profiles_dir_is_subdir_of_config() {
        // Ensure override is cleared
        if let Ok(mut lock) = CONFIG_ROOT_OVERRIDE.write() {
            *lock = None;
        }

        // Ensure stable environment for this test
        let prev_xdg = env::var("XDG_CONFIG_HOME").ok();
        let prev_home = env::var("HOME").ok();

        // Set a known HOME to ensure consistent behavior
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
            env::set_var("HOME", "/tmp/test_home");
        }

        let config = config_dir();
        let devices = device_profiles_dir();

        #[cfg(windows)]
        assert!(devices
            .to_string_lossy()
            .to_lowercase()
            .starts_with(&config.to_string_lossy().to_lowercase()));
        #[cfg(not(windows))]
        assert!(devices.starts_with(&config));
        assert!(devices.ends_with("devices"));

        // Restore environment
        unsafe {
            match prev_xdg {
                Some(val) => env::set_var("XDG_CONFIG_HOME", val),
                None => env::remove_var("XDG_CONFIG_HOME"),
            }
        }
        unsafe {
            match prev_home {
                Some(val) => env::set_var("HOME", val),
                None => env::remove_var("HOME"),
            }
        }
    }

    #[test]
    #[serial]
    fn scripts_dir_is_subdir_of_config() {
        // Ensure override is cleared
        if let Ok(mut lock) = CONFIG_ROOT_OVERRIDE.write() {
            *lock = None;
        }

        // Ensure stable environment for this test
        let prev_xdg = env::var("XDG_CONFIG_HOME").ok();
        let prev_home = env::var("HOME").ok();

        // Set a known HOME to ensure consistent behavior
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
            env::set_var("HOME", "/tmp/test_home");
        }

        let config = config_dir();
        let scripts = scripts_dir();

        #[cfg(windows)]
        assert!(scripts
            .to_string_lossy()
            .to_lowercase()
            .starts_with(&config.to_string_lossy().to_lowercase()));
        #[cfg(not(windows))]
        assert!(scripts.starts_with(&config));
        assert!(scripts.ends_with(SCRIPTS_DIR));

        // Restore environment
        unsafe {
            match prev_xdg {
                Some(val) => env::set_var("XDG_CONFIG_HOME", val),
                None => env::remove_var("XDG_CONFIG_HOME"),
            }
        }
        unsafe {
            match prev_home {
                Some(val) => env::set_var("HOME", val),
                None => env::remove_var("HOME"),
            }
        }
    }

    #[test]
    #[serial]
    fn script_cache_dir_uses_home_when_available() {
        let temp = tempdir().unwrap();
        let prev_home = env::var("HOME").ok();

        unsafe {
            env::set_var("HOME", temp.path());
        }

        let path = script_cache_dir();
        assert!(path.ends_with(PathBuf::from(CACHE_DIR_NAME).join(SCRIPT_CACHE_DIR)));
        assert_eq!(
            path.parent().and_then(|p| p.file_name()),
            Some(std::ffi::OsStr::new(CACHE_DIR_NAME))
        );

        unsafe {
            match prev_home {
                Some(val) => env::set_var("HOME", val),
                None => env::remove_var("HOME"),
            }
        }
    }

    #[test]
    #[serial]
    fn home_dir_returns_home() {
        let temp = tempdir().unwrap();
        let prev_home = env::var("HOME").ok();

        unsafe {
            env::set_var("HOME", temp.path());
        }

        let home = home_dir();
        assert!(home.is_some());
        assert_eq!(home.unwrap(), temp.path());

        unsafe {
            match prev_home {
                Some(val) => env::set_var("HOME", val),
                None => env::remove_var("HOME"),
            }
        }
    }

    #[test]
    #[serial]
    fn repl_history_path_returns_path_when_home_set() {
        let temp = tempdir().unwrap();
        let prev_home = env::var("HOME").ok();

        unsafe {
            env::set_var("HOME", temp.path());
        }

        let history = repl_history_path();
        assert!(history.is_some());
        let path = history.unwrap();

        #[cfg(windows)]
        assert!(path
            .to_string_lossy()
            .to_lowercase()
            .starts_with(&temp.path().to_string_lossy().to_lowercase()));
        #[cfg(not(windows))]
        assert!(path.starts_with(temp.path()));
        assert!(path.ends_with(REPL_HISTORY_FILE));

        unsafe {
            match prev_home {
                Some(val) => env::set_var("HOME", val),
                None => env::remove_var("HOME"),
            }
        }
    }

    #[test]
    fn constants_have_expected_values() {
        assert_eq!(CONFIG_FILE_NAME, "config.toml");
        assert_eq!(SCRIPTS_DIR, "scripts");
        assert_eq!(CACHE_DIR_NAME, ".keyrx_cache");
        assert_eq!(SCRIPT_CACHE_DIR, "scripts");
        assert_eq!(REPL_HISTORY_FILE, ".keyrx_repl_history");
        assert_eq!(PERF_BASELINE_FILE, "target/perf-baseline.json");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_constants_have_expected_values() {
        assert_eq!(UINPUT_PATH, "/dev/uinput");
        assert_eq!(UINPUT_DEVICE_NAME, "KeyRx Virtual Keyboard");
    }
}
