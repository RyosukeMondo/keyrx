//! Rhai DSL parser with prefix validation
//!
//! This module provides parsing and validation for the KeyRx configuration DSL.
//! It enforces strict prefix rules:
//! - VK_ prefix: Virtual keys (output mappings)
//! - MD_ prefix: Custom modifiers (00-FE hex range)
//! - LK_ prefix: Custom locks (00-FE hex range)

use crate::error::ParseError;
use keyrx_core::config::{Condition, KeyCode};

/// List of physical modifier names that cannot be used with MD_ prefix.
#[allow(dead_code)] // Will be used by full parser in task 13
const PHYSICAL_MODIFIERS: &[&str] = &[
    "LShift", "RShift", "LCtrl", "RCtrl", "LAlt", "RAlt", "LMeta", "RMeta",
];

/// Parses a virtual key string with VK_ prefix.
///
/// # Arguments
/// * `s` - Input string (e.g., "VK_A", "VK_Enter")
///
/// # Returns
/// * `Ok(KeyCode)` - The parsed key code
/// * `Err(ParseError)` - If the prefix is missing or the key name is invalid
///
/// # Examples
/// ```
/// # use keyrx_compiler::parser::parse_virtual_key;
/// # use keyrx_core::config::KeyCode;
/// let key = parse_virtual_key("VK_A").unwrap();
/// assert_eq!(key, KeyCode::A);
/// ```
#[allow(dead_code)] // Will be used by full parser in task 13
pub fn parse_virtual_key(s: &str) -> Result<KeyCode, ParseError> {
    // Check for VK_ prefix
    if !s.starts_with("VK_") {
        return Err(ParseError::MissingPrefix {
            key: s.to_string(),
            context: "virtual key".to_string(),
        });
    }

    // Extract key name after prefix
    let key_name = &s[3..];

    // Parse key name to KeyCode
    parse_key_name(key_name)
}

/// Parses a modifier ID string with MD_ prefix.
///
/// # Arguments
/// * `s` - Input string (e.g., "MD_00", "MD_FE")
///
/// # Returns
/// * `Ok(u8)` - The parsed modifier ID (0-254)
/// * `Err(ParseError)` - If the prefix is missing, format is invalid, or ID is out of range
///
/// # Errors
/// * `MissingPrefix` - If MD_ prefix is missing
/// * `PhysicalModifierInMD` - If a physical modifier name is used (e.g., "MD_LShift")
/// * `ModifierIdOutOfRange` - If the ID is > 0xFE (254)
/// * `InvalidPrefix` - If the format is invalid
///
/// # Examples
/// ```
/// # use keyrx_compiler::parser::parse_modifier_id;
/// let id = parse_modifier_id("MD_00").unwrap();
/// assert_eq!(id, 0);
///
/// let id = parse_modifier_id("MD_FE").unwrap();
/// assert_eq!(id, 254);
///
/// // Physical modifiers are rejected
/// assert!(parse_modifier_id("MD_LShift").is_err());
/// ```
#[allow(dead_code)] // Will be used by full parser in task 13
pub fn parse_modifier_id(s: &str) -> Result<u8, ParseError> {
    // Check for MD_ prefix
    if !s.starts_with("MD_") {
        return Err(ParseError::MissingPrefix {
            key: s.to_string(),
            context: "custom modifier".to_string(),
        });
    }

    // Extract ID part after prefix
    let id_part = &s[3..];

    // Check if it's a physical modifier name (not allowed)
    if PHYSICAL_MODIFIERS.contains(&id_part) {
        return Err(ParseError::PhysicalModifierInMD {
            name: id_part.to_string(),
        });
    }

    // Parse hex ID
    let id = u16::from_str_radix(id_part, 16).map_err(|_| ParseError::InvalidPrefix {
        expected: "MD_XX (hex, 00-FE)".to_string(),
        got: s.to_string(),
        context: "custom modifier ID".to_string(),
    })?;

    // Validate range (00-FE, max 254)
    if id > 0xFE {
        return Err(ParseError::ModifierIdOutOfRange { got: id, max: 0xFE });
    }

    Ok(id as u8)
}

