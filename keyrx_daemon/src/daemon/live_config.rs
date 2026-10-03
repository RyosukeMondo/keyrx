//! Which configuration is live — the ONE resolver used at startup and on reload.
//!
//! # Model
//!
//! - `run` (no `--config`) starts from the **active profile**
//!   (`<config_dir>/.active` → `<config_dir>/profiles/<name>.krx`). No active
//!   profile, or one that fails to load, means **no config is live: the daemon
//!   grabs no keyboard** (it keeps serving IPC/web so the profile can be fixed,
//!   and status reports `config_error`). This is what the systemd unit and the
//!   desktop entries use, so the profile chosen in the UI survives restarts.
//! - `run --config FILE` is an **explicit override for this run's startup**: FILE
//!   is loaded instead of the active profile. If FILE is a profile's `.krx`, it is
//!   reported as that profile.
//! - At runtime, activating a profile (REST, MCP, WS-RPC, IPC, CLI) **switches**
//!   the daemon to that profile, whatever it started from.
//! - Any other reload (SIGHUP, active profile's config saved, tray "Reload")
//!   re-reads a pinned `--config` file, or else re-resolves the active profile.
//!
//! [`LiveConfig`] remembers what is loaded; the daemon publishes it to
//! [`DaemonSharedState`](super::DaemonSharedState) only after the new
//! remapping state is in place, so status never reports a profile that is not
//! actually remapping keys.

use std::fs;
use std::path::{Path, PathBuf};

use keyrx_core::config::DeviceConfig;
use log::info;

use super::DaemonError;
use crate::config_loader::{load_config, rebuild_if_stale};

/// Where the daemon should take its configuration from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSource {
    /// The profile named in `<config_dir>/.active` (nothing live if none).
    ActiveProfile,
    /// A named profile: `<config_dir>/profiles/<name>.krx`.
    Profile(String),
    /// An explicit `.krx` file (`run --config FILE`).
    File(PathBuf),
}

/// A configuration that was successfully read from disk.
#[derive(Debug, Clone)]
pub struct LoadedConfig {
    /// Profile name, when the file belongs to a profile.
    pub profile: Option<String>,
    /// The `.krx` file the mappings came from.
    pub path: PathBuf,
    /// Every `device_start` block, in declaration order (first match wins).
    pub devices: Vec<DeviceConfig>,
}

/// Tracks which configuration is live and resolves reloads against it.
#[derive(Debug, Clone)]
pub struct LiveConfig {
    config_dir: PathBuf,
    loaded: Option<LoadedConfig>,
}

impl LiveConfig {
    /// Creates a tracker with nothing loaded (no keyboard grabbed).
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            loaded: None,
        }
    }

    /// The keyrx config directory (`~/.config/keyrx`).
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// What is currently loaded (`None` = nothing live).
    pub fn loaded(&self) -> Option<&LoadedConfig> {
        self.loaded.as_ref()
    }

    /// Records `loaded` as live. Call only after the remapping state was swapped.
    pub fn set_loaded(&mut self, loaded: Option<LoadedConfig>) {
        self.loaded = loaded;
    }

    /// The source a reload should read: the requested profile when one was
    /// activated; a pinned `--config` file (not a profile) is re-read as is;
    /// otherwise the active profile, so a profile-based daemon follows
    /// `.active` (CLI activation + SIGHUP, deleting the active profile).
    pub fn reload_source(&self, activation: Option<String>) -> ConfigSource {
        if let Some(name) = activation {
            return ConfigSource::Profile(name);
        }
        match &self.loaded {
            Some(LoadedConfig {
                profile: None,
                path,
                ..
            }) => ConfigSource::File(path.clone()),
            _ => ConfigSource::ActiveProfile,
        }
    }

    /// Reads `source` from disk. `Ok(None)` means no config is live (no active profile).
    pub fn load(&self, source: &ConfigSource) -> Result<Option<LoadedConfig>, DaemonError> {
        match source {
            ConfigSource::ActiveProfile => match read_active_profile_name(&self.config_dir)? {
                Some(name) => self.load_profile(&name).map(Some),
                None => Ok(None),
            },
            ConfigSource::Profile(name) => self.load_profile(name).map(Some),
            ConfigSource::File(path) => {
                let profile = self.profile_name_of(path);
                load_krx(path, profile).map(Some)
            }
        }
    }

    /// Loads profile `name`, first rebuilding its `.krx` from the `.rhai`
    /// source when needed (see [`rebuild_if_stale`]).
    fn load_profile(&self, name: &str) -> Result<LoadedConfig, DaemonError> {
        let krx = self.profile_krx_path(name);
        rebuild_if_stale(&krx)?;
        load_krx(&krx, Some(name.to_string()))
    }

    fn profile_krx_path(&self, name: &str) -> PathBuf {
        self.config_dir.join("profiles").join(format!("{name}.krx"))
    }

    /// Returns the profile name if `path` is `<config_dir>/profiles/<name>.krx`.
    fn profile_name_of(&self, path: &Path) -> Option<String> {
        let name = path.file_stem()?.to_str()?;
        let same = |a: &Path, b: &Path| match (a.canonicalize(), b.canonicalize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => a == b,
        };
        same(path, &self.profile_krx_path(name)).then(|| name.to_string())
    }
}

/// Reads the profile name from `<config_dir>/.active` (JSON `{"name":..}` or
/// the legacy plain-text name). `Ok(None)` when no profile is active.
///
/// This is the ONE resolver for "the active profile" - `run` (no
/// `--config`) uses it via [`ConfigSource::ActiveProfile`], and the
/// `simulate`/`test` CLI commands use it directly so "defaults to the
/// active profile" (as their `--help` says) means the same profile the
/// daemon would actually run.
pub fn read_active_profile_name(config_dir: &Path) -> Result<Option<String>, DaemonError> {
    let active_file = config_dir.join(".active");
    if !active_file.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&active_file).map_err(|e| {
        DaemonError::RuntimeError(format!("failed to read {}: {e}", active_file.display()))
    })?;
    let content = content.trim();
    let name = match serde_json::from_str::<serde_json::Value>(content) {
        Ok(json) => json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        Err(_) => content.to_string(),
    };
    Ok((!name.is_empty()).then_some(name))
}

/// Loads a compiled `.krx` file. A missing file or a file without device
/// configurations is an error: the caller keeps whatever was live before.
fn load_krx(path: &Path, profile: Option<String>) -> Result<LoadedConfig, DaemonError> {
    let config = load_config(path)?;
    if config.devices.is_empty() {
        return Err(DaemonError::RuntimeError(format!(
            "{} has no device configurations",
            path.display()
        )));
    }
    info!(
        "Loaded {} mappings in {} device block(s) from {} (profile: {})",
        config
            .devices
            .iter()
            .map(|d| d.mappings.len())
            .sum::<usize>(),
        config.devices.len(),
        path.display(),
        profile.as_deref().unwrap_or("-")
    );
    Ok(LoadedConfig {
        profile,
        path: path.to_path_buf(),
        devices: config.devices,
    })
}

#[cfg(test)]
#[path = "live_config_tests.rs"]
mod tests;
