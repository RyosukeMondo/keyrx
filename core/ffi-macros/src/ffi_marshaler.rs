//! Derive macro for FfiMarshaler trait
//!
//! This module provides the `#[derive(FfiMarshaler)]` procedural macro that
//! automatically generates implementations of the FfiMarshaler trait.
//!
//! # Strategy Selection
//!
//! The macro supports three strategies via the `#[ffi(strategy = "...")]` attribute:
//!
//! - **`c_struct`**: Generate a `#[repr(C)]` struct and marshal directly.
//!   Best for simple types with fixed-size fields.
//!
//! - **`json`**: Marshal via JSON serialization using serde.
//!   Best for complex types or types with dynamic sizes.
//!
//! - **`auto`**: Automatically choose based on type complexity.
//!   Uses `c_struct` if all fields are primitives or fixed-size,
//!   otherwise falls back to `json`.
//!
//! # Example: C Struct Strategy
//!
//! ```ignore
//! use keyrx_ffi_macros::FfiMarshaler;
//!
//! #[derive(FfiMarshaler)]
//! #[ffi(strategy = "c_struct")]
//! struct DeviceInfo {
//!     vendor_id: u16,
//!     product_id: u16,
//!     #[ffi(string_buffer = 256)]
//!     name: String,
//! }
//! ```
//!
//! Generates:
//! - `DeviceInfoC` struct with `#[repr(C)]`
//! - `FfiMarshaler` implementation with `to_c()` and `from_c()`
//!
//! # Example: JSON Strategy
//!
//! ```ignore
//! use keyrx_ffi_macros::FfiMarshaler;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(FfiMarshaler, Serialize, Deserialize)]
//! #[ffi(strategy = "json")]
//! struct ComplexData {
//!     items: Vec<String>,
//!     metadata: HashMap<String, Value>,
//! }
//! ```
//!
//! Generates:
//! - Uses `JsonWrapper<ComplexData>` as `CRepr`
//! - Marshals via JSON serialization
//!
//! # Field Attributes
//!
//! - `#[ffi(string_buffer = N)]`: For String fields in c_struct strategy,
//!   specifies fixed buffer size (N bytes)
//! - `#[ffi(skip)]`: Skip this field during marshaling (use Default::default())
//!
//! # Requirements
//!
//! For `json` strategy:
//! - Type must implement `Serialize` and `Deserialize`
//!
//! For `c_struct` strategy:
//! - All fields must be primitives, fixed-size arrays, or other CRepr types
//! - String fields must have `#[ffi(string_buffer = N)]` attribute

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Error, Result};

/// Derive macro for FfiMarshaler
pub fn derive_ffi_marshaler_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = match syn::parse2(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };

    match generate_ffi_marshaler(&input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error(),
    }
}

/// Generate FfiMarshaler implementation
fn generate_ffi_marshaler(input: &DeriveInput) -> Result<TokenStream> {
    // Parse strategy from attributes
    let strategy = parse_strategy(input)?;

    match strategy {
        Strategy::Json => generate_json_strategy(input),
        Strategy::CStruct => generate_c_struct_strategy(input),
        Strategy::Auto => generate_auto_strategy(input),
    }
}

/// Marshaling strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strategy {
    /// Marshal via JSON (using JsonWrapper)
    Json,
    /// Marshal via C struct (direct repr(C))
    CStruct,
    /// Auto-detect based on type complexity
    Auto,
}

/// Parse strategy from #[ffi(strategy = "...")] attribute
fn parse_strategy(input: &DeriveInput) -> Result<Strategy> {
    for attr in &input.attrs {
        if !attr.path().is_ident("ffi") {
            continue;
        }

        let meta_list = match &attr.meta {
            syn::Meta::List(list) => list,
            _ => continue,
        };

        // Parse tokens as Name = Value pairs
        let result: Result<Strategy> = syn::parse2(meta_list.tokens.clone());
        if let Ok(strategy) = result {
            return Ok(strategy);
        }
    }

    // Default to Auto if no strategy specified
    Ok(Strategy::Auto)
}

impl syn::parse::Parse for Strategy {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let name: syn::Ident = input.parse()?;
        if name != "strategy" {
            return Err(Error::new_spanned(name, "expected 'strategy'"));
        }

        let _eq: syn::Token![=] = input.parse()?;
        let value: syn::LitStr = input.parse()?;

