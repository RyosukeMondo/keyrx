//! Integration tests for the #[ffi_export] macro

use keyrx_ffi_macros::ffi_export;

// Mock FfiError and serialize_ffi_result for testing
// In the actual code, these come from keyrx_core::ffi::error
mod ffi {
    pub mod error {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct FfiError {
            pub code: String,
            pub message: String,
            pub details: Option<serde_json::Value>,
        }

        impl FfiError {
            pub fn null_pointer(param: &str) -> Self {
                Self {
                    code: "NULL_POINTER".to_string(),
                    message: format!("null pointer for parameter: {}", param),
                    details: None,
                }
            }

            pub fn invalid_utf8(param: &str) -> Self {
                Self {
                    code: "INVALID_UTF8".to_string(),
                    message: format!("invalid UTF-8 in parameter: {}", param),
                    details: None,
                }
            }

            pub fn internal(msg: String) -> Self {
                Self {
                    code: "INTERNAL_ERROR".to_string(),
                    message: msg,
                    details: None,
                }
            }
        }

        pub type FfiResult<T> = Result<T, FfiError>;

        pub fn serialize_ffi_result<T: Serialize>(
            result: &FfiResult<T>,
        ) -> serde_json::Result<String> {
            match result {
                Ok(value) => {
                    let json = serde_json::to_string(value)?;
                    Ok(format!("ok:{}", json))
                }
                Err(error) => {
                    let json = serde_json::to_string(error)?;
                    Ok(format!("error:{}", json))
                }
            }
        }
    }
}

// Test with a simple function that returns Result
#[derive(serde::Serialize)]
struct TestResult {
    value: i32,
}

// Standalone functions for testing
#[ffi_export]
fn simple_function(x: u32, y: u32) -> Result<TestResult, ffi::error::FfiError> {
    Ok(TestResult {
        value: (x + y) as i32,
    })
}

#[ffi_export]
fn with_string_param(name: &str) -> Result<TestResult, ffi::error::FfiError> {
    Ok(TestResult {
        value: name.len() as i32,
    })
}

#[ffi_export]
fn panicking_function(_x: u32) -> Result<TestResult, ffi::error::FfiError> {
    panic!("intentional panic for testing");
}

#[test]
fn test_macro_compiles() {
    // If we get here, the macro expanded successfully
    assert!(true);
}

#[test]
fn test_simple_function_ffi() {
    use std::ffi::{CStr, CString};

    // Call the generated FFI function
    let result_ptr = unsafe { keyrx_simple_function(10, 20) };
    assert!(!result_ptr.is_null());

    // Convert the result to a Rust string
    let result_str = unsafe {
        let c_str = CStr::from_ptr(result_ptr);
        c_str.to_str().unwrap().to_string()
    };

    // Free the string
    unsafe {
        let _ = CString::from_raw(result_ptr);
    }

    // Verify the result format
    assert!(result_str.starts_with("ok:"));
    let json_str = &result_str[3..];
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert_eq!(parsed["value"], 30);
}

#[test]
fn test_string_param_ffi() {
    use std::ffi::{CStr, CString};

    let name = CString::new("test").unwrap();
    let result_ptr = unsafe { keyrx_with_string_param(name.as_ptr()) };
    assert!(!result_ptr.is_null());

    let result_str = unsafe {
        let c_str = CStr::from_ptr(result_ptr);
        c_str.to_str().unwrap().to_string()
    };

    unsafe {
        let _ = CString::from_raw(result_ptr);
    }

    assert!(result_str.starts_with("ok:"));
    let json_str = &result_str[3..];
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert_eq!(parsed["value"], 4); // "test".len() == 4
}

#[test]
fn test_null_pointer_handling() {
    use std::ffi::{CStr, CString};
    use std::ptr;

    let result_ptr = unsafe { keyrx_with_string_param(ptr::null()) };
    assert!(!result_ptr.is_null());

    let result_str = unsafe {
        let c_str = CStr::from_ptr(result_ptr);
        c_str.to_str().unwrap().to_string()
    };

    unsafe {
        let _ = CString::from_raw(result_ptr);
    }

    assert!(result_str.starts_with("error:"));
    let json_str = &result_str[6..];
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert_eq!(parsed["code"], "NULL_POINTER");
}

#[test]
fn test_panic_catching() {
    use std::ffi::{CStr, CString};

    let result_ptr = unsafe { keyrx_panicking_function(42) };
    assert!(!result_ptr.is_null());

    let result_str = unsafe {
        let c_str = CStr::from_ptr(result_ptr);
        c_str.to_str().unwrap().to_string()
    };

    unsafe {
        let _ = CString::from_raw(result_ptr);
    }

    assert!(result_str.starts_with("error:"));
    let json_str = &result_str[6..];
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert_eq!(parsed["code"], "INTERNAL_ERROR");
    assert!(parsed["message"].as_str().unwrap().contains("panic"));
}
