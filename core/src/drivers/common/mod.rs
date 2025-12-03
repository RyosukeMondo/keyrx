//! Common types and utilities shared across platform-specific drivers.

pub mod error;
mod types;

// Re-export types for backward compatibility
pub use types::{extract_panic_message, DeviceInfo};
