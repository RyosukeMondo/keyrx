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
    // 1. Check for explicit override (used by tests and custom setups)
    if let Ok(dir) = std::env::var("KEYRX_CONFIG_DIR") {
        return Ok(PathBuf::from(dir));
    }

    // 2. Check XDG_CONFIG_HOME (Linux standard)
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(xdg_config).join("keyrx"));
        }
    }

    // 3. Platform-specific default config directory
    //    - Windows: %APPDATA%/keyrx (via dirs::config_dir, typically AppData/Roaming)
    //    - Linux:   $HOME/.config/keyrx
    #[cfg(target_os = "windows")]
    {
        if let Some(config) = dirs::config_dir() {
            return Ok(config.join("keyrx"));
        }
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Could not determine home directory")?;

    Ok(PathBuf::from(home).join(".config").join("keyrx"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_keyrx_config_dir_override() {
        // Save and clear other env vars
        let old_keyrx = env::var("KEYRX_CONFIG_DIR").ok();
        let old_xdg = env::var("XDG_CONFIG_HOME").ok();

        env::set_var("KEYRX_CONFIG_DIR", "/custom/config");
        env::set_var("XDG_CONFIG_HOME", "/should/not/be/used");

        let dir = get_config_dir().unwrap();
        assert_eq!(dir, PathBuf::from("/custom/config"));

        // Restore
        if let Some(val) = old_keyrx {
            env::set_var("KEYRX_CONFIG_DIR", val);
        } else {
            env::remove_var("KEYRX_CONFIG_DIR");
        }
        if let Some(val) = old_xdg {
            env::set_var("XDG_CONFIG_HOME", val);
        } else {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_xdg_config_home() {
        let old_keyrx = env::var("KEYRX_CONFIG_DIR").ok();
        let old_xdg = env::var("XDG_CONFIG_HOME").ok();

        env::remove_var("KEYRX_CONFIG_DIR");
        env::set_var("XDG_CONFIG_HOME", "/xdg/config");

        let dir = get_config_dir().unwrap();
        assert_eq!(dir, PathBuf::from("/xdg/config/keyrx"));

        // Restore
        if let Some(val) = old_keyrx {
            env::set_var("KEYRX_CONFIG_DIR", val);
        }
        if let Some(val) = old_xdg {
            env::set_var("XDG_CONFIG_HOME", val);
        } else {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[test]
    #[serial]
    fn test_home_fallback() {
        // Clear KEYRX_CONFIG_DIR to ensure we test the fallback
        let old_keyrx = env::var("KEYRX_CONFIG_DIR").ok();
        env::remove_var("KEYRX_CONFIG_DIR");

        let old_xdg = env::var("XDG_CONFIG_HOME").ok();
        let old_home = env::var("HOME").ok();
        let old_userprofile = env::var("USERPROFILE").ok();

        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("HOME");
        env::remove_var("USERPROFILE");

        #[cfg(unix)]
        env::set_var("HOME", "/home/testuser");
        #[cfg(windows)]
        env::set_var("USERPROFILE", "C:\\Users\\testuser");

        let dir = get_config_dir().unwrap();

        #[cfg(unix)]
        assert_eq!(dir, PathBuf::from("/home/testuser/.config/keyrx"));
        #[cfg(windows)]
        {
            // When HOME/USERPROFILE are set but dirs::config_dir() returns AppData/Roaming,
            // the result depends on system state. Just verify it ends with "keyrx".
            assert!(
                dir.ends_with("keyrx"),
                "Expected dir ending with keyrx, got {:?}",
                dir
            );
        }

        // Restore
        if let Some(val) = old_keyrx {
            env::set_var("KEYRX_CONFIG_DIR", val);
        }
        if let Some(val) = old_xdg {
            env::set_var("XDG_CONFIG_HOME", val);
        }
        if let Some(val) = old_home {
            env::set_var("HOME", val);
        }
        if let Some(val) = old_userprofile {
            env::set_var("USERPROFILE", val);
        }
    }
}
