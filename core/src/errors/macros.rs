//! Macros for convenient error definition and creation.
//!
//! This module provides declarative macros for:
//! - Defining error sets with compile-time duplicate detection
//! - Creating KeyrxError instances with context
//! - Early return with errors (bail pattern)

/// Define a set of error codes with compile-time duplicate detection.
///
/// This macro simplifies defining multiple ErrorDef constants at once
/// and performs compile-time checks to ensure no duplicate error codes.
///
/// # Syntax
///
/// ```ignore
/// define_errors! {
///     category: Config,
///     base: 1000,
///
///     errors: {
///         CONFIG_NOT_FOUND = 1 => {
///             message: "Configuration file not found: {path}",
///             hint: "Check that the file exists and has read permissions",
///             severity: Error,
///             doc: "https://docs.keyrx.com/errors#KRX-C1001",
///         },
///
///         CONFIG_PARSE_ERROR = 2 => {
///             message: "Failed to parse configuration: {error}",
///             hint: "Verify the TOML syntax is correct",
///             severity: Error,
///         },
///     }
/// }
/// ```
///
/// # Generated Code
///
/// For each error, the macro generates:
/// - A public constant of type `ErrorDef`
/// - The error code is computed as `base + number`
/// - Optional fields (hint, doc) can be omitted
///
/// # Duplicate Detection
///
/// The macro generates a compile-time check that fails if any error
/// codes are duplicated within the same category.
#[macro_export]
macro_rules! define_errors {
    // Main entry point: process all errors and generate duplicate check
    (
        category: $category:ident,
        base: $base:literal,

        errors: {
            $(
                $name:ident = $num:literal => {
                    message: $msg:literal
                    $(, hint: $hint:literal)?
                    $(, severity: $severity:ident)?
                    $(, doc: $doc:literal)?
                    $(,)?
                }
            ),* $(,)?
        }
    ) => {
        // Generate each error definition
        $(
            #[allow(missing_docs)]
            pub const $name: $crate::errors::ErrorDef = $crate::errors::ErrorDef {
                code: $crate::errors::ErrorCode::new(
                    $crate::errors::ErrorCategory::$category,
                    $base + $num,
                ),
                message_template: $msg,
                hint: define_errors!(@hint $($hint)?),
                severity: define_errors!(@severity $($severity)?),
                doc_link: define_errors!(@doc $($doc)?),
            };
        )*

        // Compile-time duplicate detection
        // This const function will fail to compile if any codes are duplicated
        const _: () = {
            const fn check_duplicates() {
                // Create array of all error codes in this set
                const CODES: &[u16] = &[$($base + $num),*];

                // Check for duplicates by comparing each pair
                let mut i = 0;
                while i < CODES.len() {
                    let mut j = i + 1;
                    while j < CODES.len() {
                        // This will fail at compile time if any codes match
                        assert!(CODES[i] != CODES[j], "Duplicate error code detected");
                        j += 1;
                    }
                    i += 1;
                }
            }

            check_duplicates();
        };
    };

    // Helper rules for optional fields
    (@hint) => { None };
    (@hint $hint:literal) => { Some($hint) };

    (@severity) => { $crate::errors::ErrorSeverity::Error };
    (@severity $severity:ident) => { $crate::errors::ErrorSeverity::$severity };

    (@doc) => { None };
    (@doc $doc:literal) => { Some($doc) };
}

/// Create a KeyrxError with context.
///
/// This macro provides a convenient way to create KeyrxError instances
/// with compile-time error code checking and runtime context.
///
/// # Syntax
///
/// ```ignore
/// // Simple error with no context
/// let err = keyrx_err!(CONFIG_NOT_FOUND);
///
/// // Error with context
/// let err = keyrx_err!(CONFIG_NOT_FOUND, path = "/etc/keyrx.toml");
///
/// // Error with multiple context values
/// let err = keyrx_err!(CONFIG_PARSE_ERROR,
///     file = "config.toml",
///     line = "42",
///     error = parse_err.to_string()
/// );
///
/// // Error with source
/// let err = keyrx_err!(CONFIG_PARSE_ERROR,
///     file = "config.toml";
///     source = io_err
/// );
/// ```
#[macro_export]
macro_rules! keyrx_err {
    // Simple error with no context
    ($def:expr) => {
        $crate::errors::KeyrxError::simple(&$def)
    };

    // Error with context, no source
    ($def:expr, $($key:ident = $value:expr),+ $(,)?) => {
        $crate::errors::KeyrxError::new(
            &$def,
            vec![
                $(
                    (stringify!($key).to_string(), $value.to_string()),
                )*
            ],
            None,
        )
    };

    // Error with context and source
    ($def:expr, $($key:ident = $value:expr),+ ; source = $source:expr $(,)?) => {
        $crate::errors::KeyrxError::new(
            &$def,
            vec![
                $(
                    (stringify!($key).to_string(), $value.to_string()),
                )*
            ],
            Some(Box::new($source)),
        )
    };
}

