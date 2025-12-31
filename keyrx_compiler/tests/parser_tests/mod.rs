//! Shared test utilities for parser tests.
//!
//! This module provides common imports and helper functions used across
//! parser test modules.

// Re-export commonly used types
pub use keyrx_compiler::parser::core::Parser;
pub use keyrx_core::config::{BaseKeyMapping, KeyCode, KeyMapping};
pub use std::path::PathBuf;

// Declare test modules
mod devices_tests;
mod maps_tests;
mod modifiers_tests;
mod taps_tests;
mod when_device_tests;
mod when_not_tests;
mod when_tests;
