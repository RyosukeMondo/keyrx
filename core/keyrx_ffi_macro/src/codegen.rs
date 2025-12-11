//! Code generation for FFI wrapper functions.
//!
//! This module generates Rust code for FFI functions based on contract definitions.
//! It handles parameter parsing, result serialization, and full function generation.

// Allow dead_code until Task 14 integrates this module
#![allow(dead_code)]

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::contract_loader::ParameterContract;
use crate::type_mapper::{map_param_type, FfiType};

/// Parsed parameter ready for code generation.
#[derive(Debug)]
pub struct ParsedParam {
    /// The parameter name as an identifier
    pub ident: Ident,
    /// The FFI type of the parameter
    pub ffi_type: FfiType,
    /// The contract type name
    pub contract_type: String,
}

impl ParsedParam {
    /// Create a parsed parameter from a contract parameter.
    pub fn from_contract(param: &ParameterContract) -> Self {
        Self {
            ident: Ident::new(&param.name, Span::call_site()),
            ffi_type: map_param_type(&param.param_type),
            contract_type: param.param_type.clone(),
        }
    }
}

/// Generate FFI parameter declarations for a function signature.
///
/// Creates the parameter list for an `extern "C"` function, starting with
/// the error pointer.
///
/// # Arguments
///
/// * `params` - The parsed parameters from the contract
///
/// # Returns
///
/// A `TokenStream` representing the function parameter declarations.
pub fn generate_param_declarations(params: &[ParsedParam]) -> TokenStream {
    let param_decls: Vec<TokenStream> = params
        .iter()
        .map(|p| {
            let ident = &p.ident;
            let ty = p.ffi_type.to_param_tokens();
            quote! { #ident: #ty }
        })
        .collect();

    if param_decls.is_empty() {
        quote! {
            error: *mut *mut ::std::os::raw::c_char
        }
    } else {
        quote! {
            error: *mut *mut ::std::os::raw::c_char,
            #(#param_decls),*
        }
    }
}

/// Generate code for parsing a single FFI parameter.
///
/// Generates the parsing logic for one parameter based on its FFI type.
/// Uses the runtime helpers from `keyrx_ffi_runtime`.
///
/// # Arguments
///
/// * `param` - The parsed parameter
///
/// # Returns
///
/// A `TokenStream` with the parsing code for this parameter.
pub fn generate_param_parser(param: &ParsedParam) -> TokenStream {
    let ident = &param.ident;
    let name_str = ident.to_string();

    match param.ffi_type {
        FfiType::CString => {
            // String parameters need to be parsed from C strings
            quote! {
                let #ident = unsafe {
                    ::keyrx_ffi_runtime::parse_c_string(#ident, #name_str)?
                };
            }
        }
        FfiType::Int32 => {
            // i32 can be used directly - no parsing needed
            quote! {
                let #ident = #ident;
            }
        }
        FfiType::Uint8 => {
            // u8 can be used directly
            quote! {
                let #ident = #ident;
            }
        }
        FfiType::Uint32 => {
            // u32 can be used directly
            quote! {
                let #ident = #ident;
            }
        }
        FfiType::Uint64 => {
            // u64 can be used directly
            quote! {
                let #ident = #ident;
            }
        }
        FfiType::Float64 => {
            // f64 can be used directly
            quote! {
                let #ident = #ident;
            }
        }
        FfiType::Bool => {
            // bool can be used directly
            quote! {
                let #ident = #ident;
            }
        }
        FfiType::Void | FfiType::JsonReturn => {
            // These types are not used as parameters
            quote! {}
        }
    }
}

/// Generate parsing code for all parameters of a function.
///
/// # Arguments
///
/// * `params` - All parsed parameters for the function
///
/// # Returns
///
/// A `TokenStream` containing the combined parsing logic.
pub fn generate_all_param_parsers(params: &[ParsedParam]) -> TokenStream {
    let parsers: Vec<TokenStream> = params.iter().map(generate_param_parser).collect();

    quote! {
        #(#parsers)*
    }
}

/// Generate the list of parameter identifiers for a function call.
///
/// # Arguments
///
/// * `params` - The parsed parameters
///
/// # Returns
///
/// A `TokenStream` containing comma-separated identifiers.
pub fn generate_call_args(params: &[ParsedParam]) -> TokenStream {
    let idents: Vec<&Ident> = params.iter().map(|p| &p.ident).collect();

    if idents.is_empty() {
        quote! {}
    } else {
        quote! { #(#idents),* }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract_loader::ParameterContract;

    fn make_param(name: &str, param_type: &str) -> ParameterContract {
        ParameterContract {
            name: name.to_string(),
            param_type: param_type.to_string(),
            description: "test".to_string(),
            required: true,
        }
    }

    #[test]
    fn parsed_param_from_contract() {
        let contract_param = make_param("my_string", "string");
        let parsed = ParsedParam::from_contract(&contract_param);

        assert_eq!(parsed.ident.to_string(), "my_string");
        assert_eq!(parsed.ffi_type, FfiType::CString);
        assert_eq!(parsed.contract_type, "string");
    }

    #[test]
    fn param_declarations_empty() {
        let decls = generate_param_declarations(&[]);
        let output = decls.to_string();
        assert!(output.contains("error"));
        assert!(output.contains("c_char"));
    }

    #[test]
    fn param_declarations_with_params() {
        let params = vec![
            ParsedParam::from_contract(&make_param("key", "string")),
            ParsedParam::from_contract(&make_param("count", "int32")),
        ];
        let decls = generate_param_declarations(&params);
        let output = decls.to_string();
        assert!(output.contains("error"));
        assert!(output.contains("key"));
        assert!(output.contains("count"));
        assert!(output.contains("i32"));
    }

    #[test]
    fn param_parser_string() {
        let param = ParsedParam::from_contract(&make_param("name", "string"));
        let parser = generate_param_parser(&param);
        let output = parser.to_string();
        assert!(output.contains("parse_c_string"));
        assert!(output.contains("name"));
    }

    #[test]
    fn param_parser_int() {
        let param = ParsedParam::from_contract(&make_param("count", "int32"));
        let parser = generate_param_parser(&param);
        let output = parser.to_string();
        assert!(output.contains("count"));
    }

    #[test]
    fn param_parser_bool() {
        let param = ParsedParam::from_contract(&make_param("enabled", "bool"));
        let parser = generate_param_parser(&param);
        let output = parser.to_string();
        assert!(output.contains("enabled"));
    }

    #[test]
    fn all_param_parsers() {
        let params = vec![
            ParsedParam::from_contract(&make_param("key", "string")),
            ParsedParam::from_contract(&make_param("value", "string")),
        ];
        let parsers = generate_all_param_parsers(&params);
        let output = parsers.to_string();
        assert!(output.contains("key"));
        assert!(output.contains("value"));
        assert!(output.matches("parse_c_string").count() == 2);
    }

    #[test]
    fn call_args_empty() {
        let args = generate_call_args(&[]);
        assert!(args.is_empty());
    }

    #[test]
    fn call_args_with_params() {
        let params = vec![
            ParsedParam::from_contract(&make_param("a", "string")),
            ParsedParam::from_contract(&make_param("b", "int32")),
        ];
        let args = generate_call_args(&params);
        let output = args.to_string();
        assert!(output.contains("a"));
        assert!(output.contains("b"));
    }
}
