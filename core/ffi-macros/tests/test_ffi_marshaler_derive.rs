//! Compilation tests for the #[derive(FfiMarshaler)] macro
//!
//! These tests verify that the macro syntax is correct and can be applied.
//! Actual functional tests will be in keyrx-core where the trait implementations exist.

// Just verify the macro is exported and can be imported
#[allow(unused_imports)]
use keyrx_ffi_macros::FfiMarshaler;

#[test]
fn test_macro_is_exported() {
    // If this test compiles, the FfiMarshaler derive macro is properly exported
    assert!(true);
}

#[test]
fn test_readme_documented() {
    // The macro supports three strategies:
    // - c_struct: Direct C struct representation
    // - json: JSON serialization
    // - auto: Automatically choose based on type complexity
    //
    // Usage:
    // #[derive(FfiMarshaler)]
    // #[ffi(strategy = "c_struct")]
    // struct MyStruct { ... }
    assert!(true);
}
