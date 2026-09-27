//! Argument and subcommand definitions for `keyrx profiles`.

use crate::config::profile_manager::ProfileTemplate;
use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Profile management subcommands.
#[derive(Args, Debug)]
pub struct ProfilesArgs {
    #[command(subcommand)]
    pub(super) command: ProfilesCommands,

    /// Output as JSON.
    #[arg(long, global = true)]
    pub(super) json: bool,
}

#[derive(Subcommand, Debug)]
pub(super) enum ProfilesCommands {
    /// List all profiles.
    List,

    /// Create a new profile from a template.
    Create {
        /// Profile name (max 32 chars).
        name: String,

        /// Template to use: "blank" (default) or "qmk-layers".
        #[arg(long, default_value = "blank", value_parser = parse_template)]
        template: ProfileTemplate,
    },

    /// Activate a profile (hot-reload with compilation).
    Activate {
        /// Profile name to activate.
        name: String,
    },

    /// Delete a profile.
    Delete {
        /// Profile name to delete.
        name: String,

        /// Skip confirmation prompt.
        #[arg(long)]
        confirm: bool,
    },

    /// Duplicate a profile.
    Duplicate {
        /// Source profile name.
        src: String,

        /// Destination profile name.
        dest: String,
    },

    /// Export a profile to a file.
    Export {
        /// Profile name to export.
        name: String,

        /// Output file path.
        output: PathBuf,
    },

    /// Import a profile from a file.
    Import {
        /// Input file path.
        input: PathBuf,

        /// Profile name.
        name: String,
    },
}

/// Parse template string to ProfileTemplate enum.
pub(super) fn parse_template(s: &str) -> Result<ProfileTemplate, String> {
    match s.to_lowercase().as_str() {
        "blank" => Ok(ProfileTemplate::Blank),
        "simple_remap" | "simple-remap" => Ok(ProfileTemplate::SimpleRemap),
        "capslock_escape" | "capslock-escape" => Ok(ProfileTemplate::CapslockEscape),
        "vim_navigation" | "vim-navigation" => Ok(ProfileTemplate::VimNavigation),
        "gaming" => Ok(ProfileTemplate::Gaming),
        _ => Err(format!(
            "Invalid template '{}'. Valid templates: blank, simple_remap, capslock_escape, vim_navigation, gaming",
            s
        )),
    }
}
