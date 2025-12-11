//! FfiMarshaler implementations for common types.
//!
//! This module provides ready-to-use implementations of the [`super::traits::FfiMarshaler`]
//! trait for standard Rust types. These implementations handle the conversion
//! between Rust types and their C-compatible representations.
//!
//! # Modules
//!
//! - [`primitives`]: Zero-copy marshaling for primitive types (u8, u16, u32, u64, bool, f32, f64, etc.)
//! - [`string`]: String and `&str` marshaling with null-terminated C strings
//! - `array`: `Vec<T>` marshaling with length-prefixed arrays
//! - [`json`]: JSON-based marshaling for complex types
//! - [`session`]: Streaming marshaling for large session data (recording/replay)
//!
//! # Usage
//!
//! These implementations are automatically available when the types are in scope:
//!
//! ```
//! use keyrx_core::ffi::marshal::traits::FfiMarshaler;
//!
//! // Primitives work out of the box
//! let value: u32 = 42;
//! let c_value = value.to_c().unwrap();
//! assert_eq!(c_value, 42);
//! ```

pub mod array;
pub mod json;
pub mod primitives;
pub mod session;
pub mod string;
