//! FfiMarshaler implementations for array and Vec types.
//!
//! This module provides marshaling for Rust `Vec<T>` to C-compatible length-prefixed
//! arrays. It handles batch encoding for efficiency and ensures safe transfer of
//! collection data across the FFI boundary.
//!
//! # Array Representation
//!
//! Arrays are represented in C as a pointer to data plus a length field:
//!
//! ```c
//! typedef struct {
//!     void* data;      // Pointer to array elements
//!     size_t len;      // Number of elements
//!     size_t capacity; // Allocated capacity (for optimization hints)
//! } FfiArray;
//! ```
//!
//! # Memory Management
//!
//! - **To C**: Allocates C array on the heap and converts each element
//! - **From C**: Validates length and converts each element back to Rust
//! - **Ownership**: C side must call the appropriate free function
//!
//! # Type Requirements
//!
//! The element type `T` must implement `FfiMarshaler`. This ensures:
//! - Each element can be safely converted to/from C representation
//! - Type safety is maintained across the boundary
//! - Proper error handling for element conversions
//!
//! # Batch Encoding
//!
//! For efficiency, elements are converted in a single pass:
//! - Allocates the full array upfront
//! - Converts elements sequentially
//! - Fails fast on first error
//!
//! # Example
//!
//! ```
//! use keyrx_core::ffi::marshal::traits::FfiMarshaler;
//! use keyrx_core::ffi::marshal::impls::array::free_ffi_array;
//!
//! let numbers = vec![1u32, 2, 3, 4, 5];
//! let ffi_array = numbers.to_c().unwrap();
//!
//! // FfiArray contains pointer and length
//! assert!(!ffi_array.is_null());
//!
//! // Reconstruct Rust Vec
//! let restored = Vec::<u32>::from_c(ffi_array).unwrap();
//! assert_eq!(restored, vec![1, 2, 3, 4, 5]);
//!
//! // Don't forget to free the FfiArray!
//! unsafe {
//!     free_ffi_array::<u32>(ffi_array);
//! }
//! ```

use crate::ffi::error::FfiResult;
use crate::ffi::marshal::traits::{CRepr, FfiMarshaler};
use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::slice;

/// C-compatible array representation.
///
/// This struct represents a length-prefixed array that can cross the FFI boundary.
/// It contains:
/// - `data`: Pointer to the array elements (C representation)
/// - `len`: Number of elements in the array
/// - `capacity`: Allocated capacity (optimization hint, may equal len)
///
/// # Safety
///
/// The wrapper is `Send` because:
/// 1. It represents ownership transfer to C
/// 2. The data pointer is owned by C side after marshaling
/// 3. Thread safety is handled by the C side or explicit synchronization
///
/// # Memory Layout
///
/// The struct is `#[repr(C)]` to ensure stable layout across FFI boundary.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiArray<T> {
    /// Pointer to array data (C representation of elements)
    data: *mut T,
    /// Number of elements
    len: usize,
    /// Allocated capacity (hint for optimization)
    capacity: usize,
    /// PhantomData for type safety
    _phantom: PhantomData<T>,
}

#[allow(unsafe_code)]
unsafe impl<T: Send> Send for FfiArray<T> {}

impl<T> FfiArray<T> {
    /// Create a new FfiArray from raw parts.
    ///
    /// # Safety
    ///
    /// - `data` must be a valid pointer to an array of `T` with at least `len` elements
    /// - The caller must ensure the data remains valid for the lifetime of this FfiArray
    #[allow(unsafe_code)]
    pub unsafe fn from_raw_parts(data: *mut T, len: usize, capacity: usize) -> Self {
        Self {
            data,
            len,
            capacity,
            _phantom: PhantomData,
        }
    }

    /// Create an FfiArray representing a null/empty array.
    pub fn null() -> Self {
        Self {
            data: ptr::null_mut(),
            len: 0,
            capacity: 0,
            _phantom: PhantomData,
        }
    }

    /// Get the raw data pointer.
    pub fn as_ptr(&self) -> *mut T {
        self.data
    }

    /// Get the length of the array.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Get the capacity of the array.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Check if the array is null/empty.
    pub fn is_null(&self) -> bool {
        self.data.is_null()
    }

    /// Check if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Convert to a slice.
    ///
    /// # Safety
    ///
    /// - The data pointer must be valid and point to at least `len` elements
    /// - The elements must be properly initialized
    #[allow(unsafe_code)]
    pub unsafe fn as_slice(&self) -> &[T] {
        if self.is_null() || self.is_empty() {
            &[]
        } else {
            slice::from_raw_parts(self.data, self.len)
        }
    }
}

