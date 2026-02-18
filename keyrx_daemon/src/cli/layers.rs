//! Layer management CLI commands.
//!
//! This module implements the `keyrx layers` command and all its subcommands
//! for managing configuration layers in Rhai profiles.

use crate::cli::logging;
use crate::config::profile_manager::ProfileManager;
use crate::config::rhai_generator::{LayerMode, RhaiGenerator};
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// Layer management subcommands.
#[derive(Args, Debug)]
pub struct LayersArgs {
    #[command(subcommand)]
    command: LayersCommands,

    /// Output as JSON.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum LayersCommands {
    /// List all layers in a profile.
    List {
        /// Profile name (defaults to active profile).
        #[arg(long)]
        profile: Option<String>,
    },

    /// Create a new layer.
    Create {
        /// Layer ID (must start with MD_).
        layer_id: String,

        /// Layer name/description.
        name: String,

        /// Profile name (defaults to active profile).
        #[arg(long)]
        profile: Option<String>,

        /// Layer mode: "single" or "multiple".
        #[arg(long, default_value = "single", value_parser = parse_layer_mode)]
        mode: LayerMode,
    },

    /// Rename a layer.
    Rename {
        /// Current layer ID.
        old_id: String,

        /// New layer ID (must start with MD_).
        new_id: String,

        /// Profile name (defaults to active profile).
        #[arg(long)]
        profile: Option<String>,
    },

    /// Delete a layer.
    Delete {
        /// Layer ID to delete.
        layer_id: String,

        /// Profile name (defaults to active profile).
        #[arg(long)]
        profile: Option<String>,

        /// Skip confirmation prompt.
        #[arg(long)]
        confirm: bool,
    },

    /// Show layer details and all its mappings.
    Show {
        /// Layer ID to show.
        layer_id: String,

        /// Profile name (defaults to active profile).
        #[arg(long)]
        profile: Option<String>,
    },
}

/// JSON output structure for layer list.
#[derive(Serialize)]
struct LayerListOutput {
    profile: String,
    layers: Vec<LayerInfo>,
}

/// Information about a single layer.
#[derive(Serialize)]
struct LayerInfo {
    id: String,
    mapping_count: usize,
}

/// JSON output structure for layer show.
#[derive(Serialize)]
struct LayerShowOutput {
    profile: String,
    layer_id: String,
    mappings: Vec<String>,
}

/// JSON output structure for layer creation.
#[derive(Serialize)]
struct LayerCreateOutput {
    success: bool,
    profile: String,
    layer_id: String,
    error: Option<String>,
}

/// JSON output structure for layer operations.
#[derive(Serialize)]
struct LayerOperationOutput {
    success: bool,
    error: Option<String>,
}

/// Execute layers command.
pub fn execute(args: LayersArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        LayersCommands::List { profile } => handle_list(profile.as_deref(), args.json),
        LayersCommands::Create {
            layer_id,
            name,
            profile,
            mode,
        } => handle_create(&layer_id, &name, profile.as_deref(), mode, args.json),
        LayersCommands::Rename {
            old_id,
            new_id,
            profile,
        } => handle_rename(&old_id, &new_id, profile.as_deref(), args.json),
        LayersCommands::Delete {
            layer_id,
            profile,
            confirm,
        } => handle_delete(&layer_id, profile.as_deref(), confirm, args.json),
        LayersCommands::Show { layer_id, profile } => {
            handle_show(&layer_id, profile.as_deref(), args.json)
        }
    }
}

