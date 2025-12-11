//! Procedural macro for generating FFI code from contracts.
//!
//! This crate provides the `#[keyrx_ffi]` attribute macro that generates FFI wrapper
//! functions based on contract definitions (`.ffi-contract.json` files).
//!
//! # Overview
//!
//! The macro reads a contract file at compile time and generates type-safe FFI wrappers
//! that adhere to the contract specification. This ensures that:
//!
//! - FFI function signatures match the contract exactly
//! - Error handling is consistent across all FFI functions
//! - Panic safety is maintained at the FFI boundary
//!
//! # Usage
//!
//! ```ignore
//! use keyrx_ffi_macro::keyrx_ffi;
//!
//! #[keyrx_ffi(domain = "config")]
//! impl ConfigDomain {
//!     // Methods are matched against the contract and FFI wrappers are generated
//!     fn get_config(&self, key: &str) -> Result<String, ConfigError> {
//!         // implementation
//!     }
//! }
//! ```
//!
//! # Contract Files
//!
//! Contract files are JSON files that specify the FFI interface:
//!
//! ```json
//! {
//!   "domain": "config",
//!   "version": "1.0.0",
//!   "functions": [
//!     {
//!       "name": "keyrx_config_get",
//!       "parameters": [
//!         { "name": "key", "type": "string" }
//!       ],
//!       "return_type": "string"
//!     }
//!   ]
//! }
//! ```
//!
//! # Generated Code
//!
//! For each function in the contract, the macro generates:
//!
//! - A `#[no_mangle] pub extern "C"` wrapper function
//! - Parameter validation and conversion
//! - Panic catching at the FFI boundary
//! - JSON serialization for return values
//! - Error pointer handling for error reporting

use proc_macro::TokenStream;

mod contract_loader;
mod parse;
mod type_mapper;

// Future modules will be added here as the implementation progresses:
// mod codegen;         // Tasks 11-13: Code generation

/// Attribute macro to generate FFI wrappers from contract definitions.
///
/// This macro reads a contract file at compile time and generates type-safe
/// FFI wrapper functions for each method in the impl block that matches
/// a function in the contract.
///
/// # Arguments
///
/// - `domain`: The domain name, used to locate the contract file.
///   The contract file is expected at `contracts/{domain}.ffi-contract.json`
///   relative to the crate root.
///
/// # Example
///
/// ```ignore
/// use keyrx_ffi_macro::keyrx_ffi;
///
/// #[keyrx_ffi(domain = "config")]
/// impl ConfigDomain {
///     fn get_config(&self, key: &str) -> Result<String, ConfigError> {
///         // implementation
///     }
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates FFI wrappers that:
///
/// 1. Use `#[no_mangle]` for C linkage
/// 2. Accept an error pointer for error reporting
/// 3. Catch panics at the FFI boundary
/// 4. Serialize results to JSON
///
/// # Errors
///
/// Compile-time errors are generated for:
///
/// - Missing contract file
/// - Invalid contract JSON
/// - Mismatched function signatures
/// - Invalid domain parameter
#[proc_macro_attribute]
pub fn keyrx_ffi(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Placeholder implementation - will be completed in Task 14
    // For now, just return the item unchanged to allow the crate to compile
    let _ = attr; // Suppress unused warning

    // Parse the item to validate it's an impl block
    let input = syn::parse_macro_input!(item as syn::ItemImpl);

    // Return the original impl unchanged for now
    // Full implementation will be added in subsequent tasks
    quote::quote! { #input }.into()
}
