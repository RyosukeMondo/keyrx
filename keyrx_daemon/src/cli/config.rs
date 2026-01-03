//! Configuration management CLI commands.
//!
//! This module implements the `keyrx config` command and all its subcommands
//! for managing key mappings, validating configurations, and inspecting
//! compiled .krx files.

use crate::cli::logging;
use crate::config::profile_manager::ProfileManager;
use crate::config::rhai_generator::{KeyAction, MacroStep, RhaiGenerator};
use crate::error::{CliError, ConfigError, DaemonResult};
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// Configuration management subcommands.
#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommands,

    /// Output as JSON.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Set a simple key mapping.
    SetKey {
        /// Source key (e.g., "VK_A").
        key: String,

        /// Target key (e.g., "VK_B").
        target: String,

        /// Layer name (default: "base").
        #[arg(long, default_value = "base")]
        layer: String,

        /// Profile name (default: active profile).
        #[arg(long)]
        profile: Option<String>,
    },

    /// Set a tap-hold mapping.
    SetTapHold {
        /// Source key (e.g., "VK_Space").
        key: String,

        /// Tap action (e.g., "VK_Space").
        tap: String,

        /// Hold action (e.g., "MD_00").
        hold: String,

        /// Threshold in milliseconds (default: 200).
        #[arg(long, default_value = "200")]
        threshold: u16,

        /// Layer name (default: "base").
        #[arg(long, default_value = "base")]
        layer: String,

        /// Profile name (default: active profile).
        #[arg(long)]
        profile: Option<String>,
    },

    /// Set a macro mapping.
    SetMacro {
        /// Source key (e.g., "VK_F1").
        key: String,

        /// Macro sequence (e.g., "press:VK_A,wait:50,release:VK_A").
        sequence: String,

        /// Layer name (default: "base").
        #[arg(long, default_value = "base")]
        layer: String,

        /// Profile name (default: active profile).
        #[arg(long)]
        profile: Option<String>,
    },

    /// Get a key mapping.
    GetKey {
        /// Key to query (e.g., "VK_A").
        key: String,

        /// Layer name (default: "base").
        #[arg(long, default_value = "base")]
        layer: String,

        /// Profile name (default: active profile).
        #[arg(long)]
        profile: Option<String>,
    },

    /// Delete a key mapping.
    DeleteKey {
        /// Key to delete (e.g., "VK_A").
        key: String,

        /// Layer name (default: "base").
        #[arg(long, default_value = "base")]
        layer: String,

        /// Profile name (default: active profile).
        #[arg(long)]
        profile: Option<String>,
    },

    /// Validate a profile (dry-run compilation).
    Validate {
        /// Profile name to validate (default: active profile).
        profile: Option<String>,
    },

    /// Show KRX metadata for a profile.
    Show {
        /// Profile name (default: active profile).
        profile: Option<String>,
    },

    /// Compare two profiles.
    Diff {
        /// First profile name.
        profile1: String,

        /// Second profile name.
        profile2: String,
    },
}

/// JSON output for set-key operations.
#[derive(Serialize)]
struct SetKeyOutput {
    success: bool,
    key: String,
    layer: String,
    profile: String,
    compile_time_ms: Option<u64>,
}

/// JSON output for get-key operations.
#[derive(Serialize)]
struct GetKeyOutput {
    key: String,
    layer: String,
    mapping: Option<String>,
}

/// JSON output for validation.
#[derive(Serialize)]
struct ValidationOutput {
    success: bool,
    profile: String,
    errors: Vec<String>,
}

/// JSON output for show command.
#[derive(Serialize)]
struct ShowOutput {
    profile: String,
    device_id: String,
    layers: Vec<String>,
    mapping_count: usize,
}

/// JSON output for diff command.
#[derive(Serialize)]
struct DiffOutput {
    profile1: String,
    profile2: String,
    differences: Vec<String>,
}

