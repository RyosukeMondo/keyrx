//! Error code registry system for KeyRx.
//!
//! This module provides a structured error handling system with:
//! - Unique error codes in KRX-CXXX format
//! - Category-based organization
//! - Template-based error messages
//! - Automatic documentation generation
//! - Runtime error type with context chaining
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::errors::{ErrorCode, ErrorCategory, KeyrxError};
//!
//! let code = ErrorCode::new(ErrorCategory::Config, 1001);
//! assert_eq!(code.to_string(), "KRX-C1001");
//!
//! // Create runtime error
//! let err = KeyrxError::simple(&CONFIG_NOT_FOUND);
//! ```

pub mod code;
pub mod config;
pub mod definition;
pub mod doc_generator;
pub mod driver;
pub mod error;
#[macro_use]
pub mod macros;
pub mod registry;
pub mod runtime;
pub mod validation;

pub use code::{ErrorCategory, ErrorCode};
pub use definition::{ErrorDef, ErrorSeverity};
pub use doc_generator::ErrorDocGenerator;
pub use error::KeyrxError;
pub use registry::ErrorRegistry;
