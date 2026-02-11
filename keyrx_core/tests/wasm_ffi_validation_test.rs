//! WASM FFI validation test suite entry point.
//!
//! This test file is the entry point for wasm-pack testing.
//!
//! # Running Tests
//!
//! ```bash
//! # Firefox (headless)
//! wasm-pack test --headless --firefox keyrx_core --test wasm_ffi_validation
//!
//! # Chrome (headless)
//! wasm-pack test --headless --chrome keyrx_core --test wasm_ffi_validation
//!
//! # Safari (requires Safari driver)
//! wasm-pack test --headless --safari keyrx_core --test wasm_ffi_validation
//! ```

#![cfg(target_arch = "wasm32")]

mod wasm_ffi;
