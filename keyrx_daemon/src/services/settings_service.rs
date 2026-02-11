//! Settings management service.
//!
//! This service provides global daemon settings management including
//! default keyboard layout configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Re-export from SSOT (daemon_config)
pub use crate::daemon_config::DEFAULT_PORT;

/// Global daemon settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSettings {
    /// Default keyboard layout for newly detected devices
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_layout: Option<String>,

    /// Web server port (default: 9867)
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

impl Default for DaemonSettings {
    fn default() -> Self {
        Self {
            global_layout: None,
            port: DEFAULT_PORT,
        }
    }
}

/// Settings management service
pub struct SettingsService {
    settings_path: PathBuf,
}

impl SettingsService {
    /// Create a new SettingsService with the given config directory
    pub fn new(config_dir: PathBuf) -> Self {
        let settings_path = config_dir.join("settings.json");
        Self { settings_path }
    }

    /// Load settings from disk (public for startup use)
    pub fn load_settings(&self) -> Result<DaemonSettings, String> {
        match std::fs::read_to_string(&self.settings_path) {
            Ok(contents) => serde_json::from_str(&contents)
                .map_err(|e| format!("Failed to parse settings: {}", e)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File doesn't exist, return default settings
                Ok(DaemonSettings::default())
            }
            Err(e) => Err(format!("Failed to read settings file: {}", e)),
        }
    }

    /// Save settings to disk
    fn save_settings(&self, settings: &DaemonSettings) -> Result<(), String> {
        // Ensure parent directory exists
        if let Some(parent) = self.settings_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(&self.settings_path, json)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        log::info!("Saved settings to {:?}", self.settings_path);
        Ok(())
    }

    /// Get global layout setting
    pub async fn get_global_layout(&self) -> Result<Option<String>, String> {
        let settings = self.load_settings()?;
        Ok(settings.global_layout)
    }

    /// Set global layout setting
    pub async fn set_global_layout(&self, layout: Option<String>) -> Result<(), String> {
        // Validate layout if provided
        if let Some(ref layout_str) = layout {
            validate_layout(layout_str)?;
        }

        let layout_for_log = layout.as_deref().map(String::from);
        let mut settings = self.load_settings()?;
        settings.global_layout = layout;
        self.save_settings(&settings)?;

        log::info!("Set global layout to: {:?}", layout_for_log);
        Ok(())
    }

    /// Get current port setting
    pub fn get_port(&self) -> u16 {
        self.load_settings().map(|s| s.port).unwrap_or(DEFAULT_PORT)
    }

    /// Set port setting (saves to disk)
    pub fn set_port(&self, port: u16) -> Result<(), String> {
        if port == 0 {
            return Err("Port cannot be 0".to_string());
        }

        let mut settings = self.load_settings()?;
        settings.port = port;
        self.save_settings(&settings)?;

        log::info!("Saved port {} to settings", port);
        Ok(())
    }

    /// Get path to settings file
    pub fn settings_path(&self) -> &PathBuf {
        &self.settings_path
    }
}

/// Validate layout name (must be one of the supported presets)
fn validate_layout(layout: &str) -> Result<(), String> {
    const VALID_LAYOUTS: &[&str] = &["ANSI_104", "ISO_105", "JIS_109", "HHKB", "NUMPAD"];

    if layout.is_empty() {
        return Err("Layout name cannot be empty".to_string());
    }

    if layout.len() > 32 {
        return Err(format!(
            "Layout name too long: {} chars (max 32)",
            layout.len()
        ));
    }

    if !VALID_LAYOUTS.contains(&layout) {
        return Err(format!(
            "Invalid layout '{}'. Must be one of: {}",
            layout,
            VALID_LAYOUTS.join(", ")
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_default_settings() {
        let temp_dir = TempDir::new().unwrap();
        let service = SettingsService::new(temp_dir.path().to_path_buf());

        let layout = service.get_global_layout().await.unwrap();
        assert_eq!(layout, None);
    }

    #[tokio::test]
    async fn test_set_and_get_global_layout() {
        let temp_dir = TempDir::new().unwrap();
        let service = SettingsService::new(temp_dir.path().to_path_buf());

        service
            .set_global_layout(Some("ANSI_104".to_string()))
            .await
            .unwrap();
        let layout = service.get_global_layout().await.unwrap();
        assert_eq!(layout, Some("ANSI_104".to_string()));
    }

    #[tokio::test]
    async fn test_persist_settings() {
        let temp_dir = TempDir::new().unwrap();
        let service = SettingsService::new(temp_dir.path().to_path_buf());

        service
            .set_global_layout(Some("ISO_105".to_string()))
            .await
            .unwrap();

        // Create new service instance to test persistence
        let service2 = SettingsService::new(temp_dir.path().to_path_buf());
        let layout = service2.get_global_layout().await.unwrap();
        assert_eq!(layout, Some("ISO_105".to_string()));
    }

    #[tokio::test]
    async fn test_clear_global_layout() {
        let temp_dir = TempDir::new().unwrap();
        let service = SettingsService::new(temp_dir.path().to_path_buf());

        service
            .set_global_layout(Some("ANSI_104".to_string()))
            .await
            .unwrap();
        service.set_global_layout(None).await.unwrap();

        let layout = service.get_global_layout().await.unwrap();
        assert_eq!(layout, None);
    }

    #[test]
    fn test_validate_layout_valid() {
        assert!(validate_layout("ANSI_104").is_ok());
        assert!(validate_layout("ISO_105").is_ok());
        assert!(validate_layout("JIS_109").is_ok());
        assert!(validate_layout("HHKB").is_ok());
        assert!(validate_layout("NUMPAD").is_ok());
    }

    #[test]
    fn test_validate_layout_invalid() {
        assert!(validate_layout("INVALID_LAYOUT").is_err());
        assert!(validate_layout("").is_err());
        assert!(validate_layout(&"a".repeat(33)).is_err());
    }
}
