//! Implementation of the rhai_doc attribute macro

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, ItemFn, Lit, Meta};

/// Documentation extracted from function attributes and comments
struct DocInfo {
    module: String,
    function_name: String,
    description: String,
    parameters: Vec<ParamInfo>,
    returns: Option<ReturnInfo>,
    examples: Vec<String>,
    notes: Option<String>,
    since: Option<String>,
    deprecated: Option<String>,
}

/// Information about a function parameter
struct ParamInfo {
    name: String,
    type_name: String,
    description: String,
    optional: bool,
}

/// Information about return value
struct ReturnInfo {
    type_name: String,
    description: String,
}

/// Generate the rhai_doc macro expansion
pub fn generate_rhai_doc(
    attr_tokens: TokenStream,
    func: &ItemFn,
) -> syn::Result<TokenStream> {
    // Parse attributes
    let attr_map = parse_attr_tokens(attr_tokens)?;

    let module = attr_map
        .get("module")
        .ok_or_else(|| syn::Error::new_spanned(func, "rhai_doc requires 'module' attribute"))?
        .clone();

    let since = attr_map.get("since").cloned();
    let deprecated = attr_map.get("deprecated").cloned();

    // Extract documentation from attributes
    let doc_info = extract_doc_info(&func.attrs, &func.sig, module, since, deprecated)?;

    // Generate the registration code
    let registration_code = generate_registration_code(&doc_info);

    // Keep the original function unchanged
    let func_attrs = &func.attrs;
    let func_vis = &func.vis;
    let func_sig = &func.sig;
    let func_block = &func.block;

    // Generate a unique identifier for the registration function
    let func_name = &func.sig.ident;
    let register_fn_name = quote::format_ident!("__register_doc_{}", func_name);

    Ok(quote! {
        // Keep the original function
        #(#func_attrs)*
        #func_vis #func_sig
        #func_block

        // Generate a registration function that should be called during initialization
        #[doc(hidden)]
        #[allow(non_snake_case)]
        pub fn #register_fn_name() {
            use std::sync::Once;
            static INIT: Once = Once::new();
            INIT.call_once(|| {
                #registration_code
            });
        }
    })
}

/// Extract documentation information from function attributes and signature
fn extract_doc_info(
    attrs: &[Attribute],
    sig: &syn::Signature,
    module: String,
    since: Option<String>,
    deprecated: Option<String>,
) -> syn::Result<DocInfo> {
    let function_name = sig.ident.to_string();

    // Extract doc comments
    let doc_comments = extract_doc_comments(attrs);

    // Parse doc comments into structured information
    let (description, param_docs, return_doc, examples, notes) = parse_doc_comments(&doc_comments);

    // Extract parameter information from function signature
    let parameters = extract_parameters(sig, &param_docs)?;

    // Extract return type information
    let returns = extract_return_type(sig, return_doc)?;

    Ok(DocInfo {
        module,
        function_name,
        description,
        parameters,
        returns,
        examples,
        notes,
        since,
        deprecated,
    })
}

/// Extract doc comments from attributes
fn extract_doc_comments(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            return Some(lit_str.value());
                        }
                    }
                }
            }
            None
        })
        .collect()
}

/// Parsed documentation sections from doc comments
type ParsedDocs = (
    String,                 // description
    Vec<(String, String)>,  // parameters: (name, description)
    Option<String>,         // returns
    Vec<String>,            // examples
    Option<String>,         // notes
);

/// Parse doc comments into structured sections
fn parse_doc_comments(comments: &[String]) -> ParsedDocs {
    let mut description = String::new();
    let mut parameters = Vec::new();
    let mut returns = None;
    let mut examples = Vec::new();
    let mut notes = None;

    let mut current_section = Section::Description;
    let mut current_example = String::new();
    let mut in_code_block = false;

    for line in comments {
        let trimmed = line.trim();

        // Check for section headers
        if trimmed.starts_with("# Parameters") {
            current_section = Section::Parameters;
            continue;
        } else if trimmed.starts_with("# Returns") {
            current_section = Section::Returns;
            continue;
        } else if trimmed.starts_with("# Examples") {
            current_section = Section::Examples;
            continue;
        } else if trimmed.starts_with("# Notes") {
            current_section = Section::Notes;
            continue;
        }

        // Process content based on current section
        match current_section {
            Section::Description => {
                if !description.is_empty() {
                    description.push('\n');
                }
                description.push_str(trimmed);
            }
            Section::Parameters => {
                // Parse parameter documentation: * `param_name` - description
                if let Some(param_doc) = parse_param_doc(trimmed) {
                    parameters.push(param_doc);
                }
            }
            Section::Returns => {
                if returns.is_none() {
                    returns = Some(String::new());
                }
                if let Some(ref mut ret) = returns {
                    if !ret.is_empty() {
                        ret.push(' ');
                    }
                    ret.push_str(trimmed);
                }
            }
            Section::Examples => {
                // Handle code blocks
                if trimmed.starts_with("```") {
                    if in_code_block {
                        // End of code block
                        if !current_example.trim().is_empty() {
                            examples.push(current_example.trim().to_string());
                        }
                        current_example.clear();
                        in_code_block = false;
                    } else {
                        // Start of code block
                        in_code_block = true;
                    }
                } else if in_code_block {
                    if !current_example.is_empty() {
                        current_example.push('\n');
                    }
                    current_example.push_str(trimmed);
                }
            }
            Section::Notes => {
                if notes.is_none() {
                    notes = Some(String::new());
                }
                if let Some(ref mut n) = notes {
                    if !n.is_empty() {
                        n.push(' ');
                    }
                    n.push_str(trimmed);
                }
            }
        }
    }

    (description, parameters, returns, examples, notes)
}

