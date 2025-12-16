#![allow(unexpected_cfgs)]
#![allow(private_interfaces)]
//! Basic tests for #[ffi_export] macro

use ffi_export::ffi_export;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct TestState {
    running: bool,
}

#[derive(Debug)]
struct TestError;

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TestError")
    }
}

impl std::error::Error for TestError {}

/// Start the test loop.
///
/// This function initializes the test engine.
#[ffi_export(domain = "test", version = "1.0")]
pub fn start_test(profile_id: &str) -> Result<TestState, TestError> {
    Ok(TestState { running: true })
}

/// Get a configuration value.
#[ffi_export(domain = "config")]
pub fn get_value(_key: &str) -> Result<String, TestError> {
    Ok(format!("value_{}", _key))
}

/// Set volume with optional parameter.
#[ffi_export(domain = "audio")]
pub fn set_volume(_device_id: &str, _volume: Option<u32>) -> Result<(), TestError> {
    Ok(())
}

/// Function with numeric parameters.
#[ffi_export(domain = "math")]
pub fn add_numbers(a: i32, b: i32) -> Result<i32, TestError> {
    Ok(a + b)
}

/// Function with boolean parameter.
#[ffi_export(domain = "config")]
pub fn set_enabled(_enabled: bool) -> Result<(), TestError> {
    Ok(())
}

/// Function with custom return type.
#[ffi_export(domain = "test")]
pub fn get_state() -> Result<TestState, TestError> {
    Ok(TestState { running: false })
}

/// Function with renamed contract entry.
#[ffi_export(domain = "test", rename = "test_renamed_func")]
pub fn original_name() -> Result<(), TestError> {
    Ok(())
}

#[test]
fn test_basic_function() {
    // Test that the function still works normally
    let result = start_test("profile1");
    assert!(result.is_ok());
    assert!(result.unwrap().running);
}

#[test]
fn test_optional_parameter() {
    // Test function with optional parameter
    let result = set_volume("device1", Some(50));
    assert!(result.is_ok());

    let result = set_volume("device1", None);
    assert!(result.is_ok());
}

#[test]
fn test_numeric_function() {
    let result = add_numbers(5, 3);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 8);
}

#[test]
fn test_boolean_parameter() {
    let result = set_enabled(true);
    assert!(result.is_ok());
}

#[test]
fn test_contract_constant_exists() {
    // Verify that the contract constant is generated
    // This won't compile if the macro doesn't generate the constant
    const _: &str = __FFI_CONTRACT_START_TEST;
}
