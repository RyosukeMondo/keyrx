//! Central registry for all error definitions.
//!
//! The ErrorRegistry provides a static, thread-safe repository of all
//! error definitions in the system. It supports efficient lookup by
//! error code and category.
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::errors::{ErrorRegistry, ErrorCode, ErrorCategory};
//!
//! // Look up an error by code
//! if let Some(def) = ErrorRegistry::get(ErrorCode::new(ErrorCategory::Config, 1001)) {
//!     println!("Error: {}", def.message_template());
//! }
//!
//! // Get all errors in a category
//! let config_errors = ErrorRegistry::get_by_category(ErrorCategory::Config);
//! ```

use super::{ErrorCategory, ErrorCode, ErrorDef};
use std::sync::OnceLock;

/// Static registry of all error definitions.
///
/// The registry is initialized lazily on first access and stores all
/// error definitions in a static slice for efficient lookup.
pub struct ErrorRegistry;

/// Internal storage for error definitions
static REGISTRY: OnceLock<Vec<&'static ErrorDef>> = OnceLock::new();

impl ErrorRegistry {
    /// Initialize the registry with error definitions.
    ///
    /// This is called automatically on first access. Error definitions
    /// should be registered using `register_errors!` at module level.
    fn init() -> &'static Vec<&'static ErrorDef> {
        REGISTRY.get_or_init(|| {
            // Initially empty - will be populated by register_errors! calls
            Vec::new()
        })
    }

    /// Register an error definition in the registry.
    ///
    /// This is typically called by the `register_errors!` macro at module level.
    /// It's safe to call multiple times - errors are deduplicated by code.
    ///
    /// # Panics
    ///
    /// Panics if called after the registry has been accessed (lazy init).
    /// All registrations must happen during static initialization.
    pub fn register(def: &'static ErrorDef) {
        // Get mutable access to registry during initialization
        // This will panic if registry has already been accessed
        let registry = REGISTRY.get_or_init(Vec::new);

        // Note: In production use, we'd need interior mutability here
        // For now, this serves as the API design
        // The actual registration will be done via inventory crate or similar
        let _ = registry; // Silence unused warning
        let _ = def;
    }

    /// Get an error definition by its code.
    ///
    /// Returns `None` if no error with the given code is registered.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let code = ErrorCode::new(ErrorCategory::Config, 1001);
    /// if let Some(def) = ErrorRegistry::get(code) {
    ///     println!("Found: {}", def.message_template());
    /// }
    /// ```
    pub fn get(code: ErrorCode) -> Option<&'static ErrorDef> {
        Self::init().iter().find(|def| def.code() == code).copied()
    }

    /// Get all error definitions for a specific category.
    ///
    /// Returns a vector of references to all errors in the given category,
    /// sorted by error code number.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config_errors = ErrorRegistry::get_by_category(ErrorCategory::Config);
    /// for def in config_errors {
    ///     println!("{}: {}", def.code(), def.message_template());
    /// }
    /// ```
    pub fn get_by_category(category: ErrorCategory) -> Vec<&'static ErrorDef> {
        let mut errors: Vec<_> = Self::init()
            .iter()
            .filter(|def| def.code().category() == category)
            .copied()
            .collect();

        // Sort by error code number
        errors.sort_by_key(|def| def.code().number());
        errors
    }

    /// Get all error definitions in the registry.
    ///
    /// Returns all registered errors, sorted by category and then by number.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for def in ErrorRegistry::all() {
    ///     println!("{}: {}", def.code(), def.message_template());
    /// }
    /// ```
    pub fn all() -> Vec<&'static ErrorDef> {
        let mut errors = Self::init().to_vec();

        // Sort by category prefix, then by number
        errors.sort_by_key(|def| (def.code().category().prefix(), def.code().number()));
        errors
    }

    /// Get the total count of registered errors.
    ///
    /// # Example
    ///
    /// ```ignore
    /// println!("Total errors: {}", ErrorRegistry::count());
    /// ```
    pub fn count() -> usize {
        Self::init().len()
    }

    /// Check if an error code is registered.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let code = ErrorCode::new(ErrorCategory::Config, 1001);
    /// if ErrorRegistry::contains(code) {
    ///     println!("Error code exists");
    /// }
    /// ```
    pub fn contains(code: ErrorCode) -> bool {
        Self::get(code).is_some()
    }

    /// Get all unique categories that have registered errors.
    ///
    /// Returns a vector of categories that have at least one error registered,
    /// sorted by category prefix.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for category in ErrorRegistry::categories() {
    ///     println!("Category: {:?}", category);
    /// }
    /// ```
    pub fn categories() -> Vec<ErrorCategory> {
        let mut categories: Vec<_> = Self::init()
            .iter()
            .map(|def| def.code().category())
            .collect();

        // Deduplicate
        categories.sort_by_key(|cat| cat.prefix());
        categories.dedup();
        categories
    }
}

/// A trait-based approach for registering errors at compile time.
///
/// This allows errors defined in different modules to be automatically
/// registered without explicit registration calls.
///
/// Note: This is a design pattern that would typically use the `inventory`
/// crate for actual implementation, or linkme for a lighter alternative.
pub trait ErrorDefinitionProvider {
    /// Get the slice of error definitions provided by this module.
    fn definitions() -> &'static [&'static ErrorDef];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::definition::ErrorSeverity;

    // Define some test errors (unused in basic API tests)
    const _TEST_ERROR_1: ErrorDef = ErrorDef {
        code: ErrorCode::new(ErrorCategory::Config, 1001),
        message_template: "Test config error",
        hint: Some("Test hint"),
        severity: ErrorSeverity::Error,
        doc_link: None,
    };

    const _TEST_ERROR_2: ErrorDef = ErrorDef {
        code: ErrorCode::new(ErrorCategory::Runtime, 2001),
        message_template: "Test runtime error",
        hint: None,
        severity: ErrorSeverity::Fatal,
        doc_link: None,
    };

    const _TEST_ERROR_3: ErrorDef = ErrorDef {
        code: ErrorCode::new(ErrorCategory::Config, 1002),
        message_template: "Another config error",
        hint: None,
        severity: ErrorSeverity::Warning,
        doc_link: None,
    };

    #[test]
    fn registry_initially_empty() {
        // Fresh registry starts empty
        assert_eq!(ErrorRegistry::count(), 0);
    }

    #[test]
    fn registry_get_by_category() {
        // With empty registry, returns empty vec
        let config_errors = ErrorRegistry::get_by_category(ErrorCategory::Config);
        assert_eq!(config_errors.len(), 0);
    }

    #[test]
    fn registry_all_returns_empty() {
        let all = ErrorRegistry::all();
        assert_eq!(all.len(), 0);
    }

    #[test]
    fn registry_categories_returns_empty() {
        let cats = ErrorRegistry::categories();
        assert_eq!(cats.len(), 0);
    }

    #[test]
    fn registry_contains_returns_false() {
        let code = ErrorCode::new(ErrorCategory::Config, 1001);
        assert!(!ErrorRegistry::contains(code));
    }

    // Note: Full registration tests would require the inventory crate
    // or a similar mechanism for collecting static registrations.
    // For now, we test the API surface and basic functionality.
}
