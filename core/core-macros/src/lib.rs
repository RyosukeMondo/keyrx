//! Procedural macros for KeyRX Core
//!
//! This crate provides the `#[rhai_doc]` attribute macro for automatic
//! documentation extraction and registration of Rhai API functions.
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core_macros::rhai_doc;
//!
//! /// Emits a key press event
//! ///
//! /// # Parameters
//! /// * `key` - The key code to emit
//! ///
//! /// # Returns
//! /// Returns nothing
//! ///
//! /// # Examples
//! /// ```rhai
//! /// emit_key(Key::A);
//! /// emit_key(Key::Enter);
//! /// ```
//! #[rhai_doc(module = "keys")]
//! fn emit_key(key: KeyCode) -> Result<(), Error> {
//!     // implementation
//! }
//! ```

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

mod rhai_doc;

/// Attribute macro to extract documentation and register with DocRegistry
///
/// This macro:
/// 1. Extracts doc comments from the function
/// 2. Parses function signature to extract parameters and return type
/// 3. Extracts examples from doc comments (code blocks marked with `rhai`)
/// 4. Generates code to register documentation with the global DocRegistry
///
/// # Required attributes
///
/// - `module`: The module this function belongs to (e.g., "keys", "layers")
///
/// # Optional attributes
///
/// - `since`: Version when this function was added (e.g., "0.1.0")
/// - `deprecated`: Deprecation message if this function is deprecated
///
/// # Doc comment format
///
/// The macro parses doc comments with the following structure:
///
/// ```text
/// /// Brief description (first paragraph)
/// ///
/// /// # Parameters
/// /// * `param_name` - Parameter description
/// ///
/// /// # Returns
/// /// Return value description
/// ///
/// /// # Examples
/// /// ```rhai
/// /// example_code();
/// /// ```
/// ///
/// /// # Notes
/// /// Additional notes or warnings
/// ```
///
/// # Example
///
/// ```ignore
/// #[rhai_doc(module = "keys")]
/// fn emit_key(key: KeyCode) -> Result<(), Error> {
///     // implementation
/// }
/// ```
#[proc_macro_attribute]
pub fn rhai_doc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let attr_tokens = proc_macro2::TokenStream::from(attr);

    match rhai_doc::generate_rhai_doc(attr_tokens, &input) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
