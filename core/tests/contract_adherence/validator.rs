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

use super::parser::{ParsedFunction, ParsedParam, ParsedType};
use super::type_mapper::{is_error_pointer_str, map_contract_to_rust, validate_type_match};
use keyrx_core::ffi::contract::FunctionContract;

/// Validates that a parsed Rust function matches its contract definition.
///
/// Checks parameter count (accounting for error pointer), parameter types,
/// and return type. Returns the first validation error found.
pub fn validate_function(
    contract: &FunctionContract,
    parsed: &ParsedFunction,
) -> Result<(), ValidationError> {
    let location = FileLocation::new(parsed.file_path.clone(), parsed.line_number);

    // Get contract parameters and parsed parameters
    let contract_params = &contract.parameters;

    // Count non-error-pointer params in parsed function
    let parsed_params: Vec<&ParsedParam> = parsed
        .params
        .iter()
        .filter(|p| !is_error_pointer_str(&p.rust_type))
        .collect();

    // Check parameter count (contract params should match non-error params)
    if contract_params.len() != parsed_params.len() {
        return Err(ValidationError::ParameterCountMismatch {
            function: parsed.name.clone(),
            expected: contract_params.len(),
            found: parsed_params.len(),
            location,
        });
    }

    // Validate each parameter type
    for (i, (contract_param, parsed_param)) in
        contract_params.iter().zip(parsed_params.iter()).enumerate()
    {
        let parsed_type = param_to_parsed_type(parsed_param);
        if let Err(mismatch) = validate_type_match(&contract_param.param_type, &parsed_type) {
            return Err(ValidationError::ParameterTypeMismatch {
                function: parsed.name.clone(),
                param_name: contract_param.name.clone(),
                param_index: i,
                expected_type: map_contract_to_rust(&contract_param.param_type).to_display_string(),
                found_type: mismatch.found,
                location,
            });
        }
    }

    // Validate return type
    let contract_return_type = contract.returns.type_name();
    if let Err(mismatch) = validate_type_match(contract_return_type, &parsed.return_type) {
        return Err(ValidationError::ReturnTypeMismatch {
            function: parsed.name.clone(),
            expected_type: map_contract_to_rust(contract_return_type).to_display_string(),
            found_type: mismatch.found,
            location,
        });
    }

    // Validate error pointer exists (all FFI functions should have one as last param)
    if !parsed.has_error_pointer() && !parsed.params.is_empty() {
        // Check if the last parameter looks like an error pointer
        if let Some(last_param) = parsed.params.last() {
            if !is_error_pointer_str(&last_param.rust_type) {
                return Err(ValidationError::MissingErrorPointer {
                    function: parsed.name.clone(),
                    location,
                });
            }
        }
    } else if parsed.params.is_empty() && !contract_params.is_empty() {
        // If parsed has no params but contract has params, error pointer is missing
        return Err(ValidationError::MissingErrorPointer {
            function: parsed.name.clone(),
            location,
        });
    }

    Ok(())
}

