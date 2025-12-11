//! FFI bindings code generator
//!
//! Generates Dart FFI typedef declarations and function pointer lookups
//! from FFI contract definitions.

use crate::templates::{
    context, render, to_camel_case, DART_TYPEDEF_TEMPLATE, FUNCTION_POINTER_TEMPLATE,
    NATIVE_TYPEDEF_TEMPLATE, PARAM_FREE_UTF8, PARAM_TO_NATIVE_UTF8,
    RESULT_JSON_DECODE_CONVERSION, RESULT_STRING_CONVERSION, RESULT_VOID_TEMPLATE,
    WRAPPER_FUNCTION_WITH_ERROR_TEMPLATE,
};
use crate::type_mapper::{map_to_dart_ffi_type, map_to_dart_native_type, TypeMappingError};
use crate::types::DartFfiType;
use keyrx_core::ffi::contract::{FfiContract, FunctionContract, TypeDefinition};

/// Error type for binding generation
#[derive(Debug, Clone)]
pub struct BindingGenError {
    pub function_name: String,
    pub message: String,
}

impl std::fmt::Display for BindingGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to generate binding for '{}': {}",
            self.function_name, self.message
        )
    }
}

impl std::error::Error for BindingGenError {}

impl From<TypeMappingError> for BindingGenError {
    fn from(err: TypeMappingError) -> Self {
        BindingGenError {
            function_name: String::new(),
            message: err.to_string(),
        }
    }
}

/// Generated FFI signature for a single function
#[derive(Debug, Clone)]
pub struct FfiSignature {
    /// The full FFI function name (e.g., keyrx_config_save)
    pub ffi_name: String,
    /// The native typedef declaration
    pub native_typedef: String,
    /// The Dart typedef declaration
    pub dart_typedef: String,
    /// The function pointer lookup
    pub function_pointer: String,
}

/// Generate FFI signatures for all functions in a contract
pub fn generate_ffi_signatures(contract: &FfiContract) -> Result<Vec<FfiSignature>, BindingGenError> {
    contract
        .functions
        .iter()
        .map(|func| generate_function_signature(contract, func))
        .collect()
}

/// Generate FFI signature for a single function
fn generate_function_signature(
    contract: &FfiContract,
    func: &FunctionContract,
) -> Result<FfiSignature, BindingGenError> {
    let ffi_name = func
        .rust_name
        .clone()
        .unwrap_or_else(|| format!("keyrx_{}_{}", contract.domain, func.name));

    let return_type = map_return_type(&func.returns)?;
    let (native_params, dart_params) = build_parameter_lists(func)?;

    let native_typedef = generate_native_typedef(&ffi_name, &return_type, &native_params);
    let dart_typedef = generate_dart_typedef(&ffi_name, &return_type, &dart_params);
    let lookup_name = to_camel_case(&func.name);
    let function_pointer = generate_function_pointer(&ffi_name, &lookup_name);

    Ok(FfiSignature {
        ffi_name,
        native_typedef,
        dart_typedef,
        function_pointer,
    })
}

/// Map return type from contract to FFI type
fn map_return_type(
    returns: &keyrx_core::ffi::contract::TypeDefinition,
) -> Result<DartFfiType, BindingGenError> {
    map_to_dart_ffi_type(returns.type_name()).map_err(|e| BindingGenError {
        function_name: String::new(),
        message: format!("Invalid return type: {}", e),
    })
}

/// Build native and Dart parameter lists for a function
fn build_parameter_lists(
    func: &FunctionContract,
) -> Result<(String, String), BindingGenError> {
    let mut native_params = Vec::new();
    let mut dart_params = Vec::new();

    for param in &func.parameters {
        let ffi_type = map_to_dart_ffi_type(&param.param_type).map_err(|e| BindingGenError {
            function_name: func.name.clone(),
            message: format!("Invalid parameter type for '{}': {}", param.name, e),
        })?;

        native_params.push(format!("{} {}", ffi_type.ffi_type(), param.name));
        dart_params.push(format!("{} {}", ffi_type.dart_ffi_function_type(), param.name));
    }

    // Add error pointer parameter (convention: all FFI functions take error ptr)
    native_params.push("Pointer<Pointer<Utf8>> error".to_string());
    dart_params.push("Pointer<Pointer<Utf8>> error".to_string());

    Ok((native_params.join(", "), dart_params.join(", ")))
}

