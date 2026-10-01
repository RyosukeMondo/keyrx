//! Layer 2: Business logic execution.

use crate::config::profile_manager::ProfileManager;
use crate::config::profile_manager::{ProfileError, ProfileMetadata};
use crate::config::rhai_generator::{GeneratorError, KeyAction, RhaiGenerator};
use crate::error::{CliError, ConfigError, DaemonResult};
use std::path::PathBuf;

/// Service layer for profile operations.
pub struct ProfileService {
    manager: ProfileManager,
}

impl ProfileService {
    /// Creates a new profile service.
    pub fn new(config_dir: PathBuf) -> DaemonResult<Self> {
        let manager =
            ProfileManager::new(config_dir.clone()).map_err(|e| CliError::CommandFailed {
                command: "config".to_string(),
                reason: format!("Failed to initialize profile manager: {}", e),
            })?;

        manager
            .scan_profiles()
            .map_err(|e| CliError::CommandFailed {
                command: "config".to_string(),
                reason: format!("Failed to scan profiles: {}", e),
            })?;

        Ok(Self { manager })
    }

    /// Gets the profile name (from argument or active profile).
    pub fn get_profile_name(&self, profile: Option<String>) -> DaemonResult<String> {
        if let Some(name) = profile {
            Ok(name)
        } else if let Ok(Some(active)) = self.manager.get_active() {
            Ok(active)
        } else {
            Err(CliError::InvalidArguments {
                reason: "No active profile. Use --profile to specify one.".to_string(),
            }
            .into())
        }
    }

    /// Applies a key mapping operation.
    pub fn apply_key_mapping(
        &mut self,
        profile_name: &str,
        layer: &str,
        key: &str,
        action: KeyAction,
    ) -> DaemonResult<u64> {
        self.edit_source(profile_name, "set-key", |gen| {
            gen.set_key_mapping(layer, key, action)
        })
    }

    /// Deletes a key mapping.
    pub fn delete_key_mapping(
        &mut self,
        profile_name: &str,
        layer: &str,
        key: &str,
    ) -> DaemonResult<u64> {
        self.edit_source(profile_name, "delete-key", |gen| {
            gen.delete_key_mapping(layer, key)
        })
    }

    /// Edits a profile's source and commits it only if the result compiles.
    ///
    /// The edit is applied in memory, and `ProfileManager::set_config` (the
    /// path REST and RPC saves take too) compiles a temp copy first and only
    /// then replaces the `.rhai`/`.krx` pair atomically - a rejected edit
    /// leaves both files untouched. Returns the compile time in ms.
    fn edit_source(
        &mut self,
        profile_name: &str,
        command: &str,
        edit: impl FnOnce(&mut RhaiGenerator) -> Result<(), GeneratorError>,
    ) -> DaemonResult<u64> {
        let meta = self.profile(profile_name)?;
        let mut gen =
            RhaiGenerator::load(&meta.rhai_path).map_err(|e| ConfigError::ParseError {
                path: meta.rhai_path.clone(),
                reason: e.to_string(),
            })?;
        edit(&mut gen).map_err(|e| CliError::CommandFailed {
            command: command.to_string(),
            reason: e.to_string(),
        })?;
        let start = std::time::Instant::now();
        self.manager
            .set_config(profile_name, &gen.to_string())
            .map_err(|e| match e {
                ProfileError::Compilation(c) => ConfigError::CompilationFailed {
                    reason: c.to_string(),
                },
                other => ConfigError::InvalidProfile {
                    name: profile_name.to_string(),
                    reason: other.to_string(),
                },
            })?;
        Ok(start.elapsed().as_millis() as u64)
    }

    fn profile(&self, name: &str) -> DaemonResult<ProfileMetadata> {
        Ok(self
            .manager
            .get(name)
            .ok_or_else(|| ConfigError::InvalidProfile {
                name: name.to_string(),
                reason: "Profile not found".to_string(),
            })?)
    }

