//! AST Parser for FFI Function Signatures
//!
//! This module provides data structures and parsing logic to extract
//! `extern "C"` function signatures from Rust source files using the `syn` crate.

use std::fs;
use std::path::{Path, PathBuf};
use syn::{Abi, FnArg, ForeignItem, Item, Pat, ReturnType, Type, Visibility};

/// Error type for parsing failures.
#[derive(Debug)]
pub enum ParseError {
    /// Failed to read the source file.
    IoError(std::io::Error),
    /// Failed to parse the Rust syntax.
    SynError(syn::Error),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(e) => write!(f, "IO error: {}", e),
            ParseError::SynError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<syn::Error> for ParseError {
    fn from(err: syn::Error) -> Self {
        ParseError::SynError(err)
    }
}

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

/// Parse a Rust source file and extract all `extern "C"` functions with `#[no_mangle]`.
///
/// This function reads the file, parses it using `syn`, and extracts all FFI-exported
/// functions that have both the `extern "C"` ABI and `#[no_mangle]` attribute.
pub fn parse_ffi_exports(file_path: &Path) -> Result<Vec<ParsedFunction>, ParseError> {
    let source = fs::read_to_string(file_path)?;
    parse_ffi_exports_from_str(&source, file_path.to_path_buf())
}

/// Parse FFI exports from a string (for testing purposes).
pub fn parse_ffi_exports_from_str(
    source: &str,
    file_path: PathBuf,
) -> Result<Vec<ParsedFunction>, ParseError> {
    let ast = syn::parse_file(source)?;
    let mut functions = Vec::new();

    for item in ast.items {
        match item {
            Item::Fn(func) => {
                if is_extern_c_no_mangle(&func.attrs, &func.sig.abi, &func.vis) {
                    let parsed = extract_function_signature(&func.sig, &file_path, source)?;
                    functions.push(parsed);
                }
            }
            Item::ForeignMod(foreign) => {
                if is_extern_c_abi(&foreign.abi) {
                    for item in foreign.items {
                        if let ForeignItem::Fn(func) = item {
                            if has_no_mangle(&func.attrs) {
                                let parsed =
                                    extract_foreign_fn_signature(&func, &file_path, source)?;
                                functions.push(parsed);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(functions)
}

/// Check if a function has `extern "C"` ABI and `#[no_mangle]` attribute.
fn is_extern_c_no_mangle(attrs: &[syn::Attribute], abi: &Option<Abi>, vis: &Visibility) -> bool {
    let is_pub = matches!(vis, Visibility::Public(_));
    let is_extern_c = abi
        .as_ref()
        .and_then(|a| a.name.as_ref())
        .map(|n| n.value() == "C")
        .unwrap_or(false);
    let has_no_mangle = has_no_mangle(attrs);

    is_pub && is_extern_c && has_no_mangle
}

/// Check if an ABI is `extern "C"`.
fn is_extern_c_abi(abi: &Abi) -> bool {
    abi.name.as_ref().map(|n| n.value() == "C").unwrap_or(false)
}

/// Check if attributes contain `#[no_mangle]`.
fn has_no_mangle(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("no_mangle"))
}

/// Extract function signature from a `syn::Signature`.
fn extract_function_signature(
    sig: &syn::Signature,
    file_path: &Path,
    source: &str,
) -> Result<ParsedFunction, ParseError> {
    let name = sig.ident.to_string();
    let line_number = get_line_number(source, sig.ident.span());

    let params = sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(extract_param(pat_type))
            } else {
                None
            }
        })
        .collect();

    let return_type = extract_return_type(&sig.output);

    Ok(ParsedFunction::new(
        name,
        params,
        return_type,
        file_path.to_path_buf(),
        line_number,
    ))
}

/// Extract function signature from a foreign function declaration.
fn extract_foreign_fn_signature(
    func: &syn::ForeignItemFn,
    file_path: &Path,
    source: &str,
) -> Result<ParsedFunction, ParseError> {
    let name = func.sig.ident.to_string();
    let line_number = get_line_number(source, func.sig.ident.span());

    let params = func
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(extract_param(pat_type))
            } else {
                None
            }
        })
        .collect();

    let return_type = extract_return_type(&func.sig.output);

    Ok(ParsedFunction::new(
        name,
        params,
        return_type,
        file_path.to_path_buf(),
        line_number,
    ))
}

/// Extract parameter information from a typed parameter.
fn extract_param(pat_type: &syn::PatType) -> ParsedParam {
    let name = match pat_type.pat.as_ref() {
        Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
        _ => "_".to_string(),
    };

    let (rust_type, is_pointer, is_mutable) = extract_type_info(&pat_type.ty);

    ParsedParam::new(name, rust_type, is_pointer, is_mutable)
}

/// Extract type information including string representation and pointer metadata.
fn extract_type_info(ty: &Type) -> (String, bool, bool) {
    match ty {
        Type::Ptr(ptr) => {
            let is_mut = ptr.mutability.is_some();
            let target_type = type_to_string(&ptr.elem);
            let full_type = if is_mut {
                format!("*mut {}", target_type)
            } else {
                format!("*const {}", target_type)
            };
            (full_type, true, is_mut)
        }
        _ => (type_to_string(ty), false, false),
    }
}

/// Convert a syn::Type to a string representation.
fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        Type::Ptr(ptr) => {
            let inner = type_to_string(&ptr.elem);
            if ptr.mutability.is_some() {
                format!("*mut {}", inner)
            } else {
                format!("*const {}", inner)
            }
        }
        Type::Tuple(tuple) if tuple.elems.is_empty() => "()".to_string(),
        Type::BareFn(bare_fn) => {
            let params: Vec<String> = bare_fn
                .inputs
                .iter()
                .map(|arg| type_to_string(&arg.ty))
                .collect();
            let ret = match &bare_fn.output {
                ReturnType::Default => "()".to_string(),
                ReturnType::Type(_, ty) => type_to_string(ty),
            };
            format!("fn({}) -> {}", params.join(", "), ret)
        }
        Type::Reference(reference) => {
            let inner = type_to_string(&reference.elem);
            if reference.mutability.is_some() {
                format!("&mut {}", inner)
            } else {
                format!("&{}", inner)
            }
        }
        _ => quote::quote!(#ty).to_string(),
    }
}

