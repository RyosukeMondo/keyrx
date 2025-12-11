//! Test: error when domain is empty.

use keyrx_ffi_macro::keyrx_ffi;

struct TestDomain;

#[keyrx_ffi(domain = "")]
impl TestDomain {
    fn do_something() -> Result<String, String> {
        Ok("done".to_string())
    }
}

fn main() {}
