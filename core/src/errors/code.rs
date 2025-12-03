//! Error codes and categories for KeyRx.
//!
//! Error codes follow the format KRX-CXXX where:
//! - C is the category prefix (C/R/D/V/F/I)
//! - XXX is a 3-digit number
//!
//! Categories organize errors by domain with non-overlapping ranges.

use std::fmt;

/// Error category with assigned number ranges.
///
/// Each category has a single-letter prefix and number range:
/// - Config (C): 1xxx - Configuration loading and parsing
/// - Runtime (R): 2xxx - Engine and processing errors
/// - Driver (D): 3xxx - Platform-specific driver errors
/// - Validation (V): 4xxx - Config validation and conflicts
/// - Ffi (F): 5xxx - FFI boundary errors
/// - Internal (I): 9xxx - Internal/unexpected errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Configuration errors (1xxx)
    Config,
    /// Runtime errors (2xxx)
    Runtime,
    /// Driver errors (3xxx)
    Driver,
    /// Validation errors (4xxx)
    Validation,
    /// FFI errors (5xxx)
    Ffi,
    /// Internal errors (9xxx)
    Internal,
}

impl ErrorCategory {
    /// Get the single-letter prefix for this category.
    pub const fn prefix(&self) -> char {
        match self {
            ErrorCategory::Config => 'C',
            ErrorCategory::Runtime => 'R',
            ErrorCategory::Driver => 'D',
            ErrorCategory::Validation => 'V',
            ErrorCategory::Ffi => 'F',
            ErrorCategory::Internal => 'I',
        }
    }

    /// Get the base number for this category's range.
    pub const fn base_number(&self) -> u16 {
        match self {
            ErrorCategory::Config => 1000,
            ErrorCategory::Runtime => 2000,
            ErrorCategory::Driver => 3000,
            ErrorCategory::Validation => 4000,
            ErrorCategory::Ffi => 5000,
            ErrorCategory::Internal => 9000,
        }
    }

    /// Check if a number is in this category's valid range.
    pub const fn contains(&self, number: u16) -> bool {
        let base = self.base_number();
        number >= base && number < base + 1000
    }
}

/// Unique error code in KRX-CXXX format.
///
/// ErrorCode combines a category with a number to create a globally unique
/// identifier. The Display implementation formats it as "KRX-C001" etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErrorCode {
    category: ErrorCategory,
    number: u16,
}

impl ErrorCode {
    /// Create a new error code.
    ///
    /// The number should be within the category's valid range (base to base+999).
    /// This is not enforced at compile time, but invalid codes will display oddly.
    pub const fn new(category: ErrorCategory, number: u16) -> Self {
        Self { category, number }
    }

    /// Get the category of this error code.
    pub const fn category(&self) -> ErrorCategory {
        self.category
    }

    /// Get the numeric code.
    pub const fn number(&self) -> u16 {
        self.number
    }

    /// Format as string (e.g., "KRX-C001").
    pub fn as_string(&self) -> String {
        format!("KRX-{}{:03}", self.category.prefix(), self.number)
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KRX-{}{:03}", self.category.prefix(), self.number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_category_prefix() {
        assert_eq!(ErrorCategory::Config.prefix(), 'C');
        assert_eq!(ErrorCategory::Runtime.prefix(), 'R');
        assert_eq!(ErrorCategory::Driver.prefix(), 'D');
        assert_eq!(ErrorCategory::Validation.prefix(), 'V');
        assert_eq!(ErrorCategory::Ffi.prefix(), 'F');
        assert_eq!(ErrorCategory::Internal.prefix(), 'I');
    }

    #[test]
    fn error_category_base_number() {
        assert_eq!(ErrorCategory::Config.base_number(), 1000);
        assert_eq!(ErrorCategory::Runtime.base_number(), 2000);
        assert_eq!(ErrorCategory::Driver.base_number(), 3000);
        assert_eq!(ErrorCategory::Validation.base_number(), 4000);
        assert_eq!(ErrorCategory::Ffi.base_number(), 5000);
        assert_eq!(ErrorCategory::Internal.base_number(), 9000);
    }

    #[test]
    fn error_category_contains() {
        assert!(ErrorCategory::Config.contains(1001));
        assert!(ErrorCategory::Config.contains(1999));
        assert!(!ErrorCategory::Config.contains(2000));
        assert!(!ErrorCategory::Config.contains(999));

        assert!(ErrorCategory::Runtime.contains(2001));
        assert!(!ErrorCategory::Runtime.contains(1999));
    }

    #[test]
    fn error_code_display() {
        let code = ErrorCode::new(ErrorCategory::Config, 1001);
        assert_eq!(code.to_string(), "KRX-C1001");

        let code = ErrorCode::new(ErrorCategory::Driver, 3042);
        assert_eq!(code.to_string(), "KRX-D3042");
    }

    #[test]
    fn error_code_as_string() {
        let code = ErrorCode::new(ErrorCategory::Runtime, 2100);
        assert_eq!(code.as_string(), "KRX-R2100");
    }

    #[test]
    fn error_code_accessors() {
        let code = ErrorCode::new(ErrorCategory::Validation, 4001);
        assert_eq!(code.category(), ErrorCategory::Validation);
        assert_eq!(code.number(), 4001);
    }

    #[test]
    fn error_code_formatting_edge_cases() {
        // Single digit
        let code = ErrorCode::new(ErrorCategory::Config, 1001);
        assert_eq!(code.to_string(), "KRX-C1001");

        // Three digits in range
        let code = ErrorCode::new(ErrorCategory::Ffi, 5999);
        assert_eq!(code.to_string(), "KRX-F5999");
    }
}
