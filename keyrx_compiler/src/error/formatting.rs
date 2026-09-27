//! Error formatting utilities for user-friendly error messages.
//!
//! This module provides colored terminal output with code snippets,
//! location information, and helpful suggestions for fixing errors.
//!
//! Split by output format:
//! - `colored`: the primary colored formatter (`format_error`)
//! - `legacy`: plain-text formatter kept for backwards compatibility
//! - `json`: machine-readable JSON formatter

#![allow(dead_code)] // Functions will be used in CLI integration

mod colored;
mod json;
mod legacy;

#[allow(unused_imports)]
pub use colored::format_error;
#[allow(unused_imports)]
pub use json::format_error_json;
#[allow(unused_imports)]
pub use legacy::format_error_user_friendly;

/// Helper function to encode bytes as hex string.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}