// Implement CRepr for FfiArray of any C-representable type
impl<T: CRepr> CRepr for FfiArray<T> {}

/// Implementation of FfiMarshaler for `Vec<T>`.
///
/// Converts Rust Vec to C-compatible array and back.
/// Each element is marshaled using its FfiMarshaler implementation.
impl<T> FfiMarshaler for Vec<T>
where
    T: FfiMarshaler,
    T::CRepr: Sized,
{
    type CRepr = FfiArray<T::CRepr>;

    #[allow(unsafe_code)]
    fn to_c(&self) -> FfiResult<Self::CRepr> {
        if self.is_empty() {
            return Ok(FfiArray::null());
        }

        // Allocate array for C representations
        let mut c_elements = Vec::with_capacity(self.len());

        // Convert each element
        for element in self.iter() {
            let c_element = element.to_c()?;
            c_elements.push(c_element);
        }

        // Transfer ownership to C side
        let len = c_elements.len();
        let capacity = c_elements.capacity();
        let data = c_elements.as_mut_ptr();
        mem::forget(c_elements); // Prevent drop

        Ok(unsafe { FfiArray::from_raw_parts(data, len, capacity) })
    }

    #[allow(unsafe_code)]
    fn from_c(c: Self::CRepr) -> FfiResult<Self> {
        if c.is_null() || c.is_empty() {
            return Ok(Vec::new());
        }

        // SAFETY: We check for null/empty above.
        // The caller guarantees the pointer is valid and points to `len` elements.
        let c_slice = unsafe { c.as_slice() };

        // Convert each element back to Rust type
        let mut rust_elements = Vec::with_capacity(c.len());
        for c_element in c_slice {
            let rust_element = T::from_c(*c_element)?;
            rust_elements.push(rust_element);
        }

        Ok(rust_elements)
    }

    fn estimated_size(&self) -> usize {
        if self.is_empty() {
            return mem::size_of::<FfiArray<T::CRepr>>();
        }

        // Array overhead plus size of all elements
        let array_overhead = mem::size_of::<FfiArray<T::CRepr>>();
        let elements_size: usize = self.iter().map(|e| e.estimated_size()).sum();

        array_overhead + elements_size
    }
}

