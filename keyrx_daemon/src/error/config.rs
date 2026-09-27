//! Configuration loading and validation error types.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

use keyrx_core::error::CoreError;

/// Configuration loading and validation errors.
///
/// This error type covers failures when loading, parsing, or validating
/// configuration files and profiles.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// Configuration file not found.
    #[error("Configuration file not found: {path:?}")]
    FileNotFound {
        /// Path to the missing configuration file.
        path: PathBuf,
    },

    /// Failed to parse configuration file.
    #[error("Failed to parse configuration at {path:?}: {reason}")]
    ParseError {
        /// Path to the configuration file.
        path: PathBuf,
        /// Reason for parse failure.
        reason: String,
    },

    /// Invalid profile configuration.
    #[error("Invalid profile '{name}': {reason}")]
    InvalidProfile {
        /// Name of the invalid profile.
        name: String,
        /// Reason why the profile is invalid.
        reason: String,
    },

    /// Configuration compilation failed.
    #[error("Failed to compile configuration: {reason}")]
    CompilationFailed {
        /// Reason for compilation failure.
        reason: String,
    },

    /// Core library error occurred during configuration processing.
    #[error("Core error: {0}")]
    Core(#[from] CoreError),

    /// IO error occurred during configuration operation.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Profile manager error occurred.
    #[error("Profile error: {0}")]
    Profile(String),

    /// Rhai generator error occurred.
    #[error("Generator error: {0}")]
    Generator(String),
}

impl From<crate::config::profile_manager::ProfileError> for ConfigError {
    fn from(err: crate::config::profile_manager::ProfileError) -> Self {
        ConfigError::Profile(err.to_string())
    }
}

impl From<crate::config::rhai_generator::GeneratorError> for ConfigError {
    fn from(err: crate::config::rhai_generator::GeneratorError) -> Self {
        ConfigError::Generator(err.to_string())
    }
}
