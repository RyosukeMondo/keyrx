//! Resource limits for evaluating a configuration script.
//!
//! ONE source of truth for every parser (the compiler, its `load()` imports
//! and the WASM parser behind the UI), so a profile that compiles in one
//! place compiles in all of them, and a hostile or accidental script
//! (`loop {}`, a file that loads itself) fails with an error instead of
//! hanging or overflowing the stack.

/// Rhai operations one script may run. A mapping costs a handful, so this
/// admits several hundred thousand mappings while an endless loop still
/// stops in well under a second.
pub const MAX_OPERATIONS: u64 = 2_000_000;

/// Maximum expression nesting (global, inside functions).
pub const MAX_EXPR_DEPTH: usize = 100;

/// Maximum depth of Rhai function calls.
pub const MAX_CALL_LEVELS: usize = 100;

/// Maximum depth of `load()` imports. Each level evaluates in its own
/// engine on the Rust stack, so this must stay small.
pub const MAX_IMPORT_DEPTH: usize = 16;

/// Largest script source accepted, in bytes.
pub const MAX_SOURCE_BYTES: u64 = 16 * 1024 * 1024;

/// Strips a leading UTF-8 byte-order mark. Windows editors add one when
/// saving "UTF-8"; Rhai rejects it as an unexpected character.
#[must_use]
pub fn strip_bom(source: &str) -> &str {
    source.strip_prefix('\u{feff}').unwrap_or(source)
}