fn handle_list(profile: Option<&str>, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;
    let manager = ProfileManager::new(config_dir)?;
    manager.scan_profiles()?;

    let profile_name = resolve_profile(profile, &manager)?;
    let profile_path = manager
        .get(&profile_name)
        .ok_or("Profile not found")?
        .rhai_path
        .clone();

    let gen = RhaiGenerator::load(&profile_path)?;
    let layers = gen.list_layers();

    if json {
        let output = LayerListOutput {
            profile: profile_name,
            layers: layers
                .iter()
                .map(|(id, count)| LayerInfo {
                    id: id.clone(),
                    mapping_count: *count,
                })
                .collect(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Profile: {}", profile_name);
        println!("\nLayers:");
        if layers.is_empty() {
            println!("  (no layers)");
        } else {
            for (id, count) in layers {
                println!("  {} ({} mappings)", id, count);
            }
        }
    }

    Ok(())
}

fn handle_create(
    layer_id: &str,
    name: &str,
    profile: Option<&str>,
    mode: LayerMode,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;
    let manager = ProfileManager::new(config_dir)?;
    manager.scan_profiles()?;

    let profile_name = resolve_profile(profile, &manager)?;
    let profile_path = manager
        .get(&profile_name)
        .ok_or("Profile not found")?
        .rhai_path
        .clone();

    let mut gen = RhaiGenerator::load(&profile_path)?;

    logging::log_command_start(
        "layers create",
        &format!("{} in {}", layer_id, profile_name),
    );

    match gen.add_layer(layer_id, name, mode) {
        Ok(()) => {
            gen.save(&profile_path)?;
            logging::log_layer_operation(&profile_name, "create", layer_id);
            logging::log_command_success("layers create", 0);

            if json {
                let output = LayerCreateOutput {
                    success: true,
                    profile: profile_name,
                    layer_id: layer_id.to_string(),
                    error: None,
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Layer created: {}", layer_id);
                println!("Profile: {}", profile_name);
            }
            Ok(())
        }
        Err(e) => {
            logging::log_command_error("layers create", &e.to_string());
            if json {
                let output = LayerCreateOutput {
                    success: false,
                    profile: profile_name,
                    layer_id: layer_id.to_string(),
                    error: Some(e.to_string()),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                eprintln!("Error creating layer: {}", e);
            }
            std::process::exit(1);
        }
    }
}

fn handle_rename(
    old_id: &str,
    new_id: &str,
    profile: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;
    let manager = ProfileManager::new(config_dir)?;
    manager.scan_profiles()?;

    let profile_name = resolve_profile(profile, &manager)?;
    let profile_path = manager
        .get(&profile_name)
        .ok_or("Profile not found")?
        .rhai_path
        .clone();

    let mut gen = RhaiGenerator::load(&profile_path)?;

    match gen.rename_layer(old_id, new_id) {
        Ok(()) => {
            gen.save(&profile_path)?;

            if json {
                let output = LayerOperationOutput {
                    success: true,
                    error: None,
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Layer renamed: {} -> {}", old_id, new_id);
            }
            Ok(())
        }
        Err(e) => {
            if json {
                let output = LayerOperationOutput {
                    success: false,
                    error: Some(e.to_string()),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                eprintln!("Error renaming layer: {}", e);
            }
            std::process::exit(1);
        }
    }
}

fn handle_delete(
    layer_id: &str,
    profile: Option<&str>,
    confirm: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !confirm && !json {
        println!(
            "Delete layer '{}'? This will remove all mappings in the layer.",
            layer_id
        );
        println!("Use --confirm to skip this prompt.");
        return Err("Operation cancelled. Use --confirm flag to proceed.".into());
    }

    let config_dir = get_config_dir()?;
    let manager = ProfileManager::new(config_dir)?;
    manager.scan_profiles()?;

    let profile_name = resolve_profile(profile, &manager)?;
    let profile_path = manager
        .get(&profile_name)
        .ok_or("Profile not found")?
        .rhai_path
        .clone();

    let mut gen = RhaiGenerator::load(&profile_path)?;

    match gen.delete_layer(layer_id) {
        Ok(()) => {
            gen.save(&profile_path)?;

            if json {
                let output = LayerOperationOutput {
                    success: true,
                    error: None,
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Layer deleted: {}", layer_id);
            }
            Ok(())
        }
        Err(e) => {
            if json {
                let output = LayerOperationOutput {
                    success: false,
                    error: Some(e.to_string()),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                eprintln!("Error deleting layer: {}", e);
            }
            std::process::exit(1);
        }
    }
}

fn handle_show(
    layer_id: &str,
    profile: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;
    let manager = ProfileManager::new(config_dir)?;
    manager.scan_profiles()?;

    let profile_name = resolve_profile(profile, &manager)?;
    let profile_path = manager
        .get(&profile_name)
        .ok_or("Profile not found")?
        .rhai_path
        .clone();

    let gen = RhaiGenerator::load(&profile_path)?;
    let mappings = gen.get_layer_mappings(layer_id)?;

    if json {
        let output = LayerShowOutput {
            profile: profile_name,
            layer_id: layer_id.to_string(),
            mappings: mappings.clone(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Profile: {}", profile_name);
        println!("Layer: {}", layer_id);
        println!("\nMappings:");
        if mappings.is_empty() {
            println!("  (no mappings)");
        } else {
            for mapping in mappings {
                println!("  {}", mapping);
            }
        }
    }

    Ok(())
}

/// Get config directory path.
fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    crate::cli::config_dir::get_config_dir()
}

/// Resolve profile name (use active if not specified).
fn resolve_profile(
    profile: Option<&str>,
    manager: &ProfileManager,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(name) = profile {
        Ok(name.to_string())
    } else if let Ok(Some(active)) = manager.get_active() {
        Ok(active)
    } else {
        Err("No active profile. Use --profile to specify a profile.".into())
    }
}

/// Parse layer mode from string.
fn parse_layer_mode(s: &str) -> Result<LayerMode, String> {
    match s.to_lowercase().as_str() {
        "single" => Ok(LayerMode::Single),
        "multiple" => Ok(LayerMode::Multiple),
        _ => Err(format!(
            "Invalid layer mode '{}'. Expected 'single' or 'multiple'",
            s
        )),
    }
}
