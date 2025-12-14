//! Type mapping from Rust types to contract JSON types

use syn::{GenericArgument, PathArguments, Type, TypePath};

/// Convert a Rust type to a contract JSON type string
pub fn rust_type_to_contract_type(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => path_type_to_contract(type_path),
        Type::Reference(type_ref) => {
            // For references, look at the inner type
            rust_type_to_contract_type(&type_ref.elem)
        }
        Type::Tuple(tuple) => {
            if tuple.elems.is_empty() {
                // Unit type ()
                "null".to_string()
            } else {
                // Tuple types map to array
                "array".to_string()
            }
        }
        _ => {
            // Default to object for unknown types
            "object".to_string()
        }
    }
}

/// Check if a type is Option<T> and extract T
pub fn is_option_type(ty: &Type) -> Option<Type> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty.clone());
                    }
                }
            }
        }
    }
    None
}

/// Extract the Ok type from Result<T, E>
pub fn extract_result_ok_type(ty: &Type) -> Option<Type> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Result" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(ok_ty)) = args.args.first() {
                        return Some(ok_ty.clone());
                    }
                }
            }
        }
    }
    None
}

/// Convert a PathType to a contract type string
fn path_type_to_contract(type_path: &TypePath) -> String {
    if let Some(segment) = type_path.path.segments.last() {
        let ident = segment.ident.to_string();

        // Handle common primitive types
        match ident.as_str() {
            // String types
            "str" | "String" => "string".to_string(),

            // Integer types
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
            | "i128" | "isize" => "number".to_string(),

            // Floating point
            "f32" | "f64" => "number".to_string(),

            // Boolean
            "bool" => "bool".to_string(),

            // Option<T> - extract inner type
            "Option" => {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return rust_type_to_contract_type(inner_ty);
                    }
                }
                "object".to_string()
            }

            // Vec<T> and arrays
            "Vec" => "array".to_string(),

            // Result<T, E> - extract Ok type
            "Result" => {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(ok_ty)) = args.args.first() {
                        return rust_type_to_contract_type(ok_ty);
                    }
                }
                "object".to_string()
            }

            // Default to object for custom types
            _ => "object".to_string(),
        }
    } else {
        "object".to_string()
    }
}

/// Extract type description from doc comments or type name
pub fn describe_type(ty: &Type) -> String {
    let type_name = rust_type_to_contract_type(ty);
    match type_name.as_str() {
        "string" => "A string value".to_string(),
        "number" => "A numeric value".to_string(),
        "bool" => "A boolean value".to_string(),
        "array" => "An array of values".to_string(),
        "null" => "No return value".to_string(),
        "object" => "An object value".to_string(),
        _ => format!("A {} value", type_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_string_types() {
        let ty: Type = parse_quote! { &str };
        assert_eq!(rust_type_to_contract_type(&ty), "string");

        let ty: Type = parse_quote! { String };
        assert_eq!(rust_type_to_contract_type(&ty), "string");
    }

    #[test]
    fn test_numeric_types() {
        let ty: Type = parse_quote! { u32 };
        assert_eq!(rust_type_to_contract_type(&ty), "number");

        let ty: Type = parse_quote! { i64 };
        assert_eq!(rust_type_to_contract_type(&ty), "number");

        let ty: Type = parse_quote! { f64 };
        assert_eq!(rust_type_to_contract_type(&ty), "number");
    }

    #[test]
    fn test_bool_type() {
        let ty: Type = parse_quote! { bool };
        assert_eq!(rust_type_to_contract_type(&ty), "bool");
    }

    #[test]
    fn test_option_type() {
        let ty: Type = parse_quote! { Option<String> };
        assert_eq!(rust_type_to_contract_type(&ty), "string");

        let ty: Type = parse_quote! { Option<u32> };
        assert_eq!(rust_type_to_contract_type(&ty), "number");
    }

    #[test]
    fn test_result_type() {
        let ty: Type = parse_quote! { Result<String, Error> };
        assert_eq!(rust_type_to_contract_type(&ty), "string");
    }

    #[test]
    fn test_vec_type() {
        let ty: Type = parse_quote! { Vec<String> };
        assert_eq!(rust_type_to_contract_type(&ty), "array");
    }

    #[test]
    fn test_custom_type() {
        let ty: Type = parse_quote! { MyCustomType };
        assert_eq!(rust_type_to_contract_type(&ty), "object");
    }

    #[test]
    fn test_unit_type() {
        let ty: Type = parse_quote! { () };
        assert_eq!(rust_type_to_contract_type(&ty), "null");
    }
}
