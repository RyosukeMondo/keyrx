//! Observability module for KeyRx.
//!
//! This module provides structured logging, metrics collection, and FFI bridges
//! for exposing observability data to the Flutter UI.

pub mod entry;
pub mod logger;

// Re-export commonly used types
pub use entry::{LogEntry, LogLevel};
pub use logger::{LogError, OutputFormat, StructuredLogger};
