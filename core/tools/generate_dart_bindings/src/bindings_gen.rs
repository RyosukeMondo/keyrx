//! FFI bindings code generator
//!
//! Generates Dart FFI typedef declarations and function pointer lookups
//! from FFI contract definitions.

use crate::templates::{
    context, render, to_camel_case, DART_TYPEDEF_TEMPLATE, FUNCTION_POINTER_TEMPLATE,
    NATIVE_TYPEDEF_TEMPLATE,
};
use crate::type_mapper::{map_to_dart_ffi_type, TypeMappingError};
use crate::types::DartFfiType;
use keyrx_core::ffi::contract::{FfiContract, FunctionContract};

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
}
