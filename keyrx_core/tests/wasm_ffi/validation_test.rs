//! WASM FFI validation boundary tests.
//!
//! These tests verify that the validate_config() function works correctly
//! across the WASM FFI boundary, ensuring proper error format and behavior
//! that matches TypeScript expectations.
//!
//! # Running Tests
//!
//! ```bash
//! # Firefox (headless)
//! wasm-pack test --headless --firefox keyrx_core --test wasm_ffi_validation
//!
//! # Chrome (headless)
//! wasm-pack test --headless --chrome keyrx_core --test wasm_ffi_validation
//! ```
//!
//! # Prerequisites
//!
//! Install wasm-pack:
//! ```bash
//! cargo install wasm-pack
//! ```

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

use keyrx_core::wasm::validate_config;
use serde_json::Value;

// Configure wasm-bindgen-test to run in browser
wasm_bindgen_test_configure!(run_in_browser);

// ============================================================================
// Valid Configuration Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_validate_config_accepts_valid_device_start_syntax() {
    // Test that validate_config accepts the new device_start/device_end syntax
    let valid_source = r#"
        device("*") {
            map("A", "B");
        }
    "#;

    let result = validate_config(valid_source);
    assert!(
        result.is_ok(),
        "validate_config should not fail on valid syntax"
    );

    if let Ok(js_value) = result {
        // Parse the returned JSON array
        let json_str = format!("{:?}", js_value);
        // Valid config should return empty array
        let errors: Vec<Value> = serde_wasm_bindgen::from_value(js_value)
            .expect("Should deserialize to Vec<ValidationError>");

        assert_eq!(
            errors.len(),
            0,
            "Valid Rhai syntax should return zero errors, got: {}",
            json_str
        );
    }
}

#[wasm_bindgen_test]
fn test_validate_config_accepts_multiple_mappings() {
    let valid_source = r#"
        device("*") {
            map("A", "B");
            map("B", "A");
        }
    "#;

    let result = validate_config(valid_source);
    assert!(
        result.is_ok(),
        "validate_config should not fail on valid syntax"
    );

    if let Ok(js_value) = result {
        let errors: Vec<Value> = serde_wasm_bindgen::from_value(js_value)
            .expect("Should deserialize to Vec<ValidationError>");

        assert_eq!(
            errors.len(),
            0,
            "Valid configuration with multiple mappings should return zero errors"
        );
    }
}

#[wasm_bindgen_test]
fn test_validate_config_accepts_empty_configuration() {
    // Empty configuration is technically valid
    let valid_source = "";

    let result = validate_config(valid_source);
    assert!(
        result.is_ok(),
        "validate_config should not fail on empty config"
    );

    if let Ok(js_value) = result {
        let errors: Vec<Value> = serde_wasm_bindgen::from_value(js_value)
            .expect("Should deserialize to Vec<ValidationError>");

        assert_eq!(
            errors.len(),
            0,
            "Empty configuration should return zero errors"
        );
    }
}

// ============================================================================
// Invalid Configuration Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_validate_config_rejects_invalid_syntax_with_structured_error() {
    // Missing closing brace
    let invalid_source = r#"
        device("*") {
            map("A", "B");
    "#;

    let result = validate_config(invalid_source);
    assert!(
        result.is_ok(),
        "validate_config should return Ok with error array, not Err"
    );

    if let Ok(js_value) = result {
        let errors: Vec<Value> = serde_wasm_bindgen::from_value(js_value)
            .expect("Should deserialize to Vec<ValidationError>");

        assert!(
            !errors.is_empty(),
            "Invalid syntax should return at least one error"
        );

        // Verify error structure matches TypeScript ValidationError interface
        let first_error = &errors[0];
        assert!(
            first_error.get("line").is_some(),
            "Error should have 'line' field"
        );
        assert!(
            first_error.get("column").is_some(),
            "Error should have 'column' field"
        );
        assert!(
            first_error.get("length").is_some(),
            "Error should have 'length' field"
        );
        assert!(
            first_error.get("message").is_some(),
            "Error should have 'message' field"
        );

        // Verify types
        assert!(
            first_error["line"].is_u64() || first_error["line"].is_i64(),
            "line should be a number"
        );
        assert!(
            first_error["column"].is_u64() || first_error["column"].is_i64(),
            "column should be a number"
        );
        assert!(
            first_error["length"].is_u64() || first_error["length"].is_i64(),
            "length should be a number"
        );
        assert!(
            first_error["message"].is_string(),
            "message should be a string"
        );
    }
}

