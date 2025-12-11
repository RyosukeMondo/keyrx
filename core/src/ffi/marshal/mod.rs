//! FFI marshaling layer.
//!
//! This module provides unified marshaling for all data crossing the FFI boundary.
//! It offers two strategies:
//!
//! - **Direct C Marshaling**: For small data (<1MB) using C-compatible structs
//! - **Streaming Marshaling**: For large data (>1MB) using chunked transfer
//!
//! # Architecture
//!
//! The marshaling system is organized into:
//!
//! - [`traits`]: Core traits (`FfiMarshaler`, `FfiStreamMarshaler`)
//! - [`error`]: Comprehensive error types with hint and context
//! - [`result`]: FFI result types
//! - `impls`: Implementations for common types (primitives, strings, arrays, JSON)
//!
//! # Usage
//!
//! ```
//! use keyrx_core::ffi::marshal::traits::{FfiMarshaler, CRepr};
//! use keyrx_core::ffi::error::FfiResult;
//!
//! // Define C representation
//! #[repr(C)]
//! #[derive(Copy, Clone)]
//! struct DeviceIdC {
//!     value: u64,
//! }
//!
//! impl CRepr for DeviceIdC {}
//!
//! // Implement marshaler
//! struct DeviceId(u64);
//!
//! impl FfiMarshaler for DeviceId {
//!     type CRepr = DeviceIdC;
//!
//!     fn to_c(&self) -> FfiResult<Self::CRepr> {
//!         Ok(DeviceIdC { value: self.0 })
//!     }
//!
//!     fn from_c(c: Self::CRepr) -> FfiResult<Self> {
//!         Ok(DeviceId(c.value))
//!     }
//!
//!     fn estimated_size(&self) -> usize {
//!         std::mem::size_of::<DeviceIdC>()
//!     }
//! }
//! ```

pub mod callback;
pub mod error;
pub mod impls;
pub mod result;
pub mod stream;
pub mod traits;

// Re-export core types
pub use callback::{CallbackId, CallbackRegistry, FfiCallback};
pub use error::{MarshalError, MarshalErrorC};
pub use result::{FfiErrorData, FfiErrorPtr, FfiResult};
pub use stream::{
    ChunkCollector, ChunkIterator, StreamContext, DEFAULT_CHUNK_SIZE, MAX_CHUNK_SIZE,
};
pub use traits::{CRepr, FfiMarshaler, FfiStreamMarshaler, STREAMING_THRESHOLD};
