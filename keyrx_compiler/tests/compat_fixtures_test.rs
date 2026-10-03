//! Upgrade path: `.krx` files compiled by OLDER releases must keep loading
//! and mean the same thing as a fresh compile of the same source.
//!
//! The fixtures were produced by the installed `keyrx_compiler` of the
//! pre-h and pre-k rounds (see `tests/fixtures/compat/*.rhai` for the
//! sources). The binary layout (struct fields, enum discriminants) is what
//! makes them load, and `KRX_VERSION` is the only thing that tells a loader
//! the layout changed. If this test fails after a change to a serialized
//! type, the change is NOT backward compatible: either restore the layout
//! (new enum variants take the next unused discriminant) or bump
//! `KRX_VERSION` and regenerate the fixtures deliberately.

use std::path::{Path, PathBuf};

use keyrx_compiler::parser::Parser;
use keyrx_compiler::serialize::{deserialize, read_krx};
use keyrx_core::config::ConfigRoot;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/compat")
        .join(name)
}

fn load_old(name: &str) -> ConfigRoot {
    let bytes = read_krx(&fixture(&format!("{name}.krx"))).unwrap();
    let archived = deserialize(&bytes)
        .map_err(|e| e.to_string())
        .unwrap_or_else(|e| panic!("{name}.krx no longer loads: {e}"));
    match rkyv::Deserialize::deserialize(archived, &mut rkyv::Infallible) {
        Ok(config) => config,
        Err(never) => match never {},
    }
}

fn compile_now(name: &str) -> ConfigRoot {
    Parser::new()
        .parse_script(&fixture(&format!("{name}.rhai")))
        .unwrap()
}

#[test]
fn old_krx_files_load_and_equal_a_fresh_compile() {
    for name in ["simple", "tap_hold", "layers", "tap_hold_real_key"] {
        let old = load_old(name);
        let fresh = compile_now(name);
        assert_eq!(old.version, fresh.version, "{name}: version");
        assert_eq!(
            old.devices, fresh.devices,
            "{name}: devices differ from a fresh compile"
        );
        assert!(!old.devices.is_empty(), "{name}");
    }
}
