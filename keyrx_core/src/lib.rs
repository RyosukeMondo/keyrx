#![no_std]

//! keyrx_core - Platform-agnostic keyboard remapping logic
//!
//! This crate contains the core remapping engine that is OS-agnostic and WASM-compatible.
//! It uses no_std to ensure it can be compiled to any target, including browser WASM.

extern crate alloc;

pub mod config;
pub mod error;
pub mod runtime;

// Parser module: error + validators are always available (no external deps).
// Full Parser type requires the "wasm" feature (rhai, sha2, spin).
pub mod parser;

// WASM module (only included when compiling for wasm32 target)
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub mod wasm;

// Re-export public types from config module
pub use config::{
    BaseKeyMapping, Condition, ConditionItem, ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode,
    KeyMapping, Metadata, Version,
};

// Re-export error types
pub use error::{CoreError, CoreResult};
