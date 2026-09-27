//! Daemon initialization and startup error types.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Initialization and startup errors.
///
/// This error type covers failures during daemon initialization and startup,
/// including component initialization, dependency checks, and lock poisoning.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum InitError {
    /// Failed to initialize a component.
    #[error("Failed to initialize {component}: {reason}")]
    ComponentInit {
        /// Name of the component that failed to initialize.
        component: String,
        /// Reason for initialization failure.
        reason: String,
    },

    /// Required dependency is missing or unavailable.
    #[error("Required dependency missing: {dependency}")]
    MissingDependency {
        /// Name of the missing dependency.
        dependency: String,
    },

    /// Configuration directory is not accessible.
    #[error("Configuration directory not accessible: {path:?}")]
    ConfigDirUnavailable {
        /// Path to the inaccessible configuration directory.
        path: PathBuf,
    },

    /// Lock was poisoned by a panicked thread.
    #[error("Lock poisoned in {component}: {details}")]
    LockPoisoned {
        /// Component where the lock was poisoned.
        component: String,
        /// Additional details about the poisoning.
        details: String,
    },

    /// Channel was closed unexpectedly.
    #[error("Channel closed unexpectedly in {context}")]
    ChannelClosed {
        /// Context where the channel closure occurred.
        context: String,
    },

    /// JSON serialization failed.
    #[error("JSON serialization failed: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    /// IO error occurred during initialization.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
