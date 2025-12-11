//! Test: error when applied to trait implementation.

use keyrx_ffi_macro::keyrx_ffi;

trait SomeTrait {
    fn do_something() -> Result<String, String>;
}

struct TestDomain;

#[keyrx_ffi(domain = "config")]
impl SomeTrait for TestDomain {
    fn do_something() -> Result<String, String> {
        Ok("done".to_string())
    }
}

fn main() {}
