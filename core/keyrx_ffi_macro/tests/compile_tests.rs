//! Compile-time tests for keyrx_ffi macro using trybuild.
//!
//! These tests verify that the macro produces helpful error messages for invalid inputs.
//!
//! Note: Pass tests (successful expansions) are covered by integration tests in
//! `core/tests/ffi_macro_integration.rs` which can properly access contract files.

#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    // Test error cases with expected error messages
    t.compile_fail("tests/ui/fail_*.rs");
}