    /// Gets a key mapping as string.
    pub fn get_key_mapping(
        &self,
        profile_name: &str,
        layer: &str,
        key: &str,
    ) -> DaemonResult<Option<String>> {
        let meta = self.profile(profile_name)?;
        let gen = RhaiGenerator::load(&meta.rhai_path).map_err(|e| ConfigError::ParseError {
            path: meta.rhai_path.clone(),
            reason: e.to_string(),
        })?;
        Ok(gen
            .find_mapping(layer, key)
            .map_err(|e| CliError::CommandFailed {
                command: "get-key".to_string(),
                reason: e.to_string(),
            })?)
    }

    /// Validates a profile by compiling it in memory (no file is written).
    pub fn validate_profile(&self, profile_name: &str) -> DaemonResult<()> {
        let meta = self.profile(profile_name)?;
        crate::config::ProfileCompiler::new()
            .parse(&meta.rhai_path)
            .map(|_| ())
            .map_err(|e| {
                ConfigError::CompilationFailed {
                    reason: e.to_string(),
                }
                .into()
            })
    }

    /// Gets profile metadata.
    pub fn get_profile_info(
        &self,
        profile_name: &str,
    ) -> DaemonResult<(String, Vec<String>, usize)> {
        let profile_meta =
            self.manager
                .get(profile_name)
                .ok_or_else(|| ConfigError::InvalidProfile {
                    name: profile_name.to_string(),
                    reason: "Profile not found".to_string(),
                })?;

        let content = std::fs::read_to_string(&profile_meta.rhai_path).map_err(|e| {
            ConfigError::ParseError {
                path: profile_meta.rhai_path.clone(),
                reason: format!("Failed to read profile: {}", e),
            }
        })?;

        let device_id = extract_device_id(&content).unwrap_or_else(|| "*".to_string());
        let layers = extract_layer_list(&content);
        let mapping_count = count_mappings(&content);

        Ok((device_id, layers, mapping_count))
    }

    /// Compares two profiles.
    pub fn compare_profiles(&self, profile1: &str, profile2: &str) -> DaemonResult<Vec<String>> {
        let meta1 = self
            .manager
            .get(profile1)
            .ok_or_else(|| ConfigError::InvalidProfile {
                name: profile1.to_string(),
                reason: "Profile not found".to_string(),
            })?;

        let meta2 = self
            .manager
            .get(profile2)
            .ok_or_else(|| ConfigError::InvalidProfile {
                name: profile2.to_string(),
                reason: "Profile not found".to_string(),
            })?;

        let content1 =
            std::fs::read_to_string(&meta1.rhai_path).map_err(|e| ConfigError::ParseError {
                path: meta1.rhai_path.clone(),
                reason: format!("Failed to read {}: {}", profile1, e),
            })?;

        let content2 =
            std::fs::read_to_string(&meta2.rhai_path).map_err(|e| ConfigError::ParseError {
                path: meta2.rhai_path.clone(),
                reason: format!("Failed to read {}: {}", profile2, e),
            })?;

        Ok(compute_diff(&content1, &content2))
    }
}

// Helper functions for parsing Rhai content

fn extract_device_id(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("device_start(") {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return Some(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

fn extract_layer_list(content: &str) -> Vec<String> {
    let mut layers = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("when_start(") {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    layers.push(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    layers
}

fn count_mappings(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("map(") || trimmed.starts_with("tap_hold(")
        })
        .count()
}

fn compute_diff(content1: &str, content2: &str) -> Vec<String> {
    let lines1: Vec<&str> = content1.lines().collect();
    let lines2: Vec<&str> = content2.lines().collect();
    let mut differences = Vec::new();

    let max_len = lines1.len().max(lines2.len());
    for i in 0..max_len {
        let line1 = lines1.get(i).copied().unwrap_or("");
        let line2 = lines2.get(i).copied().unwrap_or("");

        if line1 != line2 {
            if !line1.is_empty() && !line2.is_empty() {
                differences.push(format!("Line {}: '{}' -> '{}'", i + 1, line1, line2));
            } else if line2.is_empty() {
                differences.push(format!("- Line {}: '{}'", i + 1, line1));
            } else {
                differences.push(format!("+ Line {}: '{}'", i + 1, line2));
            }
        }
    }

    differences
}
