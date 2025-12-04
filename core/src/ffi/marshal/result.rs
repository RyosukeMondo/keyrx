//! C-compatible result types for FFI boundary crossing.
//!
//! This module provides FFI-safe result types that can be safely passed across
//! the Rust-Dart boundary. Unlike Rust's native `Result<T, E>`, these types use
//! `#[repr(C)]` layout and raw pointers for error handling.
//!
//! # Architecture
//!
//! The design uses two key types:
//!
//! - [`FfiResult<T>`]: C-compatible result with success flag, value, and error pointer
//! - [`FfiErrorPtr`]: Opaque pointer wrapper for heap-allocated error data
//!
//! # Memory Management
//!
//! Error data is heap-allocated on the Rust side and must be freed by calling
//! [`FfiErrorPtr::free`]. The C/Dart side is responsible for calling the
//! appropriate free function when done with error data.
//!
//! # Example
//!
//! ```
//! use keyrx_core::ffi::marshal::result::FfiResult;
//! use keyrx_core::ffi::marshal::traits::CRepr;
//! use keyrx_core::ffi::error::FfiError;
//!
//! #[repr(C)]
//! #[derive(Copy, Clone)]
//! struct StatusC {
//!     code: u32,
//! }
//!
//! impl CRepr for StatusC {}
//!
//! // Create success result
//! let success: FfiResult<StatusC> = FfiResult::ok(StatusC { code: 0 });
//! assert!(success.is_ok());
//!
//! // Create error result
//! let error: FfiResult<StatusC> = FfiResult::err(FfiError::invalid_input("bad input"));
//! assert!(!error.is_ok());
//! ```

use crate::ffi::error::FfiError;
use crate::ffi::marshal::traits::CRepr;
use std::ffi::{c_char, CString};
use std::mem::MaybeUninit;
use std::ptr;

/// C-compatible result type for FFI operations.
///
/// This type can be safely passed across the FFI boundary. It uses a discriminated
/// union pattern with:
/// - `success`: Boolean flag indicating success or error
/// - `value`: Potentially uninitialized value (valid only if success=true)
/// - `error`: Error pointer (valid only if success=false)
///
/// # Memory Layout
///
/// The `#[repr(C)]` attribute ensures predictable memory layout for C interop.
/// The Dart side can safely read the `success` field and then access either
/// `value` or `error` based on the flag.
///
/// # Safety
///
/// - When `success=true`: `value` is initialized, `error` is null
/// - When `success=false`: `error` is valid, `value` is uninitialized
///
/// # Example
///
/// ```
/// # use keyrx_core::ffi::marshal::result::FfiResult;
/// # use keyrx_core::ffi::marshal::traits::CRepr;
/// # use keyrx_core::ffi::error::FfiError;
/// # #[repr(C)]
/// # #[derive(Copy, Clone)]
/// # struct DataC { value: u32 }
/// # impl CRepr for DataC {}
/// let result: FfiResult<DataC> = FfiResult::ok(DataC { value: 42 });
///
/// if result.is_ok() {
///     // Safe to access value
///     let data = result.into_result().unwrap();
///     assert_eq!(data.value, 42);
/// }
/// ```
#[repr(C)]
pub struct FfiResult<T: CRepr> {
    /// Success flag: true if value is valid, false if error is valid.
    success: u8, // Using u8 instead of bool for C compatibility
    /// The success value (uninitialized if success=false).
    value: MaybeUninit<T>,
    /// Error pointer (null if success=true).
    error: FfiErrorPtr,
}

impl<T: CRepr> FfiResult<T> {
    /// Create a success result.
    ///
    /// # Parameters
    ///
    /// * `value` - The success value to wrap
    ///
    /// # Returns
    ///
    /// An `FfiResult` with `success=true` and the value initialized.
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::result::FfiResult;
    /// # use keyrx_core::ffi::marshal::traits::CRepr;
    /// # #[repr(C)]
    /// # #[derive(Copy, Clone)]
    /// # struct StatusC { code: u32 }
    /// # impl CRepr for StatusC {}
    /// let result = FfiResult::ok(StatusC { code: 200 });
    /// assert!(result.is_ok());
    /// ```
    pub fn ok(value: T) -> Self {
        Self {
            success: 1,
            value: MaybeUninit::new(value),
            error: FfiErrorPtr::null(),
        }
    }

