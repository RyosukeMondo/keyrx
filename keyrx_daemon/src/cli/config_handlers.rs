//! Command handlers for configuration CLI operations.
//!
//! This module contains the handler functions for various config subcommands
//! such as get-key, validate, show, and diff.

use crate::cli::common::output_error;
use crate::cli::config_helpers::{
    compute_diff, count_mappings, extract_device_id, extract_layer_list, find_key_mapping,
};
use crate::config::profile_manager::ProfileManager;
use crate::config::rhai_generator::RhaiGenerator;
use serde::Serialize;

/// JSON output for get-key operations.
#[derive(Serialize)]
pub struct GetKeyOutput {
    pub key: String,
    pub layer: String,
    pub mapping: Option<String>,
}

/// JSON output for validation.
#[derive(Serialize)]
pub struct ValidationOutput {
    pub success: bool,
    pub profile: String,
    pub errors: Vec<String>,
}

/// JSON output for show command.
#[derive(Serialize)]
pub struct ShowOutput {
    pub profile: String,
    pub device_id: String,
    pub layers: Vec<String>,
    pub mapping_count: usize,
}

/// JSON output for diff command.
#[derive(Serialize)]
pub struct DiffOutput {
    pub profile1: String,
    pub profile2: String,
    pub differences: Vec<String>,
}

/// Handle get-key command.
pub fn handle_get_key(
    manager: &ProfileManager,
    key: String,
    layer: String,
    profile: Option<String>,
    json: bool,
) -> Result<(), i32> {
    let profile_name = get_profile_name(manager, profile)?;
    let profile_meta = match manager.get(&profile_name) {
        Some(meta) => meta,
        None => {
            output_error(&format!("Profile not found: {}", profile_name), 1001, json);
            return Err(1);
        }
    };

    // Load Rhai file and search for the mapping
    let content = match std::fs::read_to_string(&profile_meta.rhai_path) {
        Ok(c) => c,
        Err(e) => {
            output_error(&format!("Failed to read profile: {}", e), 1, json);
            return Err(1);
        }
    };

    // Simple line-based search for the key
    let mapping = find_key_mapping(&content, &key, &layer);

    if json {
        let output = GetKeyOutput {
            key,
            layer,
            mapping,
        };
        // SAFETY: serde_json::to_string cannot fail on simple struct with derived Serialize
        #[allow(clippy::unwrap_used)]
        {
            println!("{}", serde_json::to_string(&output).unwrap());
        }
    } else if let Some(m) = mapping {
        println!("{}", m);
    } else {
        println!("No mapping found for {} in layer '{}'", key, layer);
    }

    Ok(())
}

/// Handle validate command.
pub fn handle_validate(
    manager: &ProfileManager,
    profile: Option<String>,
    json: bool,
) -> Result<(), i32> {
    let profile_name = get_profile_name(manager, profile)?;
    let profile_meta = match manager.get(&profile_name) {
        Some(meta) => meta,
        None => {
            output_error(&format!("Profile not found: {}", profile_name), 1001, json);
            return Err(1);
        }
    };

    // Attempt dry-run compilation
    let temp_output = profile_meta.krx_path.with_extension("tmp.krx");
    let result = keyrx_compiler::compile_file(&profile_meta.rhai_path, &temp_output);

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_output);

    match result {
        Ok(_) => {
            if json {
                let output = ValidationOutput {
                    success: true,
                    profile: profile_name,
                    errors: vec![],
                };
                // SAFETY: serde_json::to_string cannot fail on simple struct with derived Serialize
                #[allow(clippy::unwrap_used)]
                {
                    println!("{}", serde_json::to_string(&output).unwrap());
                }
            } else {
                println!("✓ Profile '{}' is valid", profile_name);
            }
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            if json {
                let output = ValidationOutput {
                    success: false,
                    profile: profile_name,
                    errors: vec![error_msg],
                };
                // SAFETY: serde_json::to_string cannot fail on simple struct with derived Serialize
                #[allow(clippy::unwrap_used)]
                {
                    println!("{}", serde_json::to_string(&output).unwrap());
                }
            } else {
                println!("✗ Profile '{}' validation failed:", profile_name);
                println!("  {}", e);
            }
            Err(1)
        }
    }
}