#[wasm_bindgen_test]
fn test_validate_config_error_has_valid_line_number() {
    // Invalid syntax on line 3
    let invalid_source = r#"
        device("*") {
            map("A", "B"
        }
    "#; // Missing closing parenthesis

    let result = validate_config(invalid_source);
    assert!(result.is_ok(), "validate_config should return Ok");

    if let Ok(js_value) = result {
        let errors: Vec<Value> = serde_wasm_bindgen::from_value(js_value)
            .expect("Should deserialize to Vec<ValidationError>");

        assert!(!errors.is_empty(), "Should have at least one error");

        let line = errors[0]["line"].as_u64().expect("line should be a number");
        assert!(line > 0, "Line number should be positive (1-indexed)");

        // Line should be reasonable (not 0, not excessively large)
        assert!(
            line <= 10,
            "Line number should be within source bounds, got: {}",
            line
        );
    }
}

#[wasm_bindgen_test]
fn test_validate_config_error_has_valid_column_number() {
    let invalid_source = r#"device("*") { map("A", "B" }"#; // Missing closing paren

    let result = validate_config(invalid_source);
    assert!(result.is_ok(), "validate_config should return Ok");

    if let Ok(js_value) = result {
        let errors: Vec<Value> = serde_wasm_bindgen::from_value(js_value)
            .expect("Should deserialize to Vec<ValidationError>");

        assert!(!errors.is_empty(), "Should have at least one error");

        let column = errors[0]["column"]
            .as_u64()
            .expect("column should be a number");
        assert!(column > 0, "Column number should be positive (1-indexed)");
    }
}

#[wasm_bindgen_test]
fn test_validate_config_error_message_is_descriptive() {
    let invalid_source = r#"
        device("*") {
            map("UNSUPPORTED_KEY", "B");
        }
    "#;

    let result = validate_config(invalid_source);
    assert!(result.is_ok(), "validate_config should return Ok");

    if let Ok(js_value) = result {
        let errors: Vec<Value> = serde_wasm_bindgen::from_value(js_value)
            .expect("Should deserialize to Vec<ValidationError>");

        assert!(!errors.is_empty(), "Should have at least one error");

        let message = errors[0]["message"]
            .as_str()
            .expect("message should be a string");

        // Error message should be non-empty and descriptive
        assert!(!message.is_empty(), "Error message should not be empty");
        assert!(
            message.len() > 10,
            "Error message should be descriptive, got: {}",
            message
        );
    }
}

// ============================================================================
// TypeScript Interface Compatibility Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_validate_config_return_type_matches_typescript_interface() {
    // This test verifies that the return type matches:
    // interface ValidationError {
    //   line: number;
    //   column: number;
    //   length: number;
    //   message: string;
    // }

    let invalid_source = r#"device("*" { }"#; // Missing closing paren

    let result = validate_config(invalid_source);
    assert!(
        result.is_ok(),
        "validate_config should return Ok(JsValue) not Err"
    );

    if let Ok(js_value) = result {
        // Should deserialize as array
        let errors: Vec<Value> =
            serde_wasm_bindgen::from_value(js_value).expect("Should deserialize to Vec");

        assert!(!errors.is_empty(), "Should have errors for invalid syntax");

        for (index, error) in errors.iter().enumerate() {
            // Verify exact field names (TypeScript is case-sensitive)
            assert!(
                error.get("line").is_some(),
                "Error {} missing 'line' field",
                index
            );
            assert!(
                error.get("column").is_some(),
                "Error {} missing 'column' field",
                index
            );
            assert!(
                error.get("length").is_some(),
                "Error {} missing 'length' field",
                index
            );
            assert!(
                error.get("message").is_some(),
                "Error {} missing 'message' field",
                index
            );

            // Should not have extra fields
            let field_count = error.as_object().unwrap().len();
            assert_eq!(
                field_count, 4,
                "Error should have exactly 4 fields (line, column, length, message)"
            );
        }
    }
}

