//! CLI subcommand handlers.
//!
//! This module contains the implementation of all CLI subcommands:
//! - `compile`: Compile Rhai scripts to .krx binary format
//! - `verify`: Verify .krx binary file integrity
//! - `hash`: Extract and verify SHA256 hash from .krx files
//! - `parse`: Parse Rhai scripts and display configuration structure

pub mod compile;
pub mod hash;
pub mod parse;
pub mod verify;

// Re-export handler functions for easy access
pub use compile::handle_compile;
pub use hash::handle_hash;
pub use parse::handle_parse;
pub use verify::handle_verify;

// Re-export error types for external use
pub use compile::CompileError;
pub use hash::HashError;
pub use parse::ParseCommandError;
pub use verify::VerifyError;