/// Parse a parameter documentation line: * `param_name` - description
fn parse_param_doc(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if !line.starts_with('*') {
        return None;
    }

    let line = line[1..].trim();

    // Extract parameter name from backticks
    let start = line.find('`')?;
    let end = line[start + 1..].find('`')? + start + 1;
    let param_name = line[start + 1..end].to_string();

    // Extract description after the dash
    let dash_pos = line[end..].find('-')?;
    let description = line[end + dash_pos + 1..].trim().to_string();

    Some((param_name, description))
}

/// Section of documentation being parsed
enum Section {
    Description,
    Parameters,
    Returns,
    Examples,
    Notes,
}

/// Extract parameter information from function signature
fn extract_parameters(
    sig: &syn::Signature,
    param_docs: &[(String, String)],
) -> syn::Result<Vec<ParamInfo>> {
    let mut parameters = Vec::new();

    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            let param_name = if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                pat_ident.ident.to_string()
            } else {
                continue;
            };

            // Get type name as string
            let type_name = quote::quote!(#pat_type.ty).to_string();

            // Check if optional (Option<T>)
            let optional = is_option_type(&pat_type.ty);

            // Find documentation for this parameter
            let description = param_docs
                .iter()
                .find(|(name, _)| name == &param_name)
                .map(|(_, desc)| desc.clone())
                .unwrap_or_default();

            parameters.push(ParamInfo {
                name: param_name,
                type_name,
                description,
                optional,
            });
        }
    }

    Ok(parameters)
}

/// Check if a type is Option<T>
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Extract return type information from function signature
fn extract_return_type(
    sig: &syn::Signature,
    return_doc: Option<String>,
) -> syn::Result<Option<ReturnInfo>> {
    match &sig.output {
        syn::ReturnType::Default => Ok(None),
        syn::ReturnType::Type(_, ty) => {
            let type_name = quote::quote!(#ty).to_string();
            let description = return_doc.unwrap_or_default();

            Ok(Some(ReturnInfo {
                type_name,
                description,
            }))
        }
    }
}

/// Generate code to register documentation with DocRegistry
fn generate_registration_code(doc_info: &DocInfo) -> TokenStream {
    let module = &doc_info.module;
    let name = &doc_info.function_name;
    let description = &doc_info.description;

    // Generate parameter docs
    let param_docs: Vec<TokenStream> = doc_info
        .parameters
        .iter()
        .map(|p| {
            let name = &p.name;
            let type_name = &p.type_name;
            let desc = &p.description;
            let optional = p.optional;

            quote! {
                crate::scripting::docs::types::ParamDoc {
                    name: #name.to_string(),
                    type_name: #type_name.to_string(),
                    description: #desc.to_string(),
                    optional: #optional,
                    default: None,
                }
            }
        })
        .collect();

    // Generate signature params
    let sig_params: Vec<TokenStream> = doc_info
        .parameters
        .iter()
        .map(|p| {
            let name = &p.name;
            let type_name = &p.type_name;
            quote! {
                (#name.to_string(), #type_name.to_string())
            }
        })
        .collect();

    // Generate return doc
    let return_doc = if let Some(ret) = &doc_info.returns {
        let type_name = &ret.type_name;
        let desc = &ret.description;
        quote! {
            Some(crate::scripting::docs::types::ReturnDoc {
                type_name: #type_name.to_string(),
                description: #desc.to_string(),
            })
        }
    } else {
        quote! { None }
    };

    let return_type = if let Some(ret) = &doc_info.returns {
        let type_name = &ret.type_name;
        quote! { Some(#type_name.to_string()) }
    } else {
        quote! { None }
    };

    // Generate examples
    let examples: Vec<&String> = doc_info.examples.iter().collect();

    // Generate notes
    let notes = if let Some(n) = &doc_info.notes {
        quote! { Some(#n.to_string()) }
    } else {
        quote! { None }
    };

    // Generate since
    let since = if let Some(s) = &doc_info.since {
        quote! { Some(#s.to_string()) }
    } else {
        quote! { None }
    };

    // Generate deprecated
    let deprecated = if let Some(d) = &doc_info.deprecated {
        quote! { Some(#d.to_string()) }
    } else {
        quote! { None }
    };

    quote! {
        // Use lazy_static or similar to ensure registration happens only once
        // For now, we'll register on every call (could be optimized with lazy_static)
        {
            use crate::scripting::docs::types::{FunctionDoc, FunctionSignature};
            use crate::scripting::docs::registry;

            let doc = FunctionDoc {
                name: #name.to_string(),
                module: #module.to_string(),
                signature: FunctionSignature {
                    params: vec![#(#sig_params),*],
                    return_type: #return_type,
                },
                description: #description.to_string(),
                parameters: vec![#(#param_docs),*],
                returns: #return_doc,
                examples: vec![#(#examples.to_string()),*],
                since: #since,
                deprecated: #deprecated,
                notes: #notes,
            };

            let _ = registry::register_function(doc);
        }
    }
}

/// Parse attribute tokens into key-value pairs
/// Expects format: module = "value", since = "value"
fn parse_attr_tokens(tokens: TokenStream) -> syn::Result<std::collections::HashMap<String, String>> {
    use syn::{Token, parse::Parse, parse::ParseStream};

    struct AttrArgs {
        args: std::collections::HashMap<String, String>,
    }

    impl Parse for AttrArgs {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let mut args = std::collections::HashMap::new();

            while !input.is_empty() {
                let key: syn::Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let value: syn::LitStr = input.parse()?;

                args.insert(key.to_string(), value.value());

                if !input.is_empty() {
                    input.parse::<Token![,]>()?;
                }
            }

            Ok(AttrArgs { args })
        }
    }

    let parsed = syn::parse2::<AttrArgs>(tokens)?;
    Ok(parsed.args)
}
