//! CLI command definitions and output formatting.

pub mod commands;
mod exit_codes;
mod output;
mod result;
mod traits;

pub use exit_codes::ExitCode;
pub use output::{OutputFormat, OutputWriter};
pub use result::{CommandError, CommandResult};
pub use traits::HasExitCode;
