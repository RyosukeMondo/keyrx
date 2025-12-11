//! Test: error when domain attribute is missing.

use keyrx_ffi_macro::keyrx_ffi;

struct TestDomain;

#[keyrx_ffi]
impl TestDomain {
    fn do_something() -> Result<String, String> {
        Ok("done".to_string())
    }
}

fn main() {}