        match value.value().as_str() {
            "json" => Ok(Strategy::Json),
            "c_struct" => Ok(Strategy::CStruct),
            "auto" => Ok(Strategy::Auto),
            other => Err(Error::new_spanned(
                value,
                format!("unknown strategy '{}', expected 'json', 'c_struct', or 'auto'", other),
            )),
        }
    }
}

/// Generate JSON strategy implementation
fn generate_json_strategy(input: &DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics crate::ffi::marshal::traits::FfiMarshaler for #name #ty_generics #where_clause {
            type CRepr = crate::ffi::marshal::impls::json::JsonWrapperC;

            fn to_c(&self) -> crate::ffi::error::FfiResult<Self::CRepr> {
                crate::ffi::marshal::impls::json::JsonWrapper(self.clone()).to_c()
            }

            fn from_c(c: Self::CRepr) -> crate::ffi::error::FfiResult<Self> {
                let wrapper = crate::ffi::marshal::impls::json::JsonWrapper::<Self>::from_c(c)?;
                Ok(wrapper.0)
            }

            fn estimated_size(&self) -> usize {
                // Estimate JSON size (serialized size + overhead)
                serde_json::to_string(self)
                    .map(|s| s.len())
                    .unwrap_or(1024) // Fallback estimate
            }
        }
    })
}

/// Generate C struct strategy implementation
fn generate_c_struct_strategy(input: &DeriveInput) -> Result<TokenStream> {
    let data = match &input.data {
        syn::Data::Struct(data) => data,
        _ => {
            return Err(Error::new_spanned(
                input,
                "FfiMarshaler derive only supports structs",
            ))
        }
    };

    let fields = match &data.fields {
        syn::Fields::Named(fields) => &fields.named,
        _ => {
            return Err(Error::new_spanned(
                input,
                "FfiMarshaler derive only supports named fields",
            ))
        }
    };

    let name = &input.ident;
    let c_name = format_ident!("{}C", name);

    // Generate C struct fields and conversions
    let mut c_fields = Vec::new();
    let mut to_c_conversions = Vec::new();
    let mut from_c_conversions = Vec::new();
    let mut field_names = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        field_names.push(field_name);
        let field_type = &field.ty;

        // Check if this is a String field with buffer size
        if is_string_type(field_type) {
            let buffer_size = parse_string_buffer_size(field)?;

            c_fields.push(quote! {
                pub #field_name: [u8; #buffer_size]
            });

            to_c_conversions.push(quote! {
                let mut #field_name = [0u8; #buffer_size];
                let bytes = self.#field_name.as_bytes();
                let len = bytes.len().min(#buffer_size - 1); // Reserve for null terminator
                #field_name[..len].copy_from_slice(&bytes[..len]);
            });

            from_c_conversions.push(quote! {
                let #field_name = {
                    let len = c.#field_name.iter().position(|&b| b == 0).unwrap_or(#buffer_size);
                    String::from_utf8_lossy(&c.#field_name[..len]).into_owned()
                };
            });
        } else if is_primitive_type(field_type) {
            // Primitives pass through directly
            c_fields.push(quote! {
                pub #field_name: #field_type
            });

            to_c_conversions.push(quote! {
                let #field_name = self.#field_name;
            });

            from_c_conversions.push(quote! {
                let #field_name = c.#field_name;
            });
        } else {
            return Err(Error::new_spanned(
                field_type,
                "Unsupported field type for c_struct strategy. \
                 Use primitives, String with #[ffi(string_buffer = N)], \
                 or switch to json strategy".to_string(),
            ));
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        // Generate C-compatible struct
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct #c_name {
            #(#c_fields),*
        }

        impl crate::ffi::marshal::traits::CRepr for #c_name {}

        // Generate FfiMarshaler implementation
        impl #impl_generics crate::ffi::marshal::traits::FfiMarshaler for #name #ty_generics #where_clause {
            type CRepr = #c_name;

            fn to_c(&self) -> crate::ffi::error::FfiResult<Self::CRepr> {
                #(#to_c_conversions)*

                Ok(#c_name {
                    #(#field_names),*
                })
            }

            fn from_c(c: Self::CRepr) -> crate::ffi::error::FfiResult<Self> {
                #(#from_c_conversions)*

                Ok(#name {
                    #(#field_names),*
                })
            }

            fn estimated_size(&self) -> usize {
                std::mem::size_of::<#c_name>()
            }
        }
    })
}