/// Execute the config command.
pub fn execute(args: ConfigArgs, config_dir: Option<PathBuf>) -> DaemonResult<()> {
    // Determine config directory (priority: parameter, env var, default)
    let config_dir = config_dir
        .or_else(|| std::env::var("KEYRX_CONFIG_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("keyrx");
            path
        });

    // Save json flag for error handling
    let json = args.json;

    // Execute command and handle errors
    let result = execute_inner(args, config_dir);

    // Format errors based on JSON flag before returning
    if let Err(e) = &result {
        use crate::cli::common::output_error;
        output_error(&e.to_string(), 1, json);
    }

    result
}

/// Inner execute function that returns errors for formatting.
fn execute_inner(args: ConfigArgs, config_dir: PathBuf) -> DaemonResult<()> {
    // Initialize ProfileManager
    let mut manager =
        ProfileManager::new(config_dir.clone()).map_err(|e| CliError::CommandFailed {
            command: "config".to_string(),
            reason: format!("Failed to initialize profile manager: {}", e),
        })?;

    // Scan profiles
    manager
        .scan_profiles()
        .map_err(|e| CliError::CommandFailed {
            command: "config".to_string(),
            reason: format!("Failed to scan profiles: {}", e),
        })?;

    match args.command {
        ConfigCommands::SetKey {
            key,
            target,
            layer,
            profile,
        } => handle_set_key(&mut manager, key, target, layer, profile, args.json),
        ConfigCommands::SetTapHold {
            key,
            tap,
            hold,
            threshold,
            layer,
            profile,
        } => handle_set_tap_hold(
            &mut manager,
            key,
            tap,
            hold,
            threshold,
            layer,
            profile,
            args.json,
        ),
        ConfigCommands::SetMacro {
            key,
            sequence,
            layer,
            profile,
        } => handle_set_macro(&mut manager, key, sequence, layer, profile, args.json),
        ConfigCommands::GetKey {
            key,
            layer,
            profile,
        } => handle_get_key(&manager, key, layer, profile, args.json),
        ConfigCommands::DeleteKey {
            key,
            layer,
            profile,
        } => handle_delete_key(&mut manager, key, layer, profile, args.json),
        ConfigCommands::Validate { profile } => handle_validate(&manager, profile, args.json),
        ConfigCommands::Show { profile } => handle_show(&manager, profile, args.json),
        ConfigCommands::Diff { profile1, profile2 } => {
            handle_diff(&manager, profile1, profile2, args.json)
        }
    }
}

