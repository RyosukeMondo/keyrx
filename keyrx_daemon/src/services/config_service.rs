//! Configuration service providing business logic for config operations.
//!
//! This service acts as a single source of truth for configuration operations,
//! shared between CLI and Web API. It provides operations for reading, updating,
//! and manipulating Rhai configuration files.

use std::fs;
use std::sync::Arc;

use crate::config::rhai_generator::{GeneratorError, KeyAction, RhaiGenerator};
use crate::config::ProfileManager;
use crate::services::ProfileService;

/// Configuration information returned by get_config.
#[derive(Debug, Clone)]
pub struct ConfigInfo {
    pub code: String,
    pub hash: String,
    pub profile: String,
}

/// Layer information.
#[derive(Debug, Clone)]
pub struct LayerInfo {
    pub id: String,
    pub mapping_count: usize,
}

/// Errors that can occur during configuration operations.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Active profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Configuration file not found")]
    FileNotFound,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Generator error: {0}")]
    GeneratorError(#[from] GeneratorError),

    #[error("Configuration too large (max 1MB)")]
    ConfigTooLarge,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Layer not found: {0}")]
    LayerNotFound(String),

    #[error("Invalid key name: {0}")]
    InvalidKeyName(String),
}

/// Service for configuration operations on the active profile.
///
/// Every write goes through [`ProfileService::set_profile_config`], which
/// compiles the profile and, if the daemon has it loaded, reloads it — so an
/// edit from any transport (REST, WS-RPC) takes effect on the keyboard.
///
/// # Thread Safety
///
/// ConfigService is `Send + Sync` and can be shared across threads via `Arc`.
pub struct ConfigService {
    profile_service: Arc<ProfileService>,
}

impl ConfigService {
    /// Creates a new ConfigService writing through `profile_service`.
    pub fn new(profile_service: Arc<ProfileService>) -> Self {
        log::debug!("ConfigService initialized");
        Self { profile_service }
    }

    fn profile_manager(&self) -> &Arc<ProfileManager> {
        self.profile_service.profile_manager()
    }

    /// Name and parsed source of the active profile.
    pub async fn load_active(&self) -> Result<(String, RhaiGenerator), ConfigError> {
        let (name, path) = self.active_rhai_path()?;
        Ok((name, RhaiGenerator::load(&path)?))
    }

    fn active_rhai_path(&self) -> Result<(String, std::path::PathBuf), ConfigError> {
        let manager = self.profile_manager();
        let active = manager
            .get_active()
            .map_err(|_| ConfigError::ProfileNotFound("failed to get active profile".to_string()))?
            .ok_or_else(|| ConfigError::ProfileNotFound("no active profile".to_string()))?;
        let metadata = manager
            .get(&active)
            .ok_or_else(|| ConfigError::ProfileNotFound(active.clone()))?;
        Ok((active, metadata.rhai_path))
    }

    /// Gets the current configuration for the active profile.
    ///
    /// Returns the Rhai code, its hash, and the profile name.
    pub async fn get_config(&self) -> Result<ConfigInfo, ConfigError> {
        log::debug!("Getting current configuration");
        let (profile, path) = self.active_rhai_path()?;
        let code = fs::read_to_string(&path)?;
        let hash = Self::compute_hash(&code);
        Ok(ConfigInfo {
            code,
            hash,
            profile,
        })
    }

    /// Replaces the active profile's source (max 1MB), compiles it and
    /// reloads the daemon if the profile is loaded.
    pub async fn update_config(&self, code: String) -> Result<(), ConfigError> {
        log::info!("Updating configuration");
        const MAX_CONFIG_SIZE: usize = 1024 * 1024; // 1MB
        if code.len() > MAX_CONFIG_SIZE {
            return Err(ConfigError::ConfigTooLarge);
        }
        // Reject unparseable source before touching the file.
        RhaiGenerator::parse(&code).map_err(|e| ConfigError::InvalidConfig(e.to_string()))?;
        let (profile, _) = self.active_rhai_path()?;
        self.save(&profile, &code).await
    }

    /// Sets a single key mapping in the active profile.
    pub async fn set_key_mapping(
        &self,
        layer: String,
        key: String,
        action: KeyAction,
    ) -> Result<(), ConfigError> {
        log::debug!("Setting key mapping: layer={}, key={}", layer, key);
        let (profile, mut generator) = self.load_active().await?;
        generator
            .set_key_mapping(&layer, &key, action)
            .map_err(Self::map_generator_error)?;
        self.save(&profile, &generator.to_string()).await
    }

    /// Deletes a key mapping from the active profile.
    pub async fn delete_key_mapping(&self, layer: String, key: String) -> Result<(), ConfigError> {
        log::debug!("Deleting key mapping: layer={}, key={}", layer, key);
        let (profile, mut generator) = self.load_active().await?;
        generator
            .delete_key_mapping(&layer, &key)
            .map_err(Self::map_generator_error)?;
        self.save(&profile, &generator.to_string()).await
    }

    /// Gets all layers from the active profile.
    pub async fn get_layers(&self) -> Result<Vec<LayerInfo>, ConfigError> {
        log::debug!("Getting layers");
        let (_, generator) = self.load_active().await?;
        Ok(generator
            .list_layers()
            .into_iter()
            .map(|(id, mapping_count)| LayerInfo { id, mapping_count })
            .collect())
    }

    /// Writes, compiles and (if loaded) reloads `profile`.
    async fn save(&self, profile: &str, source: &str) -> Result<(), ConfigError> {
        self.profile_service
            .set_profile_config(profile, source)
            .await
            .map_err(|e| ConfigError::InvalidConfig(e.to_string()))?;
        log::info!("Configuration of '{profile}' saved and compiled");
        Ok(())
    }

    fn map_generator_error(e: GeneratorError) -> ConfigError {
        match e {
            GeneratorError::LayerNotFound(l) => ConfigError::LayerNotFound(l),
            GeneratorError::InvalidKeyName(k) => ConfigError::InvalidKeyName(k),
            _ => ConfigError::GeneratorError(e),
        }
    }

    /// Computes a hash of the configuration code.
    fn compute_hash(code: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let code = "let x = 42;";
        let hash1 = ConfigService::compute_hash(code);
        let hash2 = ConfigService::compute_hash(code);
        assert_eq!(hash1, hash2);

        let different_code = "let y = 43;";
        let hash3 = ConfigService::compute_hash(different_code);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_config_too_large() {
        // Create a string larger than 1MB
        let large_code = "x".repeat(1024 * 1024 + 1);
        assert!(large_code.len() > 1024 * 1024);
    }
}
