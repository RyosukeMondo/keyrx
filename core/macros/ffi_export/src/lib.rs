//! FFI Export Macro - Derive FFI Contracts from Rust Functions
//!
//! This procedural macro enables a Rust-first, AI-native FFI workflow by deriving
//! contract JSON from Rust function signatures. This establishes Rust code as the
//! single source of truth, eliminating manual JSON editing.
//!
//! # Key Features
//!
//! - **Rust as SSOT**: Derive contracts from Rust code, not the other way around
//! - **Zero manual JSON**: Contracts are generated automatically
//! - **AI-friendly**: Clear, declarative annotations that AI agents can understand
//! - **Type-safe**: Contract generation respects Rust's type system
//!
//! # Example
//!
//! ```ignore
//! use ffi_export::ffi_export;
//!
//! #[ffi_export(domain = "engine", version = "1.0")]
//! pub fn start_loop(profile_id: &str) -> Result<EngineState, EngineError> {
//!     // Implementation
//! }
//! ```
//!
//! This generates a contract JSON entry:
//!
//! ```json
//! {
//!   "name": "start_loop",
//!   "description": "Start the event processing loop",
//!   "parameters": [
//!     {
//!       "name": "profile_id",
//!       "type": "string",
//!       "description": "Profile to load before starting the loop",
//!       "required": true
//!     }
//!   ],
//!   "returns": {
//!     "type": "object",
//!     "description": "Engine start result"
//!   }
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

mod contract;
mod parser;
mod types;

/// Derive FFI contract JSON from a Rust function signature.
///
/// This macro extracts metadata from a Rust function and generates a contract
/// JSON entry that can be consumed by the FFI code generation pipeline.
///
/// # Attributes
///
/// - `domain` (required): FFI domain name (e.g., "engine", "config")
/// - `version` (optional): Contract version, defaults to "1.0"
/// - `rename` (optional): Override function name in contract
/// - `skip` (optional): Skip contract generation for this function
///
/// # Supported Types
///
/// ## Parameters
/// - `&str`, `String` → `"string"`
/// - `u32`, `i32`, etc. → `"number"`
/// - `bool` → `"bool"`
/// - Custom types → `"object"` (must implement Serialize)
/// - `Option<T>` → Makes parameter optional
///
/// ## Return Types
/// - `Result<T, E>` → Required for FFI functions
/// - `T` → Contract type based on T
/// - `()` → `"null"`
///
/// # Documentation
///
/// Doc comments (`///`) are extracted and included in the contract:
///
/// ```ignore
/// /// Start the event processing loop.
/// ///
/// /// This initializes the engine with the given profile.
/// #[ffi_export(domain = "engine")]
/// pub fn start_loop(profile_id: &str) -> Result<State, Error> {
///     // ...
/// }
/// ```
///
/// # Examples
///
/// ## Basic Function
///
/// ```ignore
/// #[ffi_export(domain = "config", version = "1.0")]
/// pub fn get_value(key: &str) -> Result<String, ConfigError> {
///     // Implementation
/// }
/// ```
///
/// ## Optional Parameters
///
/// ```ignore
/// #[ffi_export(domain = "engine")]
/// pub fn set_volume(device_id: &str, volume: Option<u32>) -> Result<(), Error> {
///     // volume parameter will be marked as optional in contract
/// }
/// ```
///
/// ## Custom Types
///
/// ```ignore
/// #[derive(Serialize)]
/// pub struct DeviceInfo {
///     vendor_id: u16,
///     product_id: u16,
/// }
///
/// #[ffi_export(domain = "device")]
/// pub fn get_device_info(device_id: &str) -> Result<DeviceInfo, Error> {
///     // Returns custom type serialized as JSON object
/// }
/// ```
///
/// ## Renaming
///
/// ```ignore
/// #[ffi_export(domain = "engine", rename = "engine_start")]
/// pub fn start(profile: &str) -> Result<State, Error> {
///     // Contract will use "engine_start" instead of "start"
/// }
/// ```
#[proc_macro_attribute]
pub fn ffi_export(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let config = match parser::parse_attr(attr) {
        Ok(cfg) => cfg,
        Err(err) => return err.to_compile_error().into(),
    };

    // Check if we should skip this function
    if config.skip {
        return quote! { #input }.into();
    }

    // Generate contract JSON from function signature
    let contract_entry = match contract::generate_contract_entry(&input, &config) {
        Ok(entry) => entry,
        Err(err) => return err.to_compile_error().into(),
    };

    // Emit the contract as a compile-time constant that can be collected later
    let contract_json = match serde_json::to_string_pretty(&contract_entry) {
        Ok(json) => json,
        Err(err) => {
            return syn::Error::new_spanned(
                &input,
                format!("Failed to serialize contract: {}", err),
            )
            .to_compile_error()
            .into();
        }
    };

    let fn_name = &input.sig.ident;
    let contract_const_name = quote::format_ident!("__FFI_CONTRACT_{}", fn_name.to_string().to_uppercase());
    let domain = &config.domain;

    // Generate output: original function + contract constant
    let output = quote! {
        #input

        // Store the contract as a compile-time constant
        // This can be collected by the codegen tool during build
        #[doc(hidden)]
        #[allow(dead_code)]
        const #contract_const_name: &str = #contract_json;

        // Also register the contract in the inventory for runtime introspection
        #[cfg(feature = "contract-inventory")]
        ::inventory::submit! {
            crate::ffi::ContractEntry {
                domain: #domain,
                contract: #contract_json,
            }
        }
    };

    output.into()
}
