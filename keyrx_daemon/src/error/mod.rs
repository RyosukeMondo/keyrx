//! Error types for the KeyRx daemon.
//!
//! This module defines a comprehensive error type hierarchy for the daemon,
//! enabling proper error propagation and recovery instead of panics.
//!
//! Each error domain lives in its own submodule; all domain error types are
//! re-exported here so existing `crate::error::*` paths keep working.

mod cli;
mod config;
mod init;
mod platform;
mod recorder;
mod registry;
mod serialization;
mod socket;
mod web;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_composed;

use thiserror::Error;

use keyrx_core::error::CoreError;

pub use cli::CliError;
pub use config::ConfigError;
pub use init::InitError;
pub use platform::PlatformError;
pub use recorder::RecorderError;
pub use registry::RegistryError;
pub use serialization::SerializationError;
pub use socket::SocketError;
pub use web::WebError;

/// Top-level daemon error type.
///
/// This is the main error type for the daemon, encompassing all possible
/// error conditions. Module-specific errors automatically convert into
/// this type via `From` implementations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DaemonError {
    /// Initialization or startup error occurred.
    #[error("Initialization error: {0}")]
    Init(#[from] InitError),

    /// Platform-specific error occurred.
    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),

    /// Serialization or deserialization error occurred.
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),

    /// Socket operation error occurred.
    #[error("Socket error: {0}")]
    Socket(#[from] SocketError),

    /// Registry operation error occurred.
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    /// Macro recorder error occurred.
    #[error("Recorder error: {0}")]
    Recorder(#[from] RecorderError),

    /// Configuration error occurred.
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Web server or API error occurred.
    #[error("Web error: {0}")]
    Web(#[from] WebError),

    /// CLI command error occurred.
    #[error("CLI error: {0}")]
    Cli(#[from] CliError),

    /// Core library error occurred.
    #[error("Core error: {0}")]
    Core(#[from] CoreError),
}

/// Result type alias for daemon operations.
///
/// This is a convenience type alias for operations that can fail with a DaemonError.
///
/// # Examples
///
/// ```
/// use keyrx_daemon::error::{DaemonResult, DaemonError, ConfigError};
/// use std::path::PathBuf;
///
/// fn load_config(path: PathBuf) -> DaemonResult<String> {
///     if !path.exists() {
///         return Err(ConfigError::FileNotFound { path }.into());
///     }
///     Ok("config data".to_string())
/// }
/// ```
pub type DaemonResult<T> = Result<T, DaemonError>;