    /// Create an error result.
    ///
    /// # Parameters
    ///
    /// * `error` - The error to wrap
    ///
    /// # Returns
    ///
    /// An `FfiResult` with `success=false` and error pointer initialized.
    /// The error data is heap-allocated and must be freed by the caller.
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::result::FfiResult;
    /// # use keyrx_core::ffi::marshal::traits::CRepr;
    /// # use keyrx_core::ffi::error::FfiError;
    /// # #[repr(C)]
    /// # #[derive(Copy, Clone)]
    /// # struct StatusC { code: u32 }
    /// # impl CRepr for StatusC {}
    /// let result: FfiResult<StatusC> = FfiResult::err(FfiError::not_found("resource"));
    /// assert!(!result.is_ok());
    /// ```
    pub fn err(error: FfiError) -> Self {
        Self {
            success: 0,
            value: MaybeUninit::uninit(),
            error: FfiErrorPtr::from_error(error),
        }
    }

    /// Check if the result is a success.
    ///
    /// # Returns
    ///
    /// `true` if the result contains a value, `false` if it contains an error.
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::result::FfiResult;
    /// # use keyrx_core::ffi::marshal::traits::CRepr;
    /// # #[repr(C)]
    /// # #[derive(Copy, Clone)]
    /// # struct DataC { value: u32 }
    /// # impl CRepr for DataC {}
    /// let result = FfiResult::ok(DataC { value: 1 });
    /// assert!(result.is_ok());
    /// ```
    pub fn is_ok(&self) -> bool {
        self.success != 0
    }

    /// Check if the result is an error.
    ///
    /// # Returns
    ///
    /// `true` if the result contains an error, `false` if it contains a value.
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::result::FfiResult;
    /// # use keyrx_core::ffi::marshal::traits::CRepr;
    /// # use keyrx_core::ffi::error::FfiError;
    /// # #[repr(C)]
    /// # #[derive(Copy, Clone)]
    /// # struct DataC { value: u32 }
    /// # impl CRepr for DataC {}
    /// let result: FfiResult<DataC> = FfiResult::err(FfiError::internal("fail"));
    /// assert!(result.is_err());
    /// ```
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Convert into a Rust Result.
    ///
    /// Consumes the `FfiResult` and converts it to a standard Rust `Result<T, FfiError>`.
    /// This is useful for working with FFI results in Rust code.
    ///
    /// # Returns
    ///
    /// - `Ok(T)`: If success=true, returns the value
    /// - `Err(FfiError)`: If success=false, reconstructs and returns the error
    ///
    /// # Safety
    ///
    /// This method assumes:
    /// - If success=true, `value` is initialized
    /// - If success=false, `error` points to valid error data
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::marshal::result::FfiResult;
    /// # use keyrx_core::ffi::marshal::traits::CRepr;
    /// # #[repr(C)]
    /// # #[derive(Copy, Clone)]
    /// # struct DataC { value: u32 }
    /// # impl CRepr for DataC {}
    /// let ffi_result = FfiResult::ok(DataC { value: 42 });
    /// let result = ffi_result.into_result();
    /// assert_eq!(result.unwrap().value, 42);
    /// ```
    #[allow(unsafe_code)]
    pub fn into_result(self) -> Result<T, FfiError> {
        if self.success != 0 {
            // Safety: value is initialized when success=true
            Ok(unsafe { self.value.assume_init() })
        } else {
            // Safety: error is valid when success=false
            Err(unsafe { self.error.into_error() })
        }
    }
}

impl<T: CRepr> From<Result<T, FfiError>> for FfiResult<T> {
    fn from(result: Result<T, FfiError>) -> Self {
        match result {
            Ok(value) => Self::ok(value),
            Err(error) => Self::err(error),
        }
    }
}

/// C-compatible error pointer wrapper.
///
/// This type wraps a heap-allocated [`FfiErrorData`] struct. The error data
/// includes error code, message, hint, and context as C strings.
///
/// # Memory Management
///
/// Error data is allocated on the Rust heap using `Box::into_raw()`. The caller
/// (typically the Dart side) must call the appropriate free function to deallocate.
///
/// # Safety
///
/// - The pointer must be null or point to valid `FfiErrorData`
/// - After calling `into_error()` or `free()`, the pointer becomes invalid
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiErrorPtr {
    /// Raw pointer to heap-allocated error data (null if no error).
    ptr: *mut FfiErrorData,
}

