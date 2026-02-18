//! Layer 2: Business logic execution.

use crate::config::profile_manager::ProfileManager;
use crate::config::rhai_generator::{KeyAction, RhaiGenerator};
use crate::error::{CliError, ConfigError, DaemonResult};
use std::path::{Path, PathBuf};

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
        let profile_meta =
            self.manager
                .get(profile_name)
                .ok_or_else(|| ConfigError::InvalidProfile {
                    name: profile_name.to_string(),
                    reason: "Profile not found".to_string(),
                })?;

        let mut gen =
            RhaiGenerator::load(&profile_meta.rhai_path).map_err(|e| ConfigError::ParseError {
                path: profile_meta.rhai_path.clone(),
                reason: e.to_string(),
            })?;

        gen.set_key_mapping(layer, key, action)
            .map_err(|e| CliError::CommandFailed {
                command: "set-key".to_string(),
                reason: e.to_string(),
            })?;

        gen.save(&profile_meta.rhai_path)
            .map_err(|e| ConfigError::ParseError {
                path: profile_meta.rhai_path.clone(),
                reason: format!("Failed to save: {}", e),
            })?;

        self.compile_profile(&profile_meta.rhai_path, &profile_meta.krx_path)
    }

    /// Compiles a profile and returns compilation time.
    fn compile_profile(&self, rhai_path: &Path, krx_path: &Path) -> DaemonResult<u64> {
        let compile_start = std::time::Instant::now();
        keyrx_compiler::compile_file(rhai_path, krx_path).map_err(|e| {
            ConfigError::CompilationFailed {
                reason: e.to_string(),
            }
        })?;
        Ok(compile_start.elapsed().as_millis() as u64)
    }

    /// Deletes a key mapping.
    pub fn delete_key_mapping(
        &mut self,
        profile_name: &str,
        layer: &str,
        key: &str,
    ) -> DaemonResult<u64> {
        let profile_meta =
            self.manager
                .get(profile_name)
                .ok_or_else(|| ConfigError::InvalidProfile {
                    name: profile_name.to_string(),
                    reason: "Profile not found".to_string(),
                })?;

        let mut gen =
            RhaiGenerator::load(&profile_meta.rhai_path).map_err(|e| ConfigError::ParseError {
                path: profile_meta.rhai_path.clone(),
                reason: e.to_string(),
            })?;

        gen.delete_key_mapping(layer, key)
            .map_err(|e| CliError::CommandFailed {
                command: "delete-key".to_string(),
                reason: e.to_string(),
            })?;

        gen.save(&profile_meta.rhai_path)
            .map_err(|e| ConfigError::ParseError {
                path: profile_meta.rhai_path.clone(),
                reason: format!("Failed to save: {}", e),
            })?;

        self.compile_profile(&profile_meta.rhai_path, &profile_meta.krx_path)
    }

    /// Gets a key mapping as string.
    pub fn get_key_mapping(
        &self,
        profile_name: &str,
        layer: &str,
        key: &str,
    ) -> DaemonResult<Option<String>> {
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

        Ok(find_key_mapping(&content, key, layer))
    }

    /// Validates a profile by dry-run compilation.
    pub fn validate_profile(&self, profile_name: &str) -> DaemonResult<()> {
        let profile_meta =
            self.manager
                .get(profile_name)
                .ok_or_else(|| ConfigError::InvalidProfile {
                    name: profile_name.to_string(),
                    reason: "Profile not found".to_string(),
                })?;

        let temp_output = profile_meta.krx_path.with_extension("tmp.krx");
        let result = keyrx_compiler::compile_file(&profile_meta.rhai_path, &temp_output);
        let _ = std::fs::remove_file(&temp_output);

        result.map_err(|e| {
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

fn find_key_mapping(content: &str, key: &str, layer: &str) -> Option<String> {
    let mut current_layer = "base";
    let mut in_when_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("when_start(") {
            in_when_block = true;
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    current_layer = &trimmed[start + 1..start + 1 + end];
                }
            }
        } else if trimmed.starts_with("when_end()") {
            in_when_block = false;
            current_layer = "base";
        } else if (current_layer == layer || (layer == "base" && !in_when_block))
            && (trimmed.starts_with("map(") || trimmed.starts_with("tap_hold("))
        {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    let first_key = &trimmed[start + 1..start + 1 + end];
                    if first_key == key {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
    }

    None
}

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