/// Generate native typedef declaration
fn generate_native_typedef(ffi_name: &str, return_type: &DartFfiType, params: &str) -> String {
    let mut ctx = context();
    ctx.insert("function_name".to_string(), ffi_name.to_string());
    ctx.insert("return_type".to_string(), return_type.ffi_type().to_string());
    ctx.insert("native_params".to_string(), params.to_string());
    render(NATIVE_TYPEDEF_TEMPLATE, &ctx)
}

/// Generate Dart typedef declaration
fn generate_dart_typedef(ffi_name: &str, return_type: &DartFfiType, params: &str) -> String {
    let mut ctx = context();
    ctx.insert("function_name".to_string(), ffi_name.to_string());
    ctx.insert(
        "return_type".to_string(),
        return_type.dart_ffi_function_type().to_string(),
    );
    ctx.insert("dart_params".to_string(), params.to_string());
    render(DART_TYPEDEF_TEMPLATE, &ctx)
}

/// Generate function pointer lookup
fn generate_function_pointer(ffi_name: &str, lookup_name: &str) -> String {
    let mut ctx = context();
    ctx.insert("function_name".to_string(), ffi_name.to_string());
    ctx.insert("lookup_name".to_string(), lookup_name.to_string());
    render(FUNCTION_POINTER_TEMPLATE, &ctx)
}

