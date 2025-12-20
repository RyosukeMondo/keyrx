#![allow(unexpected_cfgs)]
#![allow(private_interfaces)]
//! Edge case tests for #[ffi_export] macro

use ffi_export::ffi_export;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct TestError;

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TestError")
    }
}

impl std::error::Error for TestError {}

#[derive(Debug, Serialize, Deserialize)]
struct ComplexType {
    field1: String,
    field2: u32,
}

/// Function with no parameters.
#[ffi_export(domain = "test")]
pub fn no_params() -> Result<String, TestError> {
    Ok("success".to_string())
}

/// Function with unit return type.
#[ffi_export(domain = "test")]
pub fn unit_return() -> Result<(), TestError> {
    Ok(())
}

/// Function with Vec parameter.
#[ffi_export(domain = "test")]
pub fn vec_param(items: Vec<String>) -> Result<usize, TestError> {
    Ok(items.len())
}

/// Function with multiple optional parameters.
#[ffi_export(domain = "test")]
pub fn multiple_optional(
    _required: &str,
    _opt1: Option<u32>,
    _opt2: Option<String>,
) -> Result<(), TestError> {
    Ok(())
}

/// Function with complex custom type.
#[ffi_export(domain = "test")]
pub fn complex_type() -> Result<ComplexType, TestError> {
    Ok(ComplexType {
        field1: "test".to_string(),
        field2: 42,
    })
}

/// Function with floating point parameter.
#[ffi_export(domain = "math")]
pub fn calculate(value: f64) -> Result<f64, TestError> {
    Ok(value * 2.0)
}

/// Function with all primitive types.
#[ffi_export(domain = "test")]
pub fn all_primitives(s: &str, u: u32, i: i32, f: f64, b: bool) -> Result<String, TestError> {
    Ok(format!("{} {} {} {} {}", s, u, i, f, b))
}

#[test]
fn test_no_params() -> Result<(), Box<dyn std::error::Error>> {
    no_params()?;
    Ok(())
}

#[test]
fn test_unit_return() -> Result<(), Box<dyn std::error::Error>> {
    unit_return()?;
    Ok(())
}

#[test]
fn test_vec_param() -> Result<(), Box<dyn std::error::Error>> {
    let items = vec!["a".to_string(), "b".to_string()];
    let result = vec_param(items)?;
    assert_eq!(result, 2);
    Ok(())
}

#[test]
fn test_multiple_optional() -> Result<(), Box<dyn std::error::Error>> {
    multiple_optional("required", Some(10), None)?;
    Ok(())
}

#[test]
fn test_complex_type() -> Result<(), Box<dyn std::error::Error>> {
    let val = complex_type()?;
    assert_eq!(val.field1, "test");
    assert_eq!(val.field2, 42);
    Ok(())
}

#[test]
fn test_floating_point() -> Result<(), Box<dyn std::error::Error>> {
    let result = calculate(5.5)?;
    assert_eq!(result, 11.0);
    Ok(())
}

#[test]
fn test_all_primitives() -> Result<(), Box<dyn std::error::Error>> {
    all_primitives("test", 42, -10, std::f64::consts::PI, true)?;
    Ok(())
}
