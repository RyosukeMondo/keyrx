//! Code generation for FFI wrapper functions.
//!
//! This module generates Rust code for FFI functions based on contract definitions.
//! It handles parameter parsing, result serialization, and full function generation.

// Allow dead_code until Task 14 integrates this module
#![allow(dead_code)]

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::contract_loader::{FfiContract, FunctionContract, ParameterContract};
use crate::type_mapper::{map_param_type, map_return_type, FfiType};

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

/// Generate code for serializing a return value to FFI format.
///
/// Handles different return types:
/// - Void: Returns null pointer
/// - Primitives (int32, bool, etc.): Direct return after wrapping
/// - Strings: Convert to C string
/// - Complex types (objects, arrays): Serialize to JSON C string
///
/// # Arguments
///
/// * `return_type` - The FFI type of the return value
///
/// # Returns
///
/// A `TokenStream` with the serialization code.
pub fn generate_result_serializer(return_type: &FfiType) -> TokenStream {
    match return_type {
        FfiType::Void => {
            // Void returns null pointer
            quote! {
                std::ptr::null()
            }
        }
        FfiType::CString => {
            // String result - allocate C string
            quote! {
                ::keyrx_ffi_runtime::serialize_to_c_string(&result)?
            }
        }
        FfiType::JsonReturn => {
            // Complex types - serialize to JSON
            quote! {
                ::keyrx_ffi_runtime::serialize_to_c_string(&result)?
            }
        }
        FfiType::Int32 | FfiType::Uint8 | FfiType::Uint32 | FfiType::Uint64 | FfiType::Float64 => {
            // Numeric primitives - return directly (wrapped in JSON for consistency)
            quote! {
                ::keyrx_ffi_runtime::serialize_to_c_string(&result)?
            }
        }
        FfiType::Bool => {
            // Boolean - serialize for FFI
            quote! {
                ::keyrx_ffi_runtime::serialize_to_c_string(&result)?
            }
        }
    }
}

/// Generate the FFI return type tokens.
///
/// # Arguments
///
/// * `return_type` - The FFI return type
///
/// # Returns
///
/// A `TokenStream` representing the return type declaration.
pub fn generate_return_type(return_type: &FfiType) -> TokenStream {
    match return_type {
        FfiType::Void => quote! { () },
        _ => quote! { *const ::std::os::raw::c_char },
    }
}

/// Generate a complete FFI function wrapper.
///
/// Creates an `extern "C"` function with `#[no_mangle]` that:
/// - Accepts an error pointer and parameters from the contract
/// - Parses parameters using runtime helpers
/// - Calls the implementation method
/// - Serializes the result and handles errors via `ffi_wrapper`
///
/// # Arguments
///
/// * `func` - The function contract definition
/// * `domain` - The domain name for the FFI function
/// * `impl_method` - The name of the implementation method to call
///
/// # Returns
///
/// A `TokenStream` containing the complete FFI function definition.
pub fn generate_ffi_function(
    func: &FunctionContract,
    domain: &str,
    impl_method: &Ident,
) -> TokenStream {
    let ffi_name = Ident::new(&func.ffi_name(domain), Span::call_site());

    // Parse parameters from contract
    let params: Vec<ParsedParam> = func.parameters.iter().map(ParsedParam::from_contract).collect();

    // Generate parameter declarations
    let param_decls = generate_param_declarations(&params);

    // Generate parameter parsers
    let param_parsers = generate_all_param_parsers(&params);

    // Generate call arguments
    let call_args = generate_call_args(&params);

    // Determine return type
    let return_ffi_type = map_return_type(&func.returns);
    let return_type = generate_return_type(&return_ffi_type);

    // Generate the function body based on whether it returns void
    let body = if matches!(return_ffi_type, FfiType::Void) {
        // Void return - no serialization needed
        quote! {
            unsafe {
                ::keyrx_ffi_runtime::ffi_wrapper(error, || {
                    #param_parsers
                    Self::#impl_method(#call_args)?;
                    Ok(())
                })
            }
        }
    } else {
        // Non-void return - serialize result
        quote! {
            unsafe {
                ::keyrx_ffi_runtime::ffi_wrapper(error, || {
                    #param_parsers
                    let result = Self::#impl_method(#call_args)?;
                    Ok(result)
                })
            }
        }
    };

    // Generate the complete function
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #ffi_name(#param_decls) -> #return_type {
            #body
        }
    }
}

