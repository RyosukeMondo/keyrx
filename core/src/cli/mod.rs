//! CLI command definitions and output formatting.
//!
//! This module uses println! and eprintln! for user-facing output,
//! which is intentional and distinct from internal logging.
#![allow(clippy::print_stdout, clippy::print_stderr)]

mod command;
pub mod commands;
mod error;
mod exit_codes;
mod output;
mod result;
mod traits;

pub use command::{Command, CommandContext, Verbosity};
pub use error::CommandError;
pub use exit_codes::ExitCode;
pub use output::{OutputFormat, OutputWriter};
pub use result::CommandResult;
pub use traits::HasExitCode;