/// Convert a ParsedParam to a ParsedType for type validation.
fn param_to_parsed_type(param: &ParsedParam) -> ParsedType {
    if !param.is_pointer {
        return ParsedType::Primitive(param.rust_type.clone());
    }

    // Parse pointer type from rust_type string
    if param.rust_type.starts_with("*mut ") {
        let target = param
            .rust_type
            .strip_prefix("*mut ")
            .unwrap_or("")
            .to_string();
        ParsedType::Pointer {
            target,
            is_mut: true,
        }
    } else if param.rust_type.starts_with("*const ") {
        let target = param
            .rust_type
            .strip_prefix("*const ")
            .unwrap_or("")
            .to_string();
        ParsedType::Pointer {
            target,
            is_mut: false,
        }
    } else {
        ParsedType::Primitive(param.rust_type.clone())
    }
}

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

    // Tests for validate_function
    use keyrx_core::ffi::contract::{ParameterContract, TypeDefinition};

    fn make_contract(name: &str, params: Vec<(&str, &str)>, return_type: &str) -> FunctionContract {
        FunctionContract {
            name: name.to_string(),
            description: "Test function".to_string(),
            rust_name: Some(name.to_string()),
            parameters: params
                .into_iter()
                .map(|(n, t)| ParameterContract {
                    name: n.to_string(),
                    param_type: t.to_string(),
                    description: "Test param".to_string(),
                    required: true,
                    constraints: None,
                })
                .collect(),
            returns: TypeDefinition::Primitive {
                type_name: return_type.to_string(),
                description: None,
                constraints: None,
            },
            errors: vec![],
            events_emitted: vec![],
            example: None,
            deprecated: false,
            since_version: None,
        }
    }

    fn make_parsed_fn(
        name: &str,
        params: Vec<(&str, &str, bool, bool)>,
        return_type: ParsedType,
    ) -> ParsedFunction {
        ParsedFunction::new(
            name.to_string(),
            params
                .into_iter()
                .map(|(n, t, is_ptr, is_mut)| {
                    ParsedParam::new(n.to_string(), t.to_string(), is_ptr, is_mut)
                })
                .collect(),
            return_type,
            PathBuf::from("test.rs"),
            10,
        )
    }

    #[test]
    fn test_validate_function_matching_signatures() {
        let contract = make_contract("keyrx_test", vec![("input", "string")], "int");

        let parsed = make_parsed_fn(
            "keyrx_test",
            vec![
                ("input", "*const c_char", true, false),
                ("error_out", "*mut *mut c_char", true, true),
            ],
            ParsedType::Primitive("i32".to_string()),
        );

        let result = validate_function(&contract, &parsed);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[test]
    fn test_validate_function_parameter_count_mismatch() {
        let contract = make_contract("keyrx_test", vec![("a", "string"), ("b", "int")], "void");

        let parsed = make_parsed_fn(
            "keyrx_test",
            vec![
                ("a", "*const c_char", true, false),
                ("error_out", "*mut *mut c_char", true, true),
            ],
            ParsedType::Unit,
        );

        let result = validate_function(&contract, &parsed);
        assert!(matches!(
            result,
            Err(ValidationError::ParameterCountMismatch {
                expected: 2,
                found: 1,
                ..
            })
        ));
    }

    #[test]
    fn test_validate_function_parameter_type_mismatch() {
        let contract = make_contract("keyrx_test", vec![("input", "string")], "void");

        let parsed = make_parsed_fn(
            "keyrx_test",
            vec![
                ("input", "i32", false, false), // Should be *const c_char
                ("error_out", "*mut *mut c_char", true, true),
            ],
            ParsedType::Unit,
        );

        let result = validate_function(&contract, &parsed);
        assert!(matches!(
            result,
            Err(ValidationError::ParameterTypeMismatch { param_index: 0, .. })
        ));
    }

    #[test]
    fn test_validate_function_return_type_mismatch() {
        let contract = make_contract("keyrx_test", vec![], "int");

        let parsed = make_parsed_fn(
            "keyrx_test",
            vec![("error_out", "*mut *mut c_char", true, true)],
            ParsedType::Unit, // Should be i32
        );

        let result = validate_function(&contract, &parsed);
        assert!(matches!(
            result,
            Err(ValidationError::ReturnTypeMismatch { .. })
        ));
    }

    #[test]
    fn test_validate_function_missing_error_pointer() {
        let contract = make_contract("keyrx_test", vec![("input", "string")], "void");

        let parsed = make_parsed_fn(
            "keyrx_test",
            vec![("input", "*const c_char", true, false)], // No error pointer
            ParsedType::Unit,
        );

        let result = validate_function(&contract, &parsed);
        assert!(matches!(
            result,
            Err(ValidationError::MissingErrorPointer { .. })
        ));
    }

    #[test]
    fn test_validate_function_void_return() {
        let contract = make_contract("keyrx_test", vec![], "void");

        let parsed = make_parsed_fn(
            "keyrx_test",
            vec![("error_out", "*mut *mut c_char", true, true)],
            ParsedType::Unit,
        );

        let result = validate_function(&contract, &parsed);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_function_complex_params() {
        let contract = make_contract(
            "keyrx_test",
            vec![("config", "object"), ("count", "int"), ("enabled", "bool")],
            "string",
        );

        let parsed = make_parsed_fn(
            "keyrx_test",
            vec![
                ("config", "*const c_char", true, false),
                ("count", "i32", false, false),
                ("enabled", "bool", false, false),
                ("error_out", "*mut *mut c_char", true, true),
            ],
            ParsedType::Pointer {
                target: "c_char".to_string(),
                is_mut: false,
            },
        );

        let result = validate_function(&contract, &parsed);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[test]
    fn test_param_to_parsed_type_primitive() {
        let param = ParsedParam::new("x".to_string(), "i32".to_string(), false, false);
        let parsed = param_to_parsed_type(&param);
        assert_eq!(parsed, ParsedType::Primitive("i32".to_string()));
    }

    #[test]
    fn test_param_to_parsed_type_const_pointer() {
        let param = ParsedParam::new("s".to_string(), "*const c_char".to_string(), true, false);
        let parsed = param_to_parsed_type(&param);
        assert_eq!(
            parsed,
            ParsedType::Pointer {
                target: "c_char".to_string(),
                is_mut: false
            }
        );
    }

    #[test]
    fn test_param_to_parsed_type_mut_pointer() {
        let param = ParsedParam::new("out".to_string(), "*mut c_char".to_string(), true, true);
        let parsed = param_to_parsed_type(&param);
        assert_eq!(
            parsed,
            ParsedType::Pointer {
                target: "c_char".to_string(),
                is_mut: true
            }
        );
    }
}
