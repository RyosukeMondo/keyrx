//! FfiMarshaler implementations for string types.
//!
//! This module provides marshaling for Rust string types (`String`, `&str`) to
//! C-compatible null-terminated strings. It handles UTF-8 validation and ensures
//! safe string transfer across the FFI boundary.
//!
//! # String Representation
//!
//! Strings are represented in C as `*const c_char` (null-terminated byte arrays).
//! This module provides:
//!
//! - **Owned Strings**: `String` ↔ `*mut c_char`
//! - **Borrowed Strings**: `&str` ↔ `*const c_char`
//!
//! # Memory Management
//!
//! - **To C**: Allocates null-terminated C string on the heap
//! - **From C**: Validates UTF-8 and creates owned Rust String
//! - **Ownership**: C side must call the appropriate free function
//!
//! # Safety Guarantees
//!
//! - All strings are validated as UTF-8
//! - Null terminators are always added
//! - Invalid UTF-8 results in [`FfiError::invalid_utf8`]
//! - Null pointers result in [`FfiError::null_pointer`]
//!
//! # Example
//!
//! ```
//! use keyrx_core::ffi::marshal::traits::FfiMarshaler;
//! use keyrx_core::ffi::marshal::impls::string::free_ffi_string;
//!
//! let rust_string = String::from("Hello, FFI!");
//! let ffi_string = rust_string.to_c().unwrap();
//!
//! // FfiString is null-terminated and can be passed to C functions
//! assert!(!ffi_string.is_null());
//!
//! // Reconstruct Rust string
//! let restored = String::from_c(ffi_string).unwrap();
//! assert_eq!(restored, "Hello, FFI!");
//!
//! // Don't forget to free the FfiString!
//! unsafe {
//!     free_ffi_string(ffi_string);
//! }
//! ```

use crate::ffi::error::{FfiError, FfiResult};
use crate::ffi::marshal::traits::{CRepr, FfiMarshaler};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// C-compatible string representation.
///
/// This is a wrapper around `*mut c_char` that implements `Send` because:
/// - The string data is owned by the C side after marshaling
/// - The pointer itself can be safely sent across threads
/// - The actual string data is immutable once created
///
/// # Safety
///
/// The wrapper is `Send` because:
/// 1. It represents ownership transfer to C
/// 2. C strings are typically immutable after creation
/// 3. Thread safety is handled by the C side or explicit synchronization
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct FfiString(*mut c_char);

#[allow(unsafe_code)]
unsafe impl Send for FfiString {}

impl FfiString {
    /// Create a new FfiString from a raw pointer.
    ///
    /// # Safety
    ///
    /// The pointer must be a valid null-terminated C string.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_char) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    pub fn as_ptr(&self) -> *mut c_char {
        self.0
    }

    /// Check if the pointer is null.
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

impl CRepr for FfiString {}

/// Implementation of FfiMarshaler for String.
///
/// Converts Rust String to null-terminated C string and back.
/// Allocates memory on the heap for the C string, which must be freed by the caller.
impl FfiMarshaler for String {
    type CRepr = FfiString;

    #[allow(unsafe_code)]
    fn to_c(&self) -> FfiResult<Self::CRepr> {
        // Convert to CString, which adds null terminator
        let c_string = CString::new(self.as_str())
            .map_err(|_| FfiError::invalid_input("string contains null byte"))?;

        // Transfer ownership to C side
        Ok(unsafe { FfiString::from_raw(c_string.into_raw()) })
    }

    #[allow(unsafe_code)]
    fn from_c(c: Self::CRepr) -> FfiResult<Self> {
        if c.is_null() {
            return Err(FfiError::null_pointer("c_string"));
        }

        // SAFETY: We check for null above. The caller guarantees the pointer is valid.
        unsafe {
            // Borrow the C string without taking ownership
            let c_str = CStr::from_ptr(c.as_ptr());

            // Convert to Rust string, validating UTF-8
            c_str
                .to_str()
                .map(|s| s.to_owned())
                .map_err(|_| FfiError::invalid_utf8("c_string"))
        }
    }

    fn estimated_size(&self) -> usize {
        // Size includes the bytes plus null terminator
        self.len() + 1
    }
}

/// Implementation of FfiMarshaler for &str.
///
/// Converts Rust string slice to null-terminated C string.
/// Note: The returned pointer must be freed by the caller.
impl FfiMarshaler for &str {
    type CRepr = FfiString;

    #[allow(unsafe_code)]
    fn to_c(&self) -> FfiResult<Self::CRepr> {
        // Convert to CString, which adds null terminator
        let c_string = CString::new(*self)
            .map_err(|_| FfiError::invalid_input("string contains null byte"))?;

        // Transfer ownership to C side
        Ok(unsafe { FfiString::from_raw(c_string.into_raw()) })
    }

    fn from_c(c: Self::CRepr) -> FfiResult<Self> {
        // For &str, we can't really return a borrowed reference from a C string
        // without tying it to a lifetime. Instead, this would typically be used
        // in conjunction with a wrapper type. For now, we'll error on from_c.
        // This is primarily for to_c direction.
        if c.is_null() {
            return Err(FfiError::null_pointer("c_string"));
        }

        Err(FfiError::internal(
            "Cannot create &str directly from C string - use String::from_c instead",
        ))
    }

    fn estimated_size(&self) -> usize {
        // Size includes the bytes plus null terminator
        self.len() + 1
    }
}

