#![no_std]

//! keyrx_core - Platform-agnostic keyboard remapping logic
//!
//! This crate contains the core remapping engine that is OS-agnostic and WASM-compatible.
//! It uses no_std to ensure it can be compiled to any target, including browser WASM.

extern crate alloc;

pub mod config;
pub mod dfa;
pub mod error;
pub mod runtime;
pub mod simulator;
pub mod state;

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
