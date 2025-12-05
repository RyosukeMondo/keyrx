//! JSON output validation tests.
//!
//! Tests JSON serialization and deserialization including:
//! - JSON output format validation
//! - Error and warning serialization
//! - Coverage data serialization
//! - JSON roundtrip preservation

use keyrx_core::validation::engine::ValidationEngine;
use keyrx_core::validation::types::ValidationOptions;

#[test]
fn json_output_is_parseable() {
    let script = r#"remap("CapsLock", "Escape");"#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    let json = serde_json::to_string(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["is_valid"], true);
    assert!(parsed["errors"].is_array());
    assert!(parsed["warnings"].is_array());
}

#[test]
fn json_errors_include_code_and_message() {
    let script = r#"layer_push("undefined");"#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    let json = serde_json::to_string(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(!parsed["errors"].as_array().unwrap().is_empty());
    let first_error = &parsed["errors"][0];
    assert!(first_error["code"].is_string());
    assert!(first_error["message"].is_string());
}

#[test]
fn json_warnings_include_category() {
    let script = r#"
        remap("A", "B");
        remap("A", "C");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    let json = serde_json::to_string(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(!parsed["warnings"].as_array().unwrap().is_empty());
    let first_warning = &parsed["warnings"][0];
    assert!(first_warning["category"].is_string());
}

#[test]
fn json_roundtrip_preserves_data() {
    let script = r#"
        remap("A", "B");
        block("C");
        layer_push("undefined");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new().with_coverage());

    let json = serde_json::to_string(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify data is preserved through JSON serialization
    assert_eq!(parsed["is_valid"], result.is_valid);
    assert_eq!(
        parsed["errors"].as_array().unwrap().len(),
        result.errors.len()
    );
    assert_eq!(
        parsed["warnings"].as_array().unwrap().len(),
        result.warnings.len()
    );
}
