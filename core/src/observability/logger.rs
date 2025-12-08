//! Structured logging configuration for KeyRx.
//!
//! This module provides a structured logging system using the `tracing` crate,
//! with support for JSON output, file logging, and configurable log levels.

use crate::observability::bridge::GLOBAL_LOG_BRIDGE;
use std::io;
use std::path::{Path, PathBuf};
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

/// Errors that can occur during logger initialization.
#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("Failed to initialize logger: {0}")]
    InitError(String),

    #[error("Failed to create log file: {0}")]
    FileError(#[from] io::Error),

    #[error("Logger already initialized")]
    AlreadyInitialized,
}

/// Output format for log messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable formatted output with colors and indentation.
    Pretty,
    /// JSON-formatted output for machine parsing.
    Json,
    /// Compact output without colors.
    Compact,
}

/// Configuration for structured logging.
///
/// # Example
/// ```rust,no_run
/// use keyrx_core::observability::logger::{StructuredLogger, OutputFormat};
/// use tracing::Level;
///
/// StructuredLogger::new()
///     .with_format(OutputFormat::Json)
///     .with_level(Level::INFO)
///     .init()
///     .expect("Failed to initialize logger");
/// ```
pub struct StructuredLogger {
    format: OutputFormat,
    level: Level,
    file_path: Option<PathBuf>,
    include_span_events: bool,
    env_filter: Option<String>,
}

impl Default for StructuredLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl StructuredLogger {
    /// Create a new logger configuration with default settings.
    ///
    /// Defaults:
    /// - Format: Pretty
    /// - Level: INFO
    /// - No file output
    /// - Span events: enabled
    pub fn new() -> Self {
        Self {
            format: OutputFormat::Pretty,
            level: Level::INFO,
            file_path: None,
            include_span_events: true,
            env_filter: None,
        }
    }

    /// Set the output format.
    ///
    /// # Arguments
    /// * `format` - The output format to use
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the minimum log level.
    ///
    /// # Arguments
    /// * `level` - The minimum level to log
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Enable file output.
    ///
    /// # Arguments
    /// * `path` - Path to the log file
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.file_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Control whether span events (enter/exit/close) are logged.
    ///
    /// # Arguments
    /// * `enabled` - Whether to include span events
    pub fn with_span_events(mut self, enabled: bool) -> Self {
        self.include_span_events = enabled;
        self
    }

    /// Set a custom environment filter directive.
    ///
    /// This allows fine-grained control over log levels per module.
    ///
    /// # Example
    /// ```rust,no_run
    /// use keyrx_core::observability::logger::StructuredLogger;
    ///
    /// StructuredLogger::new()
    ///     .with_env_filter("keyrx=debug,hyper=warn")
    ///     .init()
    ///     .expect("Failed to initialize logger");
    /// ```
    ///
    /// # Arguments
    /// * `filter` - Environment filter directive
    pub fn with_env_filter<S: Into<String>>(mut self, filter: S) -> Self {
        self.env_filter = Some(filter.into());
        self
    }

    /// Initialize the logger as the global default.
    ///
    /// This must be called only once in the application lifecycle.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The logger has already been initialized
    /// - File creation fails (if file output is enabled)
    pub fn init(self) -> Result<(), LogError> {
        // Build the environment filter
        let env_filter = if let Some(filter) = self.env_filter {
            EnvFilter::try_new(filter)
                .map_err(|e| LogError::InitError(format!("Invalid filter: {}", e)))?
        } else {
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new(format!(
                    "{}={}",
                    env!("CARGO_PKG_NAME").replace('-', "_"),
                    self.level
                ))
            })
        };

        // Configure span events
        let span_events = if self.include_span_events {
            FmtSpan::ENTER | FmtSpan::CLOSE
        } else {
            FmtSpan::NONE
        };

        // Build the subscriber based on format and output destination
        // We always include the GLOBAL_LOG_BRIDGE to support FFI log streaming
        let registry = Registry::default()
            .with(env_filter)
            .with(GLOBAL_LOG_BRIDGE.clone());

        match (self.format, &self.file_path) {
            // Pretty format to stdout
            (OutputFormat::Pretty, None) => {
                let layer = tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .with_span_events(span_events)
                    .with_ansi(true)
                    .pretty();

                registry
                    .with(layer)
                    .try_init()
                    .map_err(|_| LogError::AlreadyInitialized)?;
            }

            // JSON format to stdout
            (OutputFormat::Json, None) => {
                let layer = tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_span_events(span_events)
                    .json();

                registry
                    .with(layer)
                    .try_init()
                    .map_err(|_| LogError::AlreadyInitialized)?;
            }

            // Compact format to stdout
            (OutputFormat::Compact, None) => {
                let layer = tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_span_events(span_events)
                    .with_ansi(false)
                    .compact();

                registry
                    .with(layer)
                    .try_init()
                    .map_err(|_| LogError::AlreadyInitialized)?;
            }

            // Pretty format to file
            (OutputFormat::Pretty, Some(path)) => {
                let file = std::fs::File::create(path)?;
                let layer = tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_span_events(span_events)
                    .with_ansi(false)
                    .with_writer(std::sync::Arc::new(file))
                    .pretty();

                registry
                    .with(layer)
                    .try_init()
                    .map_err(|_| LogError::AlreadyInitialized)?;
            }

            // JSON format to file
            (OutputFormat::Json, Some(path)) => {
                let file = std::fs::File::create(path)?;
                let layer = tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_span_events(span_events)
                    .with_writer(std::sync::Arc::new(file))
                    .json();

                registry
                    .with(layer)
                    .try_init()
                    .map_err(|_| LogError::AlreadyInitialized)?;
            }

            // Compact format to file
            (OutputFormat::Compact, Some(path)) => {
                let file = std::fs::File::create(path)?;
                let layer = tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_span_events(span_events)
                    .with_ansi(false)
                    .with_writer(std::sync::Arc::new(file))
                    .compact();

                registry
                    .with(layer)
                    .try_init()
                    .map_err(|_| LogError::AlreadyInitialized)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_builder() {
        let logger = StructuredLogger::new()
            .with_format(OutputFormat::Json)
            .with_level(Level::DEBUG)
            .with_span_events(false);

        assert_eq!(logger.format, OutputFormat::Json);
        assert_eq!(logger.level, Level::DEBUG);
        assert!(!logger.include_span_events);
    }

    #[test]
    fn test_default_logger() {
        let logger = StructuredLogger::default();
        assert_eq!(logger.format, OutputFormat::Pretty);
        assert_eq!(logger.level, Level::INFO);
        assert!(logger.include_span_events);
    }

    #[test]
    fn test_with_file() {
        let logger = StructuredLogger::new().with_file("/tmp/test.log");
        assert!(logger.file_path.is_some());
        assert_eq!(logger.file_path.unwrap(), PathBuf::from("/tmp/test.log"));
    }

    #[test]
    fn test_with_env_filter() {
        let logger = StructuredLogger::new().with_env_filter("keyrx=debug,hyper=warn");
        assert!(logger.env_filter.is_some());
        assert_eq!(logger.env_filter.unwrap(), "keyrx=debug,hyper=warn");
    }
}