/// Generate auto strategy (decide between json and c_struct)
fn generate_auto_strategy(input: &DeriveInput) -> Result<TokenStream> {
    let data = match &input.data {
        syn::Data::Struct(data) => data,
        _ => return generate_json_strategy(input), // Non-structs use JSON
    };

    let fields = match &data.fields {
        syn::Fields::Named(fields) => &fields.named,
        _ => return generate_json_strategy(input), // Non-named fields use JSON
    };

    // Check if all fields are compatible with c_struct strategy
    let mut can_use_c_struct = true;

    for field in fields {
        let field_type = &field.ty;

        if is_string_type(field_type) {
            // String requires buffer size annotation
            if parse_string_buffer_size(field).is_err() {
                can_use_c_struct = false;
                break;
            }
        } else if !is_primitive_type(field_type) {
            can_use_c_struct = false;
            break;
        }
    }

    if can_use_c_struct {
        generate_c_struct_strategy(input)
    } else {
        generate_json_strategy(input)
    }
}

/// Check if type is a string type (String or &str)
fn is_string_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident == "String"
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Check if type is a primitive
fn is_primitive_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                matches!(
                    segment.ident.to_string().as_str(),
                    "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                        | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                        | "f32" | "f64"
                        | "bool"
                )
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Wrapper for parsing string buffer size
struct StringBufferSize(usize);

impl syn::parse::Parse for StringBufferSize {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let name: syn::Ident = input.parse()?;
        if name != "string_buffer" {
            return Err(Error::new_spanned(name, "expected 'string_buffer'"));
        }

        let _eq: syn::Token![=] = input.parse()?;
        let value: syn::LitInt = input.parse()?;

        Ok(StringBufferSize(value.base10_parse()?))
    }
}

/// Parse string buffer size from #[ffi(string_buffer = N)] attribute
fn parse_string_buffer_size(field: &syn::Field) -> Result<usize> {
    for attr in &field.attrs {
        if !attr.path().is_ident("ffi") {
            continue;
        }

        let meta_list = match &attr.meta {
            syn::Meta::List(list) => list,
            _ => continue,
        };

        // Parse as name = value
        let result: Result<StringBufferSize> = syn::parse2(meta_list.tokens.clone());
        if let Ok(size) = result {
            return Ok(size.0);
        }
    }

    Err(Error::new_spanned(
        field,
        "String fields in c_struct strategy require #[ffi(string_buffer = N)] attribute",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_parse_json_strategy() {
        let input = quote! {
            #[ffi(strategy = "json")]
            struct TestStruct {
                field: String,
            }
        };

        let parsed: DeriveInput = syn::parse2(input).unwrap();
        let strategy = parse_strategy(&parsed).unwrap();
        assert_eq!(strategy, Strategy::Json);
    }

    #[test]
    fn test_parse_c_struct_strategy() {
        let input = quote! {
            #[ffi(strategy = "c_struct")]
            struct TestStruct {
                field: u32,
            }
        };

        let parsed: DeriveInput = syn::parse2(input).unwrap();
        let strategy = parse_strategy(&parsed).unwrap();
        assert_eq!(strategy, Strategy::CStruct);
    }

    #[test]
    fn test_parse_auto_strategy() {
        let input = quote! {
            #[ffi(strategy = "auto")]
            struct TestStruct {
                field: u32,
            }
        };

        let parsed: DeriveInput = syn::parse2(input).unwrap();
        let strategy = parse_strategy(&parsed).unwrap();
        assert_eq!(strategy, Strategy::Auto);
    }

    #[test]
    fn test_default_strategy() {
        let input = quote! {
            struct TestStruct {
                field: u32,
            }
        };

        let parsed: DeriveInput = syn::parse2(input).unwrap();
        let strategy = parse_strategy(&parsed).unwrap();
        assert_eq!(strategy, Strategy::Auto);
    }

    #[test]
    fn test_is_string_type() {
        let string_type: syn::Type = syn::parse_quote!(String);
        assert!(is_string_type(&string_type));

        let int_type: syn::Type = syn::parse_quote!(u32);
        assert!(!is_string_type(&int_type));
    }

    #[test]
    fn test_is_primitive_type() {
        let primitives = vec!["u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f32", "f64", "bool"];

        for prim in primitives {
            let ty: syn::Type = syn::parse_str(prim).unwrap();
            assert!(is_primitive_type(&ty), "{} should be primitive", prim);
        }

        let non_prim: syn::Type = syn::parse_quote!(String);
        assert!(!is_primitive_type(&non_prim));
    }
}