fn handle_set_key(
    manager: &mut ProfileManager,
    key: String,
    target: String,
    layer: String,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    logging::log_command_start(
        "config set-key",
        &format!("{} -> {} (layer: {})", key, target, layer),
    );

    let profile_name = get_profile_name(manager, profile)?;
    let profile_meta = manager.get(&profile_name).ok_or_else(|| {
        logging::log_command_error(
            "config set-key",
            &format!("Profile not found: {}", profile_name),
        );
        ConfigError::InvalidProfile {
            name: profile_name.clone(),
            reason: "Profile not found".to_string(),
        }
    })?;

    // Load Rhai generator
    let mut gen = RhaiGenerator::load(&profile_meta.rhai_path).map_err(|e| {
        logging::log_command_error("config set-key", &format!("Failed to load profile: {}", e));
        ConfigError::ParseError {
            path: profile_meta.rhai_path.clone(),
            reason: e.to_string(),
        }
    })?;

    // Set the mapping
    let action = KeyAction::SimpleRemap {
        output: target.clone(),
    };

    gen.set_key_mapping(&layer, &key, action).map_err(|e| {
        logging::log_command_error("config set-key", &format!("Failed to set mapping: {}", e));
        CliError::CommandFailed {
            command: "set-key".to_string(),
            reason: e.to_string(),
        }
    })?;

    logging::log_config_change(&profile_name, "set_key", &key, &layer);

    // Save the file
    gen.save(&profile_meta.rhai_path)
        .map_err(|e| ConfigError::ParseError {
            path: profile_meta.rhai_path.clone(),
            reason: format!("Failed to save: {}", e),
        })?;

    // Recompile
    let compile_start = std::time::Instant::now();
    keyrx_compiler::compile_file(&profile_meta.rhai_path, &profile_meta.krx_path).map_err(|e| {
        ConfigError::CompilationFailed {
            reason: e.to_string(),
        }
    })?;
    let compile_time = compile_start.elapsed().as_millis() as u64;

    logging::log_command_success("config set-key", compile_time);

    if json {
        let output = SetKeyOutput {
            success: true,
            key,
            layer,
            profile: profile_name,
            compile_time_ms: Some(compile_time),
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(CliError::from)?
        );
    } else {
        println!(
            "✓ Set {} -> {} in layer '{}' of profile '{}'",
            key, target, layer, profile_name
        );
        println!("  Compiled in {}ms", compile_time);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn handle_set_tap_hold(
    manager: &mut ProfileManager,
    key: String,
    tap: String,
    hold: String,
    threshold: u16,
    layer: String,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    let profile_name = get_profile_name(manager, profile)?;
    let profile_meta = manager
        .get(&profile_name)
        .ok_or_else(|| ConfigError::InvalidProfile {
            name: profile_name.clone(),
            reason: "Profile not found".to_string(),
        })?;

    // Load Rhai generator
    let mut gen =
        RhaiGenerator::load(&profile_meta.rhai_path).map_err(|e| ConfigError::ParseError {
            path: profile_meta.rhai_path.clone(),
            reason: e.to_string(),
        })?;

    // Set the mapping
    let action = KeyAction::TapHold {
        tap: tap.clone(),
        hold: hold.clone(),
        threshold_ms: threshold,
    };

    gen.set_key_mapping(&layer, &key, action)
        .map_err(|e| CliError::CommandFailed {
            command: "set-tap-hold".to_string(),
            reason: e.to_string(),
        })?;

    // Save the file
    gen.save(&profile_meta.rhai_path)
        .map_err(|e| ConfigError::ParseError {
            path: profile_meta.rhai_path.clone(),
            reason: format!("Failed to save: {}", e),
        })?;

    // Recompile
    let compile_start = std::time::Instant::now();
    keyrx_compiler::compile_file(&profile_meta.rhai_path, &profile_meta.krx_path).map_err(|e| {
        ConfigError::CompilationFailed {
            reason: e.to_string(),
        }
    })?;
    let compile_time = compile_start.elapsed().as_millis() as u64;

    if json {
        let output = SetKeyOutput {
            success: true,
            key,
            layer,
            profile: profile_name,
            compile_time_ms: Some(compile_time),
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(CliError::from)?
        );
    } else {
        println!(
            "✓ Set {} -> tap:{} hold:{} ({}ms) in layer '{}' of profile '{}'",
            key, tap, hold, threshold, layer, profile_name
        );
        println!("  Compiled in {}ms", compile_time);
    }

    Ok(())
}

fn handle_set_macro(
    manager: &mut ProfileManager,
    key: String,
    sequence: String,
    layer: String,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    let profile_name = get_profile_name(manager, profile)?;
    let profile_meta = manager
        .get(&profile_name)
        .ok_or_else(|| ConfigError::InvalidProfile {
            name: profile_name.clone(),
            reason: "Profile not found".to_string(),
        })?;

    // Parse macro sequence
    let macro_steps = parse_macro_sequence(&sequence).map_err(|e| CliError::InvalidArguments {
        reason: format!("Invalid macro sequence: {}", e),
    })?;

    // Load Rhai generator
    let mut gen =
        RhaiGenerator::load(&profile_meta.rhai_path).map_err(|e| ConfigError::ParseError {
            path: profile_meta.rhai_path.clone(),
            reason: e.to_string(),
        })?;

    // Set the mapping
    let action = KeyAction::Macro {
        sequence: macro_steps,
    };

    gen.set_key_mapping(&layer, &key, action)
        .map_err(|e| CliError::CommandFailed {
            command: "set-macro".to_string(),
            reason: e.to_string(),
        })?;

    // Save the file
    gen.save(&profile_meta.rhai_path)
        .map_err(|e| ConfigError::ParseError {
            path: profile_meta.rhai_path.clone(),
            reason: format!("Failed to save: {}", e),
        })?;

    // Recompile
    let compile_start = std::time::Instant::now();
    keyrx_compiler::compile_file(&profile_meta.rhai_path, &profile_meta.krx_path).map_err(|e| {
        ConfigError::CompilationFailed {
            reason: e.to_string(),
        }
    })?;
    let compile_time = compile_start.elapsed().as_millis() as u64;

    if json {
        let output = SetKeyOutput {
            success: true,
            key,
            layer,
            profile: profile_name,
            compile_time_ms: Some(compile_time),
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(CliError::from)?
        );
    } else {
        println!(
            "✓ Set {} -> macro in layer '{}' of profile '{}'",
            key, layer, profile_name
        );
        println!("  Compiled in {}ms", compile_time);
    }

    Ok(())
}

fn handle_get_key(
    manager: &ProfileManager,
    key: String,
    layer: String,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    let profile_name = get_profile_name(manager, profile)?;
    let profile_meta = manager
        .get(&profile_name)
        .ok_or_else(|| ConfigError::InvalidProfile {
            name: profile_name.clone(),
            reason: "Profile not found".to_string(),
        })?;

    // Load Rhai file and search for the mapping
    let content =
        std::fs::read_to_string(&profile_meta.rhai_path).map_err(|e| ConfigError::ParseError {
            path: profile_meta.rhai_path.clone(),
            reason: format!("Failed to read profile: {}", e),
        })?;

    // Simple line-based search for the key
    let mapping = find_key_mapping(&content, &key, &layer);

    if json {
        let output = GetKeyOutput {
            key,
            layer,
            mapping,
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(CliError::from)?
        );
    } else if let Some(m) = mapping {
        println!("{}", m);
    } else {
        println!("No mapping found for {} in layer '{}'", key, layer);
    }

    Ok(())
}

fn handle_delete_key(
    manager: &mut ProfileManager,
    key: String,
    layer: String,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    let profile_name = get_profile_name(manager, profile)?;
    let profile_meta = manager
        .get(&profile_name)
        .ok_or_else(|| ConfigError::InvalidProfile {
            name: profile_name.clone(),
            reason: "Profile not found".to_string(),
        })?;

    // Load Rhai generator
    let mut gen =
        RhaiGenerator::load(&profile_meta.rhai_path).map_err(|e| ConfigError::ParseError {
            path: profile_meta.rhai_path.clone(),
            reason: e.to_string(),
        })?;

    // Delete the mapping
    gen.delete_key_mapping(&layer, &key)
        .map_err(|e| CliError::CommandFailed {
            command: "delete-key".to_string(),
            reason: e.to_string(),
        })?;

    // Save the file
    gen.save(&profile_meta.rhai_path)
        .map_err(|e| ConfigError::ParseError {
            path: profile_meta.rhai_path.clone(),
            reason: format!("Failed to save: {}", e),
        })?;

    // Recompile
    let compile_start = std::time::Instant::now();
    keyrx_compiler::compile_file(&profile_meta.rhai_path, &profile_meta.krx_path).map_err(|e| {
        ConfigError::CompilationFailed {
            reason: e.to_string(),
        }
    })?;
    let compile_time = compile_start.elapsed().as_millis() as u64;

    if json {
        let output = serde_json::json!({
            "success": true,
            "key": key,
            "layer": layer,
            "profile": profile_name,
            "compile_time_ms": compile_time,
        });
        println!(
            "{}",
            serde_json::to_string(&output).map_err(CliError::from)?
        );
    } else {
        println!(
            "✓ Deleted mapping for {} in layer '{}' of profile '{}'",
            key, layer, profile_name
        );
        println!("  Compiled in {}ms", compile_time);
    }

    Ok(())
}

fn handle_validate(
    manager: &ProfileManager,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    let profile_name = get_profile_name(manager, profile)?;
    let profile_meta = manager
        .get(&profile_name)
        .ok_or_else(|| ConfigError::InvalidProfile {
            name: profile_name.clone(),
            reason: "Profile not found".to_string(),
        })?;

    // Attempt dry-run compilation
    logging::log_command_start("config validate", &profile_name);
    let temp_output = profile_meta.krx_path.with_extension("tmp.krx");
    let result = keyrx_compiler::compile_file(&profile_meta.rhai_path, &temp_output);

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_output);

    match result {
        Ok(_) => {
            logging::log_config_validate(&profile_name, true, None);
            logging::log_command_success("config validate", 0);
            if json {
                let output = ValidationOutput {
                    success: true,
                    profile: profile_name,
                    errors: vec![],
                };
                println!(
                    "{}",
                    serde_json::to_string(&output).map_err(CliError::from)?
                );
            } else {
                println!("✓ Profile '{}' is valid", profile_name);
            }
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            logging::log_config_validate(&profile_name, false, Some(&error_msg));
            logging::log_command_error("config validate", &error_msg);
            if json {
                let output = ValidationOutput {
                    success: false,
                    profile: profile_name.clone(),
                    errors: vec![error_msg.clone()],
                };
                println!(
                    "{}",
                    serde_json::to_string(&output).map_err(CliError::from)?
                );
            } else {
                println!("✗ Profile '{}' validation failed:", profile_name);
                println!("  {}", e);
            }
            Err(ConfigError::CompilationFailed { reason: error_msg }.into())
        }
    }
}

fn handle_show(manager: &ProfileManager, profile: Option<String>, json: bool) -> DaemonResult<()> {
    let profile_name = get_profile_name(manager, profile)?;
    let profile_meta = manager
        .get(&profile_name)
        .ok_or_else(|| ConfigError::InvalidProfile {
            name: profile_name.clone(),
            reason: "Profile not found".to_string(),
        })?;

    // Load the Rhai file and analyze it
    let _gen =
        RhaiGenerator::load(&profile_meta.rhai_path).map_err(|e| ConfigError::ParseError {
            path: profile_meta.rhai_path.clone(),
            reason: e.to_string(),
        })?;

    let content =
        std::fs::read_to_string(&profile_meta.rhai_path).map_err(|e| ConfigError::ParseError {
            path: profile_meta.rhai_path.clone(),
            reason: format!("Failed to read profile: {}", e),
        })?;
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
        println!(
            "{}",
            serde_json::to_string(&output).map_err(CliError::from)?
        );
    } else {
        println!("Profile: {}", profile_name);
        println!("Device ID: {}", device_id);
        println!("Layers: {}", layers.join(", "));
        println!("Mappings: {}", mapping_count);
    }

    Ok(())
}