/// Parses a lock ID string with LK_ prefix.
///
/// # Arguments
/// * `s` - Input string (e.g., "LK_00", "LK_FE")
///
/// # Returns
/// * `Ok(u8)` - The parsed lock ID (0-254)
/// * `Err(ParseError)` - If the prefix is missing, format is invalid, or ID is out of range
///
/// # Errors
/// * `MissingPrefix` - If LK_ prefix is missing
/// * `LockIdOutOfRange` - If the ID is > 0xFE (254)
/// * `InvalidPrefix` - If the format is invalid
///
/// # Examples
/// ```
/// # use keyrx_compiler::parser::parse_lock_id;
/// let id = parse_lock_id("LK_00").unwrap();
/// assert_eq!(id, 0);
///
/// let id = parse_lock_id("LK_FE").unwrap();
/// assert_eq!(id, 254);
/// ```
#[allow(dead_code)] // Will be used by full parser in task 13
pub fn parse_lock_id(s: &str) -> Result<u8, ParseError> {
    // Check for LK_ prefix
    if !s.starts_with("LK_") {
        return Err(ParseError::MissingPrefix {
            key: s.to_string(),
            context: "custom lock".to_string(),
        });
    }

    // Extract ID part after prefix
    let id_part = &s[3..];

    // Parse hex ID
    let id = u16::from_str_radix(id_part, 16).map_err(|_| ParseError::InvalidPrefix {
        expected: "LK_XX (hex, 00-FE)".to_string(),
        got: s.to_string(),
        context: "custom lock ID".to_string(),
    })?;

    // Validate range (00-FE, max 254)
    if id > 0xFE {
        return Err(ParseError::LockIdOutOfRange { got: id, max: 0xFE });
    }

    Ok(id as u8)
}

/// Parses a condition string (MD_XX or LK_XX) into a Condition variant.
///
/// # Arguments
/// * `s` - Input string (e.g., "MD_00", "LK_01")
///
/// # Returns
/// * `Ok(Condition)` - The parsed condition
/// * `Err(ParseError)` - If the format is invalid
///
/// # Examples
/// ```
/// # use keyrx_compiler::parser::parse_condition_string;
/// # use keyrx_core::config::Condition;
/// let cond = parse_condition_string("MD_00").unwrap();
/// assert_eq!(cond, Condition::ModifierActive(0));
///
/// let cond = parse_condition_string("LK_01").unwrap();
/// assert_eq!(cond, Condition::LockActive(1));
/// ```
#[allow(dead_code)] // Will be used by full parser in task 13
pub fn parse_condition_string(s: &str) -> Result<Condition, ParseError> {
    if s.starts_with("MD_") {
        let id = parse_modifier_id(s)?;
        Ok(Condition::ModifierActive(id))
    } else if s.starts_with("LK_") {
        let id = parse_lock_id(s)?;
        Ok(Condition::LockActive(id))
    } else {
        Err(ParseError::InvalidPrefix {
            expected: "MD_XX or LK_XX".to_string(),
            got: s.to_string(),
            context: "condition".to_string(),
        })
    }
}

