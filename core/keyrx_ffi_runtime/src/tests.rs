//! Unit tests for FFI runtime helpers.

// Allow unwrap and panic in tests - panics are the expected behavior for test failures
// and the panic tests specifically test panic handling infrastructure
#![allow(clippy::unwrap_used, clippy::panic)]

use std::ffi::{c_char, CString};

// ============================================================================
// parse_c_string tests
// ============================================================================

mod parse_c_string_tests {
    use super::*;
    use crate::string::parse_c_string;

    #[test]
    fn parses_valid_c_string() {
        let s = CString::new("hello").unwrap();
        let result = unsafe { parse_c_string(s.as_ptr(), "test_param") };
        assert_eq!(result, Ok("hello".to_string()));
    }

    #[test]
    fn parses_empty_string() {
        let s = CString::new("").unwrap();
        let result = unsafe { parse_c_string(s.as_ptr(), "test_param") };
        assert_eq!(result, Ok("".to_string()));
    }

    #[test]
    fn parses_unicode_string() {
        let s = CString::new("こんにちは").unwrap();
        let result = unsafe { parse_c_string(s.as_ptr(), "test_param") };
        assert_eq!(result, Ok("こんにちは".to_string()));
    }

    #[test]
    fn returns_error_for_null_pointer() {
        let result = unsafe { parse_c_string(std::ptr::null(), "my_param") };
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("my_param"));
    }

    #[test]
    fn error_message_includes_parameter_name() {
        let result = unsafe { parse_c_string(std::ptr::null(), "profile_id") };
        let err = result.unwrap_err();
        assert!(
            err.contains("profile_id"),
            "Error should contain param name"
        );
    }

    #[test]
    fn handles_invalid_utf8() {
        // Create a C string with invalid UTF-8
        let invalid_utf8: Vec<u8> = vec![0xFF, 0xFE, 0x00]; // Invalid UTF-8 followed by null
        let result = unsafe { parse_c_string(invalid_utf8.as_ptr() as *const c_char, "data") };
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("UTF-8"));
    }
}

// ============================================================================
// serialize_to_c_string tests
// ============================================================================

mod serialize_to_c_string_tests {
    use super::*;
    use crate::json::serialize_to_c_string;
    use serde::Serialize;
    use std::ffi::CStr;

    #[derive(Serialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    #[test]
    fn serializes_struct_to_json() {
        let data = TestStruct {
            name: "test".to_string(),
            value: 42,
        };
        let result = serialize_to_c_string(&data);
        assert!(result.is_ok());

        let ptr = result.unwrap();
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap() };
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"value\":42"));

        // Free the allocated string
        unsafe { drop(CString::from_raw(ptr as *mut c_char)) };
    }

    #[test]
    fn serializes_primitive_types() {
        let result = serialize_to_c_string(&42i32);
        assert!(result.is_ok());

        let ptr = result.unwrap();
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap() };
        assert_eq!(json, "42");

        unsafe { drop(CString::from_raw(ptr as *mut c_char)) };
    }

    #[test]
    fn serializes_string() {
        let result = serialize_to_c_string(&"hello world");
        assert!(result.is_ok());

        let ptr = result.unwrap();
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap() };
        assert_eq!(json, "\"hello world\"");

        unsafe { drop(CString::from_raw(ptr as *mut c_char)) };
    }

    #[test]
    fn serializes_vec() {
        let data = vec![1, 2, 3];
        let result = serialize_to_c_string(&data);
        assert!(result.is_ok());

        let ptr = result.unwrap();
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap() };
        assert_eq!(json, "[1,2,3]");

        unsafe { drop(CString::from_raw(ptr as *mut c_char)) };
    }

    #[test]
    fn serializes_option_none() {
        let data: Option<i32> = None;
        let result = serialize_to_c_string(&data);
        assert!(result.is_ok());

        let ptr = result.unwrap();
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap() };
        assert_eq!(json, "null");

        unsafe { drop(CString::from_raw(ptr as *mut c_char)) };
    }

    #[test]
    fn serializes_option_some() {
        let data: Option<i32> = Some(42);
        let result = serialize_to_c_string(&data);
        assert!(result.is_ok());

        let ptr = result.unwrap();
        let json = unsafe { CStr::from_ptr(ptr).to_str().unwrap() };
        assert_eq!(json, "42");

        unsafe { drop(CString::from_raw(ptr as *mut c_char)) };
    }
}

// ============================================================================
// handle_panic tests
// ============================================================================

mod handle_panic_tests {
    use crate::panic::handle_panic;

    #[test]
    fn returns_ok_on_success() {
        let result = handle_panic(|| 42);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn returns_ok_with_complex_return() {
        let result = handle_panic(|| vec![1, 2, 3]);
        assert_eq!(result, Ok(vec![1, 2, 3]));
    }

    #[test]
    fn catches_panic_with_str_message() {
        let result = handle_panic(|| {
            panic!("test panic message");
        });
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("test panic message"), "Error: {}", err);
    }

