//! Test: error when using wrong attribute key.

use keyrx_ffi_macro::keyrx_ffi;

struct TestDomain;

#[keyrx_ffi(module = "config")]
impl TestDomain {
    fn do_something() -> Result<String, String> {
        Ok("done".to_string())
    }
}

fn main() {}