impl FfiErrorPtr {
    /// Create a null error pointer.
    ///
    /// Used when there's no error (success case).
    pub fn null() -> Self {
        Self {
            ptr: ptr::null_mut(),
        }
    }

    /// Check if the error pointer is null.
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    /// Create an error pointer from an FfiError.
    ///
    /// Allocates error data on the heap and returns a pointer to it.
    ///
    /// # Parameters
    ///
    /// * `error` - The error to allocate
    ///
    /// # Returns
    ///
    /// An `FfiErrorPtr` pointing to heap-allocated error data.
    pub fn from_error(error: FfiError) -> Self {
        let data = FfiErrorData::from_error(error);
        Self {
            ptr: Box::into_raw(Box::new(data)),
        }
    }

    /// Convert the error pointer into an FfiError.
    ///
    /// # Safety
    ///
    /// - The pointer must be non-null and point to valid `FfiErrorData`
    /// - After this call, the pointer becomes invalid (ownership transferred)
    ///
    /// # Panics
    ///
    /// Panics if the pointer is null.
    #[allow(unsafe_code)]
    pub unsafe fn into_error(self) -> FfiError {
        assert!(!self.ptr.is_null(), "Cannot convert null error pointer");
        let boxed = Box::from_raw(self.ptr);
        boxed.into_error()
    }

    /// Free the error data without converting to FfiError.
    ///
    /// # Safety
    ///
    /// - The pointer must be null or point to valid `FfiErrorData`
    /// - After this call, the pointer becomes invalid
    #[allow(unsafe_code)]
    pub unsafe fn free(self) {
        if !self.ptr.is_null() {
            let _ = Box::from_raw(self.ptr);
        }
    }
}

/// C-compatible error data.
///
/// This struct contains all error information as C strings (null-terminated).
/// All string fields are heap-allocated and must be freed individually.
///
/// # Memory Layout
///
/// The `#[repr(C)]` attribute ensures predictable layout:
/// - `code`: Error code as u32
/// - `message`: Required error message
/// - `hint`: Optional hint for resolution (may be null)
/// - `context`: Optional error context (may be null)
#[repr(C)]
pub struct FfiErrorData {
    /// Numeric error code (from FfiError.code string hash or custom mapping).
    pub code: u32,
    /// Error message as null-terminated C string.
    pub message: *mut c_char,
    /// Optional hint as null-terminated C string (null if not present).
    pub hint: *mut c_char,
    /// Optional context as null-terminated C string (null if not present).
    pub context: *mut c_char,
}

impl FfiErrorData {
    /// Create error data from an FfiError.
    ///
    /// Converts all string fields to null-terminated C strings.
    /// Replaces null bytes in strings with a placeholder to avoid CString errors.
    #[allow(clippy::unwrap_used)] // Static fallback strings are guaranteed to be valid
    fn from_error(error: FfiError) -> Self {
        // Hash the error code string to get a numeric code
        // Simple hash for demo - in production, use a proper error code registry
        let code = hash_error_code(&error.code);

        // Convert message to C string (required)
        // Replace null bytes to avoid CString errors
        let message_str = error.message.replace('\0', "\\0");
        let message = CString::new(message_str)
            .unwrap_or_else(|_| CString::new("Invalid error message").unwrap())
            .into_raw();

        // Convert optional hint to C string
        let hint = if let Some(details) = error.details {
            // Use details as hint if available
            let hint_str = details.to_string().replace('\0', "\\0");
            CString::new(hint_str)
                .unwrap_or_else(|_| CString::new("Invalid hint").unwrap())
                .into_raw()
        } else {
            ptr::null_mut()
        };

        // Context is not currently in FfiError, set to null
        let context = ptr::null_mut();

        Self {
            code,
            message,
            hint,
            context,
        }
    }

