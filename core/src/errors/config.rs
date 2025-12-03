//! Configuration error definitions.
//!
//! This module defines all errors related to configuration loading,
//! parsing, and validation. Config errors use the KRX-C1xxx range.

use crate::define_errors;

define_errors! {
    category: Config,
    base: 1000,

    errors: {
        CONFIG_NOT_FOUND = 1 => {
            message: "Configuration file not found: {path}",
            hint: "Create a config file at the default location (~/.config/keyrx/config.toml) or specify a valid path",
            severity: Error,
        },

        CONFIG_READ_ERROR = 2 => {
            message: "Failed to read configuration file: {path}",
            hint: "Check file permissions and ensure the file is readable",
            severity: Error,
        },

        CONFIG_PARSE_ERROR = 3 => {
            message: "Failed to parse configuration file: {error}",
            hint: "Verify the TOML syntax is correct. Check for unclosed brackets, quotes, or invalid escape sequences",
            severity: Error,
        },

        CONFIG_INVALID_VALUE = 4 => {
            message: "Invalid value for {field}: {value}",
            hint: "Check the allowed range or format for this field in the documentation",
            severity: Error,
        },

        CONFIG_MISSING_FIELD = 5 => {
            message: "Required configuration field missing: {field}",
            hint: "Add the required field to your config file or use the default value",
            severity: Error,
        },

        CONFIG_INVALID_PATH = 6 => {
            message: "Invalid path in configuration: {path}",
            hint: "Ensure the path exists and is accessible. Use absolute paths or paths relative to the config directory",
            severity: Error,
        },

        CONFIG_INVALID_TYPE = 7 => {
            message: "Invalid type for {field}: expected {expected}, got {actual}",
            hint: "Fix the field type in your config file to match the expected type",
            severity: Error,
        },

        CONFIG_OUT_OF_RANGE = 8 => {
            message: "{field} value {value} is out of valid range ({min}-{max})",
            hint: "Adjust the value to be within the valid range",
            severity: Error,
        },

        CONFIG_PERMISSION_DENIED = 9 => {
            message: "Permission denied accessing configuration file: {path}",
            hint: "Check file permissions. You may need to run with appropriate privileges or change file ownership",
            severity: Error,
        },

        CONFIG_INVALID_FORMAT = 10 => {
            message: "Invalid format for {field}: {reason}",
            hint: "Check the documentation for the correct format of this field",
            severity: Error,
        },

        CONFIG_CIRCULAR_REFERENCE = 11 => {
            message: "Circular reference detected in configuration: {path}",
            hint: "Remove circular dependencies between configuration sections or includes",
            severity: Error,
        },

        CONFIG_INCLUDE_ERROR = 12 => {
            message: "Failed to include configuration file: {path}",
            hint: "Check that the included file exists and is a valid TOML file",
            severity: Error,
        },

        CONFIG_SCHEMA_VERSION_MISMATCH = 13 => {
            message: "Configuration schema version mismatch: found {found}, expected {expected}",
            hint: "Update your configuration file to the current schema version or downgrade KeyRx",
            severity: Error,
        },

        CONFIG_DEPRECATED_FIELD = 14 => {
            message: "Deprecated configuration field: {field}",
            hint: "Replace {field} with {replacement}. The deprecated field will be removed in a future version",
            severity: Warning,
        },

        CONFIG_WRITE_ERROR = 15 => {
            message: "Failed to write configuration file: {path}",
            hint: "Check that you have write permissions for the directory and sufficient disk space",
            severity: Error,
        },

        CONFIG_INVALID_ENCODING = 16 => {
            message: "Invalid file encoding for configuration: {path}",
            hint: "Ensure the configuration file is saved as UTF-8",
            severity: Error,
        },

        CONFIG_EMPTY_FILE = 17 => {
            message: "Configuration file is empty: {path}",
            hint: "Add configuration values to the file or remove it to use defaults",
            severity: Warning,
        },

        CONFIG_UNKNOWN_FIELD = 18 => {
            message: "Unknown configuration field: {field}",
            hint: "Check for typos or refer to the documentation for valid field names",
            severity: Warning,
        },

        CONFIG_VALIDATION_FAILED = 19 => {
            message: "Configuration validation failed: {reason}",
            hint: "Fix the validation errors and try again",
            severity: Error,
        },

        CONFIG_DEFAULT_USED = 20 => {
            message: "Using default value for {field}: {value}",
            hint: "This is informational. Specify an explicit value in your config if needed",
            severity: Info,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorCategory;
    use crate::keyrx_err;

    #[test]
    fn config_error_codes_in_range() {
        assert_eq!(CONFIG_NOT_FOUND.code().number(), 1001);
        assert_eq!(CONFIG_READ_ERROR.code().number(), 1002);
        assert_eq!(CONFIG_DEFAULT_USED.code().number(), 1020);

        // Verify all are in Config category range
        assert!(ErrorCategory::Config.contains(CONFIG_NOT_FOUND.code().number()));
        assert!(ErrorCategory::Config.contains(CONFIG_DEFAULT_USED.code().number()));
    }

    #[test]
    fn config_error_categories() {
        assert_eq!(CONFIG_NOT_FOUND.code().category(), ErrorCategory::Config);
        assert_eq!(CONFIG_PARSE_ERROR.code().category(), ErrorCategory::Config);
        assert_eq!(CONFIG_OUT_OF_RANGE.code().category(), ErrorCategory::Config);
    }

    #[test]
    fn config_error_messages() {
        let err = keyrx_err!(CONFIG_NOT_FOUND, path = "/etc/keyrx/config.toml");
        assert_eq!(err.code(), "KRX-C1001");
        assert!(err.message().contains("/etc/keyrx/config.toml"));
    }

    #[test]
    fn config_error_hints() {
        assert!(CONFIG_NOT_FOUND.hint().is_some());
        assert!(CONFIG_PARSE_ERROR.hint().unwrap().contains("TOML"));
        assert!(CONFIG_PERMISSION_DENIED
            .hint()
            .unwrap()
            .contains("permissions"));
    }

    #[test]
    fn config_error_severities() {
        use crate::errors::ErrorSeverity;

        assert_eq!(CONFIG_NOT_FOUND.severity(), ErrorSeverity::Error);
        assert_eq!(CONFIG_DEPRECATED_FIELD.severity(), ErrorSeverity::Warning);
        assert_eq!(CONFIG_DEFAULT_USED.severity(), ErrorSeverity::Info);
    }

    #[test]
    fn config_error_formatting() {
        let err = keyrx_err!(
            CONFIG_OUT_OF_RANGE,
            field = "tap_timeout_ms",
            value = "2000",
            min = "50",
            max = "1000"
        );
        assert!(err.message().contains("tap_timeout_ms"));
        assert!(err.message().contains("2000"));
        assert!(err.message().contains("50"));
        assert!(err.message().contains("1000"));
    }

    #[test]
    fn config_error_context_substitution() {
        let err = keyrx_err!(
            CONFIG_INVALID_TYPE,
            field = "timing.tap_timeout",
            expected = "integer",
            actual = "string"
        );
        assert_eq!(err.code(), "KRX-C1007");
        assert!(err.message().contains("timing.tap_timeout"));
        assert!(err.message().contains("integer"));
        assert!(err.message().contains("string"));
    }
}
