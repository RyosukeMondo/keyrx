//! Common types and utilities shared across platform-specific drivers.

pub mod cache;
pub mod error;
pub mod recovery;
mod types;

// Re-export types for backward compatibility
pub use types::{extract_panic_message, DeviceInfo};
