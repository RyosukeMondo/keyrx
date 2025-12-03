//! CLI command definitions and output formatting.

pub mod commands;
mod error;
mod exit_codes;
mod output;
mod result;
mod traits;

pub use error::CommandError;
pub use exit_codes::ExitCode;
pub use output::{OutputFormat, OutputWriter};
pub use result::CommandResult;
pub use traits::HasExitCode;