/// Parses a key name (without prefix) to KeyCode.
///
/// # Arguments
/// * `name` - Key name (e.g., "A", "Enter", "Space")
///
/// # Returns
/// * `Ok(KeyCode)` - The parsed key code
/// * `Err(ParseError)` - If the key name is unknown
#[allow(dead_code)] // Will be used by full parser in task 13
fn parse_key_name(name: &str) -> Result<KeyCode, ParseError> {
    match name {
        // Letters
        "A" => Ok(KeyCode::A),
        "B" => Ok(KeyCode::B),
        "C" => Ok(KeyCode::C),
        "D" => Ok(KeyCode::D),
        "E" => Ok(KeyCode::E),
        "F" => Ok(KeyCode::F),
        "G" => Ok(KeyCode::G),
        "H" => Ok(KeyCode::H),
        "I" => Ok(KeyCode::I),
        "J" => Ok(KeyCode::J),
        "K" => Ok(KeyCode::K),
        "L" => Ok(KeyCode::L),
        "M" => Ok(KeyCode::M),
        "N" => Ok(KeyCode::N),
        "O" => Ok(KeyCode::O),
        "P" => Ok(KeyCode::P),
        "Q" => Ok(KeyCode::Q),
        "R" => Ok(KeyCode::R),
        "S" => Ok(KeyCode::S),
        "T" => Ok(KeyCode::T),
        "U" => Ok(KeyCode::U),
        "V" => Ok(KeyCode::V),
        "W" => Ok(KeyCode::W),
        "X" => Ok(KeyCode::X),
        "Y" => Ok(KeyCode::Y),
        "Z" => Ok(KeyCode::Z),

        // Numbers
        "Num0" | "0" => Ok(KeyCode::Num0),
        "Num1" | "1" => Ok(KeyCode::Num1),
        "Num2" | "2" => Ok(KeyCode::Num2),
        "Num3" | "3" => Ok(KeyCode::Num3),
        "Num4" | "4" => Ok(KeyCode::Num4),
        "Num5" | "5" => Ok(KeyCode::Num5),
        "Num6" | "6" => Ok(KeyCode::Num6),
        "Num7" | "7" => Ok(KeyCode::Num7),
        "Num8" | "8" => Ok(KeyCode::Num8),
        "Num9" | "9" => Ok(KeyCode::Num9),

        // Function keys
        "F1" => Ok(KeyCode::F1),
        "F2" => Ok(KeyCode::F2),
        "F3" => Ok(KeyCode::F3),
        "F4" => Ok(KeyCode::F4),
        "F5" => Ok(KeyCode::F5),
        "F6" => Ok(KeyCode::F6),
        "F7" => Ok(KeyCode::F7),
        "F8" => Ok(KeyCode::F8),
        "F9" => Ok(KeyCode::F9),
        "F10" => Ok(KeyCode::F10),
        "F11" => Ok(KeyCode::F11),
        "F12" => Ok(KeyCode::F12),

        // Physical modifiers
        "LShift" => Ok(KeyCode::LShift),
        "RShift" => Ok(KeyCode::RShift),
        "LCtrl" => Ok(KeyCode::LCtrl),
        "RCtrl" => Ok(KeyCode::RCtrl),
        "LAlt" => Ok(KeyCode::LAlt),
        "RAlt" => Ok(KeyCode::RAlt),
        "LMeta" => Ok(KeyCode::LMeta),
        "RMeta" => Ok(KeyCode::RMeta),

        // Special keys
        "Escape" | "Esc" => Ok(KeyCode::Escape),
        "Enter" | "Return" => Ok(KeyCode::Enter),
        "Backspace" => Ok(KeyCode::Backspace),
        "Tab" => Ok(KeyCode::Tab),
        "Space" => Ok(KeyCode::Space),
        "CapsLock" => Ok(KeyCode::CapsLock),
        "NumLock" => Ok(KeyCode::NumLock),
        "ScrollLock" => Ok(KeyCode::ScrollLock),
        "PrintScreen" => Ok(KeyCode::PrintScreen),
        "Pause" => Ok(KeyCode::Pause),
        "Insert" => Ok(KeyCode::Insert),
        "Delete" | "Del" => Ok(KeyCode::Delete),
        "Home" => Ok(KeyCode::Home),
        "End" => Ok(KeyCode::End),
        "PageUp" => Ok(KeyCode::PageUp),
        "PageDown" => Ok(KeyCode::PageDown),

        // Arrow keys
        "Left" => Ok(KeyCode::Left),
        "Right" => Ok(KeyCode::Right),
        "Up" => Ok(KeyCode::Up),
        "Down" => Ok(KeyCode::Down),

        // Additional special keys
        "LeftBracket" => Ok(KeyCode::LeftBracket),
        "RightBracket" => Ok(KeyCode::RightBracket),
        "Backslash" => Ok(KeyCode::Backslash),
        "Semicolon" => Ok(KeyCode::Semicolon),
        "Quote" => Ok(KeyCode::Quote),
        "Comma" => Ok(KeyCode::Comma),
        "Period" => Ok(KeyCode::Period),
        "Slash" => Ok(KeyCode::Slash),
        "Grave" => Ok(KeyCode::Grave),
        "Minus" => Ok(KeyCode::Minus),
        "Equal" => Ok(KeyCode::Equal),

        // Numpad keys
        "Numpad0" => Ok(KeyCode::Numpad0),
        "Numpad1" => Ok(KeyCode::Numpad1),
        "Numpad2" => Ok(KeyCode::Numpad2),
        "Numpad3" => Ok(KeyCode::Numpad3),
        "Numpad4" => Ok(KeyCode::Numpad4),
        "Numpad5" => Ok(KeyCode::Numpad5),
        "Numpad6" => Ok(KeyCode::Numpad6),
        "Numpad7" => Ok(KeyCode::Numpad7),
        "Numpad8" => Ok(KeyCode::Numpad8),
        "Numpad9" => Ok(KeyCode::Numpad9),
        "NumpadMultiply" => Ok(KeyCode::NumpadMultiply),
        "NumpadAdd" => Ok(KeyCode::NumpadAdd),
        "NumpadSubtract" => Ok(KeyCode::NumpadSubtract),
        "NumpadDivide" => Ok(KeyCode::NumpadDivide),
        "NumpadDecimal" => Ok(KeyCode::NumpadDecimal),
        "NumpadEnter" => Ok(KeyCode::NumpadEnter),

        // Unknown key name
        _ => Err(ParseError::InvalidPrefix {
            expected: "valid key name".to_string(),
            got: format!("VK_{}", name),
            context: "virtual key parsing".to_string(),
        }),
    }
}