/// Handle show command.
pub fn handle_show(
    manager: &ProfileManager,
    profile: Option<String>,
    json: bool,
) -> Result<(), i32> {
    let profile_name = get_profile_name(manager, profile)?;
    let profile_meta = match manager.get(&profile_name) {
        Some(meta) => meta,
        None => {
            output_error(&format!("Profile not found: {}", profile_name), 1001, json);
            return Err(1);
        }
    };

    // Load the Rhai file and analyze it
    let _gen = match RhaiGenerator::load(&profile_meta.rhai_path) {
        Ok(g) => g,
        Err(e) => {
            output_error(&format!("Failed to load profile: {}", e), 1, json);
            return Err(1);
        }
    };

    let content = match std::fs::read_to_string(&profile_meta.rhai_path) {
        Ok(content) => content,
        Err(e) => {
            output_error(
                &format!(
                    "Failed to read profile file '{}': {}",
                    profile_meta.rhai_path.display(),
                    e
                ),
                1,
                json,
            );
            return Err(1);
        }
    };
    let device_id = extract_device_id(&content).unwrap_or_else(|| "*".to_string());
    let layers = extract_layer_list(&content);
    let mapping_count = count_mappings(&content);

    if json {
        let output = ShowOutput {
            profile: profile_name,
            device_id,
            layers,
            mapping_count,
        };
        // SAFETY: serde_json::to_string cannot fail on simple struct with derived Serialize
        #[allow(clippy::unwrap_used)]
        {
            println!("{}", serde_json::to_string(&output).unwrap());
        }
    } else {
        println!("Profile: {}", profile_name);
        println!("Device ID: {}", device_id);
        println!("Layers: {}", layers.join(", "));
        println!("Mappings: {}", mapping_count);
    }

    Ok(())
}

/// Handle diff command.
pub fn handle_diff(
    manager: &ProfileManager,
    profile1: String,
    profile2: String,
    json: bool,
) -> Result<(), i32> {
    let meta1 = match manager.get(&profile1) {
        Some(m) => m,
        None => {
            output_error(&format!("Profile not found: {}", profile1), 1001, json);
            return Err(1);
        }
    };

    let meta2 = match manager.get(&profile2) {
        Some(m) => m,
        None => {
            output_error(&format!("Profile not found: {}", profile2), 1001, json);
            return Err(1);
        }
    };

    // Read both files
    let content1 = match std::fs::read_to_string(&meta1.rhai_path) {
        Ok(c) => c,
        Err(e) => {
            output_error(&format!("Failed to read {}: {}", profile1, e), 1, json);
            return Err(1);
        }
    };

    let content2 = match std::fs::read_to_string(&meta2.rhai_path) {
        Ok(c) => c,
        Err(e) => {
            output_error(&format!("Failed to read {}: {}", profile2, e), 1, json);
            return Err(1);
        }
    };

    // Simple line-based diff
    let differences = compute_diff(&content1, &content2);

    if json {
        let output = DiffOutput {
            profile1,
            profile2,
            differences,
        };
        // SAFETY: serde_json::to_string cannot fail on simple struct with derived Serialize
        #[allow(clippy::unwrap_used)]
        {
            println!("{}", serde_json::to_string(&output).unwrap());
        }
    } else if differences.is_empty() {
        println!("No differences between '{}' and '{}'", profile1, profile2);
    } else {
        println!("Differences between '{}' and '{}':", profile1, profile2);
        for diff in differences {
            println!("  {}", diff);
        }
    }

    Ok(())
}

/// Get profile name from option or active profile.
fn get_profile_name(manager: &ProfileManager, profile: Option<String>) -> Result<String, i32> {
    if let Some(name) = profile {
        Ok(name)
    } else if let Ok(Some(active)) = manager.get_active() {
        Ok(active)
    } else {
        eprintln!("Error: No active profile. Use --profile to specify one.");
        Err(1)
    }
}