/// Generate all typedefs for a contract as a single string
pub fn generate_typedefs_block(signatures: &[FfiSignature]) -> String {
    let mut lines = Vec::new();

    for sig in signatures {
        lines.push(format!("  {}", sig.native_typedef));
        lines.push(format!("  {}", sig.dart_typedef));
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate all function pointer lookups for a contract as a single string
pub fn generate_function_pointers_block(signatures: &[FfiSignature]) -> String {
    let mut lines = Vec::new();

    for sig in signatures {
        // Indent each line of the function pointer lookup
        for line in sig.function_pointer.lines() {
            lines.push(format!("  {}", line));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generated wrapper function for a single FFI function
#[derive(Debug, Clone)]
pub struct WrapperFunction {
    /// The Dart function name (camelCase)
    pub dart_name: String,
    /// The full generated wrapper code
    pub code: String,
}

/// Generate wrapper functions for all functions in a contract
pub fn generate_wrapper_functions(
    contract: &FfiContract,
) -> Result<Vec<WrapperFunction>, BindingGenError> {
    contract
        .functions
        .iter()
        .map(|func| generate_wrapper_function(contract, func))
        .collect()
}

/// Generate a wrapper function for a single FFI function
fn generate_wrapper_function(
    _contract: &FfiContract,
    func: &FunctionContract,
) -> Result<WrapperFunction, BindingGenError> {
    let dart_name = to_camel_case(&func.name);
    let lookup_name = dart_name.clone();
    let return_type = map_return_type(&func.returns)?;
    let dart_return = determine_dart_return_type(&return_type, &func.returns);

    let params_sig = build_dart_params_signature(func)?;
    let conversions = build_param_conversions(func);
    let ffi_args = build_ffi_call_args(func);
    let frees = build_param_frees(func);
    let result_conv = build_result_conversion(&return_type, &func.returns);

    let code = render_wrapper(&WrapperContext {
        doc_comment: func.description.clone(),
        dart_function_name: dart_name.clone(),
        dart_return_type: dart_return,
        dart_params_signature: params_sig,
        param_conversions: conversions,
        lookup_name,
        ffi_call_args: ffi_args,
        param_frees: frees,
        result_conversion: result_conv,
    });

    Ok(WrapperFunction {
        dart_name,
        code,
    })
}

/// Context for rendering wrapper functions
struct WrapperContext {
    doc_comment: String,
    dart_function_name: String,
    dart_return_type: String,
    dart_params_signature: String,
    param_conversions: String,
    lookup_name: String,
    ffi_call_args: String,
    param_frees: String,
    result_conversion: String,
}

/// Determine the Dart return type for the wrapper function
fn determine_dart_return_type(ffi_type: &DartFfiType, type_def: &TypeDefinition) -> String {
    match ffi_type {
        DartFfiType::Void => "void".to_string(),
        DartFfiType::PointerUtf8 => {
            // Check if this is a complex type that returns JSON
            if type_def.is_object() {
                "Map<String, dynamic>".to_string()
            } else if type_def.is_array() {
                "List<dynamic>".to_string()
            } else {
                "String".to_string()
            }
        }
        _ => ffi_type.dart_type().to_string(),
    }
}

/// Build the Dart parameter signature for a wrapper function
fn build_dart_params_signature(func: &FunctionContract) -> Result<String, BindingGenError> {
    let params: Result<Vec<String>, BindingGenError> = func
        .parameters
        .iter()
        .map(|p| {
            let dart_type = map_to_dart_native_type(&p.param_type).map_err(|e| BindingGenError {
                function_name: func.name.clone(),
                message: format!("Invalid parameter type for '{}': {}", p.name, e),
            })?;
            Ok(format!("{} {}", dart_type, p.name))
        })
        .collect();

    Ok(params?.join(", "))
}

/// Build parameter conversion code (Dart to FFI)
fn build_param_conversions(func: &FunctionContract) -> String {
    let conversions: Vec<String> = func
        .parameters
        .iter()
        .filter_map(|p| {
            let normalized = p.param_type.trim().to_lowercase();
            if normalized == "string" || normalized == "str" {
                let mut ctx = context();
                ctx.insert("param_name".to_string(), p.name.clone());
                Some(render(PARAM_TO_NATIVE_UTF8, &ctx))
            } else if normalized == "object" || normalized == "array" {
                // Complex types need JSON encoding
                Some(format!(
                    "    final {}Ptr = jsonEncode({}).toNativeUtf8();",
                    p.name, p.name
                ))
            } else {
                None
            }
        })
        .collect();

    conversions.join("\n")
}

/// Build FFI call arguments
fn build_ffi_call_args(func: &FunctionContract) -> String {
    let mut args: Vec<String> = func
        .parameters
        .iter()
        .map(|p| {
            let normalized = p.param_type.trim().to_lowercase();
            if normalized == "string"
                || normalized == "str"
                || normalized == "object"
                || normalized == "array"
            {
                format!("{}Ptr", p.name)
            } else {
                p.name.clone()
            }
        })
        .collect();

    // Add error pointer
    args.push("errorPtr".to_string());

    args.join(", ")
}

/// Build parameter free code
fn build_param_frees(func: &FunctionContract) -> String {
    let frees: Vec<String> = func
        .parameters
        .iter()
        .filter_map(|p| {
            let normalized = p.param_type.trim().to_lowercase();
            if normalized == "string"
                || normalized == "str"
                || normalized == "object"
                || normalized == "array"
            {
                let mut ctx = context();
                ctx.insert("param_name".to_string(), p.name.clone());
                Some(render(PARAM_FREE_UTF8, &ctx))
            } else {
                None
            }
        })
        .collect();

    frees.join("\n")
}

/// Build result conversion code
fn build_result_conversion(ffi_type: &DartFfiType, type_def: &TypeDefinition) -> String {
    match ffi_type {
        DartFfiType::Void => RESULT_VOID_TEMPLATE.to_string(),
        DartFfiType::PointerUtf8 => {
            if type_def.is_object() {
                let mut ctx = context();
                ctx.insert("dart_type".to_string(), "Map<String, dynamic>".to_string());
                render(RESULT_JSON_DECODE_CONVERSION, &ctx)
            } else if type_def.is_array() {
                let mut ctx = context();
                ctx.insert("dart_type".to_string(), "List<dynamic>".to_string());
                render(RESULT_JSON_DECODE_CONVERSION, &ctx)
            } else {
                RESULT_STRING_CONVERSION.to_string()
            }
        }
        _ => {
            // Numeric/bool types - return directly
            "    return resultPtr;".to_string()
        }
    }
}

/// Render a wrapper function using the template
fn render_wrapper(ctx: &WrapperContext) -> String {
    let mut template_ctx = context();
    template_ctx.insert("doc_comment".to_string(), ctx.doc_comment.clone());
    template_ctx.insert(
        "dart_function_name".to_string(),
        ctx.dart_function_name.clone(),
    );
    template_ctx.insert("dart_return_type".to_string(), ctx.dart_return_type.clone());
    template_ctx.insert(
        "dart_params_signature".to_string(),
        ctx.dart_params_signature.clone(),
    );
    template_ctx.insert(
        "param_conversions".to_string(),
        ctx.param_conversions.clone(),
    );
    template_ctx.insert("lookup_name".to_string(), ctx.lookup_name.clone());
    template_ctx.insert("ffi_call_args".to_string(), ctx.ffi_call_args.clone());
    template_ctx.insert("param_frees".to_string(), ctx.param_frees.clone());
    template_ctx.insert(
        "result_conversion".to_string(),
        ctx.result_conversion.clone(),
    );

    render(WRAPPER_FUNCTION_WITH_ERROR_TEMPLATE, &template_ctx)
}

/// Generate all wrapper functions for a contract as a single string
pub fn generate_wrappers_block(wrappers: &[WrapperFunction]) -> String {
    wrappers
        .iter()
        .map(|w| format!("  {}", w.code.replace('\n', "\n  ")))
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_core::ffi::contract::{ParameterContract, TypeDefinition};

    fn create_test_contract() -> FfiContract {
        FfiContract {
            schema: "https://keyrx.dev/schemas/ffi-contract-v1.json".to_string(),
            version: "1.0.0".to_string(),
            domain: "test".to_string(),
            description: "Test contract".to_string(),
            protocol_version: 1,
            functions: vec![
                FunctionContract {
                    name: "get_value".to_string(),
                    description: "Get a value".to_string(),
                    rust_name: Some("keyrx_test_get_value".to_string()),
                    parameters: vec![],
                    returns: TypeDefinition::Primitive {
                        type_name: "string".to_string(),
                        description: None,
                        constraints: None,
                    },
                    errors: vec![],
                    events_emitted: vec![],
                    example: None,
                    deprecated: false,
                    since_version: None,
                },
                FunctionContract {
                    name: "set_value".to_string(),
                    description: "Set a value".to_string(),
                    rust_name: Some("keyrx_test_set_value".to_string()),
                    parameters: vec![ParameterContract {
                        name: "json".to_string(),
                        param_type: "string".to_string(),
                        description: "JSON data".to_string(),
                        required: true,
                        constraints: None,
                    }],
                    returns: TypeDefinition::Primitive {
                        type_name: "string".to_string(),
                        description: None,
                        constraints: None,
                    },
                    errors: vec![],
                    events_emitted: vec![],
                    example: None,
                    deprecated: false,
                    since_version: None,
                },
            ],
            types: std::collections::HashMap::new(),
            events: vec![],
        }
    }

    #[test]
    fn test_generate_ffi_signatures() {
        let contract = create_test_contract();
        let signatures = generate_ffi_signatures(&contract).unwrap();

        assert_eq!(signatures.len(), 2);
        assert_eq!(signatures[0].ffi_name, "keyrx_test_get_value");
        assert_eq!(signatures[1].ffi_name, "keyrx_test_set_value");
    }

    #[test]
    fn test_native_typedef_generation() {
        let contract = create_test_contract();
        let signatures = generate_ffi_signatures(&contract).unwrap();

        let native = &signatures[0].native_typedef;
        assert!(native.contains("typedef _keyrx_test_get_value_native"));
        assert!(native.contains("Pointer<Utf8> Function"));
        assert!(native.contains("Pointer<Pointer<Utf8>> error"));
    }

    #[test]
    fn test_dart_typedef_generation() {
        let contract = create_test_contract();
        let signatures = generate_ffi_signatures(&contract).unwrap();

        let dart = &signatures[0].dart_typedef;
        assert!(dart.contains("typedef _keyrx_test_get_value ="));
        assert!(dart.contains("Pointer<Utf8> Function"));
    }

    #[test]
    fn test_function_pointer_generation() {
        let contract = create_test_contract();
        let signatures = generate_ffi_signatures(&contract).unwrap();

        let ptr = &signatures[0].function_pointer;
        assert!(ptr.contains("late final _getValue ="));
        assert!(ptr.contains("NativeFunction<_keyrx_test_get_value_native>"));
        assert!(ptr.contains("'keyrx_test_get_value'"));
    }

    #[test]
    fn test_parameter_in_typedef() {
        let contract = create_test_contract();
        let signatures = generate_ffi_signatures(&contract).unwrap();

        // set_value has a json parameter
        let native = &signatures[1].native_typedef;
        assert!(native.contains("Pointer<Utf8> json"));
    }

    #[test]
    fn test_generate_typedefs_block() {
        let contract = create_test_contract();
        let signatures = generate_ffi_signatures(&contract).unwrap();
        let block = generate_typedefs_block(&signatures);

        assert!(block.contains("_keyrx_test_get_value_native"));
        assert!(block.contains("_keyrx_test_set_value_native"));
    }

    #[test]
    fn test_generate_function_pointers_block() {
        let contract = create_test_contract();
        let signatures = generate_ffi_signatures(&contract).unwrap();
        let block = generate_function_pointers_block(&signatures);

        assert!(block.contains("late final _getValue"));
        assert!(block.contains("late final _setValue"));
    }

    // Wrapper function tests
    #[test]
    fn test_generate_wrapper_functions() {
        let contract = create_test_contract();
        let wrappers = generate_wrapper_functions(&contract).unwrap();

        assert_eq!(wrappers.len(), 2);
        assert_eq!(wrappers[0].dart_name, "getValue");
        assert_eq!(wrappers[1].dart_name, "setValue");
    }

    #[test]
    fn test_wrapper_function_no_params() {
        let contract = create_test_contract();
        let wrappers = generate_wrapper_functions(&contract).unwrap();

        let get_value = &wrappers[0].code;
        assert!(get_value.contains("String getValue()"));
        assert!(get_value.contains("final errorPtr = calloc<Pointer<Utf8>>()"));
        assert!(get_value.contains("_getValue(errorPtr)"));
        assert!(get_value.contains("throw FfiException"));
    }

    #[test]
    fn test_wrapper_function_with_string_param() {
        let contract = create_test_contract();
        let wrappers = generate_wrapper_functions(&contract).unwrap();

        let set_value = &wrappers[1].code;
        assert!(set_value.contains("String setValue(String json)"));
        assert!(set_value.contains("jsonPtr = json.toNativeUtf8()"));
        assert!(set_value.contains("_setValue(jsonPtr, errorPtr)"));
        assert!(set_value.contains("calloc.free(jsonPtr)"));
    }

    #[test]
    fn test_wrapper_error_handling() {
        let contract = create_test_contract();
        let wrappers = generate_wrapper_functions(&contract).unwrap();

        let code = &wrappers[0].code;
        assert!(code.contains("if (errorPtr.value.address != 0)"));
        assert!(code.contains("errorPtr.value.toDartString()"));
        assert!(code.contains("calloc.free(errorPtr.value)"));
        assert!(code.contains("throw FfiException(error)"));
    }

    #[test]
    fn test_wrapper_result_conversion() {
        let contract = create_test_contract();
        let wrappers = generate_wrapper_functions(&contract).unwrap();

        let code = &wrappers[0].code;
        // String return should use toDartString and free result
        assert!(code.contains("resultPtr.toDartString()"));
        assert!(code.contains("_keyrx_free_string(resultPtr)"));
    }

    #[test]
    fn test_wrapper_finally_cleanup() {
        let contract = create_test_contract();
        let wrappers = generate_wrapper_functions(&contract).unwrap();

        let code = &wrappers[0].code;
        assert!(code.contains("} finally {"));
        assert!(code.contains("calloc.free(errorPtr)"));
    }

    fn create_object_return_contract() -> FfiContract {
        FfiContract {
            schema: "https://keyrx.dev/schemas/ffi-contract-v1.json".to_string(),
            version: "1.0.0".to_string(),
            domain: "test".to_string(),
            description: "Test contract".to_string(),
            protocol_version: 1,
            functions: vec![FunctionContract {
                name: "get_profile".to_string(),
                description: "Get user profile".to_string(),
                rust_name: Some("keyrx_test_get_profile".to_string()),
                parameters: vec![],
                returns: TypeDefinition::Object {
                    type_name: "object".to_string(),
                    description: Some("User profile data".to_string()),
                    properties: std::collections::HashMap::new(),
                },
                errors: vec![],
                events_emitted: vec![],
                example: None,
                deprecated: false,
                since_version: None,
            }],
            types: std::collections::HashMap::new(),
            events: vec![],
        }
    }

    #[test]
    fn test_wrapper_object_return_type() {
        let contract = create_object_return_contract();
        let wrappers = generate_wrapper_functions(&contract).unwrap();

        let code = &wrappers[0].code;
        assert!(code.contains("Map<String, dynamic> getProfile()"));
        assert!(code.contains("jsonDecode(resultJson) as Map<String, dynamic>"));
    }

    #[test]
    fn test_generate_wrappers_block() {
        let contract = create_test_contract();
        let wrappers = generate_wrapper_functions(&contract).unwrap();
        let block = generate_wrappers_block(&wrappers);

        assert!(block.contains("String getValue()"));
        assert!(block.contains("String setValue(String json)"));
    }
}