#[wasm_bindgen_test]
fn test_validate_config_returns_array_not_single_object() {
    let invalid_source = r#"invalid syntax"#;

    let result = validate_config(invalid_source);
    assert!(result.is_ok(), "validate_config should return Ok");

    if let Ok(js_value) = result {
        // Should be an array, not a single object
        let errors: Vec<Value> =
            serde_wasm_bindgen::from_value(js_value).expect("Return value should be an array");

        // Should have at least one error
        assert!(
            !errors.is_empty(),
            "Invalid syntax should return array with errors"
        );
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_validate_config_handles_large_input() {
    // Test configuration near 1MB limit
    let mut large_config = "device(\"*\") { ".to_string();
    large_config.push_str(&"map(\"A\", \"B\"); ".repeat(10000));
    large_config.push_str(" }");

    let result = validate_config(&large_config);
    assert!(result.is_ok(), "validate_config should handle large input");

    // Large valid config should return empty errors array
    if let Ok(js_value) = result {
        let errors: Vec<Value> = serde_wasm_bindgen::from_value(js_value)
            .expect("Should deserialize to Vec<ValidationError>");

        // Should be valid (empty errors) or return size error
        if !errors.is_empty() {
            let message = errors[0]["message"]
                .as_str()
                .expect("message should be a string");
            assert!(
                message.contains("too large") || message.contains("max"),
                "Error should mention size limit"
            );
        }
    }
}

#[wasm_bindgen_test]
fn test_validate_config_handles_config_exceeding_size_limit() {
    // Test configuration exceeding 1MB limit
    let oversized_config = "x".repeat(2 * 1024 * 1024); // 2MB

    let result = validate_config(&oversized_config);
    assert!(result.is_ok(), "validate_config should return Ok");

    if let Ok(js_value) = result {
        let errors: Vec<Value> = serde_wasm_bindgen::from_value(js_value)
            .expect("Should deserialize to Vec<ValidationError>");

        assert_eq!(errors.len(), 1, "Should return exactly one error for size");

        let message = errors[0]["message"]
            .as_str()
            .expect("message should be a string");
        assert!(
            message.contains("too large"),
            "Error should mention size limit, got: {}",
            message
        );
    }
}

#[wasm_bindgen_test]
fn test_validate_config_handles_unicode_correctly() {
    let unicode_source = r#"
        device("🎮") {
            map("A", "B");
        }
    "#;

    let result = validate_config(unicode_source);
    assert!(result.is_ok(), "validate_config should handle unicode");

    if let Ok(js_value) = result {
        let errors: Vec<Value> = serde_wasm_bindgen::from_value(js_value)
            .expect("Should deserialize to Vec<ValidationError>");

        // Should be valid (device pattern can be unicode)
        assert_eq!(errors.len(), 0, "Unicode device pattern should be valid");
    }
}

// ============================================================================
// Deterministic Validation Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_validate_config_is_deterministic() {
    let source = r#"
        device("*") {
            map("A", "B");
            map("B", "A");
        }
    "#;

    // Run validation multiple times
    let result1 = validate_config(source).expect("Should return Ok");
    let result2 = validate_config(source).expect("Should return Ok");
    let result3 = validate_config(source).expect("Should return Ok");

    let errors1: Vec<Value> = serde_wasm_bindgen::from_value(result1).expect("Should deserialize");
    let errors2: Vec<Value> = serde_wasm_bindgen::from_value(result2).expect("Should deserialize");
    let errors3: Vec<Value> = serde_wasm_bindgen::from_value(result3).expect("Should deserialize");

    // Results should be identical
    assert_eq!(errors1.len(), errors2.len());
    assert_eq!(errors2.len(), errors3.len());
}

#[wasm_bindgen_test]
fn test_validate_config_error_positions_are_deterministic() {
    let invalid_source = r#"
        device("*") {
            map("A", "B"
        }
    "#; // Missing closing paren

    // Run validation multiple times
    let result1 = validate_config(invalid_source).expect("Should return Ok");
    let result2 = validate_config(invalid_source).expect("Should return Ok");

    let errors1: Vec<Value> = serde_wasm_bindgen::from_value(result1).expect("Should deserialize");
    let errors2: Vec<Value> = serde_wasm_bindgen::from_value(result2).expect("Should deserialize");

    // Error positions should be identical
    assert_eq!(errors1.len(), errors2.len());
    if !errors1.is_empty() {
        assert_eq!(errors1[0]["line"], errors2[0]["line"]);
        assert_eq!(errors1[0]["column"], errors2[0]["column"]);
        assert_eq!(errors1[0]["message"], errors2[0]["message"]);
    }
}
