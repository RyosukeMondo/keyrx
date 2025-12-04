//! Observability module for KeyRx.
//!
//! This module provides structured logging, metrics collection, and FFI bridges
//! for exposing observability data to the Flutter UI.

pub mod logger;

// Re-export commonly used types
pub use logger::{LogError, OutputFormat, StructuredLogger};
