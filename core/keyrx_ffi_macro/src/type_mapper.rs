//! Type mapping for FFI code generation.
//!
//! This module converts contract type names to Rust FFI type tokens.
//! It handles the mapping from JSON contract types to proper Rust FFI types.

// Allow dead_code until Task 11-13 integrates this module
#![allow(dead_code)]

use proc_macro2::TokenStream;
use quote::quote;

use crate::contract_loader::TypeDefinition;

/// FFI type representation for code generation.
#[derive(Debug, Clone, PartialEq)]
pub enum FfiType {
    /// `*const c_char` - string input parameter
    CString,
    /// `*const c_char` - JSON serialized return value
    JsonReturn,
    /// `i32` - signed 32-bit integer
    Int32,
    /// `u8` - unsigned 8-bit integer
    Uint8,
    /// `u32` - unsigned 32-bit integer
    Uint32,
    /// `u64` - unsigned 64-bit integer
    Uint64,
    /// `f64` - 64-bit float
    Float64,
    /// `bool` - boolean
    Bool,
    /// `()` - void return type
    Void,
}

impl FfiType {
    /// Generate FFI parameter type tokens.
    pub fn to_param_tokens(&self) -> TokenStream {
        match self {
            FfiType::CString => quote! { *const ::std::os::raw::c_char },
            FfiType::Int32 => quote! { i32 },
            FfiType::Uint8 => quote! { u8 },
            FfiType::Uint32 => quote! { u32 },
            FfiType::Uint64 => quote! { u64 },
            FfiType::Float64 => quote! { f64 },
            FfiType::Bool => quote! { bool },
            FfiType::Void | FfiType::JsonReturn => quote! { () },
        }
    }

    /// Generate FFI return type tokens.
    pub fn to_return_tokens(&self) -> TokenStream {
        match self {
            FfiType::Void => quote! { () },
            FfiType::CString | FfiType::JsonReturn => {
                quote! { *const ::std::os::raw::c_char }
            }
            FfiType::Int32 => quote! { i32 },
            FfiType::Uint8 => quote! { u8 },
            FfiType::Uint32 => quote! { u32 },
            FfiType::Uint64 => quote! { u64 },
            FfiType::Float64 => quote! { f64 },
            FfiType::Bool => quote! { bool },
        }
    }

    /// Whether this type requires JSON serialization for return values.
    pub fn needs_json_serialization(&self) -> bool {
        matches!(self, FfiType::JsonReturn)
    }

    /// Whether this type is a C string parameter requiring parsing.
    pub fn needs_string_parsing(&self) -> bool {
        matches!(self, FfiType::CString)
    }
}

/// Map a contract parameter type name to FFI type.
///
/// # Arguments
///
/// * `type_name` - The type name from the contract (e.g., "string", "int32")
///
/// # Returns
///
/// The corresponding `FfiType` for FFI code generation.
pub fn map_param_type(type_name: &str) -> FfiType {
    match type_name {
        "string" => FfiType::CString,
        "int32" | "int" => FfiType::Int32,
        "uint8" => FfiType::Uint8,
        "uint32" => FfiType::Uint32,
        "uint64" => FfiType::Uint64,
        "float64" | "float" | "double" => FfiType::Float64,
        "bool" | "boolean" => FfiType::Bool,
        // Complex types are passed as JSON strings
        "object" | "array" => FfiType::CString,
        // Unknown types default to JSON string for flexibility
        _ => FfiType::CString,
    }
}

/// Map a contract return type to FFI type.
///
/// # Arguments
///
/// * `type_def` - The type definition from the contract
///
/// # Returns
///
/// The corresponding `FfiType` for FFI return values.
pub fn map_return_type(type_def: &TypeDefinition) -> FfiType {
    let type_name = type_def.type_name();
    match type_name {
        "void" | "unit" | "()" => FfiType::Void,
        "string" => FfiType::CString,
        "int32" | "int" => FfiType::Int32,
        "uint8" => FfiType::Uint8,
        "uint32" => FfiType::Uint32,
        "uint64" => FfiType::Uint64,
        "float64" | "float" | "double" => FfiType::Float64,
        "bool" | "boolean" => FfiType::Bool,
        // Complex types (object, array) are serialized to JSON
        "object" | "array" => FfiType::JsonReturn,
        // Unknown types are serialized to JSON
        _ => FfiType::JsonReturn,
    }
}

