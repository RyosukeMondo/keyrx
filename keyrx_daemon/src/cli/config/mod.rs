//! Configuration management CLI module with layered architecture.
//!
//! This module splits configuration operations into three distinct layers:
//! 1. Input parsing and validation
//! 2. Business logic execution (service layer)
//! 3. Output formatting and serialization

pub mod handlers;
pub mod input;
pub mod output;
pub mod service;

use crate::error::DaemonResult;
use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Configuration management subcommands.
#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommands,

    /// Output as JSON.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
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

/// Execute the config command.
pub fn execute(args: ConfigArgs, config_dir: Option<PathBuf>) -> DaemonResult<()> {
    let config_dir = input::determine_config_dir(config_dir);
    let json = args.json;

    let result = execute_inner(args, config_dir);

    if let Err(e) = &result {
        use crate::cli::common::output_error;
        output_error(&e.to_string(), 1, json);
    }

    result
}

/// Inner execute function that routes to handlers.
fn execute_inner(args: ConfigArgs, config_dir: PathBuf) -> DaemonResult<()> {
    use handlers::*;

    let manager = service::ProfileService::new(config_dir)?;

    match args.command {
        ConfigCommands::SetKey {
            key,
            target,
            layer,
            profile,
        } => handle_set_key(manager, key, target, layer, profile, args.json),
        ConfigCommands::SetTapHold {
            key,
            tap,
            hold,
            threshold,
            layer,
            profile,
        } => handle_set_tap_hold(
            manager, key, tap, hold, threshold, layer, profile, args.json,
        ),
        ConfigCommands::SetMacro {
            key,
            sequence,
            layer,
            profile,
        } => handle_set_macro(manager, key, sequence, layer, profile, args.json),
        ConfigCommands::GetKey {
            key,
            layer,
            profile,
        } => handle_get_key(manager, key, layer, profile, args.json),
        ConfigCommands::DeleteKey {
            key,
            layer,
            profile,
        } => handle_delete_key(manager, key, layer, profile, args.json),
        ConfigCommands::Validate { profile } => handle_validate(manager, profile, args.json),
        ConfigCommands::Show { profile } => handle_show(manager, profile, args.json),
        ConfigCommands::Diff { profile1, profile2 } => {
            handle_diff(manager, profile1, profile2, args.json)
        }
    }
}
