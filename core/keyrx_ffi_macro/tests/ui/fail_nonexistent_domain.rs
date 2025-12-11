//! Test: error when contract file doesn't exist.

use keyrx_ffi_macro::keyrx_ffi;

struct TestDomain;

#[keyrx_ffi(domain = "nonexistent_domain_xyz")]
impl TestDomain {
    fn do_something() -> Result<String, String> {
        Ok("done".to_string())
    }
}

fn main() {}
