//! Procedural macros for KeyRX FFI exports
//!
//! This crate provides the `#[ffi_export]` attribute macro that automatically generates
//! C-ABI wrapper functions from Rust methods. The macro handles:
//! - Error conversion to FfiResult
//! - String parameter validation (null checks, UTF-8 validation)
//! - Panic catching to prevent panics from crossing FFI boundary
//! - JSON serialization of results
//!
//! # Example
//!
//! ```ignore
//! use keyrx_ffi_macros::ffi_export;
//!
//! struct MyDomain;
//!
//! impl MyDomain {
//!     #[ffi_export]
//!     fn my_function(&self, param: &str) -> Result<String, MyError> {
//!         Ok(format!("Hello, {}", param))
//!     }
//! }
//! ```

use proc_macro::TokenStream;

/// Attribute macro to generate C-ABI FFI wrappers for Rust methods
///
/// This macro transforms a Rust method into a C-compatible FFI export by:
/// 1. Creating a `#[no_mangle] pub extern "C"` wrapper function
/// 2. Adding null checks for pointer parameters
/// 3. Validating UTF-8 for string parameters
/// 4. Converting errors to FfiResult format
/// 5. Catching panics and converting them to error responses
/// 6. Serializing results to JSON
///
/// # Requirements
///
/// - The method must be part of a type implementing `FfiExportable`
/// - Return type must be `Result<T, E>` where both T and E are serializable
/// - String parameters should use `&str` or `String`
///
/// # Generated code
///
/// The macro generates a wrapper function with the same name prefixed by the domain name,
/// following the pattern: `keyrx_{domain}_{method_name}`
#[proc_macro_attribute]
pub fn ffi_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Try parsing as a standalone function first
    if let Ok(input) = syn::parse::<syn::ItemFn>(item.clone()) {
        match generate_ffi_wrapper_for_item_fn(&input) {
            Ok(output) => return output.into(),
            Err(err) => return err.to_compile_error().into(),
        }
    }

    // If that fails, try parsing as an impl method
    if let Ok(input) = syn::parse::<syn::ImplItemFn>(item.clone()) {
        match generate_ffi_wrapper_for_impl_fn(&input) {
            Ok(output) => return output.into(),
            Err(err) => return err.to_compile_error().into(),
        }
    }

    // If neither works, return an error
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "#[ffi_export] can only be applied to functions or impl methods"
    )
    .to_compile_error()
    .into()
}