/// Helper function to safely free a C string allocated by to_c().
///
/// # Safety
///
/// The pointer must have been allocated by `String::to_c()` or `&str::to_c()`.
/// The pointer must not be used after calling this function.
///
/// # Example
///
/// ```
/// use keyrx_core::ffi::marshal::traits::FfiMarshaler;
/// use keyrx_core::ffi::marshal::impls::string::free_ffi_string;
///
/// let rust_string = String::from("test");
/// let c_string = rust_string.to_c().unwrap();
///
/// // Use the C string...
///
/// // Free when done
/// unsafe {
///     free_ffi_string(c_string);
/// }
/// ```
#[no_mangle]
#[allow(unsafe_code, clippy::missing_safety_doc)]
pub unsafe extern "C" fn free_ffi_string(ffi_str: FfiString) {
    if !ffi_str.is_null() {
        // Reconstruct the CString to drop it
        drop(CString::from_raw(ffi_str.as_ptr()));
    }
}

/// Legacy function name for backwards compatibility.
/// Use `free_ffi_string` instead.
///
/// # Safety
///
/// The pointer must have been allocated by `String::to_c()` or `&str::to_c()`.
/// The pointer must not be used after calling this function.
#[no_mangle]
#[allow(unsafe_code, clippy::missing_safety_doc)]
pub unsafe extern "C" fn free_c_string(ptr: *mut c_char) {
    free_ffi_string(FfiString::from_raw(ptr));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_c() {
        let rust_string = String::from("Hello, World!");
        let c_string = rust_string.to_c().unwrap();

        // Verify it's not null
        assert!(!c_string.is_null());

        // Verify contents
        unsafe {
            let c_str = CStr::from_ptr(c_string.as_ptr());
            assert_eq!(c_str.to_str().unwrap(), "Hello, World!");

            // Clean up
            free_ffi_string(c_string);
        }
    }

    #[test]
    fn test_string_from_c() {
        // Create a C string
        let original = "Test String";
        let c_string = CString::new(original).unwrap();
        let ffi_str = unsafe { FfiString::from_raw(c_string.into_raw()) };

        // Convert back to Rust String
        let rust_string = String::from_c(ffi_str).unwrap();
        assert_eq!(rust_string, original);

        // Clean up
        unsafe {
            free_ffi_string(ffi_str);
        }
    }

    #[test]
    fn test_string_roundtrip() {
        let original = String::from("Roundtrip Test 🦀");
        let c_string = original.to_c().unwrap();

        let restored = String::from_c(c_string).unwrap();
        assert_eq!(restored, original);

        // Clean up
        unsafe {
            free_ffi_string(c_string);
        }
    }

    #[test]
    fn test_empty_string() {
        let empty = String::from("");
        let c_string = empty.to_c().unwrap();

        assert!(!c_string.is_null());

        let restored = String::from_c(c_string).unwrap();
        assert_eq!(restored, "");

        unsafe {
            free_ffi_string(c_string);
        }
    }

    #[test]
    fn test_string_with_unicode() {
        let unicode = String::from("Hello 世界 🌍");
        let c_string = unicode.to_c().unwrap();

        let restored = String::from_c(c_string).unwrap();
        assert_eq!(restored, "Hello 世界 🌍");

        unsafe {
            free_ffi_string(c_string);
        }
    }

    #[test]
    fn test_string_with_null_byte_fails() {
        let with_null = String::from("Hello\0World");
        let result = with_null.to_c();

        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code, "INVALID_INPUT");
        }
    }

    #[test]
    fn test_from_null_pointer() {
        let null_ffi_str = unsafe { FfiString::from_raw(std::ptr::null_mut()) };
        let result = String::from_c(null_ffi_str);

        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code, "NULL_POINTER");
        }
    }

    #[test]
    fn test_estimated_size() {
        let short = String::from("Hi");
        assert_eq!(short.estimated_size(), 3); // "Hi" + null = 3

        let longer = String::from("Hello, World!");
        assert_eq!(longer.estimated_size(), 14); // 13 chars + null = 14

        let empty = String::from("");
        assert_eq!(empty.estimated_size(), 1); // just null terminator
    }

    #[test]
    fn test_str_to_c() {
        let str_slice: &str = "Borrowed string";
        let c_string = str_slice.to_c().unwrap();

        assert!(!c_string.is_null());

        unsafe {
            let c_str = CStr::from_ptr(c_string.as_ptr());
            assert_eq!(c_str.to_str().unwrap(), "Borrowed string");

            free_ffi_string(c_string);
        }
    }

    #[test]
    fn test_str_from_c_errors() {
        // from_c is not supported for &str
        let c_string = CString::new("test").unwrap();
        let ffi_str = unsafe { FfiString::from_raw(c_string.into_raw()) };

        let result = <&str>::from_c(ffi_str);
        assert!(result.is_err());

        unsafe {
            free_ffi_string(ffi_str);
        }
    }

    #[test]
    fn test_str_estimated_size() {
        let s: &str = "test";
        assert_eq!(s.estimated_size(), 5); // "test" + null = 5
    }

    #[test]
    fn test_no_streaming_for_strings() {
        // Normal strings should not use streaming
        let small = String::from("Small string");
        assert!(!small.use_streaming());

        let medium = String::from("A".repeat(10_000));
        assert!(!medium.use_streaming()); // Still < 1MB

        // Very large string should use streaming
        let large = String::from("A".repeat(2_000_000));
        assert!(large.use_streaming()); // > 1MB threshold
    }

    #[test]
    fn test_free_null_pointer() {
        // Should not crash when freeing null pointer
        unsafe {
            free_ffi_string(FfiString::from_raw(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_multiline_string() {
        let multiline = String::from("Line 1\nLine 2\nLine 3");
        let c_string = multiline.to_c().unwrap();

        let restored = String::from_c(c_string).unwrap();
        assert_eq!(restored, "Line 1\nLine 2\nLine 3");

        unsafe {
            free_ffi_string(c_string);
        }
    }
}