/// Helper function to safely free a C array allocated by Vec::to_c().
///
/// # Safety
///
/// - The pointer must have been allocated by `Vec::to_c()`
/// - The pointer must not be used after calling this function
/// - `T` must be the same type used in the original Vec
///
/// # Example
///
/// ```
/// use keyrx_core::ffi::marshal::traits::FfiMarshaler;
/// use keyrx_core::ffi::marshal::impls::array::free_ffi_array;
///
/// let numbers = vec![1u32, 2, 3];
/// let c_array = numbers.to_c().unwrap();
///
/// // Use the C array...
///
/// // Free when done
/// unsafe {
///     free_ffi_array::<u32>(c_array);
/// }
/// ```
#[allow(unsafe_code, clippy::missing_safety_doc)]
pub unsafe fn free_ffi_array<T>(array: FfiArray<T>) {
    if !array.is_null() && !array.is_empty() {
        // Reconstruct the Vec to drop it properly
        drop(Vec::from_raw_parts(
            array.as_ptr(),
            array.len(),
            array.capacity(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_vec() {
        let empty: Vec<u32> = Vec::new();
        let c_array = empty.to_c().unwrap();

        assert!(c_array.is_null() || c_array.is_empty());

        let restored = Vec::<u32>::from_c(c_array).unwrap();
        assert!(restored.is_empty());
    }

    #[test]
    fn test_vec_u32_to_c() {
        let numbers = vec![1u32, 2, 3, 4, 5];
        let c_array = numbers.to_c().unwrap();

        assert!(!c_array.is_null());
        assert_eq!(c_array.len(), 5);

        // Verify contents
        unsafe {
            let slice = c_array.as_slice();
            assert_eq!(slice, &[1, 2, 3, 4, 5]);

            free_ffi_array(c_array);
        }
    }

    #[test]
    fn test_vec_from_c() {
        // Create a C array manually
        let c_elements = vec![10u32, 20, 30];
        let len = c_elements.len();
        let capacity = c_elements.capacity();
        let data = c_elements.as_ptr() as *mut u32;
        mem::forget(c_elements);

        let c_array = unsafe { FfiArray::from_raw_parts(data, len, capacity) };

        // Convert back to Rust Vec
        let rust_vec = Vec::<u32>::from_c(c_array).unwrap();
        assert_eq!(rust_vec, vec![10, 20, 30]);

        // Clean up
        unsafe {
            free_ffi_array(c_array);
        }
    }

    #[test]
    fn test_vec_roundtrip() {
        let original = vec![100u32, 200, 300, 400];
        let c_array = original.to_c().unwrap();

        let restored = Vec::<u32>::from_c(c_array).unwrap();
        assert_eq!(restored, original);

        unsafe {
            free_ffi_array(c_array);
        }
    }

    #[test]
    fn test_vec_with_different_types() {
        // Test with u8
        let bytes = vec![1u8, 2, 3];
        let c_array = bytes.to_c().unwrap();
        let restored = Vec::<u8>::from_c(c_array).unwrap();
        assert_eq!(restored, bytes);
        unsafe {
            free_ffi_array(c_array);
        }

        // Test with u64
        let large = vec![1000u64, 2000, 3000];
        let c_array = large.to_c().unwrap();
        let restored = Vec::<u64>::from_c(c_array).unwrap();
        assert_eq!(restored, large);
        unsafe {
            free_ffi_array(c_array);
        }

        // Test with bool
        let bools = vec![true, false, true];
        let c_array = bools.to_c().unwrap();
        let restored = Vec::<bool>::from_c(c_array).unwrap();
        assert_eq!(restored, bools);
        unsafe {
            free_ffi_array(c_array);
        }
    }

    #[test]
    fn test_single_element() {
        let single = vec![42u32];
        let c_array = single.to_c().unwrap();

        assert_eq!(c_array.len(), 1);

        let restored = Vec::<u32>::from_c(c_array).unwrap();
        assert_eq!(restored, vec![42]);

        unsafe {
            free_ffi_array(c_array);
        }
    }

    #[test]
    fn test_large_vec() {
        let large: Vec<u32> = (0..1000).collect();
        let c_array = large.to_c().unwrap();

        assert_eq!(c_array.len(), 1000);

        let restored = Vec::<u32>::from_c(c_array).unwrap();
        assert_eq!(restored, large);

        unsafe {
            free_ffi_array(c_array);
        }
    }

    #[test]
    fn test_estimated_size() {
        let empty: Vec<u32> = Vec::new();
        assert!(empty.estimated_size() > 0); // At least array overhead

        let small = vec![1u32, 2, 3];
        let estimated = small.estimated_size();
        // Should be overhead + 3 * size_of::<u32>()
        assert!(estimated >= mem::size_of::<FfiArray<u32>>() + 12);
    }

    #[test]
    fn test_vec_no_streaming_by_default() {
        let small = vec![1u32; 100];
        assert!(!small.use_streaming());

        let medium = vec![1u32; 10_000];
        assert!(!medium.use_streaming());
    }

    #[test]
    fn test_vec_streaming_for_large_data() {
        // Create a large vec that exceeds 1MB threshold
        // 1MB / 4 bytes = 262144 elements
        let large = vec![1u32; 300_000];
        assert!(large.use_streaming());
    }

    #[test]
    fn test_ffi_array_null() {
        let null_array: FfiArray<u32> = FfiArray::null();
        assert!(null_array.is_null());
        assert!(null_array.is_empty());
        assert_eq!(null_array.len(), 0);
    }

    #[test]
    fn test_free_null_array() {
        // Should not crash when freeing null array
        unsafe {
            free_ffi_array(FfiArray::<u32>::null());
        }
    }

    #[test]
    fn test_free_empty_array() {
        // Should not crash when freeing empty array
        let empty = vec![0u32; 0];
        let c_array = empty.to_c().unwrap();
        unsafe {
            free_ffi_array(c_array);
        }
    }

    // Test with string elements to verify FfiMarshaler composition
    #[test]
    fn test_vec_of_strings() {
        use crate::ffi::marshal::impls::string::free_ffi_string;

        let strings = vec![
            String::from("hello"),
            String::from("world"),
            String::from("test"),
        ];
        let c_array = strings.to_c().unwrap();

        assert_eq!(c_array.len(), 3);

        let restored = Vec::<String>::from_c(c_array).unwrap();
        assert_eq!(restored, vec!["hello", "world", "test"]);

        // Clean up: first free the strings, then the array
        unsafe {
            let slice = c_array.as_slice();
            for ffi_str in slice {
                free_ffi_string(*ffi_str);
            }
            free_ffi_array(c_array);
        }
    }
}
