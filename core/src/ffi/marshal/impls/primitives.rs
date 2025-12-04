//! FfiMarshaler implementations for primitive types.
//!
//! This module provides zero-copy marshaling for all Rust primitive types that
//! have direct C equivalents. Since these types are already C-compatible, the
//! marshaling is essentially a no-op with type-level validation.
//!
//! # Supported Types
//!
//! All implementations use identity marshaling (the type is its own C representation):
//!
//! - **Unsigned integers**: `u8`, `u16`, `u32`, `u64`
//! - **Signed integers**: `i8`, `i16`, `i32`, `i64`
//! - **Floating point**: `f32`, `f64`
//! - **Boolean**: `bool` (represented as `u8` in C: 0=false, 1=true)
//!
//! # Zero-Copy Guarantees
//!
//! All primitive marshalers:
//! - Use the type itself as `CRepr` (zero abstraction cost)
//! - Perform no allocations
//! - Cannot fail (return `Ok` unconditionally)
//! - Have O(1) time complexity
//!
//! # C Compatibility
//!
//! These types match C's primitive types in size and representation:
//! - `u8` ↔ `uint8_t`
//! - `u16` ↔ `uint16_t`
//! - `u32` ↔ `uint32_t`
//! - `u64` ↔ `uint64_t`
//! - `i8` ↔ `int8_t`
//! - `i16` ↔ `int16_t`
//! - `i32` ↔ `int32_t`
//! - `i64` ↔ `int64_t`
//! - `f32` ↔ `float`
//! - `f64` ↔ `double`
//! - `bool` ↔ `uint8_t` (with 0/1 values)
//!
//! # Example
//!
//! ```
//! use keyrx_core::ffi::marshal::traits::FfiMarshaler;
//!
//! // All primitives implement FfiMarshaler
//! let value: u32 = 42;
//! let c_repr = value.to_c().unwrap(); // Zero-copy, always succeeds
//! assert_eq!(c_repr, 42);
//!
//! let restored = u32::from_c(c_repr).unwrap();
//! assert_eq!(restored, 42);
//! ```

use crate::ffi::error::FfiResult;
use crate::ffi::marshal::traits::{CRepr, FfiMarshaler};

// Implement CRepr for all primitive types
// These types are already C-compatible and can be passed directly across FFI
impl CRepr for u8 {}
impl CRepr for u16 {}
impl CRepr for u32 {}
impl CRepr for u64 {}
impl CRepr for i8 {}
impl CRepr for i16 {}
impl CRepr for i32 {}
impl CRepr for i64 {}
impl CRepr for f32 {}
impl CRepr for f64 {}
impl CRepr for bool {}

// Macro to implement FfiMarshaler for primitive types with identity marshaling
macro_rules! impl_primitive_marshaler {
    ($ty:ty) => {
        impl FfiMarshaler for $ty {
            type CRepr = Self;

            #[inline]
            fn to_c(&self) -> FfiResult<Self::CRepr> {
                Ok(*self)
            }

            #[inline]
            fn from_c(c: Self::CRepr) -> FfiResult<Self> {
                Ok(c)
            }

            #[inline]
            fn estimated_size(&self) -> usize {
                std::mem::size_of::<Self>()
            }
        }
    };
}

// Implement for all unsigned integer types
impl_primitive_marshaler!(u8);
impl_primitive_marshaler!(u16);
impl_primitive_marshaler!(u32);
impl_primitive_marshaler!(u64);

// Implement for all signed integer types
impl_primitive_marshaler!(i8);
impl_primitive_marshaler!(i16);
impl_primitive_marshaler!(i32);
impl_primitive_marshaler!(i64);

// Implement for floating point types
impl_primitive_marshaler!(f32);
impl_primitive_marshaler!(f64);

// Implement for bool
impl_primitive_marshaler!(bool);

#[cfg(test)]
mod tests {
    use super::*;

    // Test unsigned integers
    #[test]
    fn test_u8_marshaler() {
        let value: u8 = 255;
        let c_repr = value.to_c().unwrap();
        assert_eq!(c_repr, 255);
        assert_eq!(u8::from_c(c_repr).unwrap(), 255);
        assert_eq!(value.estimated_size(), 1);
    }

    #[test]
    fn test_u16_marshaler() {
        let value: u16 = 65535;
        let c_repr = value.to_c().unwrap();
        assert_eq!(c_repr, 65535);
        assert_eq!(u16::from_c(c_repr).unwrap(), 65535);
        assert_eq!(value.estimated_size(), 2);
    }

    #[test]
    fn test_u32_marshaler() {
        let value: u32 = 4_294_967_295;
        let c_repr = value.to_c().unwrap();
        assert_eq!(c_repr, 4_294_967_295);
        assert_eq!(u32::from_c(c_repr).unwrap(), 4_294_967_295);
        assert_eq!(value.estimated_size(), 4);
    }

    #[test]
    fn test_u64_marshaler() {
        let value: u64 = 18_446_744_073_709_551_615;
        let c_repr = value.to_c().unwrap();
        assert_eq!(c_repr, 18_446_744_073_709_551_615);
        assert_eq!(u64::from_c(c_repr).unwrap(), 18_446_744_073_709_551_615);
        assert_eq!(value.estimated_size(), 8);
    }

