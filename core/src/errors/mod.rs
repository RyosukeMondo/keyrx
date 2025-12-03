//! Error code registry system for KeyRx.
//!
//! This module provides a structured error handling system with:
//! - Unique error codes in KRX-CXXX format
//! - Category-based organization
//! - Template-based error messages
//! - Automatic documentation generation
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::errors::{ErrorCode, ErrorCategory};
//!
//! let code = ErrorCode::new(ErrorCategory::Config, 1001);
//! assert_eq!(code.to_string(), "KRX-C1001");
//! ```

pub mod code;

pub use code::{ErrorCategory, ErrorCode};