/// Extract return type from a syn::ReturnType.
fn extract_return_type(output: &ReturnType) -> ParsedType {
    match output {
        ReturnType::Default => ParsedType::Unit,
        ReturnType::Type(_, ty) => parsed_type_from_syn(ty),
    }
}

/// Convert a syn::Type to a ParsedType.
fn parsed_type_from_syn(ty: &Type) -> ParsedType {
    match ty {
        Type::Tuple(tuple) if tuple.elems.is_empty() => ParsedType::Unit,
        Type::Ptr(ptr) => {
            let is_mut = ptr.mutability.is_some();
            let target = type_to_string(&ptr.elem);
            ParsedType::Pointer { target, is_mut }
        }
        _ => ParsedType::Primitive(type_to_string(ty)),
    }
}

/// Get line number from a span (1-indexed).
fn get_line_number(source: &str, span: proc_macro2::Span) -> usize {
    let start = span.start();
    // syn uses 1-indexed lines
    if start.line > 0 {
        start.line
    } else {
        // Fallback: count newlines manually if span doesn't have location
        source[..source.len().min(span.byte_range().start)]
            .chars()
            .filter(|&c| c == '\n')
            .count()
            + 1
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

    #[test]
    fn test_parse_simple_extern_c_function() {
        let source = r#"
            use std::ffi::c_char;

            #[no_mangle]
            pub unsafe extern "C" fn keyrx_init() -> i32 {
                0
            }
        "#;

        let funcs =
            parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).expect("parse failed");

        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "keyrx_init");
        assert_eq!(funcs[0].params.len(), 0);
        assert_eq!(
            funcs[0].return_type,
            ParsedType::Primitive("i32".to_string())
        );
    }

    #[test]
    fn test_parse_function_with_pointer_params() {
        let source = r#"
            use std::ffi::c_char;

            #[no_mangle]
            pub unsafe extern "C" fn keyrx_free_string(s: *mut c_char) {
                // free it
            }
        "#;

        let funcs =
            parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).expect("parse failed");

        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "keyrx_free_string");
        assert_eq!(funcs[0].params.len(), 1);

        let param = &funcs[0].params[0];
        assert_eq!(param.name, "s");
        assert_eq!(param.rust_type, "*mut c_char");
        assert!(param.is_pointer);
        assert!(param.is_mutable);
    }

    #[test]
    fn test_parse_function_returning_pointer() {
        let source = r#"
            use std::ffi::c_char;

            #[no_mangle]
            pub unsafe extern "C" fn keyrx_version() -> *mut c_char {
                std::ptr::null_mut()
            }
        "#;

        let funcs =
            parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).expect("parse failed");

        assert_eq!(funcs.len(), 1);
        assert_eq!(
            funcs[0].return_type,
            ParsedType::Pointer {
                target: "c_char".to_string(),
                is_mut: true,
            }
        );
    }

    #[test]
    fn test_parse_ignores_non_ffi_functions() {
        let source = r#"
            // Not extern "C" - should be ignored
            pub fn regular_function() -> i32 {
                42
            }

            // No #[no_mangle] - should be ignored
            pub unsafe extern "C" fn no_mangle_missing() -> i32 {
                42
            }

            // Not public - should be ignored
            #[no_mangle]
            unsafe extern "C" fn private_fn() -> i32 {
                42
            }

            // Valid FFI function
            #[no_mangle]
            pub unsafe extern "C" fn keyrx_valid() -> i32 {
                0
            }
        "#;

        let funcs =
            parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).expect("parse failed");

        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "keyrx_valid");
    }

    #[test]
    fn test_parse_function_with_callback_param() {
        let source = r#"
            use std::ffi::c_char;

            #[no_mangle]
            pub unsafe extern "C" fn keyrx_register_callback(
                event_type: i32,
                callback: Option<unsafe extern "C" fn(*const u8, usize)>,
            ) -> i32 {
                0
            }
        "#;

        let funcs =
            parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).expect("parse failed");

        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "keyrx_register_callback");
        assert_eq!(funcs[0].params.len(), 2);

        let event_param = &funcs[0].params[0];
        assert_eq!(event_param.name, "event_type");
        assert_eq!(event_param.rust_type, "i32");
        assert!(!event_param.is_pointer);

        let callback_param = &funcs[0].params[1];
        assert_eq!(callback_param.name, "callback");
        assert!(callback_param.rust_type.contains("Option"));
    }

    #[test]
    fn test_parse_multiple_functions() {
        let source = r#"
            use std::ffi::c_char;

            #[no_mangle]
            pub unsafe extern "C" fn keyrx_init() -> i32 {
                0
            }

            #[no_mangle]
            pub unsafe extern "C" fn keyrx_version() -> *mut c_char {
                std::ptr::null_mut()
            }

            #[no_mangle]
            pub unsafe extern "C" fn keyrx_shutdown() {
                // cleanup
            }
        "#;

        let funcs =
            parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).expect("parse failed");

        assert_eq!(funcs.len(), 3);

        let names: Vec<&str> = funcs.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"keyrx_init"));
        assert!(names.contains(&"keyrx_version"));
        assert!(names.contains(&"keyrx_shutdown"));
    }

    #[test]
    fn test_parse_function_with_void_return() {
        let source = r#"
            #[no_mangle]
            pub unsafe extern "C" fn keyrx_shutdown() {
                // no return
            }
        "#;

        let funcs =
            parse_ffi_exports_from_str(source, PathBuf::from("test.rs")).expect("parse failed");

        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].return_type, ParsedType::Unit);
    }

    #[test]
    fn test_parse_syntax_error_returns_error() {
        let source = r#"
            // Invalid Rust syntax
            pub fn broken( {
        "#;

        let result = parse_ffi_exports_from_str(source, PathBuf::from("test.rs"));
        assert!(result.is_err());
    }
}
