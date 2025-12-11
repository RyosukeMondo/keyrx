//! Signature Validator for FFI Contract Adherence
//!
//! This module provides validation error types and logic for comparing
//! FFI contract definitions against parsed Rust function signatures.

use std::path::PathBuf;

/// Represents a location in a source file for error reporting.
#[derive(Debug, Clone, PartialEq)]
pub struct FileLocation {
    /// Path to the source file
    pub file: PathBuf,
    /// Line number (1-indexed)
    pub line: usize,
}

impl FileLocation {
    /// Creates a new FileLocation.
    pub fn new(file: PathBuf, line: usize) -> Self {
        Self { file, line }
    }
}

impl std::fmt::Display for FileLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file.display(), self.line)
    }
}

/// Validation errors for FFI contract adherence checking.
///
/// Each variant contains rich context for actionable error messages,
/// including file locations, expected vs found values, and fix suggestions.
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// A function defined in the contract is not found in the Rust source.
    MissingFunction {
        /// Function name from the contract
        name: String,
        /// Path to the contract file
        contract_file: String,
    },

    /// Parameter count differs between contract and implementation.
    ParameterCountMismatch {
        /// Function name
        function: String,
        /// Expected parameter count from contract
        expected: usize,
        /// Found parameter count in implementation
        found: usize,
        /// Location in source file
        location: FileLocation,
    },

    /// Parameter type differs between contract and implementation.
    ParameterTypeMismatch {
        /// Function name
        function: String,
        /// Parameter name
        param_name: String,
        /// Parameter index (0-indexed)
        param_index: usize,
        /// Expected type from contract
        expected_type: String,
        /// Found type in implementation
        found_type: String,
        /// Location in source file
        location: FileLocation,
    },

    /// Return type differs between contract and implementation.
    ReturnTypeMismatch {
        /// Function name
        function: String,
        /// Expected return type from contract
        expected_type: String,
        /// Found return type in implementation
        found_type: String,
        /// Location in source file
        location: FileLocation,
    },

    /// A function exists in the Rust source but has no contract definition.
    UncontractedFunction {
        /// Function name
        name: String,
        /// Location in source file
        location: FileLocation,
    },

    /// Missing error pointer parameter (should be last parameter).
    MissingErrorPointer {
        /// Function name
        function: String,
        /// Location in source file
        location: FileLocation,
    },

    /// Invalid error pointer type (should be *mut *mut c_char).
    InvalidErrorPointer {
        /// Function name
        function: String,
        /// Found type for error pointer
        found_type: String,
        /// Location in source file
        location: FileLocation,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingFunction {
                name,
                contract_file,
            } => {
                write!(
                    f,
                    "Missing function '{}' (defined in contract: {})",
                    name, contract_file
                )
            }

            ValidationError::ParameterCountMismatch {
                function,
                expected,
                found,
                location,
            } => {
                write!(
                    f,
                    "Parameter count mismatch in '{}' at {}: expected {}, found {}",
                    function, location, expected, found
                )
            }

            ValidationError::ParameterTypeMismatch {
                function,
                param_name,
                param_index,
                expected_type,
                found_type,
                location,
            } => {
                write!(
                    f,
                    "Type mismatch for parameter '{}' (index {}) in '{}' at {}: \
                     expected '{}', found '{}'",
                    param_name, param_index, function, location, expected_type, found_type
                )
            }

            ValidationError::ReturnTypeMismatch {
                function,
                expected_type,
                found_type,
                location,
            } => {
                write!(
                    f,
                    "Return type mismatch in '{}' at {}: expected '{}', found '{}'",
                    function, location, expected_type, found_type
                )
            }

            ValidationError::UncontractedFunction { name, location } => {
                write!(
                    f,
                    "Uncontracted FFI function '{}' at {} (no contract definition)",
                    name, location
                )
            }

            ValidationError::MissingErrorPointer { function, location } => {
                write!(
                    f,
                    "Missing error pointer parameter in '{}' at {} \
                     (expected *mut *mut c_char as last parameter)",
                    function, location
                )
            }

            ValidationError::InvalidErrorPointer {
                function,
                found_type,
                location,
            } => {
                write!(
                    f,
                    "Invalid error pointer type in '{}' at {}: \
                     expected '*mut *mut c_char', found '{}'",
                    function, location, found_type
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

impl ValidationError {
    /// Returns a suggested fix for the error.
    pub fn fix_suggestion(&self) -> String {
        match self {
            ValidationError::MissingFunction { name, .. } => {
                format!(
                    "Implement the function '{}' with #[no_mangle] pub extern \"C\" fn",
                    name
                )
            }

            ValidationError::ParameterCountMismatch {
                expected, found, ..
            } => {
                if *expected > *found {
                    format!(
                        "Add {} missing parameter(s) to match the contract",
                        expected - found
                    )
                } else {
                    format!(
                        "Remove {} extra parameter(s) to match the contract",
                        found - expected
                    )
                }
            }

            ValidationError::ParameterTypeMismatch {
                param_name,
                expected_type,
                ..
            } => {
                format!(
                    "Change type of parameter '{}' to '{}'",
                    param_name, expected_type
                )
            }

            ValidationError::ReturnTypeMismatch { expected_type, .. } => {
                format!("Change return type to '{}'", expected_type)
            }

            ValidationError::UncontractedFunction { name, .. } => {
                format!(
                    "Add a contract definition for '{}' or remove the function if unused",
                    name
                )
            }

            ValidationError::MissingErrorPointer { .. } => {
                "Add 'error_out: *mut *mut c_char' as the last parameter".to_string()
            }

            ValidationError::InvalidErrorPointer { .. } => {
                "Change the error pointer parameter type to '*mut *mut c_char'".to_string()
            }
        }
    }

    /// Returns the function name associated with this error.
    pub fn function_name(&self) -> &str {
        match self {
            ValidationError::MissingFunction { name, .. } => name,
            ValidationError::ParameterCountMismatch { function, .. } => function,
            ValidationError::ParameterTypeMismatch { function, .. } => function,
            ValidationError::ReturnTypeMismatch { function, .. } => function,
            ValidationError::UncontractedFunction { name, .. } => name,
            ValidationError::MissingErrorPointer { function, .. } => function,
            ValidationError::InvalidErrorPointer { function, .. } => function,
        }
    }

    /// Returns the file location if available.
    pub fn location(&self) -> Option<&FileLocation> {
        match self {
            ValidationError::MissingFunction { .. } => None,
            ValidationError::ParameterCountMismatch { location, .. } => Some(location),
            ValidationError::ParameterTypeMismatch { location, .. } => Some(location),
            ValidationError::ReturnTypeMismatch { location, .. } => Some(location),
            ValidationError::UncontractedFunction { location, .. } => Some(location),
            ValidationError::MissingErrorPointer { location, .. } => Some(location),
            ValidationError::InvalidErrorPointer { location, .. } => Some(location),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_location_display() {
        let loc = FileLocation::new(PathBuf::from("src/lib.rs"), 42);
        assert_eq!(loc.to_string(), "src/lib.rs:42");
    }

    #[test]
    fn test_missing_function_error() {
        let err = ValidationError::MissingFunction {
            name: "keyrx_init".to_string(),
            contract_file: "engine.ffi-contract.json".to_string(),
        };
        assert!(err.to_string().contains("keyrx_init"));
        assert!(err.to_string().contains("engine.ffi-contract.json"));
        assert_eq!(err.function_name(), "keyrx_init");
        assert!(err.location().is_none());
    }

    #[test]
    fn test_parameter_count_mismatch_error() {
        let loc = FileLocation::new(PathBuf::from("exports.rs"), 100);
        let err = ValidationError::ParameterCountMismatch {
            function: "keyrx_test".to_string(),
            expected: 3,
            found: 2,
            location: loc.clone(),
        };
        assert!(err.to_string().contains("expected 3"));
        assert!(err.to_string().contains("found 2"));
        assert_eq!(err.location(), Some(&loc));
    }

    #[test]
    fn test_parameter_type_mismatch_error() {
        let loc = FileLocation::new(PathBuf::from("exports.rs"), 50);
        let err = ValidationError::ParameterTypeMismatch {
            function: "keyrx_test".to_string(),
            param_name: "input".to_string(),
            param_index: 0,
            expected_type: "*const c_char".to_string(),
            found_type: "i32".to_string(),
            location: loc,
        };
        assert!(err.to_string().contains("input"));
        assert!(err.to_string().contains("*const c_char"));
        assert!(err.to_string().contains("i32"));
    }

    #[test]
    fn test_return_type_mismatch_error() {
        let loc = FileLocation::new(PathBuf::from("exports.rs"), 75);
        let err = ValidationError::ReturnTypeMismatch {
            function: "keyrx_get_value".to_string(),
            expected_type: "*const c_char".to_string(),
            found_type: "()".to_string(),
            location: loc,
        };
        assert!(err.to_string().contains("Return type mismatch"));
        assert!(err.to_string().contains("*const c_char"));
    }

    #[test]
    fn test_uncontracted_function_error() {
        let loc = FileLocation::new(PathBuf::from("exports.rs"), 200);
        let err = ValidationError::UncontractedFunction {
            name: "keyrx_orphan".to_string(),
            location: loc,
        };
        assert!(err.to_string().contains("Uncontracted"));
        assert!(err.to_string().contains("keyrx_orphan"));
    }

    #[test]
    fn test_missing_error_pointer() {
        let loc = FileLocation::new(PathBuf::from("exports.rs"), 30);
        let err = ValidationError::MissingErrorPointer {
            function: "keyrx_no_error".to_string(),
            location: loc,
        };
        assert!(err.to_string().contains("Missing error pointer"));
    }

    #[test]
    fn test_invalid_error_pointer() {
        let loc = FileLocation::new(PathBuf::from("exports.rs"), 60);
        let err = ValidationError::InvalidErrorPointer {
            function: "keyrx_bad_error".to_string(),
            found_type: "*mut c_char".to_string(),
            location: loc,
        };
        assert!(err.to_string().contains("Invalid error pointer"));
        assert!(err.to_string().contains("*mut c_char"));
    }

    #[test]
    fn test_fix_suggestions() {
        let err = ValidationError::MissingFunction {
            name: "keyrx_init".to_string(),
            contract_file: "engine.json".to_string(),
        };
        assert!(err.fix_suggestion().contains("Implement"));

        let loc = FileLocation::new(PathBuf::from("test.rs"), 1);
        let err = ValidationError::ParameterCountMismatch {
            function: "test".to_string(),
            expected: 5,
            found: 3,
            location: loc.clone(),
        };
        assert!(err.fix_suggestion().contains("Add 2 missing"));

        let err = ValidationError::ParameterCountMismatch {
            function: "test".to_string(),
            expected: 2,
            found: 4,
            location: loc,
        };
        assert!(err.fix_suggestion().contains("Remove 2 extra"));
    }
}