fn handle_diff(
    manager: &ProfileManager,
    profile1: String,
    profile2: String,
    json: bool,
) -> DaemonResult<()> {
    let meta1 = manager
        .get(&profile1)
        .ok_or_else(|| ConfigError::InvalidProfile {
            name: profile1.clone(),
            reason: "Profile not found".to_string(),
        })?;

    let meta2 = manager
        .get(&profile2)
        .ok_or_else(|| ConfigError::InvalidProfile {
            name: profile2.clone(),
            reason: "Profile not found".to_string(),
        })?;

    // Read both files
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

    // Simple line-based diff
    let differences = compute_diff(&content1, &content2);

    if json {
        let output = DiffOutput {
            profile1,
            profile2,
            differences,
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(CliError::from)?
        );
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

// Helper functions

fn get_profile_name(manager: &ProfileManager, profile: Option<String>) -> DaemonResult<String> {
    if let Some(name) = profile {
        Ok(name)
    } else if let Ok(Some(active)) = manager.get_active() {
        Ok(active)
    } else {
        Err(CliError::InvalidArguments {
            reason: "No active profile. Use --profile to specify one.".to_string(),
        }
        .into())
    }
}

fn parse_macro_sequence(sequence: &str) -> Result<Vec<MacroStep>, String> {
    let mut steps = Vec::new();
    for part in sequence.split(',') {
        let part = part.trim();
        if let Some(key) = part.strip_prefix("press:") {
            steps.push(MacroStep::Press(key.to_string()));
        } else if let Some(key) = part.strip_prefix("release:") {
            steps.push(MacroStep::Release(key.to_string()));
        } else if let Some(ms) = part.strip_prefix("wait:") {
            let ms = ms
                .parse::<u16>()
                .map_err(|_| format!("Invalid wait time: {}", ms))?;
            steps.push(MacroStep::Wait(ms));
        } else {
            return Err(format!("Invalid macro step: {}", part));
        }
    }
    Ok(steps)
}

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

    // Simple line-by-line comparison (not a true diff algorithm)
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
