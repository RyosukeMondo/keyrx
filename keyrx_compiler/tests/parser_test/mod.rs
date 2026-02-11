//! Shared test utilities for parser tests.
//!
//! This module provides common imports and helper functions used across
//! parser test modules.

// Re-export commonly used types
pub use keyrx_compiler::parser::core::Parser;
pub use keyrx_core::config::{BaseKeyMapping, KeyCode, KeyMapping};
pub use std::path::PathBuf;

// Declare test modules
mod devices_test;
mod maps_test;
mod modifiers_test;
mod taps_test;
mod when_device_test;
mod when_not_test;
mod when_test;
