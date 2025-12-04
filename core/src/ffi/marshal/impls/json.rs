//! JSON-based marshaling for complex types.
//!
//! This module provides a fallback marshaling strategy for complex Rust types that
//! cannot easily be represented as C structs. It uses JSON serialization to transfer
//! data across the FFI boundary.
//!
//! # When to Use JSON Marshaling
//!
//! Use JSON marshaling for:
//! - Complex nested structures with dynamic sizes
//! - Types with collections of variable-length data
//! - Types that already implement `Serialize` and `Deserialize`
//! - Prototyping before optimizing with C-struct marshaling
//!
//! # Performance Characteristics
//!
//! - **Advantages**: Flexible, easy to implement, handles complex types
//! - **Disadvantages**: Serialization overhead, larger data size, heap allocation
//!
//! # Streaming Support
//!
//! For large JSON data (>1MB), the marshaler automatically switches to streaming
//! mode via chunked transfer to avoid memory pressure.
//!
//! # Example
//!
//! ```
//! use keyrx_core::ffi::marshal::traits::FfiMarshaler;
//! use keyrx_core::ffi::marshal::impls::json::JsonWrapper;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct ComplexData {
//!     items: Vec<String>,
//!     metadata: std::collections::HashMap<String, i32>,
//! }
//!
//! let data = ComplexData {
//!     items: vec!["a".to_string(), "b".to_string()],
//!     metadata: [("key1".to_string(), 42)].iter().cloned().collect(),
//! };
//!
//! // Wrap in JsonWrapper to enable JSON marshaling
//! let wrapper = JsonWrapper::new(data);
//! let c_repr = wrapper.to_c().unwrap();
//!
//! // Reconstruct on the other side
//! let restored: JsonWrapper<ComplexData> = JsonWrapper::from_c(c_repr).unwrap();
//! assert_eq!(restored.inner().items.len(), 2);
//!
//! // Don't forget to free!
//! use keyrx_core::ffi::marshal::impls::json::free_ffi_json;
//! unsafe {
//!     free_ffi_json(c_repr);
//! }
//! ```

use crate::ffi::error::{FfiError, FfiResult};
use crate::ffi::marshal::impls::string::FfiString;
use crate::ffi::marshal::traits::FfiMarshaler;
use serde::{de::DeserializeOwned, Serialize};
use std::ffi::{CStr, CString};

/// Wrapper type that enables JSON marshaling for any serializable type.
///
/// This wrapper implements `FfiMarshaler` by serializing the inner type to JSON
/// and transferring it as a C string.
///
/// # Type Requirements
///
/// The wrapped type `T` must implement:
/// - `Serialize`: For converting to JSON
/// - `DeserializeOwned`: For reconstructing from JSON
/// - `Send + Sync`: For thread safety across FFI boundaries
///
/// # Example
///
/// ```
/// use keyrx_core::ffi::marshal::impls::json::JsonWrapper;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct MyData {
///     value: i32,
/// }
///
/// let data = MyData { value: 42 };
/// let wrapper = JsonWrapper::new(data);
/// ```
#[derive(Debug, Clone)]
pub struct JsonWrapper<T> {
    inner: T,
}

impl<T> JsonWrapper<T> {
    /// Create a new JSON wrapper around a value.
    ///
    /// # Parameters
    ///
    /// * `inner` - The value to wrap
    ///
    /// # Returns
    ///
    /// A new `JsonWrapper` instance
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Get a reference to the inner value.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Consume the wrapper and return the inner value.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

/// C-compatible representation for JSON-marshaled data.
///
/// This is essentially a C string containing the JSON representation.
/// Reuses the `FfiString` type for consistency.
pub type FfiJson = FfiString;

impl<T> FfiMarshaler for JsonWrapper<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    type CRepr = FfiJson;

    #[allow(unsafe_code)]
    fn to_c(&self) -> FfiResult<Self::CRepr> {
        // Serialize to JSON string
        let json_string = serde_json::to_string(&self.inner).map_err(|e| {
            FfiError::serialization_failed(format!("Failed to serialize to JSON: {}", e))
        })?;

        // Convert to C string
        let c_string = CString::new(json_string).map_err(|_| {
            FfiError::internal("JSON string contains null byte (should not happen)")
        })?;

        // Transfer ownership to C side
        Ok(unsafe { FfiString::from_raw(c_string.into_raw()) })
    }