/// Generate FFI wrapper for a standalone function
fn generate_ffi_wrapper_for_item_fn(
    func: &syn::ItemFn,
) -> syn::Result<proc_macro2::TokenStream> {
    use quote::{quote, format_ident};

    let func_name = &func.sig.ident;
    let func_inputs = &func.sig.inputs;
    let func_vis = &func.vis;

    // Generate the FFI function name: keyrx_{func_name}
    let ffi_name = format_ident!("keyrx_{}", func_name);

    // Parse function parameters
    let mut ffi_params = Vec::new();
    let mut param_conversions = Vec::new();
    let mut rust_args = Vec::new();

    for input in func_inputs.iter() {
        match input {
            syn::FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    input,
                    "Standalone functions cannot have self parameter"
                ));
            }
            syn::FnArg::Typed(pat_type) => {
                let param_name = if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    &pat_ident.ident
                } else {
                    return Err(syn::Error::new_spanned(
                        &pat_type.pat,
                        "Expected identifier pattern for parameter"
                    ));
                };

                let param_type = &*pat_type.ty;

                // Convert Rust types to FFI types
                if is_string_type(param_type) {
                    ffi_params.push(quote! { #param_name: *const std::ffi::c_char });

                    param_conversions.push(quote! {
                        if #param_name.is_null() {
                            let error = FfiError::null_pointer(stringify!(#param_name));
                            let payload = serialize_ffi_result::<()>(&Err(error))
                                .unwrap_or_else(|e| format!("error:{{\"code\":\"SERIALIZATION_ERROR\",\"message\":\"{}\"}}", e));
                            return std::ffi::CString::new(payload)
                                .map_or_else(|_| std::ptr::null_mut(), std::ffi::CString::into_raw);
                        }

                        let #param_name = match std::ffi::CStr::from_ptr(#param_name).to_str() {
                            Ok(s) => s,
                            Err(_) => {
                                let error = FfiError::invalid_utf8(stringify!(#param_name));
                                let payload = serialize_ffi_result::<()>(&Err(error))
                                    .unwrap_or_else(|e| format!("error:{{\"code\":\"SERIALIZATION_ERROR\",\"message\":\"{}\"}}", e));
                                return std::ffi::CString::new(payload)
                                    .map_or_else(|_| std::ptr::null_mut(), std::ffi::CString::into_raw);
                            }
                        };
                    });

                    rust_args.push(quote! { #param_name });
                } else {
                    // Numeric types and other types pass through directly
                    ffi_params.push(quote! { #param_name: #param_type });
                    rust_args.push(quote! { #param_name });
                }
            }
        }
    }

    // Extract return type
    let return_type = match &func.sig.output {
        syn::ReturnType::Default => {
            return Err(syn::Error::new_spanned(
                &func.sig,
                "#[ffi_export] requires explicit return type (Result<T, E>)"
            ));
        }
        syn::ReturnType::Type(_, ty) => ty,
    };

    // Remove #[ffi_export] from attributes
    let mut func_attrs = func.attrs.clone();
    func_attrs.retain(|attr| !attr.path().is_ident("ffi_export"));

    let func_block = &func.block;

    Ok(quote! {
        // Keep the original function with attributes removed
        #(#func_attrs)*
        #func_vis fn #func_name(#func_inputs) -> #return_type
        #func_block

        // Generate the FFI wrapper function
        #[no_mangle]
        pub unsafe extern "C" fn #ffi_name(#(#ffi_params),*) -> *mut std::ffi::c_char {
            use crate::ffi::error::{FfiError, serialize_ffi_result};

            // Parameter conversions and validation
            #(#param_conversions)*

            // Call the actual function with panic catching
            let result: std::result::Result<#return_type, Box<dyn std::any::Any + Send>> =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    #func_name(#(#rust_args),*)
                }));

            // Handle panic
            let result = match result {
                Ok(r) => r,
                Err(panic_err) => {
                    let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };

                    let error = FfiError::internal(
                        format!("panic in {}: {}", stringify!(#func_name), panic_msg)
                    );
                    Err(error)
                }
            };

            // Serialize result to FFI format
            let payload = serialize_ffi_result(&result)
                .unwrap_or_else(|e| format!("error:{{\"code\":\"SERIALIZATION_ERROR\",\"message\":\"{}\"}}", e));

            // Convert to C string
            std::ffi::CString::new(payload)
                .map_or_else(|_| std::ptr::null_mut(), std::ffi::CString::into_raw)
        }
    })
}

/// Generate FFI wrapper for an impl method
fn generate_ffi_wrapper_for_impl_fn(
    method: &syn::ImplItemFn,
) -> syn::Result<proc_macro2::TokenStream> {
    use quote::{quote, format_ident};

    let method_name = &method.sig.ident;
    let method_inputs = &method.sig.inputs;
    let method_vis = &method.vis;

    // Generate the FFI function name: keyrx_{method_name}
    // Domain prefix will be added when we know the impl context
    let ffi_name = format_ident!("keyrx_{}", method_name);

    // Parse method parameters to generate FFI parameters
    let mut ffi_params = Vec::new();
    let mut param_conversions = Vec::new();
    let mut rust_args = Vec::new();
    let mut inner_params = Vec::new();

    for input in method_inputs.iter() {
        match input {
            syn::FnArg::Receiver(_) => {
                // Skip self parameter - we'll handle instance methods as associated functions
                continue;
            }
            syn::FnArg::Typed(pat_type) => {
                let param_name = if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    &pat_ident.ident
                } else {
                    return Err(syn::Error::new_spanned(
                        &pat_type.pat,
                        "Expected identifier pattern for parameter"
                    ));
                };

                let param_type = &*pat_type.ty;

                // Convert Rust types to FFI types
                if is_string_type(param_type) {
                    // String parameters become *const c_char
                    ffi_params.push(quote! { #param_name: *const std::ffi::c_char });

                    // Generate null check and UTF-8 validation
                    param_conversions.push(quote! {
                        if #param_name.is_null() {
                            let error = FfiError::null_pointer(stringify!(#param_name));
                            let payload = serialize_ffi_result::<()>(&Err(error))
                                .unwrap_or_else(|e| format!("error:{{\"code\":\"SERIALIZATION_ERROR\",\"message\":\"{}\"}}", e));
                            return std::ffi::CString::new(payload)
                                .map_or_else(|_| std::ptr::null_mut(), std::ffi::CString::into_raw);
                        }

                        let #param_name = match std::ffi::CStr::from_ptr(#param_name).to_str() {
                            Ok(s) => s,
                            Err(_) => {
                                let error = FfiError::invalid_utf8(stringify!(#param_name));
                                let payload = serialize_ffi_result::<()>(&Err(error))
                                    .unwrap_or_else(|e| format!("error:{{\"code\":\"SERIALIZATION_ERROR\",\"message\":\"{}\"}}", e));
                                return std::ffi::CString::new(payload)
                                    .map_or_else(|_| std::ptr::null_mut(), std::ffi::CString::into_raw);
                            }
                        };
                    });

                    rust_args.push(quote! { #param_name });
                    inner_params.push(quote! { #param_name: #param_type });
                } else {
                    // Numeric types and other types pass through directly
                    ffi_params.push(quote! { #param_name: #param_type });
                    rust_args.push(quote! { #param_name });
                    inner_params.push(quote! { #param_name: #param_type });
                }
            }
        }
    }

    // Extract return type for use in wrapper
    let return_type = match &method.sig.output {
        syn::ReturnType::Default => {
            return Err(syn::Error::new_spanned(
                &method.sig,
                "#[ffi_export] requires explicit return type (Result<T, E>)"
            ));
        }
        syn::ReturnType::Type(_, ty) => ty,
    };

    // Remove #[ffi_export] from the original method's attributes
    let mut method_attrs = method.attrs.clone();
    method_attrs.retain(|attr| !attr.path().is_ident("ffi_export"));

    let method_block = &method.block;
    let method_sig_without_attrs = syn::Signature {
        ident: method_name.clone(),
        inputs: method_inputs.clone(),
        output: method.sig.output.clone(),
        ..method.sig.clone()
    };

    // Generate the wrapper function
    Ok(quote! {
        // Keep the original method with attributes removed
        #(#method_attrs)*
        #method_vis #method_sig_without_attrs
        #method_block

        // Generate the FFI wrapper function
        #[no_mangle]
        pub unsafe extern "C" fn #ffi_name(#(#ffi_params),*) -> *mut std::ffi::c_char {
            use crate::ffi::error::{FfiError, serialize_ffi_result};

            // Helper function to encapsulate the actual call
            fn inner_call(#(#inner_params),*) -> #return_type {
                #method_name(#(#rust_args),*)
            }

            // Parameter conversions and validation
            #(#param_conversions)*

            // Call the actual method with panic catching
            let result: std::result::Result<#return_type, Box<dyn std::any::Any + Send>> =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    inner_call(#(#rust_args),*)
                }));

            // Handle panic
            let result = match result {
                Ok(r) => r,
                Err(panic_err) => {
                    let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };

                    let error = FfiError::internal(
                        format!("panic in {}: {}", stringify!(#method_name), panic_msg)
                    );
                    Err(error)
                }
            };

            // Serialize result to FFI format
            let payload = serialize_ffi_result(&result)
                .unwrap_or_else(|e| format!("error:{{\"code\":\"SERIALIZATION_ERROR\",\"message\":\"{}\"}}", e));

            // Convert to C string
            std::ffi::CString::new(payload)
                .map_or_else(|_| std::ptr::null_mut(), std::ffi::CString::into_raw)
        }
    })
}

/// Check if a type is a string type (&str, String, etc.)
fn is_string_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(type_ref) => {
            if let syn::Type::Path(type_path) = &*type_ref.elem {
                if let Some(segment) = type_path.path.segments.last() {
                    return segment.ident == "str";
                }
            }
            false
        }
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

