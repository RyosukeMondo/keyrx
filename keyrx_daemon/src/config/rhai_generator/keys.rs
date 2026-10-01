//! Key-name rules for profile edits. One definition of "the same key", so
//! `CapsLock` and `VK_CapsLock` can never become two mappings.
//!
//! The DSL rule (docs/user-guide/dsl-manual.md): input keys are bare names
//! (`CapsLock`; a `VK_` prefix is tolerated), output keys are written `VK_...`,
//! and `MD_xx` / `LK_xx` name custom modifiers and locks.

use keyrx_core::parser::validators::parse_physical_key;

use super::GeneratorError;

const MAX_KEY_LEN: usize = 64;

fn is_custom(name: &str) -> bool {
    name.starts_with("MD_") || name.starts_with("LK_")
}

fn check_len(name: &str) -> Result<(), GeneratorError> {
    if name.is_empty() || name.len() > MAX_KEY_LEN {
        return Err(GeneratorError::InvalidKeyName(format!(
            "key name must be 1 to {MAX_KEY_LEN} characters: '{name}'"
        )));
    }
    Ok(())
}

fn unknown(name: &str) -> GeneratorError {
    GeneratorError::InvalidKeyName(format!(
        "unknown key '{name}' (use a name like CapsLock, A, Escape, F13; MD_xx for custom modifiers)"
    ))
}

/// An input key as the DSL spells it: bare (`CapsLock`), or `MD_xx`/`LK_xx`.
/// A `VK_` prefix is accepted and dropped.
///
/// # Errors
///
/// [`GeneratorError::InvalidKeyName`] if the name is not a known key.
pub fn canonical_input(key: &str) -> Result<String, GeneratorError> {
    check_len(key)?;
    if is_custom(key) {
        return Ok(key.to_string());
    }
    let bare = key.strip_prefix("VK_").unwrap_or(key);
    parse_physical_key(bare).map_err(|_| unknown(key))?;
    Ok(bare.to_string())
}

/// An output key as the DSL spells it: `VK_...`, `MD_xx` or `LK_xx`. A bare
/// key name is accepted and gets its `VK_` prefix.
///
/// # Errors
///
/// [`GeneratorError::InvalidKeyName`] if the name is not a known key.
pub fn canonical_output(key: &str) -> Result<String, GeneratorError> {
    check_len(key)?;
    if is_custom(key) {
        return Ok(key.to_string());
    }
    let bare = key.strip_prefix("VK_").unwrap_or(key);
    parse_physical_key(bare).map_err(|_| unknown(key))?;
    Ok(format!("VK_{bare}"))
}

/// Whether two spellings denote the same key (`A` and `VK_A`, `Esc` and
/// `Escape` if the parser treats them alike).
#[must_use]
pub fn same_key(a: &str, b: &str) -> bool {
    identity(a) == identity(b)
}

fn identity(name: &str) -> String {
    if is_custom(name) {
        return name.to_string();
    }
    let bare = name.strip_prefix("VK_").unwrap_or(name);
    parse_physical_key(bare).map_or_else(|_| name.to_string(), |code| format!("{code:?}"))
}