    #[test]
    fn catches_panic_with_string_message() {
        let result = handle_panic(|| {
            panic!("{}", "formatted panic".to_string());
        });
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("formatted panic"), "Error: {}", err);
    }

    #[test]
    fn catches_panic_with_format_args() {
        let result = handle_panic(|| {
            panic!("value is {}", 42);
        });
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("value is 42"), "Error: {}", err);
    }

    #[test]
    fn error_contains_panic_prefix() {
        let result: Result<(), String> = handle_panic(|| {
            panic!("oops");
        });
        let err = result.unwrap_err();
        assert!(
            err.starts_with("Panic:"),
            "Error should start with 'Panic:': {}",
            err
        );
    }
}

// ============================================================================
// ffi_wrapper tests
// ============================================================================

mod ffi_wrapper_tests {
    use super::*;
    use crate::wrapper::ffi_wrapper;
    use serde::Serialize;
    use std::ffi::CStr;

    #[derive(Serialize)]
    struct Response {
        status: String,
        code: i32,
    }

    fn free_c_string(ptr: *const c_char) {
        if !ptr.is_null() {
            unsafe { drop(CString::from_raw(ptr as *mut c_char)) };
        }
    }

    #[test]
    fn returns_json_on_success() {
        let mut error: *mut c_char = std::ptr::null_mut();
        let result = unsafe {
            ffi_wrapper(&mut error, || {
                Ok(Response {
                    status: "ok".to_string(),
                    code: 200,
                })
            })
        };

        assert!(!result.is_null());
        assert!(error.is_null());

        let json = unsafe { CStr::from_ptr(result).to_str().unwrap() };
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"code\":200"));

        free_c_string(result);
    }

    #[test]
    fn returns_null_and_sets_error_on_err() {
        let mut error: *mut c_char = std::ptr::null_mut();
        let result = unsafe {
            ffi_wrapper(&mut error, || {
                Err::<Response, _>("something went wrong".to_string())
            })
        };

        assert!(result.is_null());
        assert!(!error.is_null());

        let err_msg = unsafe { CStr::from_ptr(error).to_str().unwrap() };
        assert_eq!(err_msg, "something went wrong");

        free_c_string(error as *const c_char);
    }

    #[test]
    fn returns_null_and_sets_error_on_panic() {
        let mut error: *mut c_char = std::ptr::null_mut();
        let result = unsafe {
            ffi_wrapper(&mut error, || {
                panic!("unexpected panic");
                #[allow(unreachable_code)]
                Ok::<Response, String>(Response {
                    status: "never".to_string(),
                    code: 0,
                })
            })
        };

        assert!(result.is_null());
        assert!(!error.is_null());

        let err_msg = unsafe { CStr::from_ptr(error).to_str().unwrap() };
        assert!(err_msg.contains("unexpected panic"), "Error: {}", err_msg);

        free_c_string(error as *const c_char);
    }

    #[test]
    fn handles_null_error_pointer_on_success() {
        let result = unsafe {
            ffi_wrapper(std::ptr::null_mut(), || {
                Ok(Response {
                    status: "ok".to_string(),
                    code: 200,
                })
            })
        };

        assert!(!result.is_null());
        free_c_string(result);
    }

    #[test]
    fn handles_null_error_pointer_on_error() {
        let result = unsafe {
            ffi_wrapper(std::ptr::null_mut(), || {
                Err::<Response, _>("error message".to_string())
            })
        };

        assert!(result.is_null());
        // No error pointer to free, and no crash - success
    }

    #[test]
    fn handles_null_error_pointer_on_panic() {
        let result = unsafe {
            ffi_wrapper(std::ptr::null_mut(), || {
                panic!("panic with null error ptr");
                #[allow(unreachable_code)]
                Ok::<Response, String>(Response {
                    status: "never".to_string(),
                    code: 0,
                })
            })
        };

        assert!(result.is_null());
        // No error pointer to free, and no crash - success
    }

    #[test]
    fn serializes_primitive_return() {
        let mut error: *mut c_char = std::ptr::null_mut();
        let result = unsafe { ffi_wrapper(&mut error, || Ok::<_, String>(42i32)) };

        assert!(!result.is_null());
        assert!(error.is_null());

        let json = unsafe { CStr::from_ptr(result).to_str().unwrap() };
        assert_eq!(json, "42");

        free_c_string(result);
    }

    #[test]
    fn serializes_vec_return() {
        let mut error: *mut c_char = std::ptr::null_mut();
        let result = unsafe { ffi_wrapper(&mut error, || Ok::<_, String>(vec!["a", "b", "c"])) };

        assert!(!result.is_null());
        assert!(error.is_null());

        let json = unsafe { CStr::from_ptr(result).to_str().unwrap() };
        assert_eq!(json, "[\"a\",\"b\",\"c\"]");

        free_c_string(result);
    }
}