    /// Convert error data back into an FfiError.
    ///
    /// # Safety
    ///
    /// - All non-null pointers must point to valid C strings
    /// - This consumes ownership of the C strings (they are freed)
    #[allow(unsafe_code)]
    fn into_error(self) -> FfiError {
        // Safety: We allocated these strings, so they're valid
        let message = unsafe {
            assert!(!self.message.is_null(), "Error message cannot be null");
            CString::from_raw(self.message)
                .into_string()
                .unwrap_or_else(|_| "Invalid UTF-8 in error message".to_string())
        };

        // Convert code back to string (simplified - in production use registry)
        let code = format!("ERROR_{}", self.code);

        // Handle optional hint
        #[allow(unsafe_code)]
        let details = if !self.hint.is_null() {
            let hint = unsafe { CString::from_raw(self.hint).into_string().ok() };
            hint.and_then(|s| serde_json::from_str(&s).ok())
        } else {
            None
        };

        // Free context if present
        #[allow(unsafe_code)]
        if !self.context.is_null() {
            unsafe {
                let _ = CString::from_raw(self.context);
            }
        }

        FfiError {
            code,
            message,
            details,
        }
    }
}

/// Simple hash function for error codes.
///
/// Converts error code strings to u32 values. In production, this should use
/// a proper error code registry for stable, documented codes.
fn hash_error_code(code: &str) -> u32 {
    // Simple DJB2 hash
    let mut hash: u32 = 5381;
    for byte in code.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test type for FfiResult
    #[repr(C)]
    #[derive(Copy, Clone, Debug, PartialEq)]
    struct TestC {
        value: u32,
    }

    impl CRepr for TestC {}

    #[test]
    fn test_ffi_result_ok() {
        let result = FfiResult::ok(TestC { value: 42 });
        assert!(result.is_ok());
        assert!(!result.is_err());

        let rust_result = result.into_result();
        assert_eq!(rust_result.unwrap().value, 42);
    }

    #[test]
    fn test_ffi_result_err() {
        let error = FfiError::invalid_input("test error");
        let result: FfiResult<TestC> = FfiResult::err(error);
        assert!(!result.is_ok());
        assert!(result.is_err());

        let rust_result = result.into_result();
        assert!(rust_result.is_err());
        let err = rust_result.unwrap_err();
        assert_eq!(err.message, "test error");
    }

    #[test]
    fn test_ffi_result_from_result() {
        let ok_result: Result<TestC, FfiError> = Ok(TestC { value: 100 });
        let ffi_result: FfiResult<TestC> = ok_result.into();
        assert!(ffi_result.is_ok());

        let err_result: Result<TestC, FfiError> = Err(FfiError::not_found("resource"));
        let ffi_result: FfiResult<TestC> = err_result.into();
        assert!(ffi_result.is_err());
    }

    #[test]
    fn test_ffi_error_ptr_null() {
        let ptr = FfiErrorPtr::null();
        assert!(ptr.is_null());
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_ffi_error_ptr_roundtrip() {
        let error = FfiError::new("TEST_CODE", "test message");
        let ptr = FfiErrorPtr::from_error(error);
        assert!(!ptr.is_null());

        let recovered = unsafe { ptr.into_error() };
        assert_eq!(recovered.message, "test message");
    }

    #[test]
    fn test_ffi_error_data_from_error() {
        let error = FfiError::new("INVALID_INPUT", "bad value");
        let data = FfiErrorData::from_error(error);

        assert_ne!(data.code, 0);
        assert!(!data.message.is_null());
        assert!(data.hint.is_null());
        assert!(data.context.is_null());

        // Cleanup
        let error = data.into_error();
        assert_eq!(error.message, "bad value");
    }

    #[test]
    fn test_error_code_hash() {
        let hash1 = hash_error_code("INVALID_INPUT");
        let hash2 = hash_error_code("INVALID_INPUT");
        let hash3 = hash_error_code("NOT_FOUND");

        assert_eq!(hash1, hash2); // Same string = same hash
        assert_ne!(hash1, hash3); // Different strings = different hash
    }

    #[test]
    fn test_ffi_result_memory_layout() {
        // Verify that FfiResult has predictable C layout
        use std::mem::{align_of, size_of};

        assert_eq!(align_of::<FfiResult<TestC>>(), align_of::<usize>());
        // Size should be reasonable (not too large due to alignment)
        assert!(size_of::<FfiResult<TestC>>() > 0);
    }

    #[test]
    fn test_error_with_details() {
        let error = FfiError::with_details(
            "PARSE_ERROR",
            "failed to parse",
            serde_json::json!({"line": 42}),
        );
        let data = FfiErrorData::from_error(error);

        assert!(!data.message.is_null());
        assert!(!data.hint.is_null()); // Details converted to hint

        let recovered = data.into_error();
        assert_eq!(recovered.message, "failed to parse");
        assert!(recovered.details.is_some());
    }
}