    #[allow(unsafe_code)]
    fn from_c(c: Self::CRepr) -> FfiResult<Self> {
        if c.is_null() {
            return Err(FfiError::null_pointer("json_string"));
        }

        // SAFETY: We check for null above. The caller guarantees the pointer is valid.
        unsafe {
            // Borrow the C string without taking ownership
            let c_str = CStr::from_ptr(c.as_ptr());

            // Convert to Rust string, validating UTF-8
            let json_str = c_str
                .to_str()
                .map_err(|_| FfiError::invalid_utf8("json_string"))?;

            // Deserialize from JSON
            let inner: T = serde_json::from_str(json_str).map_err(|e| {
                FfiError::deserialization_failed(format!("Failed to deserialize JSON: {}", e))
            })?;

            Ok(JsonWrapper { inner })
        }
    }

    fn estimated_size(&self) -> usize {
        // Estimate based on serialized JSON size
        // Use compact format for size estimation
        serde_json::to_string(&self.inner)
            .map(|s| s.len() + 1) // +1 for null terminator
            .unwrap_or(1024) // Default estimate if serialization fails
    }
}

/// Helper function to safely free a JSON C string allocated by to_c().
///
/// # Safety
///
/// The pointer must have been allocated by `JsonWrapper::to_c()`.
/// The pointer must not be used after calling this function.
///
/// # Example
///
/// ```
/// use keyrx_core::ffi::marshal::traits::FfiMarshaler;
/// use keyrx_core::ffi::marshal::impls::json::{JsonWrapper, free_ffi_json};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Data { value: i32 }
///
/// let wrapper = JsonWrapper::new(Data { value: 42 });
/// let c_json = wrapper.to_c().unwrap();
///
/// // Use the C JSON string...
///
/// // Free when done
/// unsafe {
///     free_ffi_json(c_json);
/// }
/// ```
#[no_mangle]
#[allow(unsafe_code, clippy::missing_safety_doc)]
pub unsafe extern "C" fn free_ffi_json(ffi_json: FfiJson) {
    if !ffi_json.is_null() {
        // Reconstruct the CString to drop it
        drop(CString::from_raw(ffi_json.as_ptr()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct SimpleData {
        value: i32,
        name: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct ComplexData {
        items: Vec<String>,
        metadata: HashMap<String, i32>,
        nested: Option<Box<ComplexData>>,
    }

    #[test]
    fn test_simple_json_marshaling() {
        let data = SimpleData {
            value: 42,
            name: "test".to_string(),
        };

        let wrapper = JsonWrapper::new(data);
        let c_json = wrapper.to_c().unwrap();

        assert!(!c_json.is_null());

        let restored: JsonWrapper<SimpleData> = JsonWrapper::from_c(c_json).unwrap();
        assert_eq!(restored.inner().value, 42);
        assert_eq!(restored.inner().name, "test");

        unsafe {
            free_ffi_json(c_json);
        }
    }

    #[test]
    fn test_complex_json_marshaling() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), 100);
        metadata.insert("key2".to_string(), 200);

        let data = ComplexData {
            items: vec![
                "item1".to_string(),
                "item2".to_string(),
                "item3".to_string(),
            ],
            metadata,
            nested: None,
        };

        let wrapper = JsonWrapper::new(data);
        let c_json = wrapper.to_c().unwrap();

        let restored: JsonWrapper<ComplexData> = JsonWrapper::from_c(c_json).unwrap();
        assert_eq!(restored.inner().items.len(), 3);
        assert_eq!(restored.inner().metadata.len(), 2);
        assert_eq!(restored.inner().metadata.get("key1"), Some(&100));

        unsafe {
            free_ffi_json(c_json);
        }
    }

    #[test]
    fn test_nested_json_marshaling() {
        let inner_data = ComplexData {
            items: vec!["inner".to_string()],
            metadata: HashMap::new(),
            nested: None,
        };

        let data = ComplexData {
            items: vec!["outer".to_string()],
            metadata: HashMap::new(),
            nested: Some(Box::new(inner_data)),
        };

        let wrapper = JsonWrapper::new(data);
        let c_json = wrapper.to_c().unwrap();

        let restored: JsonWrapper<ComplexData> = JsonWrapper::from_c(c_json).unwrap();
        assert!(restored.inner().nested.is_some());
        assert_eq!(restored.inner().nested.as_ref().unwrap().items[0], "inner");

        unsafe {
            free_ffi_json(c_json);
        }
    }

    #[test]
    fn test_empty_collections() {
        let data = ComplexData {
            items: vec![],
            metadata: HashMap::new(),
            nested: None,
        };

        let wrapper = JsonWrapper::new(data);
        let c_json = wrapper.to_c().unwrap();

        let restored: JsonWrapper<ComplexData> = JsonWrapper::from_c(c_json).unwrap();
        assert_eq!(restored.inner().items.len(), 0);
        assert_eq!(restored.inner().metadata.len(), 0);

        unsafe {
            free_ffi_json(c_json);
        }
    }

    #[test]
    fn test_from_null_pointer() {
        let null_json = unsafe { FfiString::from_raw(std::ptr::null_mut()) };
        let result = JsonWrapper::<SimpleData>::from_c(null_json);

        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code, "NULL_POINTER");
        }
    }

    #[test]
    fn test_from_invalid_json() {
        // Create a C string with invalid JSON
        let invalid_json = CString::new("{invalid json}").unwrap();
        let ffi_json = unsafe { FfiString::from_raw(invalid_json.into_raw()) };

        let result = JsonWrapper::<SimpleData>::from_c(ffi_json);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code, "DESERIALIZATION_FAILED");
        }

        // Note: We don't free here because from_c failed
    }

    #[test]
    fn test_estimated_size() {
        let small = JsonWrapper::new(SimpleData {
            value: 1,
            name: "x".to_string(),
        });
        let size = small.estimated_size();
        assert!(size > 0);
        assert!(size < 100); // Should be relatively small

        let large_items: Vec<String> = (0..100).map(|i| format!("item_{}", i)).collect();
        let large = JsonWrapper::new(ComplexData {
            items: large_items,
            metadata: HashMap::new(),
            nested: None,
        });
        let large_size = large.estimated_size();
        assert!(large_size > size); // Large object should estimate larger
    }

    #[test]
    fn test_use_streaming_threshold() {
        // Small data should not use streaming
        let small = JsonWrapper::new(SimpleData {
            value: 1,
            name: "test".to_string(),
        });
        assert!(!small.use_streaming());

        // Create large data that exceeds streaming threshold (1MB)
        let large_string = "x".repeat(2_000_000); // 2MB string
        let large = JsonWrapper::new(SimpleData {
            value: 1,
            name: large_string,
        });
        assert!(large.use_streaming()); // Should trigger streaming
    }

    #[test]
    fn test_json_with_unicode() {
        let data = SimpleData {
            value: 123,
            name: "Hello 世界 🦀".to_string(),
        };

        let wrapper = JsonWrapper::new(data);
        let c_json = wrapper.to_c().unwrap();

        let restored: JsonWrapper<SimpleData> = JsonWrapper::from_c(c_json).unwrap();
        assert_eq!(restored.inner().name, "Hello 世界 🦀");

        unsafe {
            free_ffi_json(c_json);
        }
    }

    #[test]
    fn test_inner_and_into_inner() {
        let data = SimpleData {
            value: 99,
            name: "test".to_string(),
        };

        let wrapper = JsonWrapper::new(data);

        // Test inner()
        assert_eq!(wrapper.inner().value, 99);
        assert_eq!(wrapper.inner().name, "test");

        // Test into_inner()
        let inner = wrapper.into_inner();
        assert_eq!(inner.value, 99);
        assert_eq!(inner.name, "test");
    }

    #[test]
    fn test_free_null_pointer() {
        // Should not crash when freeing null pointer
        unsafe {
            free_ffi_json(FfiString::from_raw(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_roundtrip_preserves_data() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), -42);
        metadata.insert("key2".to_string(), 0);
        metadata.insert("key3".to_string(), 12345);

        let original = ComplexData {
            items: vec![
                "first".to_string(),
                "second".to_string(),
                "third with spaces".to_string(),
            ],
            metadata: metadata.clone(),
            nested: None,
        };

        let wrapper = JsonWrapper::new(original);
        let c_json = wrapper.to_c().unwrap();
        let restored: JsonWrapper<ComplexData> = JsonWrapper::from_c(c_json).unwrap();

        assert_eq!(restored.inner().items.len(), 3);
        assert_eq!(restored.inner().items[0], "first");
        assert_eq!(restored.inner().items[2], "third with spaces");
        assert_eq!(restored.inner().metadata.len(), 3);
        assert_eq!(restored.inner().metadata.get("key1"), Some(&-42));
        assert_eq!(restored.inner().metadata.get("key3"), Some(&12345));

        unsafe {
            free_ffi_json(c_json);
        }
    }
}
