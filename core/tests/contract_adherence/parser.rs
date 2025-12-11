//! AST Parser for FFI Function Signatures
//!
//! This module provides data structures and parsing logic to extract
//! `extern "C"` function signatures from Rust source files using the `syn` crate.

use std::path::PathBuf;

/// Represents a parsed FFI function signature extracted from Rust source code.
#[derive(Debug, Clone)]
pub struct ParsedFunction {
    /// Function name (e.g., "keyrx_engine_start")
    pub name: String,
    /// Parameters with their types
    pub params: Vec<ParsedParam>,
    /// Return type of the function
    pub return_type: ParsedType,
    /// Source file path
    pub file_path: PathBuf,
    /// Line number in the source file
    pub line_number: usize,
}

/// Represents a parsed function parameter.
#[derive(Debug, Clone)]
pub struct ParsedParam {
    /// Parameter name
    pub name: String,
    /// Full Rust type as a string (e.g., "*const c_char")
    pub rust_type: String,
    /// Whether this parameter is a pointer type
    pub is_pointer: bool,
    /// Whether this is a mutable pointer
    pub is_mutable: bool,
}

/// Represents a parsed return type.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedType {
    /// Unit type `()`
    Unit,
    /// Pointer type with target and mutability
    Pointer {
        /// The type being pointed to (e.g., "c_char")
        target: String,
        /// Whether the pointer is mutable
        is_mut: bool,
    },
    /// Primitive type (e.g., i32, bool)
    Primitive(String),
}

impl ParsedType {
    /// Returns a string representation of the type for display purposes.
    pub fn to_type_string(&self) -> String {
        match self {
            ParsedType::Unit => "()".to_string(),
            ParsedType::Pointer { target, is_mut } => {
                if *is_mut {
                    format!("*mut {}", target)
                } else {
                    format!("*const {}", target)
                }
            }
            ParsedType::Primitive(name) => name.clone(),
        }
    }
}

impl ParsedParam {
    /// Creates a new ParsedParam from components.
    pub fn new(name: String, rust_type: String, is_pointer: bool, is_mutable: bool) -> Self {
        Self {
            name,
            rust_type,
            is_pointer,
            is_mutable,
        }
    }
}

impl ParsedFunction {
    /// Creates a new ParsedFunction.
    pub fn new(
        name: String,
        params: Vec<ParsedParam>,
        return_type: ParsedType,
        file_path: PathBuf,
        line_number: usize,
    ) -> Self {
        Self {
            name,
            params,
            return_type,
            file_path,
            line_number,
        }
    }

    /// Returns the number of parameters.
    pub fn param_count(&self) -> usize {
        self.params.len()
    }

    /// Checks if this function has an error pointer as the last parameter.
    pub fn has_error_pointer(&self) -> bool {
        self.params
            .last()
            .map(|p| p.rust_type.contains("*mut *mut"))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_type_unit_display() {
        let t = ParsedType::Unit;
        assert_eq!(t.to_type_string(), "()");
    }

    #[test]
    fn test_parsed_type_const_pointer_display() {
        let t = ParsedType::Pointer {
            target: "c_char".to_string(),
            is_mut: false,
        };
        assert_eq!(t.to_type_string(), "*const c_char");
    }

    #[test]
    fn test_parsed_type_mut_pointer_display() {
        let t = ParsedType::Pointer {
            target: "c_char".to_string(),
            is_mut: true,
        };
        assert_eq!(t.to_type_string(), "*mut c_char");
    }

    #[test]
    fn test_parsed_type_primitive_display() {
        let t = ParsedType::Primitive("i32".to_string());
        assert_eq!(t.to_type_string(), "i32");
    }

    #[test]
    fn test_parsed_param_creation() {
        let param = ParsedParam::new(
            "input".to_string(),
            "*const c_char".to_string(),
            true,
            false,
        );
        assert_eq!(param.name, "input");
        assert!(param.is_pointer);
        assert!(!param.is_mutable);
    }

    #[test]
    fn test_parsed_function_creation() {
        let func = ParsedFunction::new(
            "keyrx_test_fn".to_string(),
            vec![ParsedParam::new(
                "ptr".to_string(),
                "*mut *mut c_char".to_string(),
                true,
                true,
            )],
            ParsedType::Unit,
            PathBuf::from("test.rs"),
            42,
        );
        assert_eq!(func.name, "keyrx_test_fn");
        assert_eq!(func.param_count(), 1);
        assert!(func.has_error_pointer());
    }

    #[test]
    fn test_parsed_function_no_error_pointer() {
        let func = ParsedFunction::new(
            "keyrx_test_fn".to_string(),
            vec![ParsedParam::new(
                "input".to_string(),
                "*const c_char".to_string(),
                true,
                false,
            )],
            ParsedType::Unit,
            PathBuf::from("test.rs"),
            10,
        );
        assert!(!func.has_error_pointer());
    }
}