/// Generate FFI functions for all functions in a contract.
///
/// # Arguments
///
/// * `contract` - The FFI contract
///
/// # Returns
///
/// A `TokenStream` containing all generated FFI functions.
pub fn generate_all_ffi_functions(contract: &FfiContract) -> TokenStream {
    let domain = &contract.domain;
    let functions: Vec<TokenStream> = contract
        .functions
        .iter()
        .map(|func| {
            let impl_method = Ident::new(&func.name, Span::call_site());
            generate_ffi_function(func, domain, &impl_method)
        })
        .collect();

    quote! {
        #(#functions)*
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract_loader::{ParameterContract, TypeDefinition};
    use std::collections::HashMap;

    fn make_param(name: &str, param_type: &str) -> ParameterContract {
        ParameterContract {
            name: name.to_string(),
            param_type: param_type.to_string(),
            description: "test".to_string(),
            required: true,
        }
    }

    fn make_function(name: &str, params: Vec<ParameterContract>, return_type: &str) -> FunctionContract {
        FunctionContract {
            name: name.to_string(),
            description: "Test function".to_string(),
            rust_name: None,
            parameters: params,
            returns: TypeDefinition::Simple {
                type_name: return_type.to_string(),
                description: None,
            },
            errors: vec![],
        }
    }

    fn make_contract(domain: &str, functions: Vec<FunctionContract>) -> FfiContract {
        FfiContract {
            version: "1.0.0".to_string(),
            domain: domain.to_string(),
            description: "Test contract".to_string(),
            protocol_version: 1,
            functions,
            types: HashMap::new(),
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

    #[test]
    fn result_serializer_void() {
        let serializer = generate_result_serializer(&FfiType::Void);
        let output = serializer.to_string();
        assert!(output.contains("null"));
    }

    #[test]
    fn result_serializer_string() {
        let serializer = generate_result_serializer(&FfiType::CString);
        let output = serializer.to_string();
        assert!(output.contains("serialize_to_c_string"));
    }

    #[test]
    fn result_serializer_json() {
        let serializer = generate_result_serializer(&FfiType::JsonReturn);
        let output = serializer.to_string();
        assert!(output.contains("serialize_to_c_string"));
    }

    #[test]
    fn result_serializer_primitives() {
        // All numeric types use serialization
        let int_serializer = generate_result_serializer(&FfiType::Int32);
        assert!(int_serializer.to_string().contains("serialize_to_c_string"));

        let uint_serializer = generate_result_serializer(&FfiType::Uint32);
        assert!(uint_serializer.to_string().contains("serialize_to_c_string"));

        let float_serializer = generate_result_serializer(&FfiType::Float64);
        assert!(float_serializer.to_string().contains("serialize_to_c_string"));
    }

    #[test]
    fn result_serializer_bool() {
        let serializer = generate_result_serializer(&FfiType::Bool);
        let output = serializer.to_string();
        assert!(output.contains("serialize_to_c_string"));
    }

    #[test]
    fn return_type_void() {
        let ret = generate_return_type(&FfiType::Void);
        let output = ret.to_string();
        assert!(output.contains("()"));
    }

    #[test]
    fn return_type_non_void() {
        // All non-void types return *const c_char
        let string_ret = generate_return_type(&FfiType::CString);
        assert!(string_ret.to_string().contains("c_char"));

        let json_ret = generate_return_type(&FfiType::JsonReturn);
        assert!(json_ret.to_string().contains("c_char"));

        let int_ret = generate_return_type(&FfiType::Int32);
        assert!(int_ret.to_string().contains("c_char"));
    }

    #[test]
    fn generate_ffi_function_no_params_void_return() {
        let func = make_function("do_something", vec![], "void");
        let impl_method = Ident::new("do_something", Span::call_site());
        let output = generate_ffi_function(&func, "test", &impl_method);
        let code = output.to_string();

        // Check function signature
        assert!(code.contains("no_mangle"));
        assert!(code.contains("pub unsafe extern \"C\""));
        assert!(code.contains("keyrx_test_do_something"));
        assert!(code.contains("error"));

        // Check it uses ffi_wrapper
        assert!(code.contains("ffi_wrapper"));

        // Check void return
        assert!(code.contains("Ok (())"));
    }

    #[test]
    fn generate_ffi_function_with_params() {
        let func = make_function(
            "get_value",
            vec![
                make_param("key", "string"),
                make_param("default", "int32"),
            ],
            "string",
        );
        let impl_method = Ident::new("get_value", Span::call_site());
        let output = generate_ffi_function(&func, "config", &impl_method);
        let code = output.to_string();

        // Check function name follows convention
        assert!(code.contains("keyrx_config_get_value"));

        // Check parameters are declared
        assert!(code.contains("key"));
        assert!(code.contains("default"));

        // Check string parsing is generated
        assert!(code.contains("parse_c_string"));
    }

    #[test]
    fn generate_ffi_function_with_rust_name() {
        let mut func = make_function("list_items", vec![], "object");
        func.rust_name = Some("keyrx_custom_list_items".to_string());
        let impl_method = Ident::new("list_items", Span::call_site());
        let output = generate_ffi_function(&func, "config", &impl_method);
        let code = output.to_string();

        // Check custom rust_name is used
        assert!(code.contains("keyrx_custom_list_items"));
        assert!(!code.contains("keyrx_config_list_items"));
    }

    #[test]
    fn generate_ffi_function_json_return() {
        let func = make_function("get_data", vec![], "object");
        let impl_method = Ident::new("get_data", Span::call_site());
        let output = generate_ffi_function(&func, "data", &impl_method);
        let code = output.to_string();

        // Check result is returned
        assert!(code.contains("let result"));
        assert!(code.contains("Ok (result)"));
    }

    #[test]
    fn generate_all_ffi_functions_generates_multiple() {
        let contract = make_contract(
            "mydom",
            vec![
                make_function("func_one", vec![], "void"),
                make_function("func_two", vec![make_param("x", "string")], "string"),
            ],
        );
        let output = generate_all_ffi_functions(&contract);
        let code = output.to_string();

        // Check both functions are generated
        assert!(code.contains("keyrx_mydom_func_one"));
        assert!(code.contains("keyrx_mydom_func_two"));
    }
}
