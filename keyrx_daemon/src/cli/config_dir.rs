//! Common configuration directory resolution for CLI commands.
//!
//! This module provides a single source of truth for determining the configuration
//! directory used by KeyRx. It checks environment variables in the following order:
//!
//! 1. `KEYRX_CONFIG_DIR` - Explicit override (cross-platform, used by tests)
//! 2. `XDG_CONFIG_HOME` - XDG Base Directory Specification (Linux)
//! 3. `HOME`/`USERPROFILE` - User home directory (all platforms)
//!
//! The final config directory is `$HOME/.config/keyrx` on all platforms.

use std::path::PathBuf;

/// Get the KeyRx configuration directory.
///
/// Priority order:
/// 1. `KEYRX_CONFIG_DIR` - Explicit override (for testing and custom setups)
/// 2. `XDG_CONFIG_HOME/keyrx` - XDG standard on Linux
/// 3. `$HOME/.config/keyrx` or `%USERPROFILE%\.config\keyrx` - Default fallback
///
/// # Returns
///
/// The configuration directory path if it can be determined.
///
/// # Errors
///
/// Returns an error if no home directory can be determined from environment variables.
///
/// # Examples
///
/// ```no_run
/// use keyrx_daemon::cli::config_dir::get_config_dir;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config_dir = get_config_dir()?;
/// println!("Config directory: {}", config_dir.display());
/// # Ok(())
/// # }
/// ```
pub fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    resolve_config_dir(|name| std::env::var(name).ok(), dirs::config_dir())
        .ok_or_else(|| "Could not determine home directory".into())
}

/// The resolution rules of [`get_config_dir`] over an injected environment
/// (`env`) and platform config dir (`platform_config`, i.e.
/// `dirs::config_dir()`), so they are testable without mutating process env.
fn resolve_config_dir(
    env: impl Fn(&str) -> Option<String>,
    platform_config: Option<PathBuf>,
) -> Option<PathBuf> {
    // 1. Explicit override (tests and custom setups)
    if let Some(dir) = env("KEYRX_CONFIG_DIR") {
        return Some(PathBuf::from(dir));
    }
    // 2. XDG_CONFIG_HOME (Linux standard)
    if cfg!(target_os = "linux") {
        if let Some(xdg) = env("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg).join("keyrx"));
        }
    }
    // 3. Windows: %APPDATA%\keyrx
    if cfg!(target_os = "windows") {
        if let Some(config) = platform_config {
            return Some(config.join("keyrx"));
        }
    }
    // 4. $HOME/.config/keyrx
    let home = env("HOME").or_else(|| env("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".config").join("keyrx"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn resolve(vars: &[(&str, &str)]) -> Option<PathBuf> {
        let vars: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        resolve_config_dir(|k| vars.get(k).cloned(), Some(PathBuf::from("/appdata")))
    }

    #[test]
    fn test_keyrx_config_dir_override_wins() {
        let dir = resolve(&[
            ("KEYRX_CONFIG_DIR", "/custom/config"),
            ("XDG_CONFIG_HOME", "/should/not/be/used"),
            ("HOME", "/home/u"),
        ]);
        assert_eq!(dir, Some(PathBuf::from("/custom/config")));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_xdg_config_home() {
        let dir = resolve(&[("XDG_CONFIG_HOME", "/xdg/config"), ("HOME", "/home/u")]);
        assert_eq!(dir, Some(PathBuf::from("/xdg/config/keyrx")));
    }

    #[test]
    #[cfg(unix)]
    fn test_home_fallback() {
        assert_eq!(
            resolve(&[("HOME", "/home/testuser")]),
            Some(PathBuf::from("/home/testuser/.config/keyrx"))
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_uses_appdata() {
        assert_eq!(
            resolve(&[("USERPROFILE", "C:\\Users\\u")]),
            Some(PathBuf::from("/appdata").join("keyrx"))
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_no_home_is_none() {
        assert_eq!(resolve(&[]), None);
    }
}
