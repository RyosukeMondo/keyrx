//! Attribute parser for #[ffi_export] macro

use proc_macro::TokenStream;
use syn::{punctuated::Punctuated, Expr, ExprLit, Lit, Meta, Token};

/// Configuration extracted from #[ffi_export(...)] attributes
#[derive(Debug, Clone)]
pub struct FfiExportConfig {
    pub domain: String,
    pub version: String,
    pub rename: Option<String>,
    pub skip: bool,
}

impl Default for FfiExportConfig {
    fn default() -> Self {
        Self {
            domain: String::new(),
            version: "1.0".to_string(),
            rename: None,
            skip: false,
        }
    }
}

/// Parse the attribute arguments
pub fn parse_attr(attr: TokenStream) -> syn::Result<FfiExportConfig> {
    let meta_list: Punctuated<Meta, Token![,]> = syn::parse::Parser::parse(
        Punctuated::parse_terminated,
        attr,
    )?;

    let mut config = FfiExportConfig::default();

    for meta in meta_list {
        match meta {
            syn::Meta::NameValue(nv) => {
                let path = nv.path.get_ident()
                    .ok_or_else(|| syn::Error::new_spanned(&nv.path, "Expected identifier"))?;

                match path.to_string().as_str() {
                    "domain" => {
                        config.domain = extract_string_lit(&nv.value)?;
                    }
                    "version" => {
                        config.version = extract_string_lit(&nv.value)?;
                    }
                    "rename" => {
                        config.rename = Some(extract_string_lit(&nv.value)?);
                    }
                    name => {
                        return Err(syn::Error::new_spanned(
                            &nv.path,
                            format!("Unknown attribute: {}", name),
                        ));
                    }
                }
            }
            syn::Meta::Path(path) => {
                if let Some(ident) = path.get_ident() {
                    if ident == "skip" {
                        config.skip = true;
                    } else {
                        return Err(syn::Error::new_spanned(
                            &path,
                            format!("Unknown flag: {}", ident),
                        ));
                    }
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    &meta,
                    "Expected name = \"value\" or flag",
                ));
            }
        }
    }

    // Validate required fields
    if config.domain.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Missing required attribute: domain",
        ));
    }

    Ok(config)
}

/// Extract a string literal from an expression
fn extract_string_lit(expr: &Expr) -> syn::Result<String> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = expr
    {
        Ok(lit_str.value())
    } else {
        Err(syn::Error::new_spanned(expr, "Expected string literal"))
    }
}