/// Create and return a KeyrxError (early return).
///
/// This macro is the error equivalent of `bail!` from anyhow.
/// It creates a KeyrxError and immediately returns it from the function.
///
/// # Syntax
///
/// ```ignore
/// fn load_config(path: &str) -> Result<Config, KeyrxError> {
///     if !Path::new(path).exists() {
///         bail_keyrx!(CONFIG_NOT_FOUND, path = path);
///     }
///     // ... rest of function
/// }
/// ```
#[macro_export]
macro_rules! bail_keyrx {
    ($($args:tt)*) => {
        return Err($crate::keyrx_err!($($args)*))
    };
}

#[cfg(test)]
mod tests {
    use super::super::code::ErrorCategory;
    use super::super::definition::ErrorSeverity;

    // Test the define_errors! macro
    define_errors! {
        category: Config,
        base: 1000,

        errors: {
            TEST_ERROR_1 = 1 => {
                message: "Test error 1: {detail}",
                hint: "This is a hint",
                severity: Error,
                doc: "https://example.com/errors#1001",
            },

            TEST_ERROR_2 = 2 => {
                message: "Test error 2",
                severity: Warning,
            },

            TEST_ERROR_3 = 3 => {
                message: "Test error 3: {code}",
            },
        }
    }

    #[test]
    fn define_errors_generates_constants() {
        assert_eq!(TEST_ERROR_1.code().number(), 1001);
        assert_eq!(TEST_ERROR_1.message_template(), "Test error 1: {detail}");
        assert_eq!(TEST_ERROR_1.hint(), Some("This is a hint"));
        assert_eq!(TEST_ERROR_1.severity(), ErrorSeverity::Error);
        assert_eq!(
            TEST_ERROR_1.doc_link(),
            Some("https://example.com/errors#1001")
        );

        assert_eq!(TEST_ERROR_2.code().number(), 1002);
        assert_eq!(TEST_ERROR_2.severity(), ErrorSeverity::Warning);
        assert_eq!(TEST_ERROR_2.hint(), None);

        assert_eq!(TEST_ERROR_3.code().number(), 1003);
        assert_eq!(TEST_ERROR_3.severity(), ErrorSeverity::Error); // Default
    }

    #[test]
    fn define_errors_correct_category() {
        assert_eq!(TEST_ERROR_1.code().category(), ErrorCategory::Config);
        assert_eq!(TEST_ERROR_2.code().category(), ErrorCategory::Config);
    }

    #[test]
    fn keyrx_err_simple() {
        let err = keyrx_err!(TEST_ERROR_1);
        assert_eq!(err.code(), "KRX-C1001");
        assert_eq!(err.message(), "Test error 1: {detail}"); // No substitution
    }

    #[test]
    fn keyrx_err_with_context() {
        let err = keyrx_err!(TEST_ERROR_1, detail = "something went wrong");
        assert_eq!(err.code(), "KRX-C1001");
        assert_eq!(err.message(), "Test error 1: something went wrong");
    }

    #[test]
    fn keyrx_err_with_multiple_context() {
        let err = keyrx_err!(TEST_ERROR_3, code = "42");
        assert_eq!(err.message(), "Test error 3: 42");
    }

    #[test]
    fn bail_keyrx_returns_error() {
        fn test_bail() -> Result<(), crate::errors::KeyrxError> {
            bail_keyrx!(TEST_ERROR_1, detail = "test");
        }

        let result = test_bail();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code(), "KRX-C1001");
    }

    // This test would fail at compile time if uncommented (duplicate detection)
    // define_errors! {
    //     category: Config,
    //     base: 2000,
    //
    //     errors: {
    //         DUP_ERROR_1 = 1 => { message: "Error 1" },
    //         DUP_ERROR_2 = 1 => { message: "Error 2" }, // Same number!
    //     }
    // }
}
