//! Procedural macros for KeyRX FFI exports
//!
//! This crate provides the `#[ffi_export]` attribute macro that automatically generates
//! C-ABI wrapper functions from Rust methods. The macro handles:
//! - Error conversion to FfiResult
//! - String parameter validation (null checks, UTF-8 validation)
//! - Panic catching to prevent panics from crossing FFI boundary
//! - JSON serialization of results
//!
//! # Example
//!
//! ```ignore
//! use keyrx_ffi_macros::ffi_export;
//!
//! struct MyDomain;
//!
//! impl MyDomain {
//!     #[ffi_export]
//!     fn my_function(&self, param: &str) -> Result<String, MyError> {
//!         Ok(format!("Hello, {}", param))
//!     }
//! }
//! ```

use proc_macro::TokenStream;

/// Attribute macro to generate C-ABI FFI wrappers for Rust methods
///
/// This macro transforms a Rust method into a C-compatible FFI export by:
/// 1. Creating a `#[no_mangle] pub extern "C"` wrapper function
/// 2. Adding null checks for pointer parameters
/// 3. Validating UTF-8 for string parameters
/// 4. Converting errors to FfiResult format
/// 5. Catching panics and converting them to error responses
/// 6. Serializing results to JSON
///
/// # Requirements
///
/// - The method must be part of a type implementing `FfiExportable`
/// - Return type must be `Result<T, E>` where both T and E are serializable
/// - String parameters should use `&str` or `String`
///
/// # Generated code
///
/// The macro generates a wrapper function with the same name prefixed by the domain name,
/// following the pattern: `keyrx_{domain}_{method_name}`
#[proc_macro_attribute]
pub fn ffi_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Placeholder implementation - will be fully implemented in task 6
    // For now, just return the original item unchanged so the crate compiles
    item
}