/// Generate FFI parameter type tokens from a contract type name.
///
/// This is the main entry point for parameter type mapping in code generation.
///
/// # Arguments
///
/// * `contract_type` - The type name from the contract
///
/// # Returns
///
/// A `TokenStream` representing the FFI parameter type.
pub fn map_contract_type_to_ffi(contract_type: &str) -> TokenStream {
    map_param_type(contract_type).to_param_tokens()
}

/// Generate FFI return type tokens from a type definition.
///
/// # Arguments
///
/// * `type_def` - The type definition from the contract
///
/// # Returns
///
/// A `TokenStream` representing the FFI return type.
pub fn map_return_type_to_ffi(type_def: &TypeDefinition) -> TokenStream {
    map_return_type(type_def).to_return_tokens()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_string_type() {
        assert_eq!(map_param_type("string"), FfiType::CString);
        let tokens = map_contract_type_to_ffi("string");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn map_integer_types() {
        assert_eq!(map_param_type("int32"), FfiType::Int32);
        assert_eq!(map_param_type("int"), FfiType::Int32);
        assert_eq!(map_param_type("uint8"), FfiType::Uint8);
        assert_eq!(map_param_type("uint32"), FfiType::Uint32);
        assert_eq!(map_param_type("uint64"), FfiType::Uint64);
    }

    #[test]
    fn map_float_types() {
        assert_eq!(map_param_type("float64"), FfiType::Float64);
        assert_eq!(map_param_type("float"), FfiType::Float64);
        assert_eq!(map_param_type("double"), FfiType::Float64);
    }

    #[test]
    fn map_bool_types() {
        assert_eq!(map_param_type("bool"), FfiType::Bool);
        assert_eq!(map_param_type("boolean"), FfiType::Bool);
    }

    #[test]
    fn map_complex_param_types() {
        // Object and array params are passed as JSON strings
        assert_eq!(map_param_type("object"), FfiType::CString);
        assert_eq!(map_param_type("array"), FfiType::CString);
    }

    #[test]
    fn map_unknown_param_defaults_to_string() {
        assert_eq!(map_param_type("custom_type"), FfiType::CString);
    }

    #[test]
    fn map_void_return() {
        let void_type = TypeDefinition::Simple {
            type_name: "void".to_string(),
            description: None,
        };
        assert_eq!(map_return_type(&void_type), FfiType::Void);
    }

    #[test]
    fn map_string_return() {
        let string_type = TypeDefinition::Simple {
            type_name: "string".to_string(),
            description: None,
        };
        assert_eq!(map_return_type(&string_type), FfiType::CString);
    }

    #[test]
    fn map_object_return() {
        use std::collections::HashMap;
        let object_type = TypeDefinition::Object {
            type_name: "object".to_string(),
            description: None,
            properties: HashMap::new(),
        };
        assert_eq!(map_return_type(&object_type), FfiType::JsonReturn);
    }

    #[test]
    fn map_primitive_return_types() {
        let int_type = TypeDefinition::Simple {
            type_name: "int32".to_string(),
            description: None,
        };
        assert_eq!(map_return_type(&int_type), FfiType::Int32);

        let bool_type = TypeDefinition::Simple {
            type_name: "bool".to_string(),
            description: None,
        };
        assert_eq!(map_return_type(&bool_type), FfiType::Bool);
    }

    #[test]
    fn ffi_type_to_param_tokens() {
        let string_tokens = FfiType::CString.to_param_tokens();
        assert!(!string_tokens.is_empty());

        let int_tokens = FfiType::Int32.to_param_tokens();
        assert!(!int_tokens.is_empty());

        let void_tokens = FfiType::Void.to_param_tokens();
        assert!(!void_tokens.is_empty());
    }

    #[test]
    fn ffi_type_to_return_tokens() {
        let string_tokens = FfiType::CString.to_return_tokens();
        assert!(!string_tokens.is_empty());

        let json_tokens = FfiType::JsonReturn.to_return_tokens();
        assert!(!json_tokens.is_empty());

        let void_tokens = FfiType::Void.to_return_tokens();
        assert!(!void_tokens.is_empty());
    }

    #[test]
    fn needs_json_serialization() {
        assert!(!FfiType::CString.needs_json_serialization());
        assert!(FfiType::JsonReturn.needs_json_serialization());
        assert!(!FfiType::Int32.needs_json_serialization());
        assert!(!FfiType::Void.needs_json_serialization());
    }

    #[test]
    fn needs_string_parsing() {
        assert!(FfiType::CString.needs_string_parsing());
        assert!(!FfiType::JsonReturn.needs_string_parsing());
        assert!(!FfiType::Int32.needs_string_parsing());
    }
}
