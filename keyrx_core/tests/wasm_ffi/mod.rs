//! WASM FFI boundary tests module.
//!
//! This module contains tests that verify the WASM FFI boundary works correctly,
//! ensuring that data structures and error formats match TypeScript expectations.

#![cfg(target_arch = "wasm32")]

pub mod validation_test;
