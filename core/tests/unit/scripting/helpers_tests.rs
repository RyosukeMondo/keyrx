//! Unit tests for scripting::helpers module.

use keyrx_core::engine::KeyCode;
use keyrx_core::scripting::helpers::parse_key_or_error;

#[test]
fn parse_key_or_error_valid_key() {
    let result = parse_key_or_error("a", "test");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), KeyCode::A);
}

#[test]
fn parse_key_or_error_valid_key_uppercase() {
    let result = parse_key_or_error("A", "test");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), KeyCode::A);
}

#[test]
fn parse_key_or_error_modifier() {
    let result = parse_key_or_error("LeftCtrl", "block");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), KeyCode::LeftCtrl);
}

#[test]
fn parse_key_or_error_space() {
    let result = parse_key_or_error("Space", "pass");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), KeyCode::Space);
}

#[test]
fn parse_key_or_error_invalid_key() {
    let result = parse_key_or_error("InvalidKey123", "remap");
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("InvalidKey123"));
    assert!(err_str.contains(".spec-workflow/steering/tech.md (Key Naming & Aliases)."));
}

#[test]
fn parse_key_or_error_empty_string() {
    let result = parse_key_or_error("", "block");
    assert!(result.is_err());
}
