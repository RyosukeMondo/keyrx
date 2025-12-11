//! Attribute parsing for the keyrx_ffi macro.
//!
//! This module handles parsing of the `#[keyrx_ffi(domain = "...")]` attribute.

// Allow dead_code until Task 14 integrates this module
#![allow(dead_code)]

use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr, Token,
};

/// Configuration extracted from `#[keyrx_ffi(domain = "...")]` attribute.
#[derive(Debug, Clone)]
pub struct MacroConfig {
    /// The domain name from the attribute.
    pub domain: String,
}

/// Parser for the macro attribute arguments.
///
/// Parses: `domain = "config"` or `domain = "keyring"`
struct MacroArgs {
    domain: LitStr,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let domain_ident: Ident = input.parse()?;
        if domain_ident != "domain" {
            return Err(syn::Error::new(
                domain_ident.span(),
                format!("expected `domain`, found `{domain_ident}`"),
            ));
        }

        input.parse::<Token![=]>()?;
        let domain: LitStr = input.parse()?;

        if domain.value().is_empty() {
            return Err(syn::Error::new(
                domain.span(),
                "domain cannot be empty",
            ));
        }

        Ok(MacroArgs { domain })
    }
}

/// Parse macro attribute tokens into configuration.
///
/// # Arguments
///
/// * `attr` - The attribute token stream (contents inside the parentheses)
///
/// # Returns
///
/// * `Ok(MacroConfig)` - Successfully parsed configuration
/// * `Err(syn::Error)` - Parse error with span information
///
/// # Example
///
/// ```ignore
/// // For attribute: #[keyrx_ffi(domain = "config")]
/// let config = parse_macro_attr(attr_tokens)?;
/// assert_eq!(config.domain, "config");
/// ```
pub fn parse_macro_attr(attr: TokenStream) -> syn::Result<MacroConfig> {
    let args: MacroArgs = syn::parse2(attr)?;
    Ok(MacroConfig {
        domain: args.domain.value(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parse_valid_domain() {
        let attr = quote! { domain = "config" };
        let config = parse_macro_attr(attr).expect("should parse");
        assert_eq!(config.domain, "config");
    }

    #[test]
    fn parse_domain_with_hyphen() {
        let attr = quote! { domain = "my-domain" };
        let config = parse_macro_attr(attr).expect("should parse");
        assert_eq!(config.domain, "my-domain");
    }

    #[test]
    fn parse_domain_with_underscore() {
        let attr = quote! { domain = "my_domain" };
        let config = parse_macro_attr(attr).expect("should parse");
        assert_eq!(config.domain, "my_domain");
    }

    #[test]
    fn error_on_empty_domain() {
        let attr = quote! { domain = "" };
        let result = parse_macro_attr(attr);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("domain cannot be empty"));
    }

    #[test]
    fn error_on_wrong_key() {
        let attr = quote! { module = "config" };
        let result = parse_macro_attr(attr);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("expected `domain`"));
    }

    #[test]
    fn error_on_missing_value() {
        let attr = quote! { domain = };
        let result = parse_macro_attr(attr);
        assert!(result.is_err());
    }

    #[test]
    fn error_on_missing_equals() {
        let attr = quote! { domain "config" };
        let result = parse_macro_attr(attr);
        assert!(result.is_err());
    }

    #[test]
    fn error_on_empty_attr() {
        let attr = quote! {};
        let result = parse_macro_attr(attr);
        assert!(result.is_err());
    }
}