#[cfg(test)]
mod prefix_tests {
    use super::*;

    #[test]
    fn test_parse_virtual_key_accepts_valid() {
        assert_eq!(parse_virtual_key("VK_A").unwrap(), KeyCode::A);
        assert_eq!(parse_virtual_key("VK_Enter").unwrap(), KeyCode::Enter);
        assert_eq!(parse_virtual_key("VK_Space").unwrap(), KeyCode::Space);
        assert_eq!(parse_virtual_key("VK_Escape").unwrap(), KeyCode::Escape);
        assert_eq!(parse_virtual_key("VK_F1").unwrap(), KeyCode::F1);
        assert_eq!(parse_virtual_key("VK_0").unwrap(), KeyCode::Num0);
    }

    #[test]
    fn test_parse_virtual_key_rejects_missing_prefix() {
        let result = parse_virtual_key("A");
        assert!(result.is_err());
        match result {
            Err(ParseError::MissingPrefix { key, context }) => {
                assert_eq!(key, "A");
                assert_eq!(context, "virtual key");
            }
            _ => panic!("Expected MissingPrefix error"),
        }
    }

    #[test]
    fn test_parse_virtual_key_rejects_unknown_key() {
        let result = parse_virtual_key("VK_Unknown");
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidPrefix { .. }) => {}
            _ => panic!("Expected InvalidPrefix error"),
        }
    }

    #[test]
    fn test_parse_modifier_id_accepts_valid() {
        assert_eq!(parse_modifier_id("MD_00").unwrap(), 0);
        assert_eq!(parse_modifier_id("MD_01").unwrap(), 1);
        assert_eq!(parse_modifier_id("MD_FE").unwrap(), 254);
        assert_eq!(parse_modifier_id("MD_0A").unwrap(), 10);
        assert_eq!(parse_modifier_id("MD_FF").is_err(), true); // FF is out of range
    }

    #[test]
    fn test_parse_modifier_id_rejects_physical_names() {
        let physical_names = ["MD_LShift", "MD_RShift", "MD_LCtrl", "MD_RCtrl"];
        for name in &physical_names {
            let result = parse_modifier_id(name);
            assert!(result.is_err());
            match result {
                Err(ParseError::PhysicalModifierInMD { .. }) => {}
                _ => panic!("Expected PhysicalModifierInMD error for {}", name),
            }
        }
    }

    #[test]
    fn test_parse_modifier_id_rejects_out_of_range() {
        let result = parse_modifier_id("MD_FF");
        assert!(result.is_err());
        match result {
            Err(ParseError::ModifierIdOutOfRange { got, max }) => {
                assert_eq!(got, 255);
                assert_eq!(max, 254);
            }
            _ => panic!("Expected ModifierIdOutOfRange error"),
        }
    }

    #[test]
    fn test_parse_modifier_id_rejects_missing_prefix() {
        let result = parse_modifier_id("00");
        assert!(result.is_err());
        match result {
            Err(ParseError::MissingPrefix { key, context }) => {
                assert_eq!(key, "00");
                assert_eq!(context, "custom modifier");
            }
            _ => panic!("Expected MissingPrefix error"),
        }
    }

    #[test]
    fn test_parse_modifier_id_rejects_invalid_format() {
        let result = parse_modifier_id("MD_XY");
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidPrefix { .. }) => {}
            _ => panic!("Expected InvalidPrefix error"),
        }
    }

    #[test]
    fn test_parse_lock_id_accepts_valid() {
        assert_eq!(parse_lock_id("LK_00").unwrap(), 0);
        assert_eq!(parse_lock_id("LK_01").unwrap(), 1);
        assert_eq!(parse_lock_id("LK_FE").unwrap(), 254);
        assert_eq!(parse_lock_id("LK_0A").unwrap(), 10);
    }

    #[test]
    fn test_parse_lock_id_rejects_out_of_range() {
        let result = parse_lock_id("LK_FF");
        assert!(result.is_err());
        match result {
            Err(ParseError::LockIdOutOfRange { got, max }) => {
                assert_eq!(got, 255);
                assert_eq!(max, 254);
            }
            _ => panic!("Expected LockIdOutOfRange error"),
        }
    }

    #[test]
    fn test_parse_lock_id_rejects_missing_prefix() {
        let result = parse_lock_id("00");
        assert!(result.is_err());
        match result {
            Err(ParseError::MissingPrefix { key, context }) => {
                assert_eq!(key, "00");
                assert_eq!(context, "custom lock");
            }
            _ => panic!("Expected MissingPrefix error"),
        }
    }

    #[test]
    fn test_parse_condition_string_handles_modifiers() {
        let cond = parse_condition_string("MD_00").unwrap();
        assert_eq!(cond, Condition::ModifierActive(0));

        let cond = parse_condition_string("MD_0A").unwrap();
        assert_eq!(cond, Condition::ModifierActive(10));
    }

    #[test]
    fn test_parse_condition_string_handles_locks() {
        let cond = parse_condition_string("LK_00").unwrap();
        assert_eq!(cond, Condition::LockActive(0));

        let cond = parse_condition_string("LK_0B").unwrap();
        assert_eq!(cond, Condition::LockActive(11));
    }

    #[test]
    fn test_parse_condition_string_rejects_invalid_prefix() {
        let result = parse_condition_string("VK_A");
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidPrefix {
                expected,
                got,
                context,
            }) => {
                assert_eq!(expected, "MD_XX or LK_XX");
                assert_eq!(got, "VK_A");
                assert_eq!(context, "condition");
            }
            _ => panic!("Expected InvalidPrefix error"),
        }
    }
}
