//! Observability module for KeyRx.
//!
//! This module provides structured logging, metrics collection, and FFI bridges
//! for exposing observability data to the Flutter UI.

pub mod bridge;
pub mod entry;
pub mod logger;

// Re-export commonly used types
pub use bridge::LogBridge;
pub use entry::{LogEntry, LogLevel};
pub use logger::{LogError, OutputFormat, StructuredLogger};
