//! Contract JSON generation from Rust function signatures

use crate::parser::FfiExportConfig;
use crate::types::{describe_type, extract_result_ok_type, is_option_type, rust_type_to_contract_type};
use serde::{Deserialize, Serialize};
use syn::{FnArg, ItemFn, Pat, ReturnType};

/// A single function entry in the FFI contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractFunction {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ContractParameter>,
    pub returns: ContractReturnType,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ContractError>,
}

/// A function parameter in the contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

/// Return type specification in the contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractReturnType {
    #[serde(rename = "type")]
    pub return_type: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
}

/// Error specification in the contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractError {
    pub code: String,
    pub description: String,
}

/// Generate a contract function entry from a Rust function
pub fn generate_contract_entry(
    func: &ItemFn,
    config: &FfiExportConfig,
) -> syn::Result<ContractFunction> {
    // Extract function name (use rename if provided)
    let name = config
        .rename
        .clone()
        .unwrap_or_else(|| func.sig.ident.to_string());

    // Extract documentation comments
    let description = extract_doc_comments(&func.attrs);

    // Extract parameters
    let parameters = extract_parameters(&func.sig.inputs)?;

    // Extract return type
    let returns = extract_return_type(&func.sig.output)?;

    // Generate error specifications (placeholder for now)
    let errors = vec![
        ContractError {
            code: "INTERNAL_ERROR".to_string(),
            description: "Internal error occurred".to_string(),
        },
    ];

    Ok(ContractFunction {
        name,
        description,
        parameters,
        returns,
        errors,
    })
}

/// Extract documentation comments from attributes
fn extract_doc_comments(attrs: &[syn::Attribute]) -> String {
    let mut docs = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        let doc_str = lit_str.value();
                        // Remove leading space that rustdoc adds
                        let trimmed = doc_str.trim_start().trim_end();
                        if !trimmed.is_empty() {
                            docs.push(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }

    if docs.is_empty() {
        "No description provided".to_string()
    } else {
        docs.join("\n")
    }
}

/// Extract parameters from function signature
fn extract_parameters(inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) -> syn::Result<Vec<ContractParameter>> {
    let mut parameters = Vec::new();

    for input in inputs {
        match input {
            FnArg::Receiver(_) => {
                // Skip self parameters
                continue;
            }
            FnArg::Typed(pat_type) => {
                // Extract parameter name
                let param_name = if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    pat_ident.ident.to_string()
                } else {
                    return Err(syn::Error::new_spanned(
                        &pat_type.pat,
                        "Expected identifier pattern for parameter",
                    ));
                };

                let param_type = &*pat_type.ty;

                // Check if parameter is Option<T> (optional)
                let (required, inner_type) = if let Some(inner) = is_option_type(param_type) {
                    (false, inner)
                } else {
                    (true, param_type.clone())
                };

                // Map Rust type to contract type
                let contract_type = rust_type_to_contract_type(&inner_type);

                // Generate description
                let description = describe_type(&inner_type);

                parameters.push(ContractParameter {
                    name: param_name,
                    param_type: contract_type,
                    description,
                    required,
                });
            }
        }
    }

    Ok(parameters)
}

/// Extract return type from function signature
fn extract_return_type(output: &ReturnType) -> syn::Result<ContractReturnType> {
    match output {
        ReturnType::Default => {
            // No return type means ()
            Ok(ContractReturnType {
                return_type: "null".to_string(),
                description: "No return value".to_string(),
                properties: None,
            })
        }
        ReturnType::Type(_, ty) => {
            // For Result<T, E>, extract T
            let actual_type = if let Some(ok_type) = extract_result_ok_type(ty) {
                ok_type
            } else {
                (**ty).clone()
            };

            let contract_type = rust_type_to_contract_type(&actual_type);
            let description = describe_type(&actual_type);

            Ok(ContractReturnType {
                return_type: contract_type,
                description,
                properties: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_doc_comments() {
        let func: ItemFn = parse_quote! {
            /// This is a test function.
            /// It does something useful.
            pub fn test_fn() -> Result<(), Error> {
                Ok(())
            }
        };

        let description = extract_doc_comments(&func.attrs);
        assert_eq!(description, "This is a test function.\nIt does something useful.");
    }

    #[test]
    fn test_extract_parameters() {
        let func: ItemFn = parse_quote! {
            pub fn test_fn(name: &str, count: u32, optional: Option<String>) -> Result<(), Error> {
                Ok(())
            }
        };

        let params = extract_parameters(&func.sig.inputs).unwrap();
        assert_eq!(params.len(), 3);

        assert_eq!(params[0].name, "name");
        assert_eq!(params[0].param_type, "string");
        assert!(params[0].required);

        assert_eq!(params[1].name, "count");
        assert_eq!(params[1].param_type, "number");
        assert!(params[1].required);

        assert_eq!(params[2].name, "optional");
        assert_eq!(params[2].param_type, "string");
        assert!(!params[2].required);
    }

    #[test]
    fn test_extract_return_type() {
        let func: ItemFn = parse_quote! {
            pub fn test_fn() -> Result<String, Error> {
                Ok("test".to_string())
            }
        };

        let return_type = extract_return_type(&func.sig.output).unwrap();
        assert_eq!(return_type.return_type, "string");
    }

    #[test]
    fn test_generate_contract_entry() {
        let func: ItemFn = parse_quote! {
            /// Start the engine loop.
            pub fn start_loop(profile_id: &str) -> Result<State, Error> {
                unimplemented!()
            }
        };

        let config = FfiExportConfig {
            domain: "engine".to_string(),
            version: "1.0".to_string(),
            rename: None,
            skip: false,
        };

        let entry = generate_contract_entry(&func, &config).unwrap();
        assert_eq!(entry.name, "start_loop");
        assert_eq!(entry.description, "Start the engine loop.");
        assert_eq!(entry.parameters.len(), 1);
        assert_eq!(entry.parameters[0].name, "profile_id");
        assert_eq!(entry.returns.return_type, "object");
    }
}
