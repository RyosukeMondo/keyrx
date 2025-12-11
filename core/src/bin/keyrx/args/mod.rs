//! CLI argument definitions for the keyrx binary.
//!
//! This module contains all clap argument structs and subcommand enums
//! for the keyrx CLI.
//!
//! # Submodules
//! - `config`: Configuration subcommands (devices, hardware, layout, keymap, runtime)
//! - `subcommands`: Top-level command definitions

mod config;
mod subcommands;

pub use config::{
    DeviceCommands, GoldenCommands, HardwareCommands, KeymapCommands, LayoutCommands,
    RuntimeCommands,
};
pub use subcommands::{Cli, Commands};
