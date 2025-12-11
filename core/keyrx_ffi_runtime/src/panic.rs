//! Panic handling utilities for FFI boundaries.
//!
//! Provides panic catching to prevent Rust panics from crossing FFI boundaries,
//! which would cause undefined behavior.

use std::panic::{catch_unwind, AssertUnwindSafe};

/// Catch any panics from the given closure and convert them to an error.
///
/// This function wraps a closure in `catch_unwind` to prevent panics from
/// crossing the FFI boundary, which would be undefined behavior.
///
/// # Arguments
///
/// * `f` - The closure to execute (wrapped in `AssertUnwindSafe`)
///
/// # Returns
///
/// * `Ok(T)` - The closure executed successfully
/// * `Err(String)` - The closure panicked; contains the panic message
///
/// # Example
///
/// ```ignore
/// let result = handle_panic(|| {
///     // potentially panicking code
///     42
/// });
/// ```
pub fn handle_panic<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> T,
{
    catch_unwind(AssertUnwindSafe(f)).map_err(|panic_payload| {
        if let Some(s) = panic_payload.downcast_ref::<&str>() {
            format!("Panic: {}", s)
        } else if let Some(s) = panic_payload.downcast_ref::<String>() {
            format!("Panic: {}", s)
        } else {
            "Panic: unknown panic payload".to_string()
        }
    })
}