    // Test signed integers
    #[test]
    fn test_i8_marshaler() {
        let value: i8 = -128;
        let c_repr = value.to_c().unwrap();
        assert_eq!(c_repr, -128);
        assert_eq!(i8::from_c(c_repr).unwrap(), -128);
        assert_eq!(value.estimated_size(), 1);
    }

    #[test]
    fn test_i16_marshaler() {
        let value: i16 = -32768;
        let c_repr = value.to_c().unwrap();
        assert_eq!(c_repr, -32768);
        assert_eq!(i16::from_c(c_repr).unwrap(), -32768);
        assert_eq!(value.estimated_size(), 2);
    }

    #[test]
    fn test_i32_marshaler() {
        let value: i32 = -2_147_483_648;
        let c_repr = value.to_c().unwrap();
        assert_eq!(c_repr, -2_147_483_648);
        assert_eq!(i32::from_c(c_repr).unwrap(), -2_147_483_648);
        assert_eq!(value.estimated_size(), 4);
    }

    #[test]
    fn test_i64_marshaler() {
        let value: i64 = -9_223_372_036_854_775_808;
        let c_repr = value.to_c().unwrap();
        assert_eq!(c_repr, -9_223_372_036_854_775_808);
        assert_eq!(i64::from_c(c_repr).unwrap(), -9_223_372_036_854_775_808);
        assert_eq!(value.estimated_size(), 8);
    }

    // Test floating point
    #[test]
    fn test_f32_marshaler() {
        let value: f32 = 3.14159;
        let c_repr = value.to_c().unwrap();
        assert_eq!(c_repr, 3.14159);
        assert_eq!(f32::from_c(c_repr).unwrap(), 3.14159);
        assert_eq!(value.estimated_size(), 4);
    }

    #[test]
    fn test_f64_marshaler() {
        let value: f64 = 2.718281828459045;
        let c_repr = value.to_c().unwrap();
        assert_eq!(c_repr, 2.718281828459045);
        assert_eq!(f64::from_c(c_repr).unwrap(), 2.718281828459045);
        assert_eq!(value.estimated_size(), 8);
    }

    // Test boolean
    #[test]
    fn test_bool_marshaler() {
        let true_val = true;
        let c_repr = true_val.to_c().unwrap();
        assert!(c_repr);
        assert!(bool::from_c(c_repr).unwrap());
        assert_eq!(true_val.estimated_size(), 1);

        let false_val = false;
        let c_repr = false_val.to_c().unwrap();
        assert!(!c_repr);
        assert!(!bool::from_c(c_repr).unwrap());
        assert_eq!(false_val.estimated_size(), 1);
    }

    // Test zero-copy property
    #[test]
    fn test_zero_copy() {
        let value: u64 = 12345;
        let c_repr = value.to_c().unwrap();

        // Verify that the C representation is the same type
        let _type_check: u64 = c_repr;

        // Verify no heap allocation occurred (compile-time size check)
        assert_eq!(
            std::mem::size_of_val(&value),
            std::mem::size_of_val(&c_repr)
        );
    }

    // Test roundtrip for all types
    #[test]
    fn test_roundtrip_all_types() {
        // Unsigned
        assert_eq!(u8::from_c(42u8.to_c().unwrap()).unwrap(), 42u8);
        assert_eq!(u16::from_c(1234u16.to_c().unwrap()).unwrap(), 1234u16);
        assert_eq!(u32::from_c(123456u32.to_c().unwrap()).unwrap(), 123456u32);
        assert_eq!(
            u64::from_c(123456789u64.to_c().unwrap()).unwrap(),
            123456789u64
        );

        // Signed
        assert_eq!(i8::from_c((-42i8).to_c().unwrap()).unwrap(), -42i8);
        assert_eq!(i16::from_c((-1234i16).to_c().unwrap()).unwrap(), -1234i16);
        assert_eq!(
            i32::from_c((-123456i32).to_c().unwrap()).unwrap(),
            -123456i32
        );
        assert_eq!(
            i64::from_c((-123456789i64).to_c().unwrap()).unwrap(),
            -123456789i64
        );

        // Floating point
        assert_eq!(f32::from_c(1.5f32.to_c().unwrap()).unwrap(), 1.5f32);
        assert_eq!(f64::from_c(2.5f64.to_c().unwrap()).unwrap(), 2.5f64);

        // Boolean
        assert_eq!(bool::from_c(true.to_c().unwrap()).unwrap(), true);
        assert_eq!(bool::from_c(false.to_c().unwrap()).unwrap(), false);
    }

    // Test that primitives don't use streaming (they're too small)
    #[test]
    fn test_no_streaming_for_primitives() {
        assert!(!42u8.use_streaming());
        assert!(!1234u16.use_streaming());
        assert!(!123456u32.use_streaming());
        assert!(!123456789u64.use_streaming());
        assert!(!(-42i8).use_streaming());
        assert!(!3.14f32.use_streaming());
        assert!(!2.718f64.use_streaming());
        assert!(!true.use_streaming());
    }
}
