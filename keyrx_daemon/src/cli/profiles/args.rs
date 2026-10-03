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

        /// Template to start from (blank, simple_remap, capslock_escape,
        /// vim_navigation, gaming). Every template matches ALL keyboards
        /// ("*"); edit its device_start() to limit it.
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

    /// Import a layout file (.krx compiled layout or .rhai source) as a new
    /// profile. A .krx is converted to Rhai so it can be edited; nothing is
    /// stored if the file is invalid.
    Import {
        /// Input file path (.krx or .rhai).
        input: PathBuf,

        /// Profile name (default: the file name without its extension).
        name: Option<String>,

        /// Make the imported profile the active one.
        #[arg(long)]
        activate: bool,
    },
}

/// Parse template string to ProfileTemplate enum.
pub(super) fn parse_template(s: &str) -> Result<ProfileTemplate, String> {
    ProfileTemplate::from_name(s)
}
