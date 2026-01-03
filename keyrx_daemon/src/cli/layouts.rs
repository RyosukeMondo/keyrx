//! Keyboard layout management CLI commands.
//!
//! This module implements the `keyrx layouts` command and all its subcommands
//! for managing keyboard layouts in KLE (keyboard-layout-editor.com) JSON format.

use crate::config::layout_manager::{LayoutManager, LayoutSource};
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// Layout management subcommands.
#[derive(Args)]
pub struct LayoutsArgs {
    #[command(subcommand)]
    command: LayoutsCommands,

    /// Output as JSON.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum LayoutsCommands {
    /// List all available layouts (builtin and custom).
    List,

    /// Show layout details as KLE JSON.
    Show {
        /// Layout name to display.
        name: String,
    },

    /// Import a custom layout from a KLE JSON file.
    Import {
        /// Path to the KLE JSON file.
        path: PathBuf,

        /// Name for the imported layout.
        name: String,
    },

    /// Delete a custom layout.
    Delete {
        /// Name of the layout to delete.
        name: String,

        /// Skip confirmation prompt.
        #[arg(long)]
        confirm: bool,
    },
}

/// JSON output structure for layout list.
#[derive(Serialize)]
struct LayoutListOutput {
    layouts: Vec<LayoutListItem>,
    builtin_count: usize,
    custom_count: usize,
}

/// Information about a single layout in the list.
#[derive(Serialize)]
struct LayoutListItem {
    name: String,
    source: String,
}

/// JSON output structure for layout show.
#[derive(Serialize)]
struct LayoutShowOutput {
    name: String,
    source: String,
    kle_json: serde_json::Value,
}

/// JSON output structure for layout operations.
#[derive(Serialize)]
struct LayoutOperationOutput {
    success: bool,
    message: Option<String>,
    error: Option<String>,
}

/// Execute layouts command.
pub fn execute(args: LayoutsArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        LayoutsCommands::List => handle_list(args.json),
        LayoutsCommands::Show { name } => handle_show(&name, args.json),
        LayoutsCommands::Import { path, name } => handle_import(&path, &name, args.json),
        LayoutsCommands::Delete { name, confirm } => handle_delete(&name, confirm, args.json),
    }
}

/// Handle `layouts list` command.
fn handle_list(json_output: bool) -> Result<(), Box<dyn std::error::Error>> {
    let layouts_dir = get_layouts_dir();
    let manager = LayoutManager::new(layouts_dir)?;
    let all_layouts = manager.list();

    if json_output {
        let output = LayoutListOutput {
            layouts: all_layouts
                .iter()
                .map(|l| LayoutListItem {
                    name: l.name.clone(),
                    source: format!("{:?}", l.source).to_lowercase(),
                })
                .collect(),
            builtin_count: manager.builtin_count(),
            custom_count: manager.custom_count(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Available keyboard layouts:\n");
        println!("{:<32} SOURCE", "NAME");
        println!("{}", "-".repeat(45));
        for layout in all_layouts {
            let source = match layout.source {
                LayoutSource::Builtin => "builtin",
                LayoutSource::Custom => "custom",
            };
            println!("{:<32} {}", layout.name, source);
        }
        println!(
            "\nTotal: {} ({} builtin, {} custom)",
            manager.builtin_count() + manager.custom_count(),
            manager.builtin_count(),
            manager.custom_count()
        );
    }

    Ok(())
}

/// Handle `layouts show` command.
fn handle_show(name: &str, json_output: bool) -> Result<(), Box<dyn std::error::Error>> {
    let layouts_dir = get_layouts_dir();
    let manager = LayoutManager::new(layouts_dir)?;

    let layout = manager
        .get(name)
        .ok_or_else(|| format!("Layout '{}' not found", name))?;

    if json_output {
        let output = LayoutShowOutput {
            name: layout.name.clone(),
            source: format!("{:?}", layout.source).to_lowercase(),
            kle_json: layout.kle_json.clone(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Layout: {}", layout.name);
        println!("Source: {:?}", layout.source);
        println!("\nKLE JSON:");
        println!("{}", serde_json::to_string_pretty(&layout.kle_json)?);
    }

    Ok(())
}

/// Handle `layouts import` command.
fn handle_import(
    path: &std::path::Path,
    name: &str,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let layouts_dir = get_layouts_dir();
    let mut manager = LayoutManager::new(layouts_dir)?;

    match manager.import(path, name) {
        Ok(_layout) => {
            if json_output {
                let output = LayoutOperationOutput {
                    success: true,
                    message: Some(format!("Layout '{}' imported successfully", name)),
                    error: None,
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("✓ Layout '{}' imported successfully", name);
            }
            Ok(())
        }
        Err(e) => {
            if json_output {
                let output = LayoutOperationOutput {
                    success: false,
                    message: None,
                    error: Some(e.to_string()),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                eprintln!("✗ Error importing layout: {}", e);
            }
            std::process::exit(1);
        }
    }
}

/// Handle `layouts delete` command.
fn handle_delete(
    name: &str,
    confirm: bool,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let layouts_dir = get_layouts_dir();
    let mut manager = LayoutManager::new(layouts_dir)?;

    // Check if layout exists
    let layout = manager.get(name);
    if layout.is_none() {
        if json_output {
            let output = LayoutOperationOutput {
                success: false,
                message: None,
                error: Some(format!("Layout '{}' not found", name)),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("✗ Layout '{}' not found", name);
        }
        std::process::exit(1);
    }

    // SAFETY: layout is guaranteed to be Some() here because we exit above if it's None
    #[allow(clippy::unwrap_used)]
    let layout = layout.unwrap();

    // Check if it's a builtin layout
    if matches!(layout.source, LayoutSource::Builtin) {
        if json_output {
            let output = LayoutOperationOutput {
                success: false,
                message: None,
                error: Some(format!("Cannot delete builtin layout '{}'", name)),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("✗ Cannot delete builtin layout '{}'", name);
        }
        std::process::exit(1);
    }

    // Confirmation prompt (unless --confirm flag is used)
    if !confirm && !json_output {
        use std::io::{self, Write};
        print!("Delete layout '{}'? [y/N] ", name);
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled");
            return Ok(());
        }
    }

    // Perform deletion
    match manager.delete(name) {
        Ok(()) => {
            if json_output {
                let output = LayoutOperationOutput {
                    success: true,
                    message: Some(format!("Layout '{}' deleted successfully", name)),
                    error: None,
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("✓ Layout '{}' deleted successfully", name);
            }
            Ok(())
        }
        Err(e) => {
            if json_output {
                let output = LayoutOperationOutput {
                    success: false,
                    message: None,
                    error: Some(e.to_string()),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                eprintln!("✗ Error deleting layout: {}", e);
            }
            std::process::exit(1);
        }
    }
}

/// Get the layouts directory path.
fn get_layouts_dir() -> PathBuf {
    // Use environment variable if set, otherwise use default
    if let Ok(config_dir) = std::env::var("KEYRX_CONFIG_DIR") {
        PathBuf::from(config_dir).join("layouts")
    } else {
        // Default to ~/.config/keyrx/layouts
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("keyrx")
            .join("layouts")
    }
}
